//! # Bollinger Bands
//!
//! Bandas de Bollinger - indicador de volatilidade que consiste em:
//! - Banda central: SMA de N períodos
//! - Banda superior: SMA + (K × desvio padrão)
//! - Banda inferior: SMA - (K × desvio padrão)
//!
//! ## Interpretação
//! - Preço tocando banda superior: Possível sobrecompra
//! - Preço tocando banda inferior: Possível sobrevenda
//! - Bandas contraindo: Baixa volatilidade (possível breakout)
//! - Bandas expandindo: Alta volatilidade
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::BollingerBands;
//!
//! let bb = BollingerBands::default();
//! let prices = vec![dec!(100), dec!(102), dec!(104), /* ... */];
//! let result = bb.calculate_bands(&prices)?;
//! ```

use super::{validate_data_length, Indicator, SMA};
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Bollinger Bands
#[derive(Debug, Clone)]
pub struct BollingerBands {
    /// Período para SMA (padrão: 20)
    period: usize,
    /// Número de desvios padrão (padrão: 2.0)
    std_dev_multiplier: Decimal,
}

/// Resultado do cálculo das Bandas de Bollinger
#[derive(Debug, Clone)]
pub struct BollingerResult {
    /// Banda superior
    pub upper: Option<Decimal>,
    /// Banda central (SMA)
    pub middle: Option<Decimal>,
    /// Banda inferior
    pub lower: Option<Decimal>,
    /// Largura das bandas (upper - lower) / middle
    pub bandwidth: Option<Decimal>,
    /// %B - posição do preço em relação às bandas
    pub percent_b: Option<Decimal>,
}

impl BollingerBands {
    /// Cria novas Bandas de Bollinger com parâmetros customizados
    pub fn new(period: usize, std_dev_multiplier: Decimal) -> Self {
        Self {
            period,
            std_dev_multiplier,
        }
    }

    /// Bandas de Bollinger com parâmetros padrão (20, 2.0)
    pub fn default_params() -> Self {
        Self::new(20, Decimal::from(2))
    }

    /// Calcula as bandas completas
    pub fn calculate_bands(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<BollingerResult>> {
        validate_data_length(prices.len(), self.period)?;

        let sma = SMA::new(self.period);
        let middle_values = sma.calculate(prices)?;

        let mut result = Vec::with_capacity(prices.len());

        for i in 0..prices.len() {
            if let Some(middle) = middle_values[i] {
                // Calcula desvio padrão
                let start_idx = i + 1 - self.period;
                let window = &prices[start_idx..=i];
                let std_dev = calculate_std_dev(window, middle);

                let band_width = std_dev * self.std_dev_multiplier;
                let upper = middle + band_width;
                let lower = middle - band_width;

                // Bandwidth = (Upper - Lower) / Middle
                let bandwidth = if middle != Decimal::ZERO {
                    Some((upper - lower) / middle)
                } else {
                    None
                };

                // %B = (Price - Lower) / (Upper - Lower)
                let percent_b = if upper != lower {
                    Some((prices[i] - lower) / (upper - lower))
                } else {
                    None
                };

                result.push(BollingerResult {
                    upper: Some(upper),
                    middle: Some(middle),
                    lower: Some(lower),
                    bandwidth,
                    percent_b,
                });
            } else {
                result.push(BollingerResult {
                    upper: None,
                    middle: None,
                    lower: None,
                    bandwidth: None,
                    percent_b: None,
                });
            }
        }

        Ok(result)
    }

    /// Verifica se preço está acima da banda superior
    pub fn is_above_upper(price: Decimal, result: &BollingerResult) -> bool {
        result.upper.map_or(false, |upper| price > upper)
    }

    /// Verifica se preço está abaixo da banda inferior
    pub fn is_below_lower(price: Decimal, result: &BollingerResult) -> bool {
        result.lower.map_or(false, |lower| price < lower)
    }

    /// Verifica se preço está dentro das bandas
    pub fn is_within_bands(price: Decimal, result: &BollingerResult) -> bool {
        match (result.upper, result.lower) {
            (Some(upper), Some(lower)) => price <= upper && price >= lower,
            _ => false,
        }
    }

    /// Verifica squeeze (bandas contraídas - baixa volatilidade)
    pub fn is_squeeze(&self, result: &BollingerResult, threshold: Decimal) -> bool {
        result.bandwidth.map_or(false, |bw| bw < threshold)
    }
}

/// Calcula o desvio padrão de uma série de valores
fn calculate_std_dev(values: &[Decimal], mean: Decimal) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }

    let variance: Decimal = values
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<Decimal>()
        / Decimal::from(values.len());

    // Calcula raiz quadrada usando conversão para f64
    let variance_f64 = variance.to_f64().unwrap_or(0.0);
    Decimal::try_from(variance_f64.sqrt()).unwrap_or(Decimal::ZERO)
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::default_params()
    }
}

impl Indicator for BollingerBands {
    fn name(&self) -> &'static str {
        "Bollinger Bands"
    }

    fn period(&self) -> usize {
        self.period
    }

    /// Retorna a banda central (SMA) para compatibilidade com trait
    fn calculate(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<Option<Decimal>>> {
        let bands = self.calculate_bands(prices)?;
        Ok(bands.into_iter().map(|b| b.middle).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_bollinger_calculation() {
        let bb = BollingerBands::new(3, dec!(2));
        let prices = vec![dec!(100), dec!(102), dec!(104), dec!(103), dec!(105)];

        let result = bb.calculate_bands(&prices).unwrap();

        assert_eq!(result.len(), 5);

        // Primeiros 2 valores devem ser None
        assert!(result[0].middle.is_none());
        assert!(result[1].middle.is_none());

        // Terceiro valor em diante deve ter bandas
        assert!(result[2].middle.is_some());
        assert!(result[2].upper.is_some());
        assert!(result[2].lower.is_some());

        // Banda superior > middle > banda inferior
        if let (Some(upper), Some(middle), Some(lower)) = (result[2].upper, result[2].middle, result[2].lower) {
            assert!(upper > middle);
            assert!(middle > lower);
        }
    }

    #[test]
    fn test_bollinger_percent_b() {
        let bb = BollingerBands::new(3, dec!(2));
        let prices = vec![dec!(100), dec!(100), dec!(100), dec!(100), dec!(100)];

        let result = bb.calculate_bands(&prices).unwrap();

        // Com todos preços iguais, desvio padrão = 0, %B seria indefinido
        // Neste caso, %B deve ser None ou 0.5 (preço na média)
        if let Some(percent_b) = result[4].percent_b {
            // Se upper = lower, percent_b é None
            assert!(percent_b >= dec!(0) && percent_b <= dec!(1));
        }
    }

    #[test]
    fn test_bollinger_band_position() {
        let result = BollingerResult {
            upper: Some(dec!(110)),
            middle: Some(dec!(100)),
            lower: Some(dec!(90)),
            bandwidth: Some(dec!(0.2)),
            percent_b: Some(dec!(0.5)),
        };

        assert!(BollingerBands::is_above_upper(dec!(115), &result));
        assert!(!BollingerBands::is_above_upper(dec!(105), &result));

        assert!(BollingerBands::is_below_lower(dec!(85), &result));
        assert!(!BollingerBands::is_below_lower(dec!(95), &result));

        assert!(BollingerBands::is_within_bands(dec!(100), &result));
        assert!(!BollingerBands::is_within_bands(dec!(115), &result));
    }

    #[test]
    fn test_bollinger_squeeze() {
        let bb = BollingerBands::default();

        let result = BollingerResult {
            upper: Some(dec!(102)),
            middle: Some(dec!(100)),
            lower: Some(dec!(98)),
            bandwidth: Some(dec!(0.04)), // 4%
            percent_b: Some(dec!(0.5)),
        };

        // Com threshold de 5%, 4% é squeeze
        assert!(bb.is_squeeze(&result, dec!(0.05)));
        // Com threshold de 3%, 4% não é squeeze
        assert!(!bb.is_squeeze(&result, dec!(0.03)));
    }
}
