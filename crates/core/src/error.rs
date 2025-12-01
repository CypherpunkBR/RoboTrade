//! Tipos de erro unificados do RoboTrade
//!
//! Hierarquia de erros para todas as crates do sistema.

use thiserror::Error;

/// Erro principal do sistema
#[derive(Error, Debug)]
pub enum RoboTradeError {
  /// Erro de domínio/core
  #[error("Erro de domínio: {0}")]
  Core(#[from] CoreError),

  /// Erro de dados de mercado
  #[error("Erro de dados de mercado: {0}")]
  MarketData(#[from] MarketDataError),

  /// Erro de exchange/gateway
  #[error("Erro de exchange: {0}")]
  Exchange(#[from] ExchangeError),

  /// Erro de trading
  #[error("Erro de trading: {0}")]
  Trading(#[from] TradingError),

  /// Erro de infraestrutura
  #[error("Erro de infraestrutura: {0}")]
  Infra(#[from] InfraError),

  /// Erro de analytics
  #[error("Erro de analytics: {0}")]
  Analytics(#[from] AnalyticsError),
}

/// Erros do core/domínio
#[derive(Error, Debug)]
pub enum CoreError {
  /// Entidade não encontrada
  #[error("Entidade não encontrada: {entity_type} com ID {id}")]
  NotFound { entity_type: String, id: String },

  /// Validação falhou
  #[error("Validação falhou: {0}")]
  ValidationFailed(String),

  /// Operação inválida para o estado atual
  #[error("Operação inválida: {0}")]
  InvalidOperation(String),

  /// Conversão de tipo falhou
  #[error("Conversão falhou: {0}")]
  ConversionError(String),

  /// Serialização/deserialização
  #[error("Erro de serialização: {0}")]
  Serialization(String),
}

/// Erros de dados de mercado
#[derive(Error, Debug)]
pub enum MarketDataError {
  /// Provedor não disponível
  #[error("Provedor '{provider}' não disponível: {reason}")]
  ProviderUnavailable { provider: String, reason: String },

  /// Símbolo não encontrado
  #[error("Símbolo '{symbol}' não encontrado na exchange '{exchange}'")]
  SymbolNotFound { symbol: String, exchange: String },

  /// Dados insuficientes
  #[error("Dados insuficientes: esperado {expected}, recebido {received}")]
  InsufficientData { expected: usize, received: usize },

  /// Rate limit atingido
  #[error("Rate limit atingido para '{provider}', retry após {retry_after_secs}s")]
  RateLimited {
    provider: String,
    retry_after_secs: u64,
  },

  /// Timeout na requisição
  #[error("Timeout ao buscar dados de '{provider}': {reason}")]
  Timeout { provider: String, reason: String },

  /// Erro de parse dos dados
  #[error("Erro ao parsear dados: {0}")]
  ParseError(String),

  /// Conexão websocket perdida
  #[error("Conexão websocket perdida: {0}")]
  WebSocketDisconnected(String),
}

/// Erros de exchange/gateway
#[derive(Error, Debug)]
pub enum ExchangeError {
  /// Autenticação falhou
  #[error("Autenticação falhou na exchange '{exchange}': {reason}")]
  AuthenticationFailed { exchange: String, reason: String },

  /// Ordem rejeitada
  #[error("Ordem rejeitada pela exchange '{exchange}': {reason}")]
  OrderRejected { exchange: String, reason: String },

  /// Saldo insuficiente
  #[error("Saldo insuficiente: necessário {required} {asset}, disponível {available}")]
  InsufficientBalance {
    asset: String,
    required: String,
    available: String,
  },

  /// Posição não encontrada
  #[error("Posição não encontrada: {symbol} na exchange '{exchange}'")]
  PositionNotFound { symbol: String, exchange: String },

  /// Quantidade inválida
  #[error("Quantidade inválida para {symbol}: {reason}")]
  InvalidQuantity { symbol: String, reason: String },

  /// Preço inválido
  #[error("Preço inválido para {symbol}: {reason}")]
  InvalidPrice { symbol: String, reason: String },

  /// Alavancagem inválida
  #[error("Alavancagem {leverage}x não permitida para {symbol}")]
  InvalidLeverage { symbol: String, leverage: u32 },

  /// Exchange em manutenção
  #[error("Exchange '{exchange}' em manutenção")]
  Maintenance { exchange: String },

  /// Erro de API
  #[error("Erro de API da exchange '{exchange}': código {code} - {message}")]
  ApiError {
    exchange: String,
    code: i32,
    message: String,
  },

  /// Modo paper trading apenas
  #[error("Operação não permitida em modo paper trading")]
  PaperTradingOnly,
}

/// Erros de trading
#[derive(Error, Debug)]
pub enum TradingError {
  /// Estratégia não encontrada
  #[error("Estratégia '{strategy_id}' não encontrada")]
  StrategyNotFound { strategy_id: String },

  /// Sinal expirado
  #[error("Sinal '{signal_id}' expirou")]
  SignalExpired { signal_id: String },

  /// Posição já existe
  #[error("Já existe posição aberta para {symbol}")]
  PositionAlreadyExists { symbol: String },

  /// Limite de posições atingido
  #[error("Limite máximo de {max} posições simultâneas atingido")]
  MaxPositionsReached { max: u32 },

  /// Limite de risco excedido
  #[error("Limite de risco excedido: {reason}")]
  RiskLimitExceeded { reason: String },

  /// Trading desabilitado
  #[error("Trading desabilitado: {reason}")]
  TradingDisabled { reason: String },

  /// Confirmação necessária
  #[error("Operação requer confirmação: {operation}")]
  ConfirmationRequired { operation: String },

  /// Circuit breaker ativo
  #[error("Circuit breaker ativo para {context}: {reason}")]
  CircuitBreakerOpen { context: String, reason: String },
}

/// Erros de infraestrutura
#[derive(Error, Debug)]
pub enum InfraError {
  /// Erro de banco de dados
  #[error("Erro de banco de dados: {0}")]
  Database(String),

  /// Erro de configuração
  #[error("Erro de configuração: {key} - {reason}")]
  Configuration { key: String, reason: String },

  /// Erro de rede/HTTP
  #[error("Erro de rede: {0}")]
  Network(String),

  /// Erro de IO
  #[error("Erro de IO: {0}")]
  Io(String),

  /// Erro de criptografia
  #[error("Erro de criptografia: {0}")]
  Crypto(String),

  /// Recurso não encontrado
  #[error("Recurso não encontrado: {0}")]
  ResourceNotFound(String),

  /// Erro de scheduler
  #[error("Erro de scheduler: {0}")]
  Scheduler(String),
}

/// Erros de analytics
#[derive(Error, Debug)]
pub enum AnalyticsError {
  /// Indicador não encontrado
  #[error("Indicador '{indicator}' não encontrado")]
  IndicatorNotFound { indicator: String },

  /// Parâmetros inválidos
  #[error("Parâmetros inválidos para indicador '{indicator}': {reason}")]
  InvalidParameters { indicator: String, reason: String },

  /// Dados insuficientes para cálculo
  #[error("Dados insuficientes para calcular '{calculation}': mínimo {min_required} candles")]
  InsufficientDataForCalculation {
    calculation: String,
    min_required: usize,
  },

  /// Erro no backtest
  #[error("Erro no backtest: {0}")]
  BacktestError(String),

  /// Divisão por zero
  #[error("Divisão por zero em '{context}'")]
  DivisionByZero { context: String },
}

/// Result type padronizado do RoboTrade
pub type Result<T> = std::result::Result<T, RoboTradeError>;

/// Result type para operações de core
pub type CoreResult<T> = std::result::Result<T, CoreError>;

/// Result type para operações de market data
pub type MarketDataResult<T> = std::result::Result<T, MarketDataError>;

/// Result type para operações de exchange
pub type ExchangeResult<T> = std::result::Result<T, ExchangeError>;

/// Result type para operações de trading
pub type TradingResult<T> = std::result::Result<T, TradingError>;

/// Result type para operações de infra
pub type InfraResult<T> = std::result::Result<T, InfraError>;

/// Result type para operações de analytics
pub type AnalyticsResult<T> = std::result::Result<T, AnalyticsError>;

// Implementações de conversão para erros comuns

impl From<serde_json::Error> for CoreError {
  fn from(e: serde_json::Error) -> Self {
    CoreError::Serialization(e.to_string())
  }
}

impl From<std::io::Error> for InfraError {
  fn from(e: std::io::Error) -> Self {
    InfraError::Io(e.to_string())
  }
}

impl From<uuid::Error> for CoreError {
  fn from(e: uuid::Error) -> Self {
    CoreError::ConversionError(format!("UUID inválido: {}", e))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_error_display() {
    let err = CoreError::NotFound {
      entity_type: "Order".into(),
      id: "123".into(),
    };
    assert!(err.to_string().contains("Order"));
    assert!(err.to_string().contains("123"));
  }

  #[test]
  fn test_error_conversion() {
    let core_err = CoreError::ValidationFailed("teste".into());
    let main_err: RoboTradeError = core_err.into();

    match main_err {
      RoboTradeError::Core(_) => (),
      _ => panic!("Conversão incorreta"),
    }
  }

  #[test]
  fn test_exchange_error() {
    let err = ExchangeError::InsufficientBalance {
      asset: "USDT".into(),
      required: "1000".into(),
      available: "500".into(),
    };

    let msg = err.to_string();
    assert!(msg.contains("USDT"));
    assert!(msg.contains("1000"));
    assert!(msg.contains("500"));
  }
}
