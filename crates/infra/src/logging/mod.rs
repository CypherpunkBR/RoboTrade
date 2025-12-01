//! Sistema de logging estruturado do RoboTrade
//!
//! Configuração do tracing com output para console e arquivo.

use robotrade_core::error::{InfraError, InfraResult};
use tokio::fs;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::{logs_path, LoggingConfig};

/// Guard que mantém o logging ativo
/// IMPORTANTE: Este guard deve ser mantido enquanto a aplicação estiver rodando
pub struct LoggingGuard {
  _file_guard: Option<WorkerGuard>,
}

/// Inicializa o sistema de logging
pub async fn init_logging(config: &LoggingConfig) -> InfraResult<LoggingGuard> {
  let _level = parse_level(&config.level);

  // Cria filtro baseado no nível
  let filter = EnvFilter::try_new(format!("robotrade={},warn", config.level.to_lowercase()))
    .unwrap_or_else(|_| EnvFilter::new("info"));

  // Layer para console com cores
  let console_layer = fmt::layer()
    .with_target(true)
    .with_thread_ids(false)
    .with_thread_names(false)
    .with_file(false)
    .with_line_number(false)
    .with_ansi(true);

  // Configura file logging se habilitado
  let (file_layer, file_guard) = if config.file_logging {
    let logs_dir = logs_path();

    // Cria diretório de logs
    fs::create_dir_all(&logs_dir)
      .await
      .map_err(|e| InfraError::Configuration {
        key: "logs_dir".into(),
        reason: format!("Erro ao criar diretório de logs: {}", e),
      })?;

    // Configura rolling file appender (novo arquivo por dia)
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &logs_dir, "robotrade.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_fmt = if config.json_format {
      fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .json()
        .with_writer(non_blocking)
    } else {
      // Para formato texto, precisamos de uma abordagem diferente
      // Por simplicidade, usamos o mesmo formato
      fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .json()
        .with_writer(non_blocking)
    };

    (Some(file_fmt), Some(guard))
  } else {
    (None, None)
  };

  // Inicializa subscriber
  let subscriber = tracing_subscriber::registry()
    .with(filter)
    .with(console_layer);

  if let Some(file_layer) = file_layer {
    subscriber.with(file_layer).init();
  } else {
    subscriber.init();
  }

  tracing::info!(
      level = %config.level,
      file_logging = %config.file_logging,
      "Sistema de logging inicializado"
  );

  Ok(LoggingGuard {
    _file_guard: file_guard,
  })
}

/// Converte string para Level
fn parse_level(level: &str) -> Level {
  match level.to_lowercase().as_str() {
    "trace" => Level::TRACE,
    "debug" => Level::DEBUG,
    "info" => Level::INFO,
    "warn" | "warning" => Level::WARN,
    "error" => Level::ERROR,
    _ => Level::INFO,
  }
}

/// Macro helper para log estruturado de trades
#[macro_export]
macro_rules! log_trade {
    ($level:ident, $($field:tt)*) => {
        tracing::$level!(
            target: "robotrade::trades",
            $($field)*
        )
    };
}

/// Macro helper para log estruturado de sinais
#[macro_export]
macro_rules! log_signal {
    ($level:ident, $($field:tt)*) => {
        tracing::$level!(
            target: "robotrade::signals",
            $($field)*
        )
    };
}

/// Macro helper para log estruturado de ordens
#[macro_export]
macro_rules! log_order {
    ($level:ident, $($field:tt)*) => {
        tracing::$level!(
            target: "robotrade::orders",
            $($field)*
        )
    };
}

/// Macro helper para log estruturado de dados de mercado
#[macro_export]
macro_rules! log_market_data {
    ($level:ident, $($field:tt)*) => {
        tracing::$level!(
            target: "robotrade::market_data",
            $($field)*
        )
    };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_level() {
    assert_eq!(parse_level("debug"), Level::DEBUG);
    assert_eq!(parse_level("INFO"), Level::INFO);
    assert_eq!(parse_level("Warning"), Level::WARN);
    assert_eq!(parse_level("invalid"), Level::INFO);
  }
}
