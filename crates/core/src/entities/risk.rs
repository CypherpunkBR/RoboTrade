//! Entidades de Risk Management
//!
//! Modelos para limites de risco e eventos de risco.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use super::{AccountId, ExchangeId, OrderId, PositionId, StrategyId};

/// ID único de um limite de risco
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RiskLimitId(pub Uuid);

impl RiskLimitId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for RiskLimitId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RiskLimitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Escopo de aplicação do limite de risco
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLimitScope {
    /// Aplica globalmente
    Global,
    /// Aplica a uma exchange específica
    Exchange(ExchangeId),
    /// Aplica a um símbolo específico
    Symbol(i64),
    /// Aplica a uma estratégia específica
    Strategy(StrategyId),
}

impl fmt::Display for RiskLimitScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLimitScope::Global => write!(f, "global"),
            RiskLimitScope::Exchange(id) => write!(f, "exchange:{}", id),
            RiskLimitScope::Symbol(id) => write!(f, "symbol:{}", id),
            RiskLimitScope::Strategy(id) => write!(f, "strategy:{}", id),
        }
    }
}

/// Limite de risco
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimit {
    /// ID único
    pub id: RiskLimitId,
    /// Nome do limite
    pub name: String,
    /// Descrição
    pub description: Option<String>,
    /// Escopo de aplicação
    pub scope: RiskLimitScope,

    // Limites de posição
    /// Tamanho máximo de posição (quantidade)
    pub max_position_size: Option<Decimal>,
    /// Valor máximo de posição (em USDT)
    pub max_position_value: Option<Decimal>,
    /// Máximo de posições por símbolo
    pub max_positions_per_symbol: Option<u32>,
    /// Máximo de posições totais
    pub max_total_positions: Option<u32>,

    // Limites de exposição
    /// Exposição máxima total
    pub max_total_exposure: Option<Decimal>,
    /// Alavancagem máxima
    pub max_leverage: Option<u32>,

    // Limites de perda
    /// Perda máxima por trade (absoluta)
    pub max_loss_per_trade: Option<Decimal>,
    /// Perda máxima por trade (%)
    pub max_loss_per_trade_pct: Option<Decimal>,
    /// Perda máxima diária (absoluta)
    pub max_daily_loss: Option<Decimal>,
    /// Perda máxima diária (%)
    pub max_daily_loss_pct: Option<Decimal>,
    /// Perda máxima semanal
    pub max_weekly_loss: Option<Decimal>,
    /// Drawdown máximo (%)
    pub max_drawdown_pct: Option<Decimal>,

    // Limites de ordem
    /// Tamanho máximo de ordem
    pub max_order_size: Option<Decimal>,
    /// Máximo de ordens por minuto
    pub max_orders_per_minute: Option<u32>,

    // Circuit breaker
    /// Limite de perdas consecutivas
    pub consecutive_loss_limit: Option<u32>,
    /// Minutos de pausa após atingir o limite
    pub pause_after_losses_minutes: Option<u32>,

    /// Está habilitado?
    pub is_enabled: bool,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl RiskLimit {
    /// Cria um novo limite de risco global
    pub fn new_global(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RiskLimitId::new(),
            name: name.into(),
            description: None,
            scope: RiskLimitScope::Global,
            max_position_size: None,
            max_position_value: None,
            max_positions_per_symbol: None,
            max_total_positions: None,
            max_total_exposure: None,
            max_leverage: None,
            max_loss_per_trade: None,
            max_loss_per_trade_pct: None,
            max_daily_loss: None,
            max_daily_loss_pct: None,
            max_weekly_loss: None,
            max_drawdown_pct: None,
            max_order_size: None,
            max_orders_per_minute: None,
            consecutive_loss_limit: None,
            pause_after_losses_minutes: None,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Define limites de posição
    pub fn with_position_limits(
        mut self,
        max_size: Option<Decimal>,
        max_value: Option<Decimal>,
        max_total: Option<u32>,
    ) -> Self {
        self.max_position_size = max_size;
        self.max_position_value = max_value;
        self.max_total_positions = max_total;
        self
    }

    /// Define limites de perda
    pub fn with_loss_limits(
        mut self,
        per_trade_pct: Option<Decimal>,
        daily_pct: Option<Decimal>,
        weekly: Option<Decimal>,
    ) -> Self {
        self.max_loss_per_trade_pct = per_trade_pct;
        self.max_daily_loss_pct = daily_pct;
        self.max_weekly_loss = weekly;
        self
    }

    /// Define circuit breaker
    pub fn with_circuit_breaker(
        mut self,
        consecutive_losses: u32,
        pause_minutes: u32,
    ) -> Self {
        self.consecutive_loss_limit = Some(consecutive_losses);
        self.pause_after_losses_minutes = Some(pause_minutes);
        self
    }

    /// Verifica se o limite está habilitado
    pub fn is_active(&self) -> bool {
        self.is_enabled
    }
}

/// Tipo de evento de risco
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskEventType {
    /// Limite violado
    LimitBreach,
    /// Circuit breaker ativado
    CircuitBreaker,
    /// Aviso de aproximação de limite
    Warning,
    /// Liquidação iminente
    LiquidationWarning,
    /// Margem baixa
    LowMargin,
}

impl fmt::Display for RiskEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskEventType::LimitBreach => write!(f, "limit_breach"),
            RiskEventType::CircuitBreaker => write!(f, "circuit_breaker"),
            RiskEventType::Warning => write!(f, "warning"),
            RiskEventType::LiquidationWarning => write!(f, "liquidation_warning"),
            RiskEventType::LowMargin => write!(f, "low_margin"),
        }
    }
}

impl std::str::FromStr for RiskEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "limit_breach" => Ok(RiskEventType::LimitBreach),
            "circuit_breaker" => Ok(RiskEventType::CircuitBreaker),
            "warning" => Ok(RiskEventType::Warning),
            "liquidation_warning" => Ok(RiskEventType::LiquidationWarning),
            "low_margin" => Ok(RiskEventType::LowMargin),
            _ => Err(format!("Tipo de evento de risco inválido: {}", s)),
        }
    }
}

/// Severidade do evento de risco
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    /// Informativo
    Info,
    /// Aviso
    Warning,
    /// Crítico
    Critical,
}

impl fmt::Display for RiskSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskSeverity::Info => write!(f, "info"),
            RiskSeverity::Warning => write!(f, "warning"),
            RiskSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for RiskSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(RiskSeverity::Info),
            "warning" => Ok(RiskSeverity::Warning),
            "critical" => Ok(RiskSeverity::Critical),
            _ => Err(format!("Severidade de risco inválida: {}", s)),
        }
    }
}

/// Ação tomada em resposta a um evento de risco
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskAction {
    /// Nenhuma ação (apenas log)
    None,
    /// Ordem rejeitada
    OrderRejected,
    /// Posição fechada
    PositionClosed,
    /// Trading pausado
    TradingPaused,
    /// Notificação enviada
    NotificationSent,
    /// Alavancagem reduzida
    LeverageReduced,
}

impl fmt::Display for RiskAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskAction::None => write!(f, "none"),
            RiskAction::OrderRejected => write!(f, "order_rejected"),
            RiskAction::PositionClosed => write!(f, "position_closed"),
            RiskAction::TradingPaused => write!(f, "trading_paused"),
            RiskAction::NotificationSent => write!(f, "notification_sent"),
            RiskAction::LeverageReduced => write!(f, "leverage_reduced"),
        }
    }
}

/// Evento de risco registrado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvent {
    /// ID único (auto-incrementado)
    pub id: i64,
    /// Tipo do evento
    pub event_type: RiskEventType,
    /// Severidade
    pub severity: RiskSeverity,
    /// ID do limite de risco relacionado
    pub risk_limit_id: Option<RiskLimitId>,
    /// ID da conta relacionada
    pub account_id: Option<AccountId>,
    /// ID do símbolo relacionado
    pub symbol_id: Option<i64>,
    /// ID da posição relacionada
    pub position_id: Option<PositionId>,
    /// ID da ordem relacionada
    pub order_id: Option<OrderId>,
    /// Nome do limite violado
    pub limit_name: Option<String>,
    /// Valor do limite
    pub limit_value: Option<Decimal>,
    /// Valor atual
    pub actual_value: Option<Decimal>,
    /// Percentual de violação
    pub breach_pct: Option<Decimal>,
    /// Ação tomada
    pub action_taken: RiskAction,
    /// Resolvido?
    pub resolved: bool,
    /// Data de resolução
    pub resolved_at: Option<DateTime<Utc>>,
    /// Notas de resolução
    pub resolution_notes: Option<String>,
    /// Metadados adicionais
    pub metadata: Option<serde_json::Value>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
}

impl RiskEvent {
    /// Cria um novo evento de risco
    pub fn new(event_type: RiskEventType, severity: RiskSeverity) -> Self {
        Self {
            id: 0,
            event_type,
            severity,
            risk_limit_id: None,
            account_id: None,
            symbol_id: None,
            position_id: None,
            order_id: None,
            limit_name: None,
            limit_value: None,
            actual_value: None,
            breach_pct: None,
            action_taken: RiskAction::None,
            resolved: false,
            resolved_at: None,
            resolution_notes: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Cria evento de violação de limite
    pub fn limit_breach(
        limit_id: RiskLimitId,
        limit_name: impl Into<String>,
        limit_value: Decimal,
        actual_value: Decimal,
    ) -> Self {
        let breach_pct = if limit_value != Decimal::ZERO {
            ((actual_value - limit_value).abs() / limit_value) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Self {
            id: 0,
            event_type: RiskEventType::LimitBreach,
            severity: RiskSeverity::Critical,
            risk_limit_id: Some(limit_id),
            account_id: None,
            symbol_id: None,
            position_id: None,
            order_id: None,
            limit_name: Some(limit_name.into()),
            limit_value: Some(limit_value),
            actual_value: Some(actual_value),
            breach_pct: Some(breach_pct),
            action_taken: RiskAction::None,
            resolved: false,
            resolved_at: None,
            resolution_notes: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Define ação tomada
    pub fn with_action(mut self, action: RiskAction) -> Self {
        self.action_taken = action;
        self
    }

    /// Associa a uma posição
    pub fn for_position(mut self, position_id: PositionId) -> Self {
        self.position_id = Some(position_id);
        self
    }

    /// Associa a uma ordem
    pub fn for_order(mut self, order_id: OrderId) -> Self {
        self.order_id = Some(order_id);
        self
    }

    /// Marca como resolvido
    pub fn resolve(&mut self, notes: Option<String>) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = notes;
    }

    /// Verifica se é crítico
    pub fn is_critical(&self) -> bool {
        self.severity == RiskSeverity::Critical
    }
}

/// Status de risco atual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatus {
    /// Exposição total atual
    pub total_exposure: Decimal,
    /// Limite de exposição
    pub exposure_limit: Option<Decimal>,
    /// Exposição utilizada (%)
    pub exposure_used_pct: Decimal,
    /// Perda diária atual
    pub daily_loss: Decimal,
    /// Limite de perda diária
    pub daily_loss_limit: Option<Decimal>,
    /// Perda diária utilizada (%)
    pub daily_loss_used_pct: Decimal,
    /// Posições abertas
    pub open_positions: u32,
    /// Limite de posições
    pub position_limit: Option<u32>,
    /// Perdas consecutivas atuais
    pub consecutive_losses: u32,
    /// Circuit breaker ativo?
    pub circuit_breaker_active: bool,
    /// Tempo até liberar circuit breaker
    pub circuit_breaker_expires_at: Option<DateTime<Utc>>,
    /// Eventos de risco não resolvidos
    pub unresolved_events: u32,
    /// Última atualização
    pub last_updated: DateTime<Utc>,
}

impl Default for RiskStatus {
    fn default() -> Self {
        Self {
            total_exposure: Decimal::ZERO,
            exposure_limit: None,
            exposure_used_pct: Decimal::ZERO,
            daily_loss: Decimal::ZERO,
            daily_loss_limit: None,
            daily_loss_used_pct: Decimal::ZERO,
            open_positions: 0,
            position_limit: None,
            consecutive_losses: 0,
            circuit_breaker_active: false,
            circuit_breaker_expires_at: None,
            unresolved_events: 0,
            last_updated: Utc::now(),
        }
    }
}

impl RiskStatus {
    /// Verifica se pode abrir novas posições
    pub fn can_open_position(&self) -> bool {
        if self.circuit_breaker_active {
            return false;
        }

        if let Some(limit) = self.position_limit {
            if self.open_positions >= limit {
                return false;
            }
        }

        true
    }

    /// Verifica se está em zona de risco
    pub fn is_at_risk(&self) -> bool {
        self.exposure_used_pct > Decimal::from(80)
            || self.daily_loss_used_pct > Decimal::from(80)
            || self.circuit_breaker_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_risk_limit_creation() {
        let limit = RiskLimit::new_global("Default Risk Limits")
            .with_position_limits(Some(dec!(1.0)), Some(dec!(50000)), Some(5))
            .with_loss_limits(Some(dec!(2)), Some(dec!(5)), Some(dec!(1000)))
            .with_circuit_breaker(3, 30);

        assert!(limit.is_active());
        assert_eq!(limit.max_position_size, Some(dec!(1.0)));
        assert_eq!(limit.consecutive_loss_limit, Some(3));
    }

    #[test]
    fn test_risk_event_limit_breach() {
        let event = RiskEvent::limit_breach(
            RiskLimitId::new(),
            "Max Daily Loss",
            dec!(1000),
            dec!(1200),
        )
        .with_action(RiskAction::TradingPaused);

        assert_eq!(event.event_type, RiskEventType::LimitBreach);
        assert_eq!(event.severity, RiskSeverity::Critical);
        assert_eq!(event.breach_pct, Some(dec!(20))); // 20% over limit
        assert_eq!(event.action_taken, RiskAction::TradingPaused);
    }

    #[test]
    fn test_risk_status() {
        let mut status = RiskStatus::default();
        status.open_positions = 3;
        status.position_limit = Some(5);

        assert!(status.can_open_position());

        status.open_positions = 5;
        assert!(!status.can_open_position());

        status.open_positions = 3;
        status.circuit_breaker_active = true;
        assert!(!status.can_open_position());
    }
}
