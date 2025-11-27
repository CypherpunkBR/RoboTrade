//! Reconciliation State - Simplified Implementation
//!
//! This module provides the reconciliation state for comparing ledger balances
//! with exchange balances. The actual reconciliation service integration will
//! be implemented incrementally.

use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::traits::ExchangeGateway;
use robotrade_infra::database::DbPool;
use robotrade_trading_worker::ws_manager::Exchange;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::exchange_service::ExchangeService;
use crate::ledger_state::LedgerState;

/// Reconciliation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationStatus {
    Matched,
    Discrepancy,
    Pending,
    Error(String),
}

/// Discrepancy type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscrepancyType {
    ExchangeHigher,
    LedgerHigher,
    Missing,
}

/// Discrepancy details
#[derive(Debug, Clone)]
pub struct Discrepancy {
    pub asset: String,
    pub discrepancy_type: DiscrepancyType,
    pub exchange_balance: Decimal,
    pub ledger_balance: Decimal,
    pub difference: Decimal,
    pub difference_pct: Decimal,
}

/// Reconciliation snapshot
#[derive(Debug, Clone)]
pub struct ReconciliationSnapshot {
    pub id: String,
    pub exchange: Exchange,
    pub status: ReconciliationStatus,
    pub discrepancies: Vec<Discrepancy>,
    pub total_discrepancy_value: Decimal,
    pub created_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Reconciliation health
#[derive(Debug, Clone)]
pub struct ReconciliationHealth {
    pub exchange: Exchange,
    pub status: HealthStatus,
    pub last_reconciliation: Option<DateTime<Utc>>,
    pub consecutive_discrepancies: u32,
    pub total_discrepancy_value: Decimal,
}

/// State container for reconciliation services
pub struct ReconciliationState {
    inner: Arc<RwLock<ReconciliationStateInner>>,
}

struct ReconciliationStateInner {
    pool: Option<DbPool>,
    exchange_service: Option<ExchangeService>,
    ledger_state: Option<LedgerState>,
    threshold: Decimal,
}

impl ReconciliationState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ReconciliationStateInner {
                pool: None,
                exchange_service: None,
                ledger_state: None,
                threshold: Decimal::new(1, 8), // 0.00000001 default threshold
            })),
        }
    }

    /// Initialize with dependencies
    pub fn initialize(
        &self,
        pool: DbPool,
        exchange_service: ExchangeService,
        ledger_state: LedgerState,
    ) {
        debug!("Initializing ReconciliationState");
        let mut inner = futures::executor::block_on(self.inner.write());
        inner.pool = Some(pool);
        inner.exchange_service = Some(exchange_service);
        inner.ledger_state = Some(ledger_state);
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        if let Ok(inner) = self.inner.try_read() {
            inner.pool.is_some()
        } else {
            false
        }
    }

    /// Run reconciliation for an exchange
    pub async fn run_reconciliation(&self, exchange: Exchange) -> Result<ReconciliationSnapshot, String> {
        let inner = self.inner.read().await;

        let exchange_service = inner
            .exchange_service
            .as_ref()
            .ok_or("Exchange service not initialized")?;

        let ledger_state = inner
            .ledger_state
            .as_ref()
            .ok_or("Ledger state not initialized")?;

        let pool = inner
            .pool
            .as_ref()
            .ok_or("Pool not initialized")?;

        // Fetch balances from exchange
        let exchange_balances = match exchange {
            Exchange::Binance => {
                let guard = exchange_service.binance_client().await;
                if let Some(ref client) = *guard {
                    match client.get_balances().await {
                        Ok(balances) => {
                            let mut map = HashMap::new();
                            for b in balances {
                                map.insert(b.asset.clone(), b.free + b.locked);
                            }
                            map
                        }
                        Err(e) => return Err(format!("Failed to fetch Binance balances: {}", e)),
                    }
                } else {
                    return Err("Binance client not initialized".to_string());
                }
            }
            Exchange::Kraken => {
                let guard = exchange_service.kraken_client().await;
                if let Some(ref client) = *guard {
                    match client.get_balances().await {
                        Ok(balances) => {
                            let mut map = HashMap::new();
                            for b in balances {
                                map.insert(b.asset.clone(), b.free + b.locked);
                            }
                            map
                        }
                        Err(e) => return Err(format!("Failed to fetch Kraken balances: {}", e)),
                    }
                } else {
                    return Err("Kraken client not initialized".to_string());
                }
            }
        };

        // Fetch balances from ledger
        let ledger_balances_list = ledger_state.get_all_balances(exchange).await;
        let mut ledger_balances: HashMap<String, Decimal> = HashMap::new();
        for ab in ledger_balances_list {
            ledger_balances.insert(ab.asset, ab.balance);
        }

        // Compare and find discrepancies
        let mut discrepancies = Vec::new();
        let mut total_discrepancy = Decimal::ZERO;
        let threshold = inner.threshold;

        // Check all exchange assets
        for (asset, exchange_balance) in &exchange_balances {
            let ledger_balance = ledger_balances
                .get(asset)
                .copied()
                .unwrap_or(Decimal::ZERO);

            let difference = *exchange_balance - ledger_balance;
            let abs_diff = difference.abs();

            if abs_diff > threshold {
                let diff_pct = if *exchange_balance != Decimal::ZERO {
                    (abs_diff / *exchange_balance) * Decimal::from(100)
                } else {
                    Decimal::from(100)
                };

                discrepancies.push(Discrepancy {
                    asset: asset.clone(),
                    discrepancy_type: if difference > Decimal::ZERO {
                        DiscrepancyType::ExchangeHigher
                    } else {
                        DiscrepancyType::LedgerHigher
                    },
                    exchange_balance: *exchange_balance,
                    ledger_balance,
                    difference: abs_diff,
                    difference_pct: diff_pct,
                });

                total_discrepancy += abs_diff;
            }
        }

        // Check ledger assets not in exchange
        for (asset, ledger_balance) in &ledger_balances {
            if !exchange_balances.contains_key(asset) && *ledger_balance > threshold {
                discrepancies.push(Discrepancy {
                    asset: asset.clone(),
                    discrepancy_type: DiscrepancyType::LedgerHigher,
                    exchange_balance: Decimal::ZERO,
                    ledger_balance: *ledger_balance,
                    difference: *ledger_balance,
                    difference_pct: Decimal::from(100),
                });

                total_discrepancy += *ledger_balance;
            }
        }

        let status = if discrepancies.is_empty() {
            ReconciliationStatus::Matched
        } else {
            ReconciliationStatus::Discrepancy
        };

        let snapshot = ReconciliationSnapshot {
            id: Uuid::new_v4().to_string(),
            exchange,
            status,
            discrepancies,
            total_discrepancy_value: total_discrepancy,
            created_at: Utc::now(),
            notes: None,
        };

        // Save snapshot to database
        self.save_snapshot(pool, &snapshot).await?;

        info!(
            "Reconciliation completed for {:?}: {} discrepancies, total: {}",
            exchange,
            snapshot.discrepancies.len(),
            total_discrepancy
        );

        Ok(snapshot)
    }

    async fn save_snapshot(&self, pool: &DbPool, snapshot: &ReconciliationSnapshot) -> Result<(), String> {
        let exchange_id = match snapshot.exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };
        let status_str = match &snapshot.status {
            ReconciliationStatus::Matched => "Matched",
            ReconciliationStatus::Discrepancy => "Discrepancy",
            ReconciliationStatus::Pending => "Pending",
            ReconciliationStatus::Error(_) => "Error",
        };
        let discrepancies_json = serde_json::to_string(&snapshot.discrepancies.iter().map(|d| {
            serde_json::json!({
                "asset": d.asset,
                "type": format!("{:?}", d.discrepancy_type),
                "exchange_balance": d.exchange_balance.to_string(),
                "ledger_balance": d.ledger_balance.to_string(),
                "difference": d.difference.to_string(),
                "difference_pct": d.difference_pct.to_string(),
            })
        }).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            r#"
            INSERT INTO reconciliation_snapshots (
                id, exchange_id, status, discrepancies,
                total_discrepancy_value, created_at, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&snapshot.id)
        .bind(exchange_id)
        .bind(status_str)
        .bind(&discrepancies_json)
        .bind(snapshot.total_discrepancy_value.to_string())
        .bind(snapshot.created_at.timestamp())
        .bind(&snapshot.notes)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to save reconciliation snapshot: {}", e))?;

        Ok(())
    }

    /// Get recent reconciliation snapshots
    pub async fn get_recent_snapshots(
        &self,
        exchange: Option<Exchange>,
        limit: usize,
    ) -> Vec<ReconciliationSnapshot> {
        let inner = self.inner.read().await;
        let pool = match inner.pool.as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut query = String::from(
            "SELECT id, exchange_id, status, discrepancies, total_discrepancy_value, created_at, notes FROM reconciliation_snapshots"
        );

        if let Some(ex) = exchange {
            let exchange_id = match ex {
                Exchange::Binance => "binance_futures",
                Exchange::Kraken => "kraken_futures",
            };
            query.push_str(&format!(" WHERE exchange_id = '{}'", exchange_id));
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

        let rows: Vec<ReconciliationSnapshotRow> = sqlx::query_as(&query)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        rows.into_iter()
            .filter_map(|r| r.try_into_snapshot().ok())
            .collect()
    }

    /// Get specific snapshot
    pub async fn get_snapshot(&self, id: &str) -> Option<ReconciliationSnapshot> {
        let inner = self.inner.read().await;
        let pool = inner.pool.as_ref()?;

        let row: Option<ReconciliationSnapshotRow> = sqlx::query_as(
            "SELECT id, exchange_id, status, discrepancies, total_discrepancy_value, created_at, notes FROM reconciliation_snapshots WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        row.and_then(|r| r.try_into_snapshot().ok())
    }

    /// Get reconciliation health
    pub async fn get_health(&self, exchange: Exchange) -> ReconciliationHealth {
        let inner = self.inner.read().await;
        let pool = match inner.pool.as_ref() {
            Some(p) => p,
            None => {
                return ReconciliationHealth {
                    exchange,
                    status: HealthStatus::Unknown,
                    last_reconciliation: None,
                    consecutive_discrepancies: 0,
                    total_discrepancy_value: Decimal::ZERO,
                };
            }
        };

        let exchange_id = match exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };

        // Get last snapshot
        let last_snapshot: Option<ReconciliationSnapshotRow> = sqlx::query_as(
            "SELECT id, exchange_id, status, discrepancies, total_discrepancy_value, created_at, notes FROM reconciliation_snapshots WHERE exchange_id = ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(exchange_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        // Count consecutive discrepancies
        let consecutive: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (SELECT status FROM reconciliation_snapshots WHERE exchange_id = ? ORDER BY created_at DESC LIMIT 10) WHERE status = 'Discrepancy'"
        )
        .bind(exchange_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        match last_snapshot {
            Some(snap) => {
                let snapshot = snap.try_into_snapshot().ok();
                match snapshot {
                    Some(s) => {
                        let status = if s.discrepancies.is_empty() {
                            HealthStatus::Healthy
                        } else if consecutive > 3 {
                            HealthStatus::Critical
                        } else {
                            HealthStatus::Warning
                        };
                        ReconciliationHealth {
                            exchange,
                            status,
                            last_reconciliation: Some(s.created_at),
                            consecutive_discrepancies: consecutive as u32,
                            total_discrepancy_value: s.total_discrepancy_value,
                        }
                    }
                    None => ReconciliationHealth {
                        exchange,
                        status: HealthStatus::Unknown,
                        last_reconciliation: None,
                        consecutive_discrepancies: 0,
                        total_discrepancy_value: Decimal::ZERO,
                    },
                }
            }
            None => ReconciliationHealth {
                exchange,
                status: HealthStatus::Unknown,
                last_reconciliation: None,
                consecutive_discrepancies: 0,
                total_discrepancy_value: Decimal::ZERO,
            },
        }
    }
}

impl Default for ReconciliationState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ReconciliationState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ReconciliationSnapshotRow {
    id: String,
    exchange_id: String,
    status: String,
    discrepancies: String,
    total_discrepancy_value: String,
    created_at: i64,
    notes: Option<String>,
}

impl ReconciliationSnapshotRow {
    fn try_into_snapshot(self) -> Result<ReconciliationSnapshot, String> {
        use std::str::FromStr;

        let exchange = match self.exchange_id.as_str() {
            "binance_futures" | "binance" => Exchange::Binance,
            "kraken_futures" | "kraken" => Exchange::Kraken,
            _ => Exchange::Binance,
        };

        let status = match self.status.as_str() {
            "Matched" => ReconciliationStatus::Matched,
            "Discrepancy" => ReconciliationStatus::Discrepancy,
            "Pending" => ReconciliationStatus::Pending,
            _ => ReconciliationStatus::Error(self.status.clone()),
        };

        // Parse discrepancies JSON
        let discrepancies: Vec<Discrepancy> = serde_json::from_str::<Vec<serde_json::Value>>(&self.discrepancies)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                Some(Discrepancy {
                    asset: v.get("asset")?.as_str()?.to_string(),
                    discrepancy_type: match v.get("type")?.as_str()? {
                        "ExchangeHigher" => DiscrepancyType::ExchangeHigher,
                        "LedgerHigher" => DiscrepancyType::LedgerHigher,
                        _ => DiscrepancyType::Missing,
                    },
                    exchange_balance: Decimal::from_str(v.get("exchange_balance")?.as_str()?).ok()?,
                    ledger_balance: Decimal::from_str(v.get("ledger_balance")?.as_str()?).ok()?,
                    difference: Decimal::from_str(v.get("difference")?.as_str()?).ok()?,
                    difference_pct: Decimal::from_str(v.get("difference_pct")?.as_str()?).ok()?,
                })
            })
            .collect();

        Ok(ReconciliationSnapshot {
            id: self.id,
            exchange,
            status,
            discrepancies,
            total_discrepancy_value: Decimal::from_str(&self.total_discrepancy_value)
                .unwrap_or(Decimal::ZERO),
            created_at: Utc
                .timestamp_opt(self.created_at, 0)
                .single()
                .unwrap_or_else(Utc::now),
            notes: self.notes,
        })
    }
}
