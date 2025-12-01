//! Estratégia baseada no Fear & Greed Index
//!
//! Compra quando o mercado está em "Extreme Fear" e vende quando está em "Extreme Greed"

use async_trait::async_trait;
use parking_lot::RwLock;
use robotrade_core::entities::{
  Candle, FearGreedData, Signal, SignalStrength, SignalType, TradeDirection,
};
use robotrade_core::error::RoboTradeError;
use std::sync::Arc;
use tracing::{debug, info};

use super::Strategy;

/// Símbolo padrão para sinais quando não especificado
const DEFAULT_SYMBOL: &str = "BTCUSDT";

/// Configuração da estratégia Fear & Greed
#[derive(Debug, Clone)]
pub struct FearGreedStrategyConfig {
  /// Limite para sinal de compra (abaixo deste valor = compra)
  pub buy_threshold: u8,
  /// Limite para sinal de venda (acima deste valor = venda)
  pub sell_threshold: u8,
  /// Força mínima do sinal para operar
  pub min_signal_strength: SignalStrength,
  /// Se deve usar confirmação adicional (média móvel do índice)
  pub use_confirmation: bool,
  /// Janela da média móvel para confirmação
  pub confirmation_window: usize,
}

impl Default for FearGreedStrategyConfig {
  fn default() -> Self {
    Self {
      buy_threshold: 25,  // Extreme Fear
      sell_threshold: 75, // Extreme Greed
      min_signal_strength: SignalStrength::Moderate,
      use_confirmation: false,
      confirmation_window: 7,
    }
  }
}

/// Estratégia Fear & Greed
///
/// Lógica:
/// - Compra quando Fear & Greed <= buy_threshold (mercado com medo extremo)
/// - Vende quando Fear & Greed >= sell_threshold (mercado com ganância extrema)
/// - Força do sinal baseada em quão distante está do threshold
pub struct FearGreedStrategy {
  config: FearGreedStrategyConfig,
  /// Dados do Fear & Greed Index (atualizado externamente)
  fear_greed_data: Arc<RwLock<Option<FearGreedData>>>,
  /// Histórico do Fear & Greed para confirmação
  fear_greed_history: Arc<RwLock<Vec<FearGreedData>>>,
}

impl FearGreedStrategy {
  /// Cria uma nova estratégia Fear & Greed
  pub fn new(config: FearGreedStrategyConfig) -> Self {
    Self {
      config,
      fear_greed_data: Arc::new(RwLock::new(None)),
      fear_greed_history: Arc::new(RwLock::new(Vec::new())),
    }
  }

  /// Atualiza o Fear & Greed Index atual
  pub fn update_fear_greed(&self, data: FearGreedData) {
    // Adiciona ao histórico
    {
      let mut history = self.fear_greed_history.write();
      history.push(data.clone());

      // Mantém apenas os últimos N dias
      let max_history = self.config.confirmation_window * 2;
      while history.len() > max_history {
        history.remove(0);
      }
    }

    // Atualiza o dado atual
    *self.fear_greed_data.write() = Some(data);
  }

  /// Retorna o Fear & Greed Index atual
  pub fn current_fear_greed(&self) -> Option<FearGreedData> {
    self.fear_greed_data.read().clone()
  }

  /// Calcula a força do sinal baseado no valor do índice
  fn calculate_signal_strength(&self, value: u8, threshold: u8, is_buy: bool) -> SignalStrength {
    let distance = if is_buy {
      threshold.saturating_sub(value)
    } else {
      value.saturating_sub(threshold)
    };

    match distance {
      0..=5 => SignalStrength::Weak,
      6..=15 => SignalStrength::Moderate,
      _ => SignalStrength::Strong,
    }
  }

  /// Verifica confirmação com média móvel do índice
  fn check_confirmation(&self, current_value: u8, is_buy: bool) -> bool {
    if !self.config.use_confirmation {
      return true;
    }

    let history = self.fear_greed_history.read();
    if history.len() < self.config.confirmation_window {
      return true; // Sem dados suficientes, aceita o sinal
    }

    let recent: Vec<_> = history
      .iter()
      .rev()
      .take(self.config.confirmation_window)
      .collect();

    let avg: f64 =
      recent.iter().map(|d| d.value as f64).sum::<f64>() / self.config.confirmation_window as f64;

    if is_buy {
      // Para compra, a média também deve estar baixa
      (current_value as f64) < avg
    } else {
      // Para venda, a média também deve estar alta
      (current_value as f64) > avg
    }
  }
}

#[async_trait]
impl Strategy for FearGreedStrategy {
  fn name(&self) -> &str {
    "FearGreed"
  }

  fn description(&self) -> &str {
    "Estratégia baseada no Fear & Greed Index - compra no medo extremo, vende na ganância extrema"
  }

  async fn generate_signal(&self, candles: &[Candle]) -> Result<Option<Signal>, RoboTradeError> {
    let fear_greed = match self.fear_greed_data.read().clone() {
      Some(fg) => fg,
      None => {
        debug!("Sem dados de Fear & Greed disponíveis");
        return Ok(None);
      }
    };

    let candle = match candles.last() {
      Some(c) => c,
      None => return Ok(None),
    };

    let value = fear_greed.value;

    // Verifica sinal de compra
    if value <= self.config.buy_threshold {
      let strength = self.calculate_signal_strength(value, self.config.buy_threshold, true);

      if strength < self.config.min_signal_strength {
        debug!(
            value = %value,
            threshold = %self.config.buy_threshold,
            strength = ?strength,
            "Sinal de compra muito fraco"
        );
        return Ok(None);
      }

      if !self.check_confirmation(value, true) {
        debug!("Sinal de compra não confirmado pela média móvel");
        return Ok(None);
      }

      info!(
          value = %value,
          classification = ?fear_greed.classification,
          strength = ?strength,
          "Gerando sinal de COMPRA (Fear)"
      );

      return Ok(Some(Signal::new(
        "fear_greed_v1",
        "Fear & Greed Strategy",
        DEFAULT_SYMBOL,
        SignalType::FearGreed {
          index_value: value,
          threshold: self.config.buy_threshold,
        },
        TradeDirection::Long,
        strength,
        candle.close,
        format!(
          "Medo extremo detectado: {} ({:?})",
          value, fear_greed.classification
        ),
      )));
    }

    // Verifica sinal de venda
    if value >= self.config.sell_threshold {
      let strength = self.calculate_signal_strength(value, self.config.sell_threshold, false);

      if strength < self.config.min_signal_strength {
        debug!(
            value = %value,
            threshold = %self.config.sell_threshold,
            strength = ?strength,
            "Sinal de venda muito fraco"
        );
        return Ok(None);
      }

      if !self.check_confirmation(value, false) {
        debug!("Sinal de venda não confirmado pela média móvel");
        return Ok(None);
      }

      info!(
          value = %value,
          classification = ?fear_greed.classification,
          strength = ?strength,
          "Gerando sinal de VENDA (Greed)"
      );

      return Ok(Some(Signal::new(
        "fear_greed_v1",
        "Fear & Greed Strategy",
        DEFAULT_SYMBOL,
        SignalType::FearGreed {
          index_value: value,
          threshold: self.config.sell_threshold,
        },
        TradeDirection::Short,
        strength,
        candle.close,
        format!(
          "Ganância extrema detectada: {} ({:?})",
          value, fear_greed.classification
        ),
      )));
    }

    debug!(
        value = %value,
        "Fear & Greed em zona neutra, sem sinal"
    );

    Ok(None)
  }

  fn is_ready(&self) -> bool {
    self.fear_greed_data.read().is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use rust_decimal_macros::dec;

  fn create_test_candle(_symbol: &str, price: rust_decimal::Decimal) -> Candle {
    Candle {
      open_time: Utc::now(),
      close_time: Utc::now(),
      open: price,
      high: price * dec!(1.01),
      low: price * dec!(0.99),
      close: price,
      volume: dec!(1000),
      quote_volume: Some(dec!(1000) * price),
      trade_count: Some(100),
    }
  }

  fn create_fear_greed(value: u8) -> FearGreedData {
    FearGreedData::new(value, Utc::now().date_naive())
  }

  #[tokio::test]
  async fn test_buy_signal_extreme_fear() {
    let config = FearGreedStrategyConfig {
      buy_threshold: 25,
      sell_threshold: 75,
      min_signal_strength: SignalStrength::Weak,
      use_confirmation: false,
      confirmation_window: 7,
    };

    let strategy = FearGreedStrategy::new(config);
    strategy.update_fear_greed(create_fear_greed(10)); // Extreme Fear

    let candles = vec![create_test_candle("BTCUSDT", dec!(50000))];
    let signal = strategy.generate_signal(&candles).await.unwrap();

    assert!(signal.is_some());
    let signal = signal.unwrap();
    assert_eq!(signal.direction, TradeDirection::Long);
  }

  #[tokio::test]
  async fn test_sell_signal_extreme_greed() {
    let config = FearGreedStrategyConfig {
      buy_threshold: 25,
      sell_threshold: 75,
      min_signal_strength: SignalStrength::Weak,
      use_confirmation: false,
      confirmation_window: 7,
    };

    let strategy = FearGreedStrategy::new(config);
    strategy.update_fear_greed(create_fear_greed(90)); // Extreme Greed

    let candles = vec![create_test_candle("BTCUSDT", dec!(50000))];
    let signal = strategy.generate_signal(&candles).await.unwrap();

    assert!(signal.is_some());
    let signal = signal.unwrap();
    assert_eq!(signal.direction, TradeDirection::Short);
  }

  #[tokio::test]
  async fn test_no_signal_neutral() {
    let config = FearGreedStrategyConfig::default();
    let strategy = FearGreedStrategy::new(config);
    strategy.update_fear_greed(create_fear_greed(50)); // Neutral

    let candles = vec![create_test_candle("BTCUSDT", dec!(50000))];
    let signal = strategy.generate_signal(&candles).await.unwrap();

    assert!(signal.is_none());
  }

  #[tokio::test]
  async fn test_signal_strength() {
    let config = FearGreedStrategyConfig::default();
    let strategy = FearGreedStrategy::new(config);

    // Valor muito baixo = sinal forte
    assert_eq!(
      strategy.calculate_signal_strength(5, 25, true),
      SignalStrength::Strong
    );

    // Valor próximo do threshold = sinal fraco
    assert_eq!(
      strategy.calculate_signal_strength(23, 25, true),
      SignalStrength::Weak
    );

    // Valor moderadamente distante = sinal moderado
    assert_eq!(
      strategy.calculate_signal_strength(15, 25, true),
      SignalStrength::Moderate
    );
  }

  #[tokio::test]
  async fn test_no_data_no_signal() {
    let strategy = FearGreedStrategy::new(FearGreedStrategyConfig::default());

    let candles = vec![create_test_candle("BTCUSDT", dec!(50000))];
    let signal = strategy.generate_signal(&candles).await.unwrap();

    assert!(signal.is_none());
    assert!(!strategy.is_ready());
  }
}
