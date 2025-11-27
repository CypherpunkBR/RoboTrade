//! # ATR - Average True Range
//!
//! Indicador de volatilidade que mede o range médio verdadeiro.
//!
//! ## Fórmula
//! ```text
//! True Range = max(
//!     High - Low,
//!     |High - Previous Close|,
//!     |Low - Previous Close|
//! )
//! ATR = SMA ou EMA do True Range
//! ```
//!
//! ## Uso
//! - Definir stop-loss baseado em volatilidade
//! - Identificar períodos de alta/baixa volatilidade
//! - Position sizing
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::ATR;
//!
//! let atr = ATR::new(14);
//! let candles = vec![/* candles... */];
//! let result = atr.calculate_from_candles(&candles)?;
//! ```

use super::{validate_data_length, CandleIndicator};
use robotrade_core::entities::Candle;
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;

/// Average True Range
#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    use_ema: bool,
}

impl ATR {
    /// Cria um novo indicador ATR
    ///
    /// # Arguments
    /// * `period` - Número de períodos para o cálculo (padrão: 14)
    pub fn new(period: usize) -> Self {
        Self {
            period,
            use_ema: true, // Usa EMA por padrão (Wilder's smoothing)
        }
    }

    /// ATR com período padrão de 14
    pub fn default_period() -> Self {
        Self::new(14)
    }

    /// Configura para usar SMA em vez de EMA
    pub fn with_sma(mut self) -> Self {
        self.use_ema = false;
        self
    }

    /// Calcula o True Range para um candle dado o close anterior
    pub fn true_range(candle: &Candle, previous_close: Option<Decimal>) -> Decimal {
        let high_low = candle.high - candle.low;

        match previous_close {
            Some(prev_close) => {
                let high_prev_close = (candle.high - prev_close).abs();
                let low_prev_close = (candle.low - prev_close).abs();
                high_low.max(high_prev_close).max(low_prev_close)
            }
            None => high_low,
        }
    }

    /// Calcula stop-loss baseado em ATR
    ///
    /// # Arguments
    /// * `current_price` - Preço atual
    /// * `atr_value` - Valor do ATR
    /// * `multiplier` - Multiplicador (ex: 2.0 para 2x ATR)
    /// * `is_long` - Se é posição long (true) ou short (false)
    pub fn calculate_stop_loss(
        current_price: Decimal,
        atr_value: Decimal,
        multiplier: Decimal,
        is_long: bool,
    ) -> Decimal {
        let atr_distance = atr_value * multiplier;
        if is_long {
            current_price - atr_distance
        } else {
            current_price + atr_distance
        }
    }

    /// Calcula take-profit baseado em ATR
    pub fn calculate_take_profit(
        current_price: Decimal,
        atr_value: Decimal,
        multiplier: Decimal,
        is_long: bool,
    ) -> Decimal {
        let atr_distance = atr_value * multiplier;
        if is_long {
            current_price + atr_distance
        } else {
            current_price - atr_distance
        }
    }
}

impl CandleIndicator for ATR {
    fn name(&self) -> &'static str {
        "ATR"
    }

    fn period(&self) -> usize {
        self.period
    }

    fn calculate_from_candles(&self, candles: &[Candle]) -> AnalyticsResult<Vec<Option<Decimal>>> {
        validate_data_length(candles.len(), self.period + 1)?;

        let mut true_ranges = Vec::with_capacity(candles.len());

        // Primeiro TR não tem close anterior
        true_ranges.push(candles[0].high - candles[0].low);

        // Calcula TR para cada candle
        for i in 1..candles.len() {
            let tr = Self::true_range(&candles[i], Some(candles[i - 1].close));
            true_ranges.push(tr);
        }

        let mut result = Vec::with_capacity(candles.len());

        // Primeiros (period - 1) valores são None
        for _ in 0..self.period - 1 {
            result.push(None);
        }

        if self.use_ema {
            // Primeiro ATR é calculado como SMA
            let initial_sum: Decimal = true_ranges[..self.period].iter().copied().sum();
            let mut current_atr = initial_sum / Decimal::from(self.period);
            result.push(Some(current_atr));

            // ATR seguintes usam Wilder's smoothing
            // ATR = ((ATR anterior × (período - 1)) + TR atual) / período
            let period_dec = Decimal::from(self.period);
            let period_minus_one = period_dec - Decimal::ONE;

            for &tr in true_ranges.iter().skip(self.period) {
                current_atr = (current_atr * period_minus_one + tr) / period_dec;
                result.push(Some(current_atr));
            }
        } else {
            // SMA simples
            for i in (self.period - 1)..candles.len() {
                let start_idx = i + 1 - self.period;
                let sum: Decimal = true_ranges[start_idx..=i].iter().copied().sum();
                result.push(Some(sum / Decimal::from(self.period)));
            }
        }

        Ok(result)
    }
}

/// Configuração de multiplicadores ATR comuns
#[derive(Debug, Clone)]
pub struct ATRMultipliers {
    /// Multiplicador para stop-loss (padrão: 2.0)
    pub stop_loss: Decimal,
    /// Multiplicador para take-profit (padrão: 3.0)
    pub take_profit: Decimal,
    /// Multiplicador para trailing stop (padrão: 2.5)
    pub trailing_stop: Decimal,
}

impl Default for ATRMultipliers {
    fn default() -> Self {
        Self {
            stop_loss: Decimal::from(2),
            take_profit: Decimal::from(3),
            trailing_stop: Decimal::TWO + Decimal::new(5, 1), // 2.5
        }
    }
}

impl ATRMultipliers {
    /// Multiplicadores conservadores
    pub fn conservative() -> Self {
        Self {
            stop_loss: Decimal::from(3),
            take_profit: Decimal::from(4),
            trailing_stop: Decimal::from(3),
        }
    }

    /// Multiplicadores agressivos
    pub fn aggressive() -> Self {
        Self {
            stop_loss: Decimal::new(15, 1), // 1.5
            take_profit: Decimal::from(2),
            trailing_stop: Decimal::from(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_candle(open: Decimal, high: Decimal, low: Decimal, close: Decimal) -> Candle {
        Candle::new(open, high, low, close, dec!(1000), Utc::now(), Utc::now())
    }

    #[test]
    fn test_true_range_no_gap() {
        let candle = create_candle(dec!(100), dec!(110), dec!(95), dec!(105));
        let tr = ATR::true_range(&candle, Some(dec!(100)));

        // TR = max(110-95, |110-100|, |95-100|) = max(15, 10, 5) = 15
        assert_eq!(tr, dec!(15));
    }

    #[test]
    fn test_true_range_gap_up() {
        let candle = create_candle(dec!(110), dec!(115), dec!(108), dec!(112));
        let tr = ATR::true_range(&candle, Some(dec!(100)));

        // TR = max(115-108, |115-100|, |108-100|) = max(7, 15, 8) = 15
        assert_eq!(tr, dec!(15));
    }

    #[test]
    fn test_true_range_gap_down() {
        let candle = create_candle(dec!(90), dec!(92), dec!(85), dec!(88));
        let tr = ATR::true_range(&candle, Some(dec!(100)));

        // TR = max(92-85, |92-100|, |85-100|) = max(7, 8, 15) = 15
        assert_eq!(tr, dec!(15));
    }

    #[test]
    fn test_atr_calculation() {
        let atr = ATR::new(3);
        let candles = vec![
            create_candle(dec!(100), dec!(105), dec!(98), dec!(103)),
            create_candle(dec!(103), dec!(108), dec!(101), dec!(106)),
            create_candle(dec!(106), dec!(110), dec!(104), dec!(108)),
            create_candle(dec!(108), dec!(112), dec!(106), dec!(110)),
            create_candle(dec!(110), dec!(115), dec!(108), dec!(113)),
        ];

        let result = atr.calculate_from_candles(&candles).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());
        assert!(result[4].is_some());
    }

    #[test]
    fn test_stop_loss_calculation() {
        let current_price = dec!(100);
        let atr_value = dec!(5);
        let multiplier = dec!(2);

        let long_stop = ATR::calculate_stop_loss(current_price, atr_value, multiplier, true);
        let short_stop = ATR::calculate_stop_loss(current_price, atr_value, multiplier, false);

        assert_eq!(long_stop, dec!(90)); // 100 - (5 * 2) = 90
        assert_eq!(short_stop, dec!(110)); // 100 + (5 * 2) = 110
    }

    #[test]
    fn test_take_profit_calculation() {
        let current_price = dec!(100);
        let atr_value = dec!(5);
        let multiplier = dec!(3);

        let long_tp = ATR::calculate_take_profit(current_price, atr_value, multiplier, true);
        let short_tp = ATR::calculate_take_profit(current_price, atr_value, multiplier, false);

        assert_eq!(long_tp, dec!(115)); // 100 + (5 * 3) = 115
        assert_eq!(short_tp, dec!(85)); // 100 - (5 * 3) = 85
    }
}
