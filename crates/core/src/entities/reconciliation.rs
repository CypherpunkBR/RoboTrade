//! Entidades relacionadas à Reconciliação
//!
//! Comparação de saldos calculados vs reportados pela exchange.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// ID único de um snapshot de reconciliação
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReconciliationSnapshotId(pub String);

impl ReconciliationSnapshotId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl Default for ReconciliationSnapshotId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ReconciliationSnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ReconciliationSnapshotId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Tipo de snapshot de reconciliação
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SnapshotType {
    /// Reconciliação diária automática
    Daily,
    /// Reconciliação manual
    Manual,
    /// Após sincronização de dados
    PostSync,
    /// Antes de executar trade
    PreTrade,
    /// Agendada
    Scheduled,
}

impl fmt::Display for SnapshotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotType::Daily => write!(f, "DAILY"),
            SnapshotType::Manual => write!(f, "MANUAL"),
            SnapshotType::PostSync => write!(f, "POST_SYNC"),
            SnapshotType::PreTrade => write!(f, "PRE_TRADE"),
            SnapshotType::Scheduled => write!(f, "SCHEDULED"),
        }
    }
}

impl std::str::FromStr for SnapshotType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DAILY" => Ok(SnapshotType::Daily),
            "MANUAL" => Ok(SnapshotType::Manual),
            "POST_SYNC" => Ok(SnapshotType::PostSync),
            "PRE_TRADE" => Ok(SnapshotType::PreTrade),
            "SCHEDULED" => Ok(SnapshotType::Scheduled),
            _ => Err(format!("Unknown snapshot type: {}", s)),
        }
    }
}

/// Status da reconciliação
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationStatus {
    /// Saldos conferem
    Matched,
    /// Discrepância encontrada
    Discrepancy,
    /// Aguardando revisão
    PendingReview,
    /// Resolvido manualmente
    Resolved,
    /// Ignorado (discrepância conhecida)
    Ignored,
}

impl fmt::Display for ReconciliationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReconciliationStatus::Matched => write!(f, "MATCHED"),
            ReconciliationStatus::Discrepancy => write!(f, "DISCREPANCY"),
            ReconciliationStatus::PendingReview => write!(f, "PENDING_REVIEW"),
            ReconciliationStatus::Resolved => write!(f, "RESOLVED"),
            ReconciliationStatus::Ignored => write!(f, "IGNORED"),
        }
    }
}

impl std::str::FromStr for ReconciliationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MATCHED" => Ok(ReconciliationStatus::Matched),
            "DISCREPANCY" => Ok(ReconciliationStatus::Discrepancy),
            "PENDING_REVIEW" => Ok(ReconciliationStatus::PendingReview),
            "RESOLVED" => Ok(ReconciliationStatus::Resolved),
            "IGNORED" => Ok(ReconciliationStatus::Ignored),
            _ => Err(format!("Unknown reconciliation status: {}", s)),
        }
    }
}

/// Discrepância encontrada na reconciliação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    /// Ativo com discrepância
    pub asset: String,
    /// Saldo calculado (do ledger)
    pub calculated: Decimal,
    /// Saldo reportado (da exchange)
    pub reported: Decimal,
    /// Diferença absoluta
    pub difference: Decimal,
    /// Diferença percentual
    pub difference_pct: Decimal,
}

impl Discrepancy {
    /// Cria uma nova discrepância
    pub fn new(asset: String, calculated: Decimal, reported: Decimal) -> Self {
        let difference = (calculated - reported).abs();
        let difference_pct = if reported.is_zero() {
            if calculated.is_zero() {
                Decimal::ZERO
            } else {
                Decimal::from(100)
            }
        } else {
            (difference / reported.abs()) * Decimal::from(100)
        };

        Self {
            asset,
            calculated,
            reported,
            difference,
            difference_pct,
        }
    }

    /// Verifica se a discrepância está dentro de um threshold
    pub fn is_within_threshold(&self, threshold: Decimal) -> bool {
        self.difference <= threshold
    }

    /// Verifica se o saldo calculado é maior que o reportado
    pub fn calculated_higher(&self) -> bool {
        self.calculated > self.reported
    }
}

/// Snapshot de reconciliação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSnapshot {
    /// ID único do snapshot
    pub id: ReconciliationSnapshotId,
    /// ID da exchange
    pub exchange_id: String,
    /// Tipo de snapshot
    pub snapshot_type: SnapshotType,
    /// Saldos calculados do ledger
    pub calculated_balances: HashMap<String, Decimal>,
    /// Saldos reportados pela exchange
    pub reported_balances: HashMap<String, Decimal>,
    /// Discrepâncias encontradas
    pub discrepancies: Vec<Discrepancy>,
    /// Status da reconciliação
    pub status: ReconciliationStatus,
    /// Quando foi revisado
    pub reviewed_at: Option<DateTime<Utc>>,
    /// Quem revisou
    pub reviewed_by: Option<String>,
    /// Notas de resolução
    pub resolution_notes: Option<String>,
    /// Quando os saldos foram capturados
    pub snapshot_timestamp: DateTime<Utc>,
    /// Criado em
    pub created_at: DateTime<Utc>,
}

impl ReconciliationSnapshot {
    /// Cria um novo snapshot de reconciliação
    pub fn new(
        exchange_id: String,
        snapshot_type: SnapshotType,
        calculated_balances: HashMap<String, Decimal>,
        reported_balances: HashMap<String, Decimal>,
        threshold: Decimal,
    ) -> Self {
        let now = Utc::now();
        let discrepancies = Self::find_discrepancies(&calculated_balances, &reported_balances, threshold);
        let status = if discrepancies.is_empty() {
            ReconciliationStatus::Matched
        } else {
            ReconciliationStatus::Discrepancy
        };

        Self {
            id: ReconciliationSnapshotId::new(),
            exchange_id,
            snapshot_type,
            calculated_balances,
            reported_balances,
            discrepancies,
            status,
            reviewed_at: None,
            reviewed_by: None,
            resolution_notes: None,
            snapshot_timestamp: now,
            created_at: now,
        }
    }

    /// Encontra discrepâncias entre saldos calculados e reportados
    fn find_discrepancies(
        calculated: &HashMap<String, Decimal>,
        reported: &HashMap<String, Decimal>,
        threshold: Decimal,
    ) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();

        // Verifica ativos no saldo calculado
        for (asset, &calc_amount) in calculated {
            let rep_amount = reported.get(asset).copied().unwrap_or(Decimal::ZERO);
            let diff = (calc_amount - rep_amount).abs();

            if diff > threshold {
                discrepancies.push(Discrepancy::new(
                    asset.clone(),
                    calc_amount,
                    rep_amount,
                ));
            }
        }

        // Verifica ativos que só existem no reportado
        for (asset, &rep_amount) in reported {
            if !calculated.contains_key(asset) && rep_amount.abs() > threshold {
                discrepancies.push(Discrepancy::new(
                    asset.clone(),
                    Decimal::ZERO,
                    rep_amount,
                ));
            }
        }

        discrepancies
    }

    /// Marca como revisado
    pub fn mark_reviewed(&mut self, reviewed_by: String, notes: Option<String>) {
        self.reviewed_at = Some(Utc::now());
        self.reviewed_by = Some(reviewed_by);
        self.resolution_notes = notes;
        self.status = ReconciliationStatus::Resolved;
    }

    /// Marca como ignorado
    pub fn mark_ignored(&mut self, reason: String) {
        self.reviewed_at = Some(Utc::now());
        self.resolution_notes = Some(reason);
        self.status = ReconciliationStatus::Ignored;
    }

    /// Verifica se há discrepâncias
    pub fn has_discrepancies(&self) -> bool {
        !self.discrepancies.is_empty()
    }

    /// Retorna a maior discrepância absoluta
    pub fn max_discrepancy(&self) -> Option<&Discrepancy> {
        self.discrepancies.iter().max_by(|a, b| {
            a.difference.partial_cmp(&b.difference).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Total de diferenças em valor absoluto
    pub fn total_discrepancy_value(&self) -> Decimal {
        self.discrepancies.iter().map(|d| d.difference).sum()
    }
}

/// Resultado de uma reconciliação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationResult {
    /// Snapshot gerado
    pub snapshot: ReconciliationSnapshot,
    /// Se passou na reconciliação
    pub success: bool,
    /// Mensagem de status
    pub message: String,
}

impl ReconciliationResult {
    /// Cria resultado de sucesso
    pub fn success(snapshot: ReconciliationSnapshot) -> Self {
        Self {
            success: true,
            message: "Reconciliation completed - all balances match".to_string(),
            snapshot,
        }
    }

    /// Cria resultado com discrepâncias
    pub fn with_discrepancies(snapshot: ReconciliationSnapshot) -> Self {
        let count = snapshot.discrepancies.len();
        Self {
            success: false,
            message: format!("Reconciliation completed - {} discrepancies found", count),
            snapshot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_discrepancy_creation() {
        let disc = Discrepancy::new("USDT".to_string(), dec!(1000), dec!(999));

        assert_eq!(disc.difference, dec!(1));
        assert!(disc.calculated_higher());
    }

    #[test]
    fn test_reconciliation_snapshot_matched() {
        let mut calculated = HashMap::new();
        calculated.insert("USDT".to_string(), dec!(1000));
        calculated.insert("BTC".to_string(), dec!(0.5));

        let mut reported = HashMap::new();
        reported.insert("USDT".to_string(), dec!(1000));
        reported.insert("BTC".to_string(), dec!(0.5));

        let snapshot = ReconciliationSnapshot::new(
            "binance".to_string(),
            SnapshotType::Manual,
            calculated,
            reported,
            dec!(0.00000001), // 1 satoshi threshold
        );

        assert_eq!(snapshot.status, ReconciliationStatus::Matched);
        assert!(!snapshot.has_discrepancies());
    }

    #[test]
    fn test_reconciliation_snapshot_discrepancy() {
        let mut calculated = HashMap::new();
        calculated.insert("USDT".to_string(), dec!(1000));

        let mut reported = HashMap::new();
        reported.insert("USDT".to_string(), dec!(990));

        let snapshot = ReconciliationSnapshot::new(
            "binance".to_string(),
            SnapshotType::Manual,
            calculated,
            reported,
            dec!(0.00000001),
        );

        assert_eq!(snapshot.status, ReconciliationStatus::Discrepancy);
        assert!(snapshot.has_discrepancies());
        assert_eq!(snapshot.discrepancies.len(), 1);
        assert_eq!(snapshot.discrepancies[0].difference, dec!(10));
    }

    #[test]
    fn test_reconciliation_mark_reviewed() {
        let mut snapshot = ReconciliationSnapshot::new(
            "binance".to_string(),
            SnapshotType::Manual,
            HashMap::new(),
            HashMap::new(),
            dec!(0.00000001),
        );

        snapshot.mark_reviewed("admin".to_string(), Some("All good".to_string()));

        assert_eq!(snapshot.status, ReconciliationStatus::Resolved);
        assert_eq!(snapshot.reviewed_by, Some("admin".to_string()));
    }
}
