//! Reconciliation Service
//!
//! Compares ledger balances with exchange-reported balances to detect discrepancies.
//!
//! Features:
//! - Automatic balance comparison
//! - Discrepancy detection and classification
//! - Snapshot creation for audit trail
//! - Resolution tracking

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::ws_manager::Exchange;

/// Reconciliation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationStatus {
    /// All balances match
    Matched,
    /// Discrepancies found
    Discrepancies,
    /// Reconciliation in progress
    InProgress,
    /// Reconciliation failed
    Failed,
}

/// Type of discrepancy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscrepancyType {
    /// Ledger balance is higher than exchange
    LedgerHigher,
    /// Exchange balance is higher than ledger
    ExchangeHigher,
    /// Asset exists in ledger but not on exchange
    MissingOnExchange,
    /// Asset exists on exchange but not in ledger
    MissingInLedger,
}

/// A discrepancy between ledger and exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub asset: String,
    pub discrepancy_type: DiscrepancyType,
    pub ledger_balance: Decimal,
    pub exchange_balance: Decimal,
    pub difference: Decimal,
    pub difference_pct: Decimal,
}

/// Balance snapshot from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub asset: String,
    pub balance: Decimal,
}

/// Reconciliation snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSnapshot {
    pub id: String,
    pub exchange: Exchange,
    pub status: ReconciliationStatus,
    pub ledger_balances: Vec<BalanceSnapshot>,
    pub exchange_balances: Vec<BalanceSnapshot>,
    pub discrepancies: Vec<Discrepancy>,
    pub total_discrepancy_value: Decimal,
    pub created_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Reconciliation configuration
#[derive(Debug, Clone)]
pub struct ReconciliationConfig {
    /// Tolerance for balance differences (as a fraction, e.g., 0.001 for 0.1%)
    pub tolerance_pct: Decimal,
    /// Minimum absolute difference to consider (ignore dust)
    pub min_difference: Decimal,
    /// Whether to auto-resolve small discrepancies
    pub auto_resolve_dust: bool,
    /// Dust threshold for auto-resolution
    pub dust_threshold: Decimal,
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            tolerance_pct: dec!(0.001), // 0.1%
            min_difference: dec!(0.00000001), // 1 satoshi
            auto_resolve_dust: true,
            dust_threshold: dec!(0.01), // $0.01
        }
    }
}

/// Trait for fetching exchange balances
#[async_trait::async_trait]
pub trait BalanceFetcher: Send + Sync {
    /// Fetch current balances from the exchange
    async fn fetch_balances(&self, exchange: Exchange) -> Result<Vec<BalanceSnapshot>, String>;
}

/// Trait for fetching ledger balances
#[async_trait::async_trait]
pub trait LedgerBalanceFetcher: Send + Sync {
    /// Fetch current balances from the ledger
    async fn fetch_balances(&self, exchange: Exchange) -> Result<Vec<BalanceSnapshot>, String>;
}

/// Trait for snapshot persistence
#[async_trait::async_trait]
pub trait ReconciliationStore: Send + Sync {
    /// Save a reconciliation snapshot
    async fn save_snapshot(&self, snapshot: &ReconciliationSnapshot) -> Result<(), String>;

    /// Get recent snapshots
    async fn get_recent_snapshots(
        &self,
        exchange: Option<Exchange>,
        limit: usize,
    ) -> Vec<ReconciliationSnapshot>;

    /// Get snapshot by ID
    async fn get_snapshot(&self, id: &str) -> Option<ReconciliationSnapshot>;

    /// Get snapshots with discrepancies
    async fn get_discrepancy_snapshots(
        &self,
        exchange: Option<Exchange>,
        limit: usize,
    ) -> Vec<ReconciliationSnapshot>;
}

/// Reconciliation Service
pub struct ReconciliationService<E: BalanceFetcher, L: LedgerBalanceFetcher, S: ReconciliationStore>
{
    exchange_fetcher: E,
    ledger_fetcher: L,
    store: S,
    config: ReconciliationConfig,
}

impl<E: BalanceFetcher, L: LedgerBalanceFetcher, S: ReconciliationStore>
    ReconciliationService<E, L, S>
{
    /// Create a new reconciliation service
    pub fn new(
        exchange_fetcher: E,
        ledger_fetcher: L,
        store: S,
        config: ReconciliationConfig,
    ) -> Self {
        Self {
            exchange_fetcher,
            ledger_fetcher,
            store,
            config,
        }
    }

    /// Run reconciliation for an exchange
    pub async fn reconcile(&self, exchange: Exchange) -> Result<ReconciliationSnapshot, String> {
        info!("Starting reconciliation for {}", exchange);

        // Fetch balances from both sources
        let exchange_balances = self.exchange_fetcher.fetch_balances(exchange).await?;
        let ledger_balances = self.ledger_fetcher.fetch_balances(exchange).await?;

        // Compare balances
        let discrepancies =
            self.compare_balances(&ledger_balances, &exchange_balances);

        let status = if discrepancies.is_empty() {
            ReconciliationStatus::Matched
        } else {
            ReconciliationStatus::Discrepancies
        };

        let total_discrepancy = discrepancies
            .iter()
            .map(|d| d.difference.abs())
            .sum();

        let snapshot = ReconciliationSnapshot {
            id: Uuid::new_v4().to_string(),
            exchange,
            status,
            ledger_balances,
            exchange_balances,
            discrepancies: discrepancies.clone(),
            total_discrepancy_value: total_discrepancy,
            created_at: Utc::now(),
            notes: None,
        };

        self.store.save_snapshot(&snapshot).await?;

        if discrepancies.is_empty() {
            info!("Reconciliation complete for {} - all balances match", exchange);
        } else {
            warn!(
                "Reconciliation complete for {} - {} discrepancies found, total: {}",
                exchange,
                discrepancies.len(),
                total_discrepancy
            );
        }

        Ok(snapshot)
    }

    /// Compare ledger and exchange balances
    fn compare_balances(
        &self,
        ledger_balances: &[BalanceSnapshot],
        exchange_balances: &[BalanceSnapshot],
    ) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();

        // Create maps for easy lookup
        let ledger_map: HashMap<&str, Decimal> = ledger_balances
            .iter()
            .map(|b| (b.asset.as_str(), b.balance))
            .collect();

        let exchange_map: HashMap<&str, Decimal> = exchange_balances
            .iter()
            .map(|b| (b.asset.as_str(), b.balance))
            .collect();

        // Check ledger balances against exchange
        for (asset, ledger_balance) in &ledger_map {
            match exchange_map.get(*asset) {
                Some(&exchange_balance) => {
                    if let Some(discrepancy) = self.check_discrepancy(
                        asset,
                        *ledger_balance,
                        exchange_balance,
                    ) {
                        discrepancies.push(discrepancy);
                    }
                }
                None => {
                    // Asset in ledger but not on exchange
                    if *ledger_balance > self.config.min_difference {
                        discrepancies.push(Discrepancy {
                            asset: asset.to_string(),
                            discrepancy_type: DiscrepancyType::MissingOnExchange,
                            ledger_balance: *ledger_balance,
                            exchange_balance: Decimal::ZERO,
                            difference: *ledger_balance,
                            difference_pct: dec!(100),
                        });
                    }
                }
            }
        }

        // Check for assets on exchange but not in ledger
        for (asset, exchange_balance) in &exchange_map {
            if !ledger_map.contains_key(*asset) && *exchange_balance > self.config.min_difference {
                discrepancies.push(Discrepancy {
                    asset: asset.to_string(),
                    discrepancy_type: DiscrepancyType::MissingInLedger,
                    ledger_balance: Decimal::ZERO,
                    exchange_balance: *exchange_balance,
                    difference: *exchange_balance,
                    difference_pct: dec!(100),
                });
            }
        }

        discrepancies
    }

    /// Check if there's a discrepancy between two balances
    fn check_discrepancy(
        &self,
        asset: &str,
        ledger_balance: Decimal,
        exchange_balance: Decimal,
    ) -> Option<Discrepancy> {
        let difference = ledger_balance - exchange_balance;
        let abs_difference = difference.abs();

        // Ignore if below minimum threshold
        if abs_difference < self.config.min_difference {
            return None;
        }

        // Calculate percentage difference
        let base = ledger_balance.max(exchange_balance);
        let difference_pct = if base > Decimal::ZERO {
            (abs_difference / base) * dec!(100)
        } else {
            Decimal::ZERO
        };

        // Check if within tolerance
        if difference_pct <= self.config.tolerance_pct * dec!(100) {
            debug!(
                "Balance difference for {} ({}) within tolerance",
                asset, difference
            );
            return None;
        }

        // Auto-resolve dust if enabled
        if self.config.auto_resolve_dust && abs_difference <= self.config.dust_threshold {
            debug!("Auto-resolving dust for {}: {}", asset, difference);
            return None;
        }

        let discrepancy_type = if difference > Decimal::ZERO {
            DiscrepancyType::LedgerHigher
        } else {
            DiscrepancyType::ExchangeHigher
        };

        Some(Discrepancy {
            asset: asset.to_string(),
            discrepancy_type,
            ledger_balance,
            exchange_balance,
            difference,
            difference_pct,
        })
    }

    /// Get recent reconciliation snapshots
    pub async fn get_recent_snapshots(
        &self,
        exchange: Option<Exchange>,
        limit: usize,
    ) -> Vec<ReconciliationSnapshot> {
        self.store.get_recent_snapshots(exchange, limit).await
    }

    /// Get snapshot by ID
    pub async fn get_snapshot(&self, id: &str) -> Option<ReconciliationSnapshot> {
        self.store.get_snapshot(id).await
    }

    /// Get snapshots with discrepancies
    pub async fn get_discrepancy_history(
        &self,
        exchange: Option<Exchange>,
        limit: usize,
    ) -> Vec<ReconciliationSnapshot> {
        self.store.get_discrepancy_snapshots(exchange, limit).await
    }

    /// Calculate the overall reconciliation health
    pub async fn get_health(&self, exchange: Exchange) -> ReconciliationHealth {
        let recent = self.store.get_recent_snapshots(Some(exchange), 10).await;

        if recent.is_empty() {
            return ReconciliationHealth {
                exchange,
                status: HealthStatus::Unknown,
                last_reconciliation: None,
                consecutive_discrepancies: 0,
                total_discrepancy_value: Decimal::ZERO,
            };
        }

        let last = &recent[0];
        let consecutive_discrepancies = recent
            .iter()
            .take_while(|s| s.status == ReconciliationStatus::Discrepancies)
            .count() as u32;

        let status = match (last.status, consecutive_discrepancies) {
            (ReconciliationStatus::Matched, _) => HealthStatus::Healthy,
            (ReconciliationStatus::Discrepancies, 1..=2) => HealthStatus::Warning,
            (ReconciliationStatus::Discrepancies, _) => HealthStatus::Critical,
            _ => HealthStatus::Unknown,
        };

        ReconciliationHealth {
            exchange,
            status,
            last_reconciliation: Some(last.created_at),
            consecutive_discrepancies,
            total_discrepancy_value: last.total_discrepancy_value,
        }
    }
}

/// Health status for reconciliation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Reconciliation health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationHealth {
    pub exchange: Exchange,
    pub status: HealthStatus,
    pub last_reconciliation: Option<DateTime<Utc>>,
    pub consecutive_discrepancies: u32,
    pub total_discrepancy_value: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_config_default() {
        let config = ReconciliationConfig::default();
        assert_eq!(config.tolerance_pct, dec!(0.001));
        assert!(config.auto_resolve_dust);
    }

    #[test]
    fn test_discrepancy_type() {
        assert_eq!(
            DiscrepancyType::LedgerHigher,
            DiscrepancyType::LedgerHigher
        );
        assert_ne!(
            DiscrepancyType::LedgerHigher,
            DiscrepancyType::ExchangeHigher
        );
    }
}
