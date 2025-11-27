//! # MACD - Moving Average Convergence Divergence
//!
//! Indicador de momentum que mostra a relação entre duas EMAs.
//!
//! ## Componentes
//! - **Linha MACD**: EMA(12) - EMA(26)
//! - **Linha de Sinal**: EMA(9) da Linha MACD
//! - **Histograma**: Linha MACD - Linha de Sinal
//!
//! ## Sinais
//! - MACD cruza acima do Sinal: Bullish (compra)
//! - MACD cruza abaixo do Sinal: Bearish (venda)
//! - Divergência: Preço e MACD movem em direções opostas
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::MACD;
//!
//! let macd = MACD::default();
//! let prices = vec![dec!(100), dec!(102), dec!(104), /* ... */];
//! let result = macd.calculate(&prices)?;
//! ```

use super::{validate_data_length, Indicator, EMA};
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;

/// MACD - Moving Average Convergence Divergence
#[derive(Debug, Clone)]
pub struct MACD {
    /// Período da EMA rápida (padrão: 12)
    fast_period: usize,
    /// Período da EMA lenta (padrão: 26)
    slow_period: usize,
    /// Período da linha de sinal (padrão: 9)
    signal_period: usize,
}

/// Resultado do cálculo MACD
#[derive(Debug, Clone)]
pub struct MACDResult {
    /// Linha MACD (EMA rápida - EMA lenta)
    pub macd_line: Option<Decimal>,
    /// Linha de sinal (EMA do MACD)
    pub signal_line: Option<Decimal>,
    /// Histograma (MACD - Sinal)
    pub histogram: Option<Decimal>,
}

impl MACD {
    /// Cria um novo indicador MACD com parâmetros customizados
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
        }
    }

    /// MACD com parâmetros padrão (12, 26, 9)
    pub fn default_params() -> Self {
        Self::new(12, 26, 9)
    }

    /// Retorna o período mínimo necessário para cálculo
    pub fn min_period(&self) -> usize {
        self.slow_period + self.signal_period - 1
    }

    /// Calcula o MACD completo retornando linha, sinal e histograma
    pub fn calculate_full(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<MACDResult>> {
        validate_data_length(prices.len(), self.min_period())?;

        let ema_fast = EMA::new(self.fast_period);
        let ema_slow = EMA::new(self.slow_period);

        let fast_values = ema_fast.calculate(prices)?;
        let slow_values = ema_slow.calculate(prices)?;

        // Calcula linha MACD
        let macd_line: Vec<Option<Decimal>> = fast_values
            .iter()
            .zip(slow_values.iter())
            .map(|(fast, slow)| {
                match (fast, slow) {
                    (Some(f), Some(s)) => Some(*f - *s),
                    _ => None,
                }
            })
            .collect();

        // Extrai valores não-None do MACD para calcular a linha de sinal
        let macd_values: Vec<Decimal> = macd_line
            .iter()
            .filter_map(|&v| v)
            .collect();

        // Calcula linha de sinal (EMA do MACD)
        let signal_ema = EMA::new(self.signal_period);
        let signal_values = if macd_values.len() >= self.signal_period {
            signal_ema.calculate(&macd_values)?
        } else {
            vec![None; macd_values.len()]
        };

        // Monta resultado final
        let mut result = Vec::with_capacity(prices.len());
        let mut macd_idx = 0;

        for macd_val in &macd_line {
            let (signal, histogram) = if let Some(m) = macd_val {
                let s = if macd_idx < signal_values.len() {
                    signal_values[macd_idx]
                } else {
                    None
                };
                macd_idx += 1;
                let h = s.map(|sig| *m - sig);
                (s, h)
            } else {
                (None, None)
            };

            result.push(MACDResult {
                macd_line: *macd_val,
                signal_line: signal,
                histogram,
            });
        }

        Ok(result)
    }

    /// Verifica se há cruzamento bullish (MACD cruza acima do sinal)
    pub fn is_bullish_crossover(current: &MACDResult, previous: &MACDResult) -> bool {
        match (current.macd_line, current.signal_line, previous.macd_line, previous.signal_line) {
            (Some(curr_macd), Some(curr_sig), Some(prev_macd), Some(prev_sig)) => {
                curr_macd > curr_sig && prev_macd <= prev_sig
            }
            _ => false,
        }
    }

    /// Verifica se há cruzamento bearish (MACD cruza abaixo do sinal)
    pub fn is_bearish_crossover(current: &MACDResult, previous: &MACDResult) -> bool {
        match (current.macd_line, current.signal_line, previous.macd_line, previous.signal_line) {
            (Some(curr_macd), Some(curr_sig), Some(prev_macd), Some(prev_sig)) => {
                curr_macd < curr_sig && prev_macd >= prev_sig
            }
            _ => false,
        }
    }
}

impl Default for MACD {
    fn default() -> Self {
        Self::default_params()
    }
}

impl Indicator for MACD {
    fn name(&self) -> &'static str {
        "MACD"
    }

    fn period(&self) -> usize {
        self.slow_period
    }

    /// Retorna apenas a linha MACD (para compatibilidade com trait)
    fn calculate(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<Option<Decimal>>> {
        let full_result = self.calculate_full(prices)?;
        Ok(full_result.into_iter().map(|r| r.macd_line).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn generate_test_prices(count: usize, start: Decimal, increment: Decimal) -> Vec<Decimal> {
        (0..count)
            .map(|i| start + increment * Decimal::from(i))
            .collect()
    }

    #[test]
    fn test_macd_min_period() {
        let macd = MACD::default();
        // 26 + 9 - 1 = 34
        assert_eq!(macd.min_period(), 34);
    }

    #[test]
    fn test_macd_calculation() {
        let macd = MACD::new(3, 5, 3);
        // Precisamos de pelo menos 5 + 3 - 1 = 7 preços
        let prices = generate_test_prices(10, dec!(100), dec!(2));

        let result = macd.calculate_full(&prices).unwrap();

        assert_eq!(result.len(), 10);

        // Primeiros valores devem ser None
        assert!(result[0].macd_line.is_none());
        assert!(result[3].macd_line.is_none());

        // A partir do índice 4, MACD deve ter valor
        assert!(result[4].macd_line.is_some());
    }

    #[test]
    fn test_macd_crossover_detection() {
        let previous = MACDResult {
            macd_line: Some(dec!(5)),
            signal_line: Some(dec!(6)),
            histogram: Some(dec!(-1)),
        };

        let current = MACDResult {
            macd_line: Some(dec!(7)),
            signal_line: Some(dec!(6)),
            histogram: Some(dec!(1)),
        };

        assert!(MACD::is_bullish_crossover(&current, &previous));
        assert!(!MACD::is_bearish_crossover(&current, &previous));
    }

    #[test]
    fn test_macd_bearish_crossover() {
        let previous = MACDResult {
            macd_line: Some(dec!(7)),
            signal_line: Some(dec!(6)),
            histogram: Some(dec!(1)),
        };

        let current = MACDResult {
            macd_line: Some(dec!(5)),
            signal_line: Some(dec!(6)),
            histogram: Some(dec!(-1)),
        };

        assert!(MACD::is_bearish_crossover(&current, &previous));
        assert!(!MACD::is_bullish_crossover(&current, &previous));
    }

    #[test]
    fn test_macd_insufficient_data() {
        let macd = MACD::default();
        let prices = vec![dec!(100), dec!(102)];

        let result = macd.calculate(&prices);
        assert!(result.is_err());
    }
}
