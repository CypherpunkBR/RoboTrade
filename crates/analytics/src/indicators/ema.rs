//! # EMA - Exponential Moving Average
//!
//! Média Móvel Exponencial - dá mais peso aos preços recentes.
//!
//! ## Fórmula
//! ```text
//! Multiplicador = 2 / (período + 1)
//! EMA = (Preço atual - EMA anterior) × Multiplicador + EMA anterior
//! ```
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::EMA;
//!
//! let ema = EMA::new(12);
//! let prices = vec![dec!(100), dec!(102), dec!(104), /* ... */];
//! let result = ema.calculate(&prices)?;
//! ```

use super::{validate_data_length, Indicator};
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;

/// Exponential Moving Average (Média Móvel Exponencial)
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: Decimal,
}

impl EMA {
    /// Cria um novo indicador EMA
    ///
    /// # Arguments
    /// * `period` - Número de períodos para o cálculo (ex: 9, 12, 26)
    pub fn new(period: usize) -> Self {
        // Multiplicador = 2 / (período + 1)
        let multiplier = Decimal::from(2) / Decimal::from(period + 1);
        Self { period, multiplier }
    }

    /// Retorna o multiplicador (smoothing factor)
    pub fn multiplier(&self) -> Decimal {
        self.multiplier
    }

    /// Calcula o EMA dado o valor atual e o EMA anterior
    pub fn calculate_next(&self, current_price: Decimal, previous_ema: Decimal) -> Decimal {
        (current_price - previous_ema) * self.multiplier + previous_ema
    }
}

impl Indicator for EMA {
    fn name(&self) -> &'static str {
        "EMA"
    }

    fn period(&self) -> usize {
        self.period
    }

    fn calculate(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<Option<Decimal>>> {
        validate_data_length(prices.len(), self.period)?;

        let mut result = Vec::with_capacity(prices.len());

        // Primeiros (period - 1) valores são None
        for _ in 0..self.period - 1 {
            result.push(None);
        }

        // Primeiro EMA é calculado como SMA
        let initial_sum: Decimal = prices[..self.period].iter().copied().sum();
        let mut current_ema = initial_sum / Decimal::from(self.period);
        result.push(Some(current_ema));

        // Calcula EMA para cada ponto restante
        for &price in prices.iter().skip(self.period) {
            current_ema = self.calculate_next(price, current_ema);
            result.push(Some(current_ema));
        }

        Ok(result)
    }
}

/// EMA com períodos comuns pré-definidos
impl EMA {
    /// EMA de 9 períodos (curto prazo)
    pub fn ema9() -> Self {
        Self::new(9)
    }

    /// EMA de 12 períodos (MACD rápido)
    pub fn ema12() -> Self {
        Self::new(12)
    }

    /// EMA de 21 períodos
    pub fn ema21() -> Self {
        Self::new(21)
    }

    /// EMA de 26 períodos (MACD lento)
    pub fn ema26() -> Self {
        Self::new(26)
    }

    /// EMA de 50 períodos (médio prazo)
    pub fn ema50() -> Self {
        Self::new(50)
    }

    /// EMA de 200 períodos (longo prazo)
    pub fn ema200() -> Self {
        Self::new(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ema_multiplier() {
        let ema = EMA::new(9);
        // Multiplicador para período 9 = 2/(9+1) = 0.2
        assert_eq!(ema.multiplier(), dec!(0.2));

        let ema12 = EMA::new(12);
        // Multiplicador para período 12 ≈ 2/13 ≈ 0.153846...
        assert!(ema12.multiplier() > dec!(0.15) && ema12.multiplier() < dec!(0.16));
    }

    #[test]
    fn test_ema_calculation() {
        let ema = EMA::new(3);
        let prices = vec![dec!(10), dec!(20), dec!(30), dec!(40), dec!(50)];

        let result = ema.calculate(&prices).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        // Primeiro EMA = SMA dos primeiros 3 = (10+20+30)/3 = 20
        assert_eq!(result[2], Some(dec!(20)));
        // Segundo EMA = (40 - 20) * 0.5 + 20 = 30
        assert_eq!(result[3], Some(dec!(30)));
        // Terceiro EMA = (50 - 30) * 0.5 + 30 = 40
        assert_eq!(result[4], Some(dec!(40)));
    }

    #[test]
    fn test_ema_next() {
        let ema = EMA::new(9);
        let previous_ema = dec!(100);
        let current_price = dec!(110);

        // EMA = (110 - 100) * 0.2 + 100 = 2 + 100 = 102
        let result = ema.calculate_next(current_price, previous_ema);
        assert_eq!(result, dec!(102));
    }

    #[test]
    fn test_ema_insufficient_data() {
        let ema = EMA::new(5);
        let prices = vec![dec!(10), dec!(20)];

        let result = ema.calculate(&prices);
        assert!(result.is_err());
    }

    #[test]
    fn test_ema_presets() {
        assert_eq!(EMA::ema9().period(), 9);
        assert_eq!(EMA::ema12().period(), 12);
        assert_eq!(EMA::ema26().period(), 26);
    }
}
