//! Mensagens relacionadas a sinais de trading

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::entities::{Exchange, Signal, SignalId, SignalStrength, SignalType, TradeDirection};

/// Mensagens de sinais
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalMessage {
  /// Novo sinal gerado por estratégia
  NewSignal(Signal),

  /// Sinal confirmado (passou validação)
  SignalConfirmed { signal_id: SignalId },

  /// Sinal rejeitado (falhou validação)
  SignalRejected { signal_id: SignalId, reason: String },

  /// Sinal executado (ordem criada)
  SignalExecuted {
    signal_id: SignalId,
    order_id: String,
  },

  /// Sinal expirado
  SignalExpired { signal_id: SignalId },

  /// Alerta de preço disparado
  PriceAlert {
    symbol: String,
    condition: MessagePriceAlertCondition,
    triggered_price: Decimal,
  },

  /// Alerta de indicador
  IndicatorAlert {
    symbol: String,
    indicator: String,
    value: Decimal,
    message: String,
  },
}

/// Condição de alerta de preço para mensagens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriceAlertCondition {
  /// Preço cruzou acima
  CrossedAbove(Decimal),
  /// Preço cruzou abaixo
  CrossedBelow(Decimal),
  /// Variação percentual
  PercentChange {
    threshold: Decimal,
    is_increase: bool,
  },
}

/// Request para gerar sinal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateSignalRequest {
  /// Exchange alvo
  pub exchange: Exchange,
  /// Símbolo
  pub symbol: String,
  /// Tipo de sinal
  pub signal_type: SignalType,
  /// Direção
  pub direction: TradeDirection,
  /// Força do sinal
  pub strength: SignalStrength,
  /// Preço de entrada sugerido
  pub entry_price: Decimal,
  /// Stop loss sugerido
  pub stop_loss: Option<Decimal>,
  /// Take profit sugerido
  pub take_profit: Option<Decimal>,
  /// Razão/motivo
  pub reason: String,
  /// Estratégia que gerou
  pub strategy_id: Option<String>,
  /// Dados extras
  pub metadata: Option<serde_json::Value>,
}

/// Request para criar alerta de preço
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePriceAlertRequest {
  /// Símbolo
  pub symbol: String,
  /// Condição de disparo
  pub condition: MessagePriceAlertCondition,
  /// Mensagem personalizada
  pub message: Option<String>,
  /// Expiração (opcional)
  pub expires_at: Option<DateTime<Utc>>,
}

/// Comando de request-response para sinais
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalRequest {
  /// Obter sinal por ID
  GetSignal { signal_id: SignalId },

  /// Listar sinais pendentes
  GetPendingSignals {
    exchange: Option<Exchange>,
    symbol: Option<String>,
  },

  /// Listar sinais recentes
  GetRecentSignals {
    limit: usize,
    exchange: Option<Exchange>,
  },

  /// Cancelar sinal
  CancelSignal { signal_id: SignalId },
}

/// Resposta para requests de sinais
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalResponse {
  /// Sinal retornado
  Signal(Option<Signal>),

  /// Lista de sinais
  Signals(Vec<Signal>),

  /// Sinal cancelado
  Cancelled { signal_id: SignalId },

  /// Erro na requisição
  Error { message: String },
}
