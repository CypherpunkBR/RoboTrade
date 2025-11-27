//! Entidades de Notifications & Alerts
//!
//! Modelos para alertas e notificações.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use super::{OrderId, PositionId, SignalId, StrategyId};

/// ID único de uma definição de alerta
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlertDefinitionId(pub Uuid);

impl AlertDefinitionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for AlertDefinitionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AlertDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo de alerta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// Alerta de preço
    Price,
    /// Alerta de indicador
    Indicator,
    /// Alerta de posição
    Position,
    /// Alerta de risco
    Risk,
    /// Alerta de sistema
    System,
}

impl fmt::Display for AlertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertType::Price => write!(f, "price"),
            AlertType::Indicator => write!(f, "indicator"),
            AlertType::Position => write!(f, "position"),
            AlertType::Risk => write!(f, "risk"),
            AlertType::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for AlertType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "price" => Ok(AlertType::Price),
            "indicator" => Ok(AlertType::Indicator),
            "position" => Ok(AlertType::Position),
            "risk" => Ok(AlertType::Risk),
            "system" => Ok(AlertType::System),
            _ => Err(format!("Tipo de alerta inválido: {}", s)),
        }
    }
}

/// Operador de condição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    /// Maior que
    GreaterThan,
    /// Maior ou igual
    GreaterOrEqual,
    /// Menor que
    LessThan,
    /// Menor ou igual
    LessOrEqual,
    /// Igual
    Equal,
    /// Diferente
    NotEqual,
    /// Cruza para cima
    CrossAbove,
    /// Cruza para baixo
    CrossBelow,
}

impl fmt::Display for ConditionOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionOperator::GreaterThan => write!(f, "gt"),
            ConditionOperator::GreaterOrEqual => write!(f, "gte"),
            ConditionOperator::LessThan => write!(f, "lt"),
            ConditionOperator::LessOrEqual => write!(f, "lte"),
            ConditionOperator::Equal => write!(f, "eq"),
            ConditionOperator::NotEqual => write!(f, "neq"),
            ConditionOperator::CrossAbove => write!(f, "cross_above"),
            ConditionOperator::CrossBelow => write!(f, "cross_below"),
        }
    }
}

/// Condição de alerta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Campo a avaliar
    pub field: String,
    /// Operador
    pub operator: ConditionOperator,
    /// Valor de comparação
    pub value: serde_json::Value,
}

/// Canal de notificação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannel {
    /// Notificação desktop
    Desktop,
    /// Som
    Sound,
    /// Email
    Email,
    /// Telegram
    Telegram,
    /// Webhook
    Webhook,
    /// Push notification
    Push,
}

impl fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationChannel::Desktop => write!(f, "desktop"),
            NotificationChannel::Sound => write!(f, "sound"),
            NotificationChannel::Email => write!(f, "email"),
            NotificationChannel::Telegram => write!(f, "telegram"),
            NotificationChannel::Webhook => write!(f, "webhook"),
            NotificationChannel::Push => write!(f, "push"),
        }
    }
}

impl std::str::FromStr for NotificationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "desktop" => Ok(NotificationChannel::Desktop),
            "sound" => Ok(NotificationChannel::Sound),
            "email" => Ok(NotificationChannel::Email),
            "telegram" => Ok(NotificationChannel::Telegram),
            "webhook" => Ok(NotificationChannel::Webhook),
            "push" => Ok(NotificationChannel::Push),
            _ => Err(format!("Canal de notificação inválido: {}", s)),
        }
    }
}

/// Definição de alerta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDefinition {
    /// ID único
    pub id: AlertDefinitionId,
    /// Nome do alerta
    pub name: String,
    /// Descrição
    pub description: Option<String>,
    /// Tipo do alerta
    pub alert_type: AlertType,
    /// Condições para disparar
    pub conditions: Vec<AlertCondition>,
    /// ID do símbolo (se aplicável)
    pub symbol_id: Option<i64>,
    /// ID da estratégia (se aplicável)
    pub strategy_id: Option<StrategyId>,
    /// Canais de notificação
    pub notification_channels: Vec<NotificationChannel>,
    /// Template da mensagem
    pub message_template: Option<String>,
    /// Habilitado?
    pub is_enabled: bool,
    /// É recorrente? (pode disparar múltiplas vezes)
    pub is_recurring: bool,
    /// Cooldown entre disparos (segundos)
    pub cooldown_seconds: u32,
    /// Último disparo
    pub last_triggered_at: Option<DateTime<Utc>>,
    /// Contador de disparos
    pub trigger_count: u32,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl AlertDefinition {
    /// Cria uma nova definição de alerta
    pub fn new(name: impl Into<String>, alert_type: AlertType) -> Self {
        let now = Utc::now();
        Self {
            id: AlertDefinitionId::new(),
            name: name.into(),
            description: None,
            alert_type,
            conditions: vec![],
            symbol_id: None,
            strategy_id: None,
            notification_channels: vec![NotificationChannel::Desktop],
            message_template: None,
            is_enabled: true,
            is_recurring: false,
            cooldown_seconds: 300, // 5 minutos
            last_triggered_at: None,
            trigger_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Adiciona condição
    pub fn with_condition(mut self, condition: AlertCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Define canais de notificação
    pub fn with_channels(mut self, channels: Vec<NotificationChannel>) -> Self {
        self.notification_channels = channels;
        self
    }

    /// Torna recorrente
    pub fn recurring(mut self, cooldown_seconds: u32) -> Self {
        self.is_recurring = true;
        self.cooldown_seconds = cooldown_seconds;
        self
    }

    /// Verifica se pode disparar (respeitando cooldown)
    pub fn can_trigger(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        if let Some(last) = self.last_triggered_at {
            if !self.is_recurring {
                return false;
            }
            let elapsed = (Utc::now() - last).num_seconds();
            return elapsed >= self.cooldown_seconds as i64;
        }

        true
    }

    /// Registra disparo
    pub fn record_trigger(&mut self) {
        self.last_triggered_at = Some(Utc::now());
        self.trigger_count += 1;
        self.updated_at = Utc::now();
    }
}

/// Severidade da notificação
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    /// Informativo
    Info,
    /// Sucesso
    Success,
    /// Aviso
    Warning,
    /// Erro/Urgente
    Error,
    /// Crítico
    Critical,
}

impl Default for NotificationSeverity {
    fn default() -> Self {
        NotificationSeverity::Info
    }
}

impl fmt::Display for NotificationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationSeverity::Info => write!(f, "info"),
            NotificationSeverity::Success => write!(f, "success"),
            NotificationSeverity::Warning => write!(f, "warning"),
            NotificationSeverity::Error => write!(f, "error"),
            NotificationSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for NotificationSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(NotificationSeverity::Info),
            "success" => Ok(NotificationSeverity::Success),
            "warning" => Ok(NotificationSeverity::Warning),
            "error" => Ok(NotificationSeverity::Error),
            "critical" => Ok(NotificationSeverity::Critical),
            _ => Err(format!("Severidade de notificação inválida: {}", s)),
        }
    }
}

/// Tipo de notificação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Alerta
    Alert,
    /// Sinal de trading
    Signal,
    /// Ordem executada
    Order,
    /// Posição
    Position,
    /// Trade fechado
    Trade,
    /// Risco
    Risk,
    /// Sistema
    System,
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationType::Alert => write!(f, "alert"),
            NotificationType::Signal => write!(f, "signal"),
            NotificationType::Order => write!(f, "order"),
            NotificationType::Position => write!(f, "position"),
            NotificationType::Trade => write!(f, "trade"),
            NotificationType::Risk => write!(f, "risk"),
            NotificationType::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alert" => Ok(NotificationType::Alert),
            "signal" => Ok(NotificationType::Signal),
            "order" => Ok(NotificationType::Order),
            "position" => Ok(NotificationType::Position),
            "trade" => Ok(NotificationType::Trade),
            "risk" => Ok(NotificationType::Risk),
            "system" => Ok(NotificationType::System),
            _ => Err(format!("Tipo de notificação inválido: {}", s)),
        }
    }
}

/// Notificação gerada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// ID único (auto-incrementado)
    pub id: i64,
    /// Tipo da notificação
    pub notification_type: NotificationType,
    /// Severidade
    pub severity: NotificationSeverity,
    /// Título
    pub title: String,
    /// Mensagem
    pub message: String,
    /// ID da definição de alerta (se veio de alerta)
    pub alert_definition_id: Option<AlertDefinitionId>,
    /// ID do sinal relacionado
    pub signal_id: Option<SignalId>,
    /// ID da ordem relacionada
    pub order_id: Option<OrderId>,
    /// ID da posição relacionada
    pub position_id: Option<PositionId>,
    /// Canais para os quais foi enviada
    pub channels_sent: Vec<NotificationChannel>,
    /// Foi lida?
    pub is_read: bool,
    /// Lida em
    pub read_at: Option<DateTime<Utc>>,
    /// Foi descartada?
    pub is_dismissed: bool,
    /// Descartada em
    pub dismissed_at: Option<DateTime<Utc>>,
    /// Metadados adicionais
    pub metadata: Option<serde_json::Value>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
}

impl Notification {
    /// Cria uma nova notificação
    pub fn new(
        notification_type: NotificationType,
        severity: NotificationSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: 0, // será definido pelo banco
            notification_type,
            severity,
            title: title.into(),
            message: message.into(),
            alert_definition_id: None,
            signal_id: None,
            order_id: None,
            position_id: None,
            channels_sent: vec![],
            is_read: false,
            read_at: None,
            is_dismissed: false,
            dismissed_at: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Cria notificação de sinal
    pub fn from_signal(signal_id: SignalId, title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut notification = Self::new(
            NotificationType::Signal,
            NotificationSeverity::Info,
            title,
            message,
        );
        notification.signal_id = Some(signal_id);
        notification
    }

    /// Cria notificação de ordem
    pub fn from_order(order_id: OrderId, title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut notification = Self::new(
            NotificationType::Order,
            NotificationSeverity::Info,
            title,
            message,
        );
        notification.order_id = Some(order_id);
        notification
    }

    /// Cria notificação de risco
    pub fn risk_alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            NotificationType::Risk,
            NotificationSeverity::Critical,
            title,
            message,
        )
    }

    /// Cria notificação de sistema
    pub fn system(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            NotificationType::System,
            NotificationSeverity::Info,
            title,
            message,
        )
    }

    /// Define severidade
    pub fn with_severity(mut self, severity: NotificationSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Registra envio para um canal
    pub fn record_sent(&mut self, channel: NotificationChannel) {
        if !self.channels_sent.contains(&channel) {
            self.channels_sent.push(channel);
        }
    }

    /// Marca como lida
    pub fn mark_read(&mut self) {
        self.is_read = true;
        self.read_at = Some(Utc::now());
    }

    /// Descarta a notificação
    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
        self.dismissed_at = Some(Utc::now());
    }
}

/// Resumo de notificações não lidas
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationSummary {
    /// Total não lidas
    pub total_unread: u32,
    /// Críticas não lidas
    pub critical_unread: u32,
    /// Avisos não lidos
    pub warning_unread: u32,
    /// Informativas não lidas
    pub info_unread: u32,
    /// Última notificação
    pub last_notification_at: Option<DateTime<Utc>>,
}

impl NotificationSummary {
    /// Verifica se há notificações urgentes
    pub fn has_urgent(&self) -> bool {
        self.critical_unread > 0
    }

    /// Total pendente de atenção
    pub fn pending_attention(&self) -> u32 {
        self.critical_unread + self.warning_unread
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_definition_cooldown() {
        let mut alert = AlertDefinition::new("Price Alert", AlertType::Price)
            .recurring(60); // 1 minuto de cooldown

        assert!(alert.can_trigger());

        alert.record_trigger();
        assert!(!alert.can_trigger()); // Dentro do cooldown

        // Simula passagem de tempo
        alert.last_triggered_at = Some(Utc::now() - chrono::Duration::seconds(61));
        assert!(alert.can_trigger());
    }

    #[test]
    fn test_notification_lifecycle() {
        let mut notification = Notification::new(
            NotificationType::Alert,
            NotificationSeverity::Warning,
            "Price Alert",
            "BTC crossed above $50,000",
        );

        assert!(!notification.is_read);
        assert!(!notification.is_dismissed);

        notification.record_sent(NotificationChannel::Desktop);
        assert_eq!(notification.channels_sent.len(), 1);

        notification.mark_read();
        assert!(notification.is_read);
        assert!(notification.read_at.is_some());

        notification.dismiss();
        assert!(notification.is_dismissed);
    }

    #[test]
    fn test_notification_factories() {
        let signal_notif = Notification::from_signal(
            SignalId::new(),
            "New Signal",
            "Long entry signal for BTCUSDT",
        );
        assert_eq!(signal_notif.notification_type, NotificationType::Signal);
        assert!(signal_notif.signal_id.is_some());

        let risk_notif = Notification::risk_alert(
            "Risk Limit Breached",
            "Daily loss limit exceeded",
        );
        assert_eq!(risk_notif.notification_type, NotificationType::Risk);
        assert_eq!(risk_notif.severity, NotificationSeverity::Critical);
    }
}
