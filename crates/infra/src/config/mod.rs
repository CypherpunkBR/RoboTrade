//! Configuração do RoboTrade
//!
//! Gerenciamento de configurações via arquivo TOML.

use directories::ProjectDirs;
use robotrade_core::error::{InfraError, InfraResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tracing::{debug, info, warn};

/// Diretórios padrão da aplicação
static PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

/// Obtém os diretórios do projeto
pub fn project_dirs() -> &'static ProjectDirs {
    PROJECT_DIRS.get_or_init(|| {
        ProjectDirs::from("br", "cypherpunk", "robotrade")
            .expect("Não foi possível determinar diretórios do projeto")
    })
}

/// Caminho para o arquivo de configuração
pub fn config_path() -> PathBuf {
    project_dirs().config_dir().join("config.toml")
}

/// Caminho para o banco de dados
pub fn database_path() -> PathBuf {
    project_dirs().data_dir().join("robotrade.db")
}

/// Caminho para os logs
pub fn logs_path() -> PathBuf {
    project_dirs().data_dir().join("logs")
}

/// Configuração principal do RoboTrade
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// Configurações gerais
    #[serde(default)]
    pub general: GeneralConfig,

    /// Configurações de trading
    #[serde(default)]
    pub trading: TradingConfig,

    /// Configurações de coleta de dados
    #[serde(default)]
    pub data_collection: DataCollectionConfig,

    /// Configurações de notificação
    #[serde(default)]
    pub notifications: NotificationConfig,

    /// Configurações de logging
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Configurações de exchange
    #[serde(default)]
    pub exchange: ExchangeConfig,
}

impl AppConfig {
    /// Carrega configuração do arquivo ou cria padrão
    pub async fn load() -> InfraResult<Self> {
        let config_path = config_path();

        if config_path.exists() {
            debug!("Carregando configuração de {:?}", config_path);
            let content =
                fs::read_to_string(&config_path)
                    .await
                    .map_err(|e| InfraError::Configuration {
                        key: "config_file".into(),
                        reason: format!("Erro ao ler arquivo: {}", e),
                    })?;

            let config: Self = toml::from_str(&content).map_err(|e| InfraError::Configuration {
                key: "config_file".into(),
                reason: format!("Erro ao parsear TOML: {}", e),
            })?;

            info!("Configuração carregada com sucesso");
            Ok(config)
        } else {
            warn!("Arquivo de configuração não encontrado, usando padrões");
            let config = Self::default();
            config.save().await?;
            Ok(config)
        }
    }

    /// Salva a configuração no arquivo
    pub async fn save(&self) -> InfraResult<()> {
        let config_path = config_path();

        // Cria diretório se não existir
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| InfraError::Configuration {
                    key: "config_dir".into(),
                    reason: format!("Erro ao criar diretório: {}", e),
                })?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| InfraError::Configuration {
            key: "config_file".into(),
            reason: format!("Erro ao serializar: {}", e),
        })?;

        fs::write(&config_path, content)
            .await
            .map_err(|e| InfraError::Configuration {
                key: "config_file".into(),
                reason: format!("Erro ao escrever arquivo: {}", e),
            })?;

        debug!("Configuração salva em {:?}", config_path);
        Ok(())
    }
}

/// Configurações gerais
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Tema da interface (dark/light)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Idioma (pt-BR, en-US)
    #[serde(default = "default_language")]
    pub language: String,

    /// Iniciar minimizado no tray
    #[serde(default)]
    pub start_minimized: bool,

    /// Iniciar com o sistema
    #[serde(default)]
    pub start_with_system: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            start_minimized: false,
            start_with_system: false,
        }
    }
}

fn default_theme() -> String {
    "dark".into()
}

fn default_language() -> String {
    "pt-BR".into()
}

/// Configurações de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Modo padrão (paper/live)
    #[serde(default = "default_trading_mode")]
    pub default_mode: String,

    /// Alavancagem padrão
    #[serde(default = "default_leverage")]
    pub default_leverage: u32,

    /// Tamanho padrão da posição (% do capital)
    #[serde(default = "default_position_size")]
    pub default_position_size_pct: Decimal,

    /// Stop loss padrão (%)
    #[serde(default = "default_stop_loss")]
    pub default_stop_loss_pct: Decimal,

    /// Take profit padrão (%)
    #[serde(default = "default_take_profit")]
    pub default_take_profit_pct: Decimal,

    /// Máximo de posições simultâneas
    #[serde(default = "default_max_positions")]
    pub max_positions: u32,

    /// Máximo de risco diário (%)
    #[serde(default = "default_max_daily_risk")]
    pub max_daily_risk_pct: Decimal,

    /// Requer confirmação para sinais
    #[serde(default = "default_true")]
    pub require_signal_confirmation: bool,

    /// Trading automático habilitado
    #[serde(default)]
    pub auto_trading_enabled: bool,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            default_mode: default_trading_mode(),
            default_leverage: default_leverage(),
            default_position_size_pct: default_position_size(),
            default_stop_loss_pct: default_stop_loss(),
            default_take_profit_pct: default_take_profit(),
            max_positions: default_max_positions(),
            max_daily_risk_pct: default_max_daily_risk(),
            require_signal_confirmation: true,
            auto_trading_enabled: false,
        }
    }
}

fn default_trading_mode() -> String {
    "paper".into()
}

fn default_leverage() -> u32 {
    5
}

fn default_position_size() -> Decimal {
    Decimal::from(5)
}

fn default_stop_loss() -> Decimal {
    Decimal::from(2)
}

fn default_take_profit() -> Decimal {
    Decimal::from(4)
}

fn default_max_positions() -> u32 {
    3
}

fn default_max_daily_risk() -> Decimal {
    Decimal::from(5)
}

fn default_true() -> bool {
    true
}

/// Configurações de coleta de dados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionConfig {
    /// Intervalo de coleta do Fear & Greed (minutos)
    #[serde(default = "default_fear_greed_interval")]
    pub fear_greed_interval_mins: u32,

    /// Intervalo de coleta de candles (segundos)
    #[serde(default = "default_candle_interval")]
    pub candle_interval_secs: u32,

    /// Símbolos para monitorar
    #[serde(default = "default_symbols")]
    pub symbols: Vec<String>,

    /// Timeframes para coletar
    #[serde(default = "default_timeframes")]
    pub timeframes: Vec<String>,

    /// Manter histórico de candles (dias)
    #[serde(default = "default_candle_history")]
    pub candle_history_days: u32,
}

impl Default for DataCollectionConfig {
    fn default() -> Self {
        Self {
            fear_greed_interval_mins: default_fear_greed_interval(),
            candle_interval_secs: default_candle_interval(),
            symbols: default_symbols(),
            timeframes: default_timeframes(),
            candle_history_days: default_candle_history(),
        }
    }
}

fn default_fear_greed_interval() -> u32 {
    60 // 1 hora
}

fn default_candle_interval() -> u32 {
    60 // 1 minuto
}

fn default_symbols() -> Vec<String> {
    vec!["BTCUSDT".into(), "ETHUSDT".into()]
}

fn default_timeframes() -> Vec<String> {
    vec!["1h".into(), "4h".into(), "1d".into()]
}

fn default_candle_history() -> u32 {
    90 // 90 dias
}

/// Configurações de notificação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notificações nativas habilitadas
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Sons habilitados
    #[serde(default = "default_true")]
    pub sounds_enabled: bool,

    /// Notificar novos sinais
    #[serde(default = "default_true")]
    pub notify_signals: bool,

    /// Notificar ordens preenchidas
    #[serde(default = "default_true")]
    pub notify_orders: bool,

    /// Notificar fechamento de posições
    #[serde(default = "default_true")]
    pub notify_positions: bool,

    /// Notificar erros
    #[serde(default = "default_true")]
    pub notify_errors: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sounds_enabled: true,
            notify_signals: true,
            notify_orders: true,
            notify_positions: true,
            notify_errors: true,
        }
    }
}

/// Configurações de logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Nível de log (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log para arquivo
    #[serde(default = "default_true")]
    pub file_logging: bool,

    /// Rotação de logs (dias para manter)
    #[serde(default = "default_log_retention")]
    pub retention_days: u32,

    /// Log em formato JSON
    #[serde(default)]
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file_logging: true,
            retention_days: default_log_retention(),
            json_format: false,
        }
    }
}

fn default_log_level() -> String {
    "info".into()
}

fn default_log_retention() -> u32 {
    7
}

/// Configurações de exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// Usar testnet
    #[serde(default = "default_true")]
    pub use_testnet: bool,

    /// Timeout de requisições (segundos)
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u32,

    /// Máximo de retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// API key da Binance (armazenada no keyring, não no arquivo)
    /// Este campo é apenas para documentação
    #[serde(skip)]
    pub api_key: Option<String>,

    /// API secret da Binance (armazenada no keyring, não no arquivo)
    /// Este campo é apenas para documentação
    #[serde(skip)]
    pub api_secret: Option<String>,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            use_testnet: true,
            request_timeout_secs: default_request_timeout(),
            max_retries: default_max_retries(),
            api_key: None,
            api_secret: None,
        }
    }
}

fn default_request_timeout() -> u32 {
    30
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.theme, "dark");
        assert_eq!(config.trading.default_leverage, 5);
        assert!(config.exchange.use_testnet);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("theme"));
        assert!(toml_str.contains("default_leverage"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [general]
            theme = "light"

            [trading]
            default_leverage = 10
        "#;

        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.theme, "light");
        assert_eq!(config.trading.default_leverage, 10);
    }
}
