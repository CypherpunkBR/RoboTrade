//! Estratégias de trading
//!
//! Define a trait Strategy e implementações concretas

use async_trait::async_trait;
use robotrade_core::entities::{Candle, Signal};
use robotrade_core::error::RoboTradeError;

pub mod fear_greed;

pub use fear_greed::FearGreedStrategy;

/// Trait que define uma estratégia de trading
#[async_trait]
pub trait Strategy: Send + Sync {
  /// Nome da estratégia
  fn name(&self) -> &str;

  /// Descrição da estratégia
  fn description(&self) -> &str;

  /// Gera sinal baseado nos candles históricos
  /// Retorna None se não houver sinal
  async fn generate_signal(&self, candles: &[Candle]) -> Result<Option<Signal>, RoboTradeError>;

  /// Verifica se a estratégia está pronta para operar
  fn is_ready(&self) -> bool {
    true
  }
}

/// Estratégia mock para testes
#[cfg(test)]
pub struct MockStrategy {
  name: String,
  signals: std::sync::Mutex<std::collections::VecDeque<Option<Signal>>>,
}

#[cfg(test)]
impl MockStrategy {
  pub fn new(name: &str, signals: Vec<Option<Signal>>) -> Self {
    Self {
      name: name.to_string(),
      signals: std::sync::Mutex::new(signals.into_iter().collect()),
    }
  }
}

#[cfg(test)]
#[async_trait]
impl Strategy for MockStrategy {
  fn name(&self) -> &str {
    &self.name
  }

  fn description(&self) -> &str {
    "Estratégia mock para testes"
  }

  async fn generate_signal(&self, _candles: &[Candle]) -> Result<Option<Signal>, RoboTradeError> {
    let mut signals = self.signals.lock().unwrap();
    Ok(signals.pop_front().flatten())
  }
}
