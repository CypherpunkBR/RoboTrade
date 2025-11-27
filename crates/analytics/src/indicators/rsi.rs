//! # RSI - Relative Strength Index
//!
//! Índice de Força Relativa - oscilador de momentum que mede a velocidade
//! e magnitude das mudanças de preço.
//!
//! ## Fórmula
//! ```text
//! RS = Média de ganhos / Média de perdas
//! RSI = 100 - (100 / (1 + RS))
//! ```
//!
//! ## Interpretação
//! - RSI > 70: Sobrecomprado (possível reversão para baixo)
//! - RSI < 30: Sobrevendido (possível reversão para cima)
//! - RSI = 50: Neutro
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::RSI;
//!
//! let rsi = RSI::new(14);
//! let prices = vec![dec!(100), dec!(102), dec!(104), /* ... */];
//! let result = rsi.calculate(&prices)?;
//! ```

use super::{validate_data_length, Indicator};
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;

/// Relative Strength Index (Índice de Força Relativa)
#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
}

impl RSI {
    /// Cria um novo indicador RSI
    ///
    /// # Arguments
    /// * `period` - Número de períodos para o cálculo (padrão: 14)
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    /// RSI com período padrão de 14
    pub fn default_period() -> Self {
        Self::new(14)
    }

    /// Verifica se o RSI indica sobrecompra
    pub fn is_overbought(rsi: Decimal, threshold: Decimal) -> bool {
        rsi > threshold
    }

    /// Verifica se o RSI indica sobrevenda
    pub fn is_oversold(rsi: Decimal, threshold: Decimal) -> bool {
        rsi < threshold
    }

    /// Verifica se o RSI está em zona neutra
    pub fn is_neutral(rsi: Decimal, oversold: Decimal, overbought: Decimal) -> bool {
        rsi >= oversold && rsi <= overbought
    }
}

impl Indicator for RSI {
    fn name(&self) -> &'static str {
        "RSI"
    }

    fn period(&self) -> usize {
        self.period
    }

    fn calculate(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<Option<Decimal>>> {
        // RSI precisa de period + 1 dados (para calcular as mudanças)
        validate_data_length(prices.len(), self.period + 1)?;

        let mut result = Vec::with_capacity(prices.len());

        // Calcula as mudanças de preço
        let changes: Vec<Decimal> = prices
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        // Primeiros 'period' valores são None
        for _ in 0..self.period {
            result.push(None);
        }

        // Calcula ganhos e perdas iniciais (média simples)
        let mut avg_gain = Decimal::ZERO;
        let mut avg_loss = Decimal::ZERO;

        for &change in changes.iter().take(self.period) {
            if change > Decimal::ZERO {
                avg_gain += change;
            } else {
                avg_loss += change.abs();
            }
        }

        avg_gain /= Decimal::from(self.period);
        avg_loss /= Decimal::from(self.period);

        // Calcula primeiro RSI
        let rsi = calculate_rsi(avg_gain, avg_loss);
        result.push(Some(rsi));

        // Calcula RSI para pontos restantes usando média suavizada
        let period_dec = Decimal::from(self.period);
        let one = Decimal::ONE;

        for &change in changes.iter().skip(self.period) {
            let (current_gain, current_loss) = if change > Decimal::ZERO {
                (change, Decimal::ZERO)
            } else {
                (Decimal::ZERO, change.abs())
            };

            // Média suavizada (Wilder's smoothing)
            avg_gain = (avg_gain * (period_dec - one) + current_gain) / period_dec;
            avg_loss = (avg_loss * (period_dec - one) + current_loss) / period_dec;

            let rsi = calculate_rsi(avg_gain, avg_loss);
            result.push(Some(rsi));
        }

        Ok(result)
    }
}

/// Calcula o RSI dado ganho médio e perda média
fn calculate_rsi(avg_gain: Decimal, avg_loss: Decimal) -> Decimal {
    if avg_loss == Decimal::ZERO {
        return Decimal::from(100);
    }

    let rs = avg_gain / avg_loss;
    Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs))
}

/// Configuração de níveis RSI
#[derive(Debug, Clone)]
pub struct RSILevels {
    /// Nível de sobrevenda (padrão: 30)
    pub oversold: Decimal,
    /// Nível de sobrecompra (padrão: 70)
    pub overbought: Decimal,
    /// Nível de sobrevenda extrema (padrão: 20)
    pub extreme_oversold: Decimal,
    /// Nível de sobrecompra extrema (padrão: 80)
    pub extreme_overbought: Decimal,
}

impl Default for RSILevels {
    fn default() -> Self {
        Self {
            oversold: Decimal::from(30),
            overbought: Decimal::from(70),
            extreme_oversold: Decimal::from(20),
            extreme_overbought: Decimal::from(80),
        }
    }
}

impl RSILevels {
    /// Níveis conservadores (25/75)
    pub fn conservative() -> Self {
        Self {
            oversold: Decimal::from(25),
            overbought: Decimal::from(75),
            extreme_oversold: Decimal::from(15),
            extreme_overbought: Decimal::from(85),
        }
    }

    /// Níveis agressivos (35/65)
    pub fn aggressive() -> Self {
        Self {
            oversold: Decimal::from(35),
            overbought: Decimal::from(65),
            extreme_oversold: Decimal::from(25),
            extreme_overbought: Decimal::from(75),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_rsi_calculation() {
        let rsi = RSI::new(3);
        // Sequência de preços crescentes (deve dar RSI alto)
        let prices = vec![
            dec!(100),
            dec!(102),
            dec!(104),
            dec!(106),
            dec!(108),
        ];

        let result = rsi.calculate(&prices).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_none());
        // RSI de sequência crescente deve ser 100 (só ganhos, sem perdas)
        assert_eq!(result[3], Some(dec!(100)));
    }

    #[test]
    fn test_rsi_decreasing() {
        let rsi = RSI::new(3);
        // Sequência de preços decrescentes (deve dar RSI baixo)
        let prices = vec![
            dec!(100),
            dec!(98),
            dec!(96),
            dec!(94),
            dec!(92),
        ];

        let result = rsi.calculate(&prices).unwrap();

        // RSI de sequência decrescente deve ser 0 (só perdas, sem ganhos)
        assert_eq!(result[3], Some(dec!(0)));
    }

    #[test]
    fn test_rsi_levels() {
        assert!(RSI::is_overbought(dec!(75), dec!(70)));
        assert!(!RSI::is_overbought(dec!(65), dec!(70)));

        assert!(RSI::is_oversold(dec!(25), dec!(30)));
        assert!(!RSI::is_oversold(dec!(35), dec!(30)));

        assert!(RSI::is_neutral(dec!(50), dec!(30), dec!(70)));
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let rsi = RSI::new(14);
        let prices = vec![dec!(100), dec!(102)];

        let result = rsi.calculate(&prices);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsi_levels_presets() {
        let default = RSILevels::default();
        assert_eq!(default.oversold, dec!(30));
        assert_eq!(default.overbought, dec!(70));

        let conservative = RSILevels::conservative();
        assert_eq!(conservative.oversold, dec!(25));

        let aggressive = RSILevels::aggressive();
        assert_eq!(aggressive.oversold, dec!(35));
    }
}
