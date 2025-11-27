//! Entidades de Strategy Management
//!
//! Modelos para estratégias, versões e instâncias.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// ID único de uma estratégia
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrategyId(pub Uuid);

impl StrategyId {
    /// Cria um novo ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Cria a partir de uma string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for StrategyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StrategyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ID único de uma versão de estratégia
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrategyVersionId(pub Uuid);

impl StrategyVersionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for StrategyVersionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StrategyVersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ID único de uma instância de estratégia
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrategyInstanceId(pub Uuid);

impl StrategyInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for StrategyInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StrategyInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo de estratégia
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    /// Baseada no Fear & Greed Index
    FearGreed,
    /// Cruzamento de médias móveis
    SmaCrossover,
    /// Cruzamento de EMAs
    EmaCrossover,
    /// RSI com níveis
    RsiLevels,
    /// MACD
    Macd,
    /// Bollinger Bands
    BollingerBands,
    /// Estratégia customizada
    Custom(String),
}

impl fmt::Display for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyType::FearGreed => write!(f, "fear_greed"),
            StrategyType::SmaCrossover => write!(f, "sma_crossover"),
            StrategyType::EmaCrossover => write!(f, "ema_crossover"),
            StrategyType::RsiLevels => write!(f, "rsi_levels"),
            StrategyType::Macd => write!(f, "macd"),
            StrategyType::BollingerBands => write!(f, "bollinger_bands"),
            StrategyType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fear_greed" => Ok(StrategyType::FearGreed),
            "sma_crossover" => Ok(StrategyType::SmaCrossover),
            "ema_crossover" => Ok(StrategyType::EmaCrossover),
            "rsi_levels" => Ok(StrategyType::RsiLevels),
            "macd" => Ok(StrategyType::Macd),
            "bollinger_bands" => Ok(StrategyType::BollingerBands),
            other => {
                if let Some(name) = other.strip_prefix("custom:") {
                    Ok(StrategyType::Custom(name.to_string()))
                } else {
                    Ok(StrategyType::Custom(other.to_string()))
                }
            }
        }
    }
}

/// Definição de uma estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDefinition {
    /// ID único
    pub id: StrategyId,
    /// Nome da estratégia
    pub name: String,
    /// Descrição
    pub description: Option<String>,
    /// Tipo da estratégia
    pub strategy_type: StrategyType,
    /// Símbolos padrão
    pub default_symbols: Vec<String>,
    /// Timeframes padrão
    pub default_timeframes: Vec<String>,
    /// Habilitada?
    pub is_enabled: bool,
    /// Permitido em modo live?
    pub is_live_allowed: bool,
    /// Autor
    pub author: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// URL de documentação
    pub documentation_url: Option<String>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl StrategyDefinition {
    /// Cria uma nova definição de estratégia
    pub fn new(
        name: impl Into<String>,
        strategy_type: StrategyType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: StrategyId::new(),
            name: name.into(),
            description: None,
            strategy_type,
            default_symbols: vec![],
            default_timeframes: vec![],
            is_enabled: false,
            is_live_allowed: false,
            author: None,
            tags: vec![],
            documentation_url: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Define descrição
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Define símbolos padrão
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.default_symbols = symbols;
        self
    }

    /// Define timeframes padrão
    pub fn with_timeframes(mut self, timeframes: Vec<String>) -> Self {
        self.default_timeframes = timeframes;
        self
    }

    /// Habilita a estratégia
    pub fn enable(mut self) -> Self {
        self.is_enabled = true;
        self
    }

    /// Permite modo live
    pub fn allow_live(mut self) -> Self {
        self.is_live_allowed = true;
        self
    }
}

/// Configuração de risco para estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRiskConfig {
    /// Percentual máximo de risco por trade
    pub max_risk_per_trade_pct: rust_decimal::Decimal,
    /// Percentual de stop loss padrão
    pub default_stop_loss_pct: rust_decimal::Decimal,
    /// Percentual de take profit padrão
    pub default_take_profit_pct: rust_decimal::Decimal,
    /// Alavancagem padrão
    pub default_leverage: u32,
    /// Máximo de posições simultâneas
    pub max_positions: u32,
    /// Usar trailing stop?
    pub use_trailing_stop: bool,
    /// Percentual do trailing stop
    pub trailing_stop_pct: Option<rust_decimal::Decimal>,
}

impl Default for StrategyRiskConfig {
    fn default() -> Self {
        use rust_decimal::Decimal;
        Self {
            max_risk_per_trade_pct: Decimal::new(2, 0),  // 2%
            default_stop_loss_pct: Decimal::new(2, 0),   // 2%
            default_take_profit_pct: Decimal::new(4, 0), // 4%
            default_leverage: 1,
            max_positions: 3,
            use_trailing_stop: false,
            trailing_stop_pct: None,
        }
    }
}

/// Versão imutável de uma estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersion {
    /// ID único
    pub id: StrategyVersionId,
    /// ID da estratégia
    pub strategy_id: StrategyId,
    /// Versão semântica (ex: "1.0.0")
    pub version: String,
    /// Major version
    pub version_major: u32,
    /// Minor version
    pub version_minor: u32,
    /// Patch version
    pub version_patch: u32,
    /// Configuração principal (JSON)
    pub config: serde_json::Value,
    /// Configuração de indicadores
    pub indicators_config: Option<serde_json::Value>,
    /// Regras de entrada
    pub entry_rules: serde_json::Value,
    /// Regras de saída
    pub exit_rules: serde_json::Value,
    /// Filtros
    pub filters: Option<serde_json::Value>,
    /// Configuração de risco
    pub risk_config: StrategyRiskConfig,
    /// Versão ativa?
    pub is_active: bool,
    /// Descrição da mudança
    pub change_description: String,
    /// ID da versão pai
    pub parent_version_id: Option<StrategyVersionId>,
    /// Métricas de performance
    pub performance_metrics: Option<serde_json::Value>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Criado por
    pub created_by: String,
}

impl StrategyVersion {
    /// Cria uma nova versão
    pub fn new(
        strategy_id: StrategyId,
        version: impl Into<String>,
        config: serde_json::Value,
        entry_rules: serde_json::Value,
        exit_rules: serde_json::Value,
        change_description: impl Into<String>,
    ) -> Self {
        let version_str = version.into();
        let parts: Vec<u32> = version_str
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();

        Self {
            id: StrategyVersionId::new(),
            strategy_id,
            version: version_str,
            version_major: parts.first().copied().unwrap_or(1),
            version_minor: parts.get(1).copied().unwrap_or(0),
            version_patch: parts.get(2).copied().unwrap_or(0),
            config,
            indicators_config: None,
            entry_rules,
            exit_rules,
            filters: None,
            risk_config: StrategyRiskConfig::default(),
            is_active: false,
            change_description: change_description.into(),
            parent_version_id: None,
            performance_metrics: None,
            created_at: Utc::now(),
            created_by: "user".to_string(),
        }
    }

    /// Ativa esta versão
    pub fn activate(mut self) -> Self {
        self.is_active = true;
        self
    }

    /// Define versão pai
    pub fn with_parent(mut self, parent_id: StrategyVersionId) -> Self {
        self.parent_version_id = Some(parent_id);
        self
    }
}

/// Modo de trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    /// Paper trading (simulação)
    Paper,
    /// Trading real
    Live,
}

impl fmt::Display for TradingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingMode::Paper => write!(f, "paper"),
            TradingMode::Live => write!(f, "live"),
        }
    }
}

impl std::str::FromStr for TradingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "paper" => Ok(TradingMode::Paper),
            "live" => Ok(TradingMode::Live),
            _ => Err(format!("Modo de trading inválido: {}", s)),
        }
    }
}

/// Status de uma instância de estratégia
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrategyInstanceStatus {
    /// Parada
    Stopped,
    /// Em execução
    Running,
    /// Pausada
    Paused,
    /// Em erro
    Error,
}

impl fmt::Display for StrategyInstanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyInstanceStatus::Stopped => write!(f, "stopped"),
            StrategyInstanceStatus::Running => write!(f, "running"),
            StrategyInstanceStatus::Paused => write!(f, "paused"),
            StrategyInstanceStatus::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for StrategyInstanceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stopped" => Ok(StrategyInstanceStatus::Stopped),
            "running" => Ok(StrategyInstanceStatus::Running),
            "paused" => Ok(StrategyInstanceStatus::Paused),
            "error" => Ok(StrategyInstanceStatus::Error),
            _ => Err(format!("Status de instância inválido: {}", s)),
        }
    }
}

/// Instância de execução de uma estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInstance {
    /// ID único
    pub id: StrategyInstanceId,
    /// ID da versão da estratégia
    pub strategy_version_id: StrategyVersionId,
    /// ID do símbolo
    pub symbol_id: i64,
    /// Modo de trading
    pub trading_mode: TradingMode,
    /// Timeframe
    pub timeframe: String,
    /// Overrides de configuração
    pub config_overrides: Option<serde_json::Value>,
    /// Status atual
    pub status: StrategyInstanceStatus,
    /// Última avaliação
    pub last_evaluation_at: Option<DateTime<Utc>>,
    /// Último sinal gerado
    pub last_signal_at: Option<DateTime<Utc>>,
    /// Último erro
    pub last_error: Option<String>,
    /// Contador de erros
    pub error_count: u32,
    /// Sinais gerados
    pub signals_generated: u32,
    /// Trades executados
    pub trades_executed: u32,
    /// Início da execução
    pub started_at: Option<DateTime<Utc>>,
    /// Fim da execução
    pub stopped_at: Option<DateTime<Utc>>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl StrategyInstance {
    /// Cria uma nova instância
    pub fn new(
        strategy_version_id: StrategyVersionId,
        symbol_id: i64,
        trading_mode: TradingMode,
        timeframe: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: StrategyInstanceId::new(),
            strategy_version_id,
            symbol_id,
            trading_mode,
            timeframe: timeframe.into(),
            config_overrides: None,
            status: StrategyInstanceStatus::Stopped,
            last_evaluation_at: None,
            last_signal_at: None,
            last_error: None,
            error_count: 0,
            signals_generated: 0,
            trades_executed: 0,
            started_at: None,
            stopped_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Verifica se está em execução
    pub fn is_running(&self) -> bool {
        self.status == StrategyInstanceStatus::Running
    }

    /// Inicia a instância
    pub fn start(&mut self) {
        self.status = StrategyInstanceStatus::Running;
        self.started_at = Some(Utc::now());
        self.stopped_at = None;
        self.updated_at = Utc::now();
    }

    /// Para a instância
    pub fn stop(&mut self) {
        self.status = StrategyInstanceStatus::Stopped;
        self.stopped_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Pausa a instância
    pub fn pause(&mut self) {
        self.status = StrategyInstanceStatus::Paused;
        self.updated_at = Utc::now();
    }

    /// Registra um erro
    pub fn record_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
        self.error_count += 1;
        self.status = StrategyInstanceStatus::Error;
        self.updated_at = Utc::now();
    }

    /// Registra avaliação
    pub fn record_evaluation(&mut self) {
        self.last_evaluation_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Registra sinal gerado
    pub fn record_signal(&mut self) {
        self.signals_generated += 1;
        self.last_signal_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Registra trade executado
    pub fn record_trade(&mut self) {
        self.trades_executed += 1;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_definition_creation() {
        let strategy = StrategyDefinition::new("Fear & Greed Bot", StrategyType::FearGreed)
            .with_description("Estratégia baseada no índice Fear & Greed")
            .with_symbols(vec!["BTCUSDT".to_string()])
            .with_timeframes(vec!["4h".to_string()])
            .enable();

        assert_eq!(strategy.name, "Fear & Greed Bot");
        assert!(strategy.is_enabled);
        assert!(!strategy.is_live_allowed);
    }

    #[test]
    fn test_strategy_instance_lifecycle() {
        let mut instance = StrategyInstance::new(
            StrategyVersionId::new(),
            1,
            TradingMode::Paper,
            "4h",
        );

        assert_eq!(instance.status, StrategyInstanceStatus::Stopped);
        assert!(!instance.is_running());

        instance.start();
        assert!(instance.is_running());
        assert!(instance.started_at.is_some());

        instance.pause();
        assert_eq!(instance.status, StrategyInstanceStatus::Paused);

        instance.record_error("Test error");
        assert_eq!(instance.status, StrategyInstanceStatus::Error);
        assert_eq!(instance.error_count, 1);

        instance.stop();
        assert_eq!(instance.status, StrategyInstanceStatus::Stopped);
        assert!(instance.stopped_at.is_some());
    }

    #[test]
    fn test_strategy_version_parsing() {
        let version = StrategyVersion::new(
            StrategyId::new(),
            "1.2.3",
            serde_json::json!({}),
            serde_json::json!({}),
            serde_json::json!({}),
            "Initial version",
        );

        assert_eq!(version.version_major, 1);
        assert_eq!(version.version_minor, 2);
        assert_eq!(version.version_patch, 3);
    }
}
