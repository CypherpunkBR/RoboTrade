//! # SMA - Simple Moving Average
//!
//! Média Móvel Simples - calcula a média aritmética dos últimos N preços.
//!
//! ## Fórmula
//! ```text
//! SMA = (P1 + P2 + ... + Pn) / n
//! ```
//!
//! ## Uso
//! ```rust,ignore
//! use robotrade_analytics::indicators::SMA;
//!
//! let sma = SMA::new(14);
//! let prices = vec![dec!(100), dec!(102), dec!(104), /* ... */];
//! let result = sma.calculate(&prices)?;
//! ```

use super::{validate_data_length, Indicator};
use robotrade_core::error::AnalyticsResult;
use rust_decimal::Decimal;

/// Simple Moving Average (Média Móvel Simples)
#[derive(Debug, Clone)]
pub struct SMA {
  period: usize,
}

impl SMA {
  /// Cria um novo indicador SMA
  ///
  /// # Arguments
  /// * `period` - Número de períodos para o cálculo (ex: 14, 20, 50, 200)
  pub fn new(period: usize) -> Self {
    Self { period }
  }

  /// Calcula o SMA para um único ponto dado um slice de preços
  pub fn calculate_single(&self, prices: &[Decimal]) -> Option<Decimal> {
    if prices.len() < self.period {
      return None;
    }

    let sum: Decimal = prices[prices.len() - self.period..].iter().copied().sum();
    Some(sum / Decimal::from(self.period))
  }
}

impl Indicator for SMA {
  fn name(&self) -> &'static str {
    "SMA"
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

    // Calcula SMA para cada ponto restante
    for i in (self.period - 1)..prices.len() {
      let sum: Decimal = prices[i + 1 - self.period..=i].iter().copied().sum();
      result.push(Some(sum / Decimal::from(self.period)));
    }

    Ok(result)
  }
}

/// SMA com períodos comuns pré-definidos
impl SMA {
  /// SMA de 7 períodos (curto prazo)
  pub fn sma7() -> Self {
    Self::new(7)
  }

  /// SMA de 14 períodos
  pub fn sma14() -> Self {
    Self::new(14)
  }

  /// SMA de 20 períodos
  pub fn sma20() -> Self {
    Self::new(20)
  }

  /// SMA de 50 períodos (médio prazo)
  pub fn sma50() -> Self {
    Self::new(50)
  }

  /// SMA de 100 períodos
  pub fn sma100() -> Self {
    Self::new(100)
  }

  /// SMA de 200 períodos (longo prazo)
  pub fn sma200() -> Self {
    Self::new(200)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn test_sma_calculation() {
    let sma = SMA::new(3);
    let prices = vec![dec!(10), dec!(20), dec!(30), dec!(40), dec!(50)];

    let result = sma.calculate(&prices).unwrap();

    assert_eq!(result.len(), 5);
    assert!(result[0].is_none());
    assert!(result[1].is_none());
    assert_eq!(result[2], Some(dec!(20))); // (10+20+30)/3 = 20
    assert_eq!(result[3], Some(dec!(30))); // (20+30+40)/3 = 30
    assert_eq!(result[4], Some(dec!(40))); // (30+40+50)/3 = 40
  }

  #[test]
  fn test_sma_single() {
    let sma = SMA::new(3);
    let prices = vec![dec!(10), dec!(20), dec!(30)];

    let result = sma.calculate_single(&prices);
    assert_eq!(result, Some(dec!(20)));
  }

  #[test]
  fn test_sma_insufficient_data() {
    let sma = SMA::new(5);
    let prices = vec![dec!(10), dec!(20)];

    let result = sma.calculate(&prices);
    assert!(result.is_err());
  }

  #[test]
  fn test_sma_presets() {
    assert_eq!(SMA::sma7().period(), 7);
    assert_eq!(SMA::sma20().period(), 20);
    assert_eq!(SMA::sma50().period(), 50);
    assert_eq!(SMA::sma200().period(), 200);
  }
}
