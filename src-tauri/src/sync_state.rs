//! Sync State - Simplified Implementation
//!
//! This module provides the sync state for synchronizing historical data from exchanges.
//! The actual sync engine integration will be implemented incrementally.

use chrono::{DateTime, TimeZone, Utc};
use robotrade_infra::database::DbPool;
use robotrade_trading_worker::{
    sync_engine::SyncDataType,
    ws_manager::Exchange,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::exchange_service::ExchangeService;

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// Simplified sync state
#[derive(Debug, Clone)]
pub struct SimpleSyncState {
    pub exchange: Exchange,
    pub data_type: SyncDataType,
    pub status: SyncStatus,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub items_synced: u64,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error: Option<String>,
}

/// State container for sync services
pub struct SyncState {
    inner: Arc<RwLock<SyncStateInner>>,
}

struct SyncStateInner {
    pool: Option<DbPool>,
    exchange_service: Option<ExchangeService>,
    is_running: bool,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SyncStateInner {
                pool: None,
                exchange_service: None,
                is_running: false,
            })),
        }
    }

    /// Initialize with dependencies
    pub fn initialize(&self, pool: DbPool, exchange_service: ExchangeService) {
        debug!("Initializing SyncState");
        let mut inner = futures::executor::block_on(self.inner.write());
        inner.pool = Some(pool);
        inner.exchange_service = Some(exchange_service);
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        if let Ok(inner) = self.inner.try_read() {
            inner.pool.is_some()
        } else {
            false
        }
    }

    /// Start sync for an exchange
    pub async fn start_sync(
        &self,
        exchange: Exchange,
        data_types: Option<Vec<SyncDataType>>,
        start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        let mut inner = self.inner.write().await;

        if inner.is_running {
            return Err("Sync already running".to_string());
        }

        inner.is_running = true;

        let types = data_types.unwrap_or_else(|| {
            vec![
                SyncDataType::Trades,
                SyncDataType::Orders,
                SyncDataType::Balances,
            ]
        });

        info!(
            "Starting sync for {:?}: {:?}, start={:?}",
            exchange, types, start_time
        );

        // For now, just mark as started (actual sync will be implemented later)
        inner.is_running = false;

        Ok(())
    }

    /// Stop sync
    pub async fn stop_sync(&self) -> Result<(), String> {
        let mut inner = self.inner.write().await;
        inner.is_running = false;
        info!("Sync stopped");
        Ok(())
    }

    /// Pause sync
    pub async fn pause_sync(&self) -> Result<(), String> {
        info!("Sync paused");
        Ok(())
    }

    /// Resume sync
    pub async fn resume_sync(&self) -> Result<(), String> {
        info!("Sync resumed");
        Ok(())
    }

    /// Get sync state for a data type
    pub async fn get_sync_state(
        &self,
        exchange: Exchange,
        data_type: SyncDataType,
    ) -> Option<SimpleSyncState> {
        let inner = self.inner.read().await;
        let pool = inner.pool.as_ref()?;

        let exchange_id = match exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };
        let data_type_str = format!("{:?}", data_type);

        let row: Option<SyncStateRow> = sqlx::query_as(
            r#"
            SELECT exchange_id, data_type, status, last_timestamp,
                   items_synced, started_at, updated_at, error
            FROM sync_states
            WHERE exchange_id = ? AND data_type = ?
            "#,
        )
        .bind(exchange_id)
        .bind(&data_type_str)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        row.map(|r| r.into_sync_state(exchange, data_type))
    }

    /// Get all sync states for an exchange
    pub async fn get_all_sync_states(&self, exchange: Exchange) -> Vec<SimpleSyncState> {
        let inner = self.inner.read().await;
        let pool = match inner.pool.as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let exchange_id = match exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };

        let rows: Vec<SyncStateRow> = sqlx::query_as(
            r#"
            SELECT exchange_id, data_type, status, last_timestamp,
                   items_synced, started_at, updated_at, error
            FROM sync_states
            WHERE exchange_id = ?
            "#,
        )
        .bind(exchange_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .filter_map(|r| {
                let dt = parse_data_type(&r.data_type)?;
                Some(r.into_sync_state(exchange, dt))
            })
            .collect()
    }

    /// Reset sync for a data type
    pub async fn reset_sync(
        &self,
        exchange: Exchange,
        data_type: SyncDataType,
    ) -> Result<(), String> {
        let inner = self.inner.read().await;
        let pool = inner.pool.as_ref().ok_or("Pool not initialized")?;

        let exchange_id = match exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };
        let data_type_str = format!("{:?}", data_type);

        sqlx::query("DELETE FROM sync_states WHERE exchange_id = ? AND data_type = ?")
            .bind(exchange_id)
            .bind(&data_type_str)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to reset sync: {}", e))?;

        info!("Sync reset for {:?} {:?}", exchange, data_type);
        Ok(())
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SyncState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(sqlx::FromRow)]
struct SyncStateRow {
    exchange_id: String,
    data_type: String,
    status: String,
    last_timestamp: Option<i64>,
    items_synced: i64,
    started_at: i64,
    updated_at: i64,
    error: Option<String>,
}

impl SyncStateRow {
    fn into_sync_state(self, exchange: Exchange, data_type: SyncDataType) -> SimpleSyncState {
        let status = match self.status.as_str() {
            "Pending" => SyncStatus::Pending,
            "InProgress" => SyncStatus::InProgress,
            "Completed" => SyncStatus::Completed,
            "Failed" => SyncStatus::Failed,
            "Paused" => SyncStatus::Paused,
            _ => SyncStatus::Pending,
        };

        SimpleSyncState {
            exchange,
            data_type,
            status,
            last_timestamp: self
                .last_timestamp
                .and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            items_synced: self.items_synced as u64,
            started_at: Utc
                .timestamp_opt(self.started_at, 0)
                .single()
                .unwrap_or_else(Utc::now),
            updated_at: Utc
                .timestamp_opt(self.updated_at, 0)
                .single()
                .unwrap_or_else(Utc::now),
            error: self.error,
        }
    }
}

fn parse_data_type(s: &str) -> Option<SyncDataType> {
    match s {
        "Trades" => Some(SyncDataType::Trades),
        "Orders" => Some(SyncDataType::Orders),
        "Positions" => Some(SyncDataType::Positions),
        "Balances" => Some(SyncDataType::Balances),
        "Deposits" => Some(SyncDataType::Deposits),
        "Withdrawals" => Some(SyncDataType::Withdrawals),
        "Funding" => Some(SyncDataType::Funding),
        _ => None,
    }
}
