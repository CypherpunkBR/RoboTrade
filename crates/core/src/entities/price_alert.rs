//! Entidade para alertas de preco. @todo Deveria ter um modulo pra isso. Cade as boas praticas ?

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ID unico de um alerta de preco
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceAlertId(Uuid);

impl PriceAlertId {
    /// Cria um novo ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Retorna o UUID interno
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for PriceAlertId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PriceAlertId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo de condicao do alerta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    /// Preco acima de
    Above,
    /// Preco abaixo de
    Below,
    /// Preco cruza para cima
    CrossAbove,
    /// Preco cruza para baixo
    CrossBelow,
    /// Variacao percentual positiva
    PercentUp,
    /// Variacao percentual negativa
    PercentDown,
}

impl AlertCondition {
    /// Descricao em portugues
    pub fn description_pt(&self) -> &'static str {
        match self {
            AlertCondition::Above => "Preco acima de",
            AlertCondition::Below => "Preco abaixo de",
            AlertCondition::CrossAbove => "Preco cruza para cima de",
            AlertCondition::CrossBelow => "Preco cruza para baixo de",
            AlertCondition::PercentUp => "Variacao positiva de",
            AlertCondition::PercentDown => "Variacao negativa de",
        }
    }
}

/// Status do alerta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    /// Alerta ativo
    Active,
    /// Alerta disparado
    Triggered,
    /// Alerta desabilitado
    Disabled,
    /// Alerta expirado
    Expired,
}

/// Tipo de notificacao
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Notificacao do sistema
    System,
    /// Som
    Sound,
    /// Email (futuro)
    Email,
    /// Webhook (futuro)
    Webhook,
}

/// Alerta de preco
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    /// ID do alerta
    pub id: PriceAlertId,
    /// Simbolo do par (ex: BTCUSDT)
    pub symbol: String,
    /// Condicao do alerta
    pub condition: AlertCondition,
    /// Preco alvo
    pub target_price: Decimal,
    /// Percentual (para alertas de variacao)
    pub percent: Option<Decimal>,
    /// Preco de referencia (para calcular percentual)
    pub reference_price: Option<Decimal>,
    /// Status do alerta
    pub status: AlertStatus,
    /// Tipo de notificacao
    pub notification_type: NotificationType,
    /// Mensagem customizada
    pub message: Option<String>,
    /// Se o alerta e recorrente (pode disparar multiplas vezes)
    pub recurring: bool,
    /// Quantidade de vezes que ja disparou
    pub trigger_count: u32,
    /// Expiracao do alerta (opcional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Criado em
    pub created_at: DateTime<Utc>,
    /// Ultimo disparo
    pub triggered_at: Option<DateTime<Utc>>,
    /// Ultimo preco que verificou
    pub last_checked_price: Option<Decimal>,
}

impl PriceAlert {
    /// Cria um novo alerta de preco simples
    pub fn new(symbol: impl Into<String>, condition: AlertCondition, target_price: Decimal) -> Self {
        Self {
            id: PriceAlertId::new(),
            symbol: symbol.into(),
            condition,
            target_price,
            percent: None,
            reference_price: None,
            status: AlertStatus::Active,
            notification_type: NotificationType::System,
            message: None,
            recurring: false,
            trigger_count: 0,
            expires_at: None,
            created_at: Utc::now(),
            triggered_at: None,
            last_checked_price: None,
        }
    }

    /// Cria alerta de variacao percentual
    pub fn percent_change(
        symbol: impl Into<String>,
        is_up: bool,
        percent: Decimal,
        reference_price: Decimal,
    ) -> Self {
        let condition = if is_up {
            AlertCondition::PercentUp
        } else {
            AlertCondition::PercentDown
        };

        let target = if is_up {
            reference_price * (Decimal::ONE + percent / Decimal::from(100))
        } else {
            reference_price * (Decimal::ONE - percent / Decimal::from(100))
        };

        Self {
            id: PriceAlertId::new(),
            symbol: symbol.into(),
            condition,
            target_price: target,
            percent: Some(percent),
            reference_price: Some(reference_price),
            status: AlertStatus::Active,
            notification_type: NotificationType::System,
            message: None,
            recurring: false,
            trigger_count: 0,
            expires_at: None,
            created_at: Utc::now(),
            triggered_at: None,
            last_checked_price: None,
        }
    }

    /// Verifica se o alerta deve disparar dado o preco atual
    pub fn should_trigger(&self, current_price: Decimal, previous_price: Option<Decimal>) -> bool {
        if self.status != AlertStatus::Active {
            return false;
        }

        // Verifica expiracao
        if let Some(expires) = self.expires_at {
            if Utc::now() > expires {
                return false;
            }
        }

        match self.condition {
            AlertCondition::Above => current_price >= self.target_price,
            AlertCondition::Below => current_price <= self.target_price,
            AlertCondition::CrossAbove => {
                if let Some(prev) = previous_price {
                    prev < self.target_price && current_price >= self.target_price
                } else {
                    false
                }
            }
            AlertCondition::CrossBelow => {
                if let Some(prev) = previous_price {
                    prev > self.target_price && current_price <= self.target_price
                } else {
                    false
                }
            }
            AlertCondition::PercentUp | AlertCondition::PercentDown => {
                current_price >= self.target_price
            }
        }
    }

    /// Dispara o alerta
    pub fn trigger(&mut self) {
        self.triggered_at = Some(Utc::now());
        self.trigger_count += 1;

        if !self.recurring {
            self.status = AlertStatus::Triggered;
        }
    }

    /// Desabilita o alerta
    pub fn disable(&mut self) {
        self.status = AlertStatus::Disabled;
    }

    /// Reativa o alerta
    pub fn reactivate(&mut self) {
        self.status = AlertStatus::Active;
    }

    /// Atualiza o ultimo preco verificado
    pub fn update_last_price(&mut self, price: Decimal) {
        self.last_checked_price = Some(price);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_alert_above() {
        let alert = PriceAlert::new("BTCUSDT", AlertCondition::Above, dec!(50000));
        assert!(alert.should_trigger(dec!(50001), None));
        assert!(alert.should_trigger(dec!(50000), None));
        assert!(!alert.should_trigger(dec!(49999), None));
    }

    #[test]
    fn test_alert_below() {
        let alert = PriceAlert::new("BTCUSDT", AlertCondition::Below, dec!(50000));
        assert!(alert.should_trigger(dec!(49999), None));
        assert!(alert.should_trigger(dec!(50000), None));
        assert!(!alert.should_trigger(dec!(50001), None));
    }

    #[test]
    fn test_alert_cross_above() {
        let alert = PriceAlert::new("BTCUSDT", AlertCondition::CrossAbove, dec!(50000));
        // Cruza de baixo para cima
        assert!(alert.should_trigger(dec!(50001), Some(dec!(49999))));
        // Ja estava acima
        assert!(!alert.should_trigger(dec!(50001), Some(dec!(50000))));
        // Sem preco anterior
        assert!(!alert.should_trigger(dec!(50001), None));
    }

    #[test]
    fn test_alert_percent_up() {
        let alert = PriceAlert::percent_change("BTCUSDT", true, dec!(5), dec!(50000));
        // 5% acima de 50000 = 52500
        assert!(alert.should_trigger(dec!(52500), None));
        assert!(alert.should_trigger(dec!(53000), None));
        assert!(!alert.should_trigger(dec!(52000), None));
    }

    #[test]
    fn test_alert_trigger() {
        let mut alert = PriceAlert::new("BTCUSDT", AlertCondition::Above, dec!(50000));
        assert_eq!(alert.status, AlertStatus::Active);

        alert.trigger();
        assert_eq!(alert.status, AlertStatus::Triggered);
        assert_eq!(alert.trigger_count, 1);
        assert!(alert.triggered_at.is_some());
    }

    #[test]
    fn test_alert_recurring() {
        let mut alert = PriceAlert::new("BTCUSDT", AlertCondition::Above, dec!(50000));
        alert.recurring = true;

        alert.trigger();
        assert_eq!(alert.status, AlertStatus::Active); // Continua ativo
        assert_eq!(alert.trigger_count, 1);
    }
}
