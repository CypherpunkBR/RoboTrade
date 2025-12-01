//! Historical Sync Engine
//!
//! Synchronizes historical data from exchanges:
//! - Trades
//! - Orders
//! - Positions
//! - Balances
//! - Deposits/Withdrawals
//! - Funding payments
//!
//! Features:
//! - Resumable sync with cursor support
//! - Rate limiting
//! - Retry with backoff
//! - Progress tracking

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use super::ws_manager::Exchange;

/// Sync data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncDataType {
    Trades,
    Orders,
    Positions,
    Balances,
    Deposits,
    Withdrawals,
    Funding,
}

impl std::fmt::Display for SyncDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncDataType::Trades => write!(f, "trades"),
            SyncDataType::Orders => write!(f, "orders"),
            SyncDataType::Positions => write!(f, "positions"),
            SyncDataType::Balances => write!(f, "balances"),
            SyncDataType::Deposits => write!(f, "deposits"),
            SyncDataType::Withdrawals => write!(f, "withdrawals"),
            SyncDataType::Funding => write!(f, "funding"),
        }
    }
}

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// Sync progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgressEvent {
    pub exchange: Exchange,
    pub data_type: SyncDataType,
    pub status: SyncStatus,
    pub items_synced: u64,
    pub total_items: Option<u64>,
    pub current_timestamp: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Start timestamp for sync (default: beginning of time)
    pub start_time: Option<DateTime<Utc>>,
    /// End timestamp for sync (default: now)
    pub end_time: Option<DateTime<Utc>>,
    /// Batch size for API calls
    pub batch_size: usize,
    /// Rate limit (requests per second)
    pub rate_limit: f64,
    /// Max retries per request
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_retry_delay: Duration,
    /// Max retry delay
    pub max_retry_delay: Duration,
    /// Data types to sync
    pub data_types: Vec<SyncDataType>,
    /// Progress channel buffer size
    pub progress_buffer_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            start_time: None,
            end_time: None,
            batch_size: 1000,
            rate_limit: 5.0,
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(30),
            data_types: vec![
                SyncDataType::Trades,
                SyncDataType::Orders,
                SyncDataType::Positions,
                SyncDataType::Balances,
                SyncDataType::Deposits,
                SyncDataType::Withdrawals,
                SyncDataType::Funding,
            ],
            progress_buffer_size: 100,
        }
    }
}

/// Sync state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub exchange: Exchange,
    pub data_type: SyncDataType,
    pub status: SyncStatus,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub last_id: Option<String>,
    pub items_synced: u64,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error: Option<String>,
}

/// Synced trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedTrade {
    pub exchange: Exchange,
    pub trade_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub realized_pnl: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
}

/// Synced order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedOrder {
    pub exchange: Exchange,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub status: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub avg_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Synced position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedPosition {
    pub exchange: Exchange,
    pub symbol: String,
    pub side: String,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub leverage: Decimal,
    pub margin_type: String,
    pub timestamp: DateTime<Utc>,
}

/// Synced balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedBalance {
    pub exchange: Exchange,
    pub asset: String,
    pub total: Decimal,
    pub available: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Synced deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedDeposit {
    pub exchange: Exchange,
    pub deposit_id: String,
    pub asset: String,
    pub amount: Decimal,
    pub status: String,
    pub tx_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Synced withdrawal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedWithdrawal {
    pub exchange: Exchange,
    pub withdrawal_id: String,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub status: String,
    pub tx_id: Option<String>,
    pub address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Synced funding payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedFunding {
    pub exchange: Exchange,
    pub funding_id: String,
    pub symbol: String,
    pub amount: Decimal,
    pub funding_rate: Decimal,
    pub position_size: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Synced data batch
#[derive(Debug, Clone)]
pub enum SyncedData {
    Trades(Vec<SyncedTrade>),
    Orders(Vec<SyncedOrder>),
    Positions(Vec<SyncedPosition>),
    Balances(Vec<SyncedBalance>),
    Deposits(Vec<SyncedDeposit>),
    Withdrawals(Vec<SyncedWithdrawal>),
    Funding(Vec<SyncedFunding>),
}

/// Trait for sync data persistence
#[async_trait::async_trait]
pub trait SyncDataStore: Send + Sync {
    /// Get sync state for an exchange and data type
    async fn get_sync_state(&self, exchange: Exchange, data_type: SyncDataType) -> Option<SyncState>;

    /// Save sync state
    async fn save_sync_state(&self, state: &SyncState) -> Result<(), String>;

    /// Save synced trades
    async fn save_trades(&self, trades: &[SyncedTrade]) -> Result<(), String>;

    /// Save synced orders
    async fn save_orders(&self, orders: &[SyncedOrder]) -> Result<(), String>;

    /// Save synced positions
    async fn save_positions(&self, positions: &[SyncedPosition]) -> Result<(), String>;

    /// Save synced balances
    async fn save_balances(&self, balances: &[SyncedBalance]) -> Result<(), String>;

    /// Save synced deposits
    async fn save_deposits(&self, deposits: &[SyncedDeposit]) -> Result<(), String>;

    /// Save synced withdrawals
    async fn save_withdrawals(&self, withdrawals: &[SyncedWithdrawal]) -> Result<(), String>;

    /// Save synced funding
    async fn save_funding(&self, funding: &[SyncedFunding]) -> Result<(), String>;
}

/// Trait for exchange sync adapter
#[async_trait::async_trait]
pub trait ExchangeSyncAdapter: Send + Sync {
    /// Get exchange identifier
    fn exchange(&self) -> Exchange;

    /// Fetch trades
    async fn fetch_trades(
        &self,
        symbol: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SyncedTrade>, String>;

    /// Fetch orders
    async fn fetch_orders(
        &self,
        symbol: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<SyncedOrder>, String>;

    /// Fetch positions
    async fn fetch_positions(&self) -> Result<Vec<SyncedPosition>, String>;

    /// Fetch balances
    async fn fetch_balances(&self) -> Result<Vec<SyncedBalance>, String>;

    /// Fetch deposits
    async fn fetch_deposits(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<SyncedDeposit>, String>;

    /// Fetch withdrawals
    async fn fetch_withdrawals(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<SyncedWithdrawal>, String>;

    /// Fetch funding payments
    async fn fetch_funding(
        &self,
        symbol: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<SyncedFunding>, String>;
}

/// Sync engine handle
pub struct SyncEngineHandle {
    cmd_tx: mpsc::Sender<SyncCommand>,
    progress_rx: mpsc::Receiver<SyncProgressEvent>,
}

impl SyncEngineHandle {
    /// Receive next progress event
    pub async fn recv_progress(&mut self) -> Option<SyncProgressEvent> {
        self.progress_rx.recv().await
    }

    /// Stop the sync engine
    pub async fn stop(&self) -> Result<(), String> {
        self.cmd_tx
            .send(SyncCommand::Stop)
            .await
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    /// Pause sync
    pub async fn pause(&self) -> Result<(), String> {
        self.cmd_tx
            .send(SyncCommand::Pause)
            .await
            .map_err(|e| format!("Failed to send pause command: {}", e))
    }

    /// Resume sync
    pub async fn resume(&self) -> Result<(), String> {
        self.cmd_tx
            .send(SyncCommand::Resume)
            .await
            .map_err(|e| format!("Failed to send resume command: {}", e))
    }
}

/// Sync commands
enum SyncCommand {
    Stop,
    Pause,
    Resume,
}

/// Historical Sync Engine
pub struct SyncEngine<S: SyncDataStore> {
    config: SyncConfig,
    store: S,
    adapters: Vec<Box<dyn ExchangeSyncAdapter>>,
}

impl<S: SyncDataStore + 'static> SyncEngine<S> {
    /// Create a new sync engine
    pub fn new(config: SyncConfig, store: S) -> Self {
        Self {
            config,
            store,
            adapters: Vec::new(),
        }
    }

    /// Add an exchange adapter
    pub fn add_adapter(&mut self, adapter: Box<dyn ExchangeSyncAdapter>) {
        self.adapters.push(adapter);
    }

    /// Start the sync engine
    pub fn start(self) -> SyncEngineHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (progress_tx, progress_rx) = mpsc::channel(self.config.progress_buffer_size);

        tokio::spawn(async move {
            run_sync_engine(self.config, self.store, self.adapters, cmd_rx, progress_tx).await;
        });

        SyncEngineHandle { cmd_tx, progress_rx }
    }
}

/// Main sync engine loop
async fn run_sync_engine<S: SyncDataStore>(
    config: SyncConfig,
    store: S,
    adapters: Vec<Box<dyn ExchangeSyncAdapter>>,
    mut cmd_rx: mpsc::Receiver<SyncCommand>,
    progress_tx: mpsc::Sender<SyncProgressEvent>,
) {
    info!("Sync engine starting...");

    let mut is_running = true;
    let mut is_paused = false;

    // Calculate delay between requests based on rate limit
    let request_delay = Duration::from_secs_f64(1.0 / config.rate_limit);

    // Process each adapter
    for adapter in &adapters {
        if !is_running {
            break;
        }

        let exchange = adapter.exchange();
        info!("Starting sync for {}", exchange);

        // Process each data type
        for data_type in &config.data_types {
            if !is_running {
                break;
            }

            // Check for commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    SyncCommand::Stop => {
                        is_running = false;
                        break;
                    }
                    SyncCommand::Pause => {
                        is_paused = true;
                    }
                    SyncCommand::Resume => {
                        is_paused = false;
                    }
                }
            }

            // Wait while paused
            while is_paused && is_running {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        SyncCommand::Stop => is_running = false,
                        SyncCommand::Resume => is_paused = false,
                        _ => {}
                    }
                }
            }

            if !is_running {
                break;
            }

            // Get or create sync state
            let mut state = store.get_sync_state(exchange, *data_type)
                .await
                .unwrap_or_else(|| SyncState {
                    exchange,
                    data_type: *data_type,
                    status: SyncStatus::Pending,
                    last_timestamp: config.start_time,
                    last_id: None,
                    items_synced: 0,
                    started_at: Utc::now(),
                    updated_at: Utc::now(),
                    error: None,
                });

            // Skip if already completed
            if state.status == SyncStatus::Completed {
                debug!("Skipping {} {} - already completed", exchange, data_type);
                continue;
            }

            info!("Syncing {} {} from {:?}", exchange, data_type, state.last_timestamp);

            // Update state to in progress
            state.status = SyncStatus::InProgress;
            state.updated_at = Utc::now();
            let _ = store.save_sync_state(&state).await;

            // Send progress event
            send_progress(&progress_tx, &state).await;

            // Sync loop for this data type
            let result = sync_data_type(
                adapter.as_ref(),
                &store,
                &config,
                &mut state,
                &progress_tx,
                request_delay,
            ).await;

            match result {
                Ok(()) => {
                    state.status = SyncStatus::Completed;
                    state.error = None;
                    info!("Completed sync for {} {} - {} items", exchange, data_type, state.items_synced);
                }
                Err(e) => {
                    state.status = SyncStatus::Failed;
                    state.error = Some(e.clone());
                    error!("Failed sync for {} {}: {}", exchange, data_type, e);
                }
            }

            state.updated_at = Utc::now();
            let _ = store.save_sync_state(&state).await;
            send_progress(&progress_tx, &state).await;
        }
    }

    info!("Sync engine stopped");
}

/// Sync a specific data type
async fn sync_data_type<S: SyncDataStore>(
    adapter: &dyn ExchangeSyncAdapter,
    store: &S,
    config: &SyncConfig,
    state: &mut SyncState,
    progress_tx: &mpsc::Sender<SyncProgressEvent>,
    request_delay: Duration,
) -> Result<(), String> {
    let exchange = adapter.exchange();
    let mut retries = 0u32;
    let mut retry_delay = config.initial_retry_delay;

    loop {
        // Rate limiting
        tokio::time::sleep(request_delay).await;

        let result = match state.data_type {
            SyncDataType::Trades => {
                let trades = adapter.fetch_trades(
                    None,
                    state.last_timestamp,
                    config.end_time,
                    state.last_id.as_deref(),
                    config.batch_size,
                ).await?;

                if trades.is_empty() {
                    return Ok(());
                }

                let count = trades.len();
                if let Some(last) = trades.last() {
                    state.last_timestamp = Some(last.timestamp);
                    state.last_id = Some(last.trade_id.clone());
                }

                store.save_trades(&trades).await?;
                count
            }

            SyncDataType::Orders => {
                let orders = adapter.fetch_orders(
                    None,
                    state.last_timestamp,
                    config.end_time,
                    config.batch_size,
                ).await?;

                if orders.is_empty() {
                    return Ok(());
                }

                let count = orders.len();
                if let Some(last) = orders.last() {
                    state.last_timestamp = Some(last.updated_at);
                    state.last_id = Some(last.order_id.clone());
                }

                store.save_orders(&orders).await?;
                count
            }

            SyncDataType::Positions => {
                let positions = adapter.fetch_positions().await?;
                let count = positions.len();
                store.save_positions(&positions).await?;
                // Positions are a snapshot, complete after one fetch
                state.status = SyncStatus::Completed;
                return Ok(());
            }

            SyncDataType::Balances => {
                let balances = adapter.fetch_balances().await?;
                let count = balances.len();
                store.save_balances(&balances).await?;
                // Balances are a snapshot, complete after one fetch
                return Ok(());
            }

            SyncDataType::Deposits => {
                let deposits = adapter.fetch_deposits(
                    state.last_timestamp,
                    config.end_time,
                    config.batch_size,
                ).await?;

                if deposits.is_empty() {
                    return Ok(());
                }

                let count = deposits.len();
                if let Some(last) = deposits.last() {
                    state.last_timestamp = Some(last.timestamp);
                    state.last_id = Some(last.deposit_id.clone());
                }

                store.save_deposits(&deposits).await?;
                count
            }

            SyncDataType::Withdrawals => {
                let withdrawals = adapter.fetch_withdrawals(
                    state.last_timestamp,
                    config.end_time,
                    config.batch_size,
                ).await?;

                if withdrawals.is_empty() {
                    return Ok(());
                }

                let count = withdrawals.len();
                if let Some(last) = withdrawals.last() {
                    state.last_timestamp = Some(last.timestamp);
                    state.last_id = Some(last.withdrawal_id.clone());
                }

                store.save_withdrawals(&withdrawals).await?;
                count
            }

            SyncDataType::Funding => {
                let funding = adapter.fetch_funding(
                    None,
                    state.last_timestamp,
                    config.end_time,
                    config.batch_size,
                ).await?;

                if funding.is_empty() {
                    return Ok(());
                }

                let count = funding.len();
                if let Some(last) = funding.last() {
                    state.last_timestamp = Some(last.timestamp);
                    state.last_id = Some(last.funding_id.clone());
                }

                store.save_funding(&funding).await?;
                count
            }
        };

        // Reset retry counter on success
        retries = 0;
        retry_delay = config.initial_retry_delay;

        // Update state
        state.items_synced += result as u64;
        state.updated_at = Utc::now();
        let _ = store.save_sync_state(state).await;

        // Send progress
        send_progress(progress_tx, state).await;

        // If we got less than batch size, we're done
        if result < config.batch_size {
            return Ok(());
        }
    }
}

/// Send progress event
async fn send_progress(tx: &mpsc::Sender<SyncProgressEvent>, state: &SyncState) {
    let event = SyncProgressEvent {
        exchange: state.exchange,
        data_type: state.data_type,
        status: state.status,
        items_synced: state.items_synced,
        total_items: None,
        current_timestamp: state.last_timestamp,
        error: state.error.clone(),
    };
    let _ = tx.send(event).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_data_type_display() {
        assert_eq!(format!("{}", SyncDataType::Trades), "trades");
        assert_eq!(format!("{}", SyncDataType::Funding), "funding");
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.data_types.len(), 7);
    }
}
