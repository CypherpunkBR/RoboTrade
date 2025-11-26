//! # robotrade-infra
//!
//! Infraestrutura do sistema RoboTrade:
//! - Configuração (TOML)
//! - Logging estruturado (tracing)
//! - Banco de dados SQLite
//! - Implementações de repositórios
//!
//! Este crate depende apenas de `robotrade-core`.

pub mod config;
pub mod database;
pub mod logging;
pub mod repositories;

// Re-exports principais
pub use config::{
    config_path, database_path, logs_path, project_dirs, AppConfig, DataCollectionConfig,
    ExchangeConfig, GeneralConfig, LoggingConfig, NotificationConfig, TradingConfig,
};
pub use database::{close_database, health_check, init_database, DatabaseConfig, DbPool};
pub use logging::{init_logging, LoggingGuard};
pub use repositories::{SqliteCandleRepository, SqliteFearGreedRepository};
