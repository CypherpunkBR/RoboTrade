//! Entidade Signal (Sinal de trading)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ID único de um sinal
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignalId(pub Uuid);

impl SignalId {
    /// Cria um novo ID de sinal
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SignalId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Direção do sinal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeDirection {
    /// Compra (aposta na alta)
    Long,
    /// Venda (aposta na baixa)
    Short,
}

impl std::fmt::Display for TradeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeDirection::Long => write!(f, "long"),
            TradeDirection::Short => write!(f, "short"),
        }
    }
}

/// Força do sinal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignalStrength {
    /// Sinal fraco
    Weak,
    /// Sinal moderado
    Moderate,
    /// Sinal forte
    Strong,
}

impl SignalStrength {
    /// Converte para valor numérico (0-100)
    pub fn as_score(&self) -> u8 {
        match self {
            SignalStrength::Weak => 33,
            SignalStrength::Moderate => 66,
            SignalStrength::Strong => 100,
        }
    }
}

/// Status do sinal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStatus {
    /// Sinal ativo, aguardando execução
    Active,
    /// Aguardando confirmação manual
    AwaitingConfirmation,
    /// Confirmado pelo usuário
    Confirmed,
    /// Executado (ordem enviada)
    Executed,
    /// Expirado (não foi executado a tempo)
    Expired,
    /// Cancelado
    Cancelled,
}

/// Tipo de sinal (origem/indicador)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalType {
    /// Cruzamento de médias móveis
    MACrossover { fast_period: u32, slow_period: u32 },
    /// RSI em zona de sobrevenda/sobrecompra
    RSI { period: u32, threshold: Decimal },
    /// Rompimento de Bollinger Bands
    BollingerBreakout { period: u32, std_dev: Decimal },
    /// Baseado no Fear & Greed Index
    FearGreed { index_value: u8, threshold: u8 },
    /// Sinal composto (múltiplos indicadores)
    Composite {
        rule_id: String,
        components: Vec<String>,
    },
    /// Sinal manual/customizado
    Custom { name: String, description: String },
}

/// Sinal de trading gerado por uma estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// ID único do sinal
    pub id: SignalId,
    /// ID da estratégia que gerou o sinal
    pub strategy_id: String,
    /// Nome da estratégia
    pub strategy_name: String,
    /// Símbolo do par
    pub symbol: String,
    /// Tipo do sinal
    pub signal_type: SignalType,
    /// Direção (long/short)
    pub direction: TradeDirection,
    /// Força do sinal
    pub strength: SignalStrength,
    /// Preço no momento do sinal
    pub trigger_price: Decimal,
    /// Preço sugerido de entrada
    pub suggested_entry: Option<Decimal>,
    /// Stop loss sugerido
    pub suggested_stop_loss: Option<Decimal>,
    /// Take profit sugerido
    pub suggested_take_profit: Option<Decimal>,
    /// Risco/Recompensa calculado
    pub risk_reward_ratio: Option<Decimal>,
    /// Motivo/descrição do sinal
    pub reason: String,
    /// Confiança do sinal (0-100)
    pub confidence: u8,
    /// Status atual
    pub status: SignalStatus,
    /// Requer confirmação manual?
    pub requires_confirmation: bool,
    /// Timestamp de geração
    pub generated_at: DateTime<Utc>,
    /// Timestamp de expiração
    pub expires_at: DateTime<Utc>,
    /// Timestamp de confirmação (se confirmado)
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Timestamp de execução (se executado)
    pub executed_at: Option<DateTime<Utc>>,
    /// Metadados adicionais (JSON)
    pub metadata: Option<serde_json::Value>,
}

impl Signal {
    /// Cria um novo sinal
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strategy_id: impl Into<String>,
        strategy_name: impl Into<String>,
        symbol: impl Into<String>,
        signal_type: SignalType,
        direction: TradeDirection,
        strength: SignalStrength,
        trigger_price: Decimal,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: SignalId::new(),
            strategy_id: strategy_id.into(),
            strategy_name: strategy_name.into(),
            symbol: symbol.into(),
            signal_type,
            direction,
            strength,
            trigger_price,
            suggested_entry: None,
            suggested_stop_loss: None,
            suggested_take_profit: None,
            risk_reward_ratio: None,
            reason: reason.into(),
            confidence: strength.as_score(),
            status: SignalStatus::Active,
            requires_confirmation: false,
            generated_at: now,
            expires_at: now + chrono::Duration::hours(1), // Default: 1 hora
            confirmed_at: None,
            executed_at: None,
            metadata: None,
        }
    }

    /// Define entrada, stop loss e take profit
    pub fn with_targets(
        mut self,
        entry: Decimal,
        stop_loss: Decimal,
        take_profit: Decimal,
    ) -> Self {
        self.suggested_entry = Some(entry);
        self.suggested_stop_loss = Some(stop_loss);
        self.suggested_take_profit = Some(take_profit);

        // Calcula risk/reward
        let risk = (entry - stop_loss).abs();
        let reward = (take_profit - entry).abs();

        if risk > Decimal::ZERO {
            self.risk_reward_ratio = Some(reward / risk);
        }

        self
    }

    /// Define que requer confirmação manual
    pub fn requires_manual_confirmation(mut self) -> Self {
        self.requires_confirmation = true;
        self.status = SignalStatus::AwaitingConfirmation;
        self
    }

    /// Define tempo de expiração customizado
    pub fn with_expiration(mut self, duration: chrono::Duration) -> Self {
        self.expires_at = self.generated_at + duration;
        self
    }

    /// Verifica se o sinal expirou
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Verifica se o sinal pode ser executado
    pub fn can_execute(&self) -> bool {
        match self.status {
            SignalStatus::Active => !self.requires_confirmation && !self.is_expired(),
            SignalStatus::Confirmed => !self.is_expired(),
            _ => false,
        }
    }

    /// Marca como confirmado
    pub fn confirm(&mut self) {
        self.status = SignalStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
    }

    /// Marca como executado
    pub fn mark_executed(&mut self) {
        self.status = SignalStatus::Executed;
        self.executed_at = Some(Utc::now());
    }

    /// Marca como expirado
    pub fn mark_expired(&mut self) {
        self.status = SignalStatus::Expired;
    }

    /// Marca como cancelado
    pub fn cancel(&mut self) {
        self.status = SignalStatus::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_signal_creation() {
        let signal = Signal::new(
            "fear_greed_v1",
            "Fear & Greed Strategy",
            "BTCUSDT",
            SignalType::FearGreed {
                index_value: 20,
                threshold: 25,
            },
            TradeDirection::Long,
            SignalStrength::Strong,
            dec!(42000),
            "Medo extremo detectado",
        );

        assert_eq!(signal.direction, TradeDirection::Long);
        assert_eq!(signal.strength, SignalStrength::Strong);
        assert_eq!(signal.status, SignalStatus::Active);
        assert!(!signal.requires_confirmation);
    }

    #[test]
    fn test_signal_with_targets() {
        let signal = Signal::new(
            "test",
            "Test",
            "BTCUSDT",
            SignalType::Custom {
                name: "test".into(),
                description: "test".into(),
            },
            TradeDirection::Long,
            SignalStrength::Strong,
            dec!(40000),
            "Test",
        )
        .with_targets(dec!(40000), dec!(38000), dec!(44000));

        assert_eq!(signal.suggested_entry, Some(dec!(40000)));
        assert_eq!(signal.suggested_stop_loss, Some(dec!(38000)));
        assert_eq!(signal.suggested_take_profit, Some(dec!(44000)));
        // Risk: 2000, Reward: 4000 = R:R 2.0
        assert_eq!(signal.risk_reward_ratio, Some(dec!(2)));
    }

    #[test]
    fn test_signal_expiration() {
        let mut signal = Signal::new(
            "test",
            "Test",
            "BTCUSDT",
            SignalType::Custom {
                name: "test".into(),
                description: "test".into(),
            },
            TradeDirection::Long,
            SignalStrength::Strong,
            dec!(40000),
            "Test",
        );

        // Define expiração no passado
        signal.expires_at = Utc::now() - chrono::Duration::hours(1);

        assert!(signal.is_expired());
        assert!(!signal.can_execute());
    }
}
