# Sistema de Configuração

Este documento descreve o sistema de configuração do RoboTrade.

## Visão Geral

O RoboTrade usa um sistema de configuração em camadas:

```
┌─────────────────────────────────────────────────────────────────┐
│                    PRIORIDADE (maior → menor)                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Runtime Overrides (banco de dados)                          │
│     └─ Alterações feitas pelo usuário durante execução          │
│                                                                  │
│  2. Environment Variables (.env)                                 │
│     └─ Variáveis de ambiente específicas                        │
│                                                                  │
│  3. User Config (config.toml)                                   │
│     └─ Configuração personalizada do usuário                    │
│                                                                  │
│  4. Default Config (built-in)                                   │
│     └─ Valores padrão embutidos no código                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Arquivos de Configuração

### Localização

```rust
use directories::ProjectDirs;

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "cypherpunk", "robotrade")
}

pub fn config_path() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("config.toml")
}

pub fn database_path() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("robotrade.db")
}

pub fn logs_path() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("logs")
}
```

Caminhos típicos por OS:
- **macOS**: `~/Library/Application Support/com.cypherpunk.robotrade/`
- **Linux**: `~/.config/robotrade/` ou `~/.local/share/robotrade/`
- **Windows**: `C:\Users\<user>\AppData\Roaming\cypherpunk\robotrade\`

### config.toml

Arquivo principal de configuração:

```toml
# config.toml - Configuração do RoboTrade

[general]
# Nome da aplicação
app_name = "RoboTrade"

# Ambiente: development, staging, production
environment = "development"

# Diretório de dados personalizado (opcional)
# data_dir = "/custom/path/to/data"

[trading]
# Modo de operação: paper (simulação) ou live (real)
mode = "paper"

# Alavancagem padrão para novos trades
default_leverage = 1

# Margem padrão: isolated ou cross
default_margin_type = "isolated"

# Símbolos monitorados
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]

# Timeframes para análise
timeframes = ["1h", "4h", "1d"]

[risk]
# Tamanho máximo de posição em USD
max_position_size = "10000.00"

# Percentual máximo de risco por trade
max_risk_per_trade = "0.02"

# Perda máxima diária em USD
max_daily_loss = "500.00"

# Máximo de ordens por minuto
max_orders_per_minute = 10

# Máximo de posições abertas
max_open_positions = 5

# Alavancagem máxima permitida
max_leverage = 10

# Exigir stop loss em todas as ordens
require_stop_loss = true

# Distância mínima do stop loss em %
min_stop_loss_distance = "0.5"

# Ativar circuit breaker
circuit_breaker_enabled = true

# Perdas consecutivas para ativar circuit breaker
circuit_breaker_max_losses = 5

# Cooldown em minutos após circuit breaker
circuit_breaker_cooldown_minutes = 60

[exchanges.binance_futures]
# Habilitar esta exchange
enabled = true

# Usar testnet
testnet = true

# ID da API key no keyring
api_key_id = "binance_futures_api_key"

# Rate limits customizados (requests/minuto)
rate_limit_per_minute = 1200

# Timeout em segundos
timeout_seconds = 30

[exchanges.kraken_futures]
enabled = false
testnet = true
api_key_id = "kraken_futures_api_key"

[data_collection]
# Habilitar coleta automática
enabled = true

# Intervalo de coleta em segundos
interval_seconds = 60

# Máximo de retries por coleta
max_retries = 3

# Delay entre retries em ms
retry_delay_ms = 1000

# Providers habilitados
providers = ["binance", "fear_greed"]

[data_collection.fear_greed]
# Intervalo específico para Fear & Greed (mais lento)
interval_seconds = 3600

[strategies.fear_greed_contrarian]
enabled = true
version = "1.0.0"

# Comprar quando Fear & Greed <= threshold
buy_threshold = 25

# Vender quando Fear & Greed >= threshold
sell_threshold = 75

# Símbolos para esta estratégia
symbols = ["BTCUSDT"]

# Tamanho da posição em % do capital
position_size_pct = "0.10"

[strategies.rsi_oversold]
enabled = false
version = "1.0.0"
period = 14
oversold = 30
overbought = 70
symbols = ["BTCUSDT", "ETHUSDT"]

[backtest]
# Capital inicial padrão para backtests
default_initial_capital = "10000.00"

# Taxa de comissão padrão
default_commission_rate = "0.0004"

# Slippage simulado
default_slippage_rate = "0.001"

[logging]
# Nível de log: trace, debug, info, warn, error
level = "info"

# Formato: json ou pretty
format = "json"

# Log para arquivo
file_enabled = true

# Log para console
console_enabled = true

# Log para banco de dados (opcional)
database_enabled = false

# Rotação de logs: daily, hourly, never
rotation = "daily"

# Máximo de arquivos de log mantidos
max_files = 7

[notifications]
# Habilitar notificações
enabled = true

# Notificações nativas do sistema
native_enabled = true

# Som para notificações
sound_enabled = true

# Notificar sobre novos sinais
notify_signals = true

# Notificar sobre ordens preenchidas
notify_orders = true

# Notificar sobre posições fechadas
notify_positions = true

# Notificar sobre erros
notify_errors = true

[database]
# URL do banco de dados
# Por padrão usa SQLite no diretório de dados
# url = "sqlite://path/to/robotrade.db"

# Número máximo de conexões
max_connections = 5

# Timeout de conexão em segundos
connection_timeout = 30

[ui]
# Tema: light, dark, system
theme = "system"

# Idioma: pt-BR, en-US
language = "pt-BR"

# Atualização automática de preços em ms
price_refresh_ms = 1000

# Mostrar na system tray
system_tray_enabled = true

# Minimizar para tray ao fechar
minimize_to_tray = true
```

## Estrutura de Configuração em Rust

```rust
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub data_collection: DataCollectionConfig,
    pub strategies: HashMap<String, StrategyConfig>,
    pub backtest: BacktestConfig,
    pub logging: LoggingConfig,
    pub notifications: NotificationConfig,
    pub database: DatabaseConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub app_name: String,
    pub environment: Environment,
    pub data_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub mode: TradingMode,
    pub default_leverage: u32,
    pub default_margin_type: MarginType,
    pub symbols: Vec<String>,
    pub timeframes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    #[default]
    Paper,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarginType {
    Isolated,
    Cross,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_size: Decimal,
    pub max_risk_per_trade: Decimal,
    pub max_daily_loss: Decimal,
    pub max_orders_per_minute: u32,
    pub max_open_positions: u32,
    pub max_leverage: u32,
    pub require_stop_loss: bool,
    pub min_stop_loss_distance: Decimal,
    pub circuit_breaker_enabled: bool,
    pub circuit_breaker_max_losses: u32,
    pub circuit_breaker_cooldown_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub testnet: bool,
    pub api_key_id: String,
    pub rate_limit_per_minute: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub providers: Vec<String>,
    pub fear_greed: Option<FearGreedCollectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedCollectionConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub version: String,
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub default_initial_capital: Decimal,
    pub default_commission_rate: Decimal,
    pub default_slippage_rate: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file_enabled: bool,
    pub console_enabled: bool,
    pub database_enabled: bool,
    pub rotation: LogRotation,
    pub max_files: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    Daily,
    Hourly,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub native_enabled: bool,
    pub sound_enabled: bool,
    pub notify_signals: bool,
    pub notify_orders: bool,
    pub notify_positions: bool,
    pub notify_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: Option<String>,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: Theme,
    pub language: String,
    pub price_refresh_ms: u64,
    pub system_tray_enabled: bool,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}
```

## Carregamento de Configuração

```rust
impl AppConfig {
    /// Carrega configuração de todas as fontes
    pub fn load() -> InfraResult<Self> {
        // 1. Carregar defaults
        let mut config = Self::default();

        // 2. Carregar de arquivo TOML (se existir)
        let config_path = config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| InfraError::ConfigError {
                    message: format!("Failed to read config file: {}", e),
                })?;

            let file_config: AppConfig = toml::from_str(&content)?;
            config.merge(file_config);
        }

        // 3. Carregar de variáveis de ambiente
        config.apply_env_overrides()?;

        // 4. Validar configuração final
        config.validate()?;

        Ok(config)
    }

    /// Mescla outra configuração (valores não-nulos sobrescrevem)
    fn merge(&mut self, other: AppConfig) {
        // Implementar merge seletivo
        self.general = other.general;
        self.trading = other.trading;
        self.risk = other.risk;
        self.exchanges = other.exchanges;
        // ... etc
    }

    /// Aplica overrides de variáveis de ambiente
    fn apply_env_overrides(&mut self) -> InfraResult<()> {
        // ROBOTRADE_ENV
        if let Ok(env) = std::env::var("ROBOTRADE_ENV") {
            self.general.environment = match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "staging" => Environment::Staging,
                "production" | "prod" => Environment::Production,
                _ => return Err(InfraError::InvalidConfigValue {
                    key: "ROBOTRADE_ENV".to_string(),
                    message: format!("Invalid environment: {}", env),
                }),
            };
        }

        // ROBOTRADE_LOG_LEVEL
        if let Ok(level) = std::env::var("ROBOTRADE_LOG_LEVEL") {
            self.logging.level = level;
        }

        // ROBOTRADE_TRADING_MODE
        if let Ok(mode) = std::env::var("ROBOTRADE_TRADING_MODE") {
            self.trading.mode = match mode.to_lowercase().as_str() {
                "paper" => TradingMode::Paper,
                "live" => TradingMode::Live,
                _ => return Err(InfraError::InvalidConfigValue {
                    key: "ROBOTRADE_TRADING_MODE".to_string(),
                    message: format!("Invalid trading mode: {}", mode),
                }),
            };
        }

        Ok(())
    }

    /// Valida a configuração
    fn validate(&self) -> InfraResult<()> {
        // Validar limites de risco
        if self.risk.max_position_size <= Decimal::ZERO {
            return Err(InfraError::InvalidConfigValue {
                key: "risk.max_position_size".to_string(),
                message: "Must be positive".to_string(),
            });
        }

        if self.risk.max_risk_per_trade <= Decimal::ZERO
            || self.risk.max_risk_per_trade > Decimal::ONE
        {
            return Err(InfraError::InvalidConfigValue {
                key: "risk.max_risk_per_trade".to_string(),
                message: "Must be between 0 and 1".to_string(),
            });
        }

        // Validar modo live
        if self.trading.mode == TradingMode::Live {
            // Verificar se há pelo menos uma exchange habilitada
            let has_enabled_exchange = self.exchanges.values().any(|e| e.enabled);
            if !has_enabled_exchange {
                return Err(InfraError::InvalidConfigValue {
                    key: "trading.mode".to_string(),
                    message: "Live mode requires at least one enabled exchange".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                app_name: "RoboTrade".to_string(),
                environment: Environment::Development,
                data_dir: None,
            },
            trading: TradingConfig {
                mode: TradingMode::Paper,
                default_leverage: 1,
                default_margin_type: MarginType::Isolated,
                symbols: vec!["BTCUSDT".to_string()],
                timeframes: vec!["1h".to_string(), "4h".to_string(), "1d".to_string()],
            },
            risk: RiskConfig {
                max_position_size: Decimal::from(10000),
                max_risk_per_trade: Decimal::from_str("0.02").unwrap(),
                max_daily_loss: Decimal::from(500),
                max_orders_per_minute: 10,
                max_open_positions: 5,
                max_leverage: 10,
                require_stop_loss: true,
                min_stop_loss_distance: Decimal::from_str("0.5").unwrap(),
                circuit_breaker_enabled: true,
                circuit_breaker_max_losses: 5,
                circuit_breaker_cooldown_minutes: 60,
            },
            exchanges: HashMap::new(),
            data_collection: DataCollectionConfig {
                enabled: true,
                interval_seconds: 60,
                max_retries: 3,
                retry_delay_ms: 1000,
                providers: vec!["binance".to_string(), "fear_greed".to_string()],
                fear_greed: Some(FearGreedCollectionConfig {
                    interval_seconds: 3600,
                }),
            },
            strategies: HashMap::new(),
            backtest: BacktestConfig {
                default_initial_capital: Decimal::from(10000),
                default_commission_rate: Decimal::from_str("0.0004").unwrap(),
                default_slippage_rate: Decimal::from_str("0.001").unwrap(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Json,
                file_enabled: true,
                console_enabled: true,
                database_enabled: false,
                rotation: LogRotation::Daily,
                max_files: 7,
            },
            notifications: NotificationConfig {
                enabled: true,
                native_enabled: true,
                sound_enabled: true,
                notify_signals: true,
                notify_orders: true,
                notify_positions: true,
                notify_errors: true,
            },
            database: DatabaseConfig {
                url: None,
                max_connections: 5,
                connection_timeout: 30,
            },
            ui: UiConfig {
                theme: Theme::System,
                language: "pt-BR".to_string(),
                price_refresh_ms: 1000,
                system_tray_enabled: true,
                minimize_to_tray: true,
            },
        }
    }
}
```

## Runtime Overrides

```rust
pub struct ConfigOverrideService {
    db: DbPool,
    cache: RwLock<HashMap<String, serde_json::Value>>,
}

impl ConfigOverrideService {
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> InfraResult<Option<T>> {
        // Verificar cache primeiro
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(Some(serde_json::from_value(value.clone())?));
            }
        }

        // Buscar no banco
        let row = sqlx::query!(
            "SELECT value FROM config_overrides WHERE key = ?",
            key
        )
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(r) => {
                let value: serde_json::Value = serde_json::from_str(&r.value)?;

                // Atualizar cache
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(key.to_string(), value.clone());
                }

                Ok(Some(serde_json::from_value(value)?))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> InfraResult<()> {
        let json_value = serde_json::to_value(value)?;
        let json_string = serde_json::to_string(&json_value)?;

        sqlx::query!(
            r#"
            INSERT INTO config_overrides (key, value, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
            "#,
            key,
            json_string
        )
        .execute(&self.db)
        .await?;

        // Atualizar cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), json_value);
        }

        tracing::info!(key = key, "Config override updated");

        Ok(())
    }

    pub async fn remove(&self, key: &str) -> InfraResult<bool> {
        let result = sqlx::query!("DELETE FROM config_overrides WHERE key = ?", key)
            .execute(&self.db)
            .await?;

        // Remover do cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
```

## Comandos Tauri para Configuração

```rust
#[tauri::command]
pub async fn get_app_config(
    state: tauri::State<'_, AppState>,
) -> Result<AppConfigDto, String> {
    Ok(AppConfigDto::from(&*state.config))
}

#[tauri::command]
pub async fn update_config(
    state: tauri::State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    // Validar que a chave é permitida para atualização
    if !is_runtime_updatable(&key) {
        return Err(format!("Config key '{}' cannot be updated at runtime", key));
    }

    state.config_overrides
        .set(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    // Emitir evento de configuração alterada
    state.app_handle
        .emit_all("config-changed", ConfigChangedPayload { key: key.clone() })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn reload_config(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let new_config = AppConfig::load().map_err(|e| e.to_string())?;

    // Aplicar overrides do banco
    // ... (aplicar overrides)

    // Atualizar estado
    *state.config.write().await = new_config;

    state.app_handle
        .emit_all("config-reloaded", ())
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn is_runtime_updatable(key: &str) -> bool {
    matches!(
        key,
        "ui.theme"
            | "ui.language"
            | "ui.price_refresh_ms"
            | "notifications.enabled"
            | "notifications.sound_enabled"
            | "logging.level"
            | "data_collection.enabled"
            | "data_collection.interval_seconds"
    )
}
```

---

**Próximo**: [Telemetria](./telemetry.md)
