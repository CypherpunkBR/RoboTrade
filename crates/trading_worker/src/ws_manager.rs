//! WebSocket Manager
//!
//! Orchestrates WebSocket connections to multiple exchanges (Binance, Kraken)
//! and provides a unified stream of normalized events.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Normalized event from any exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedEvent {
    /// Balance update
    BalanceUpdate(BalanceUpdateEvent),
    /// Position update
    PositionUpdate(PositionUpdateEvent),
    /// Order update
    OrderUpdate(OrderUpdateEvent),
    /// Trade/Fill executed
    TradeExecuted(TradeEvent),
    /// Account log entry (funding, transfers, etc)
    AccountLogEntry(AccountLogEvent),
    /// Connection state changed
    ConnectionStateChanged(ConnectionStateEvent),
    /// Error occurred
    Error(ErrorEvent),
}

/// Balance update from any exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdateEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Asset/currency
    pub asset: String,
    /// Total balance
    pub total: Decimal,
    /// Available balance
    pub available: Decimal,
    /// Balance change (if available)
    pub change: Option<Decimal>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Position update from any exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdateEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Symbol (e.g., "BTCUSDT", "PI_XBTUSD")
    pub symbol: String,
    /// Position side (LONG/SHORT)
    pub side: PositionSide,
    /// Position size
    pub size: Decimal,
    /// Entry price
    pub entry_price: Decimal,
    /// Unrealized PnL
    pub unrealized_pnl: Decimal,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Order update from any exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Symbol
    pub symbol: String,
    /// Exchange order ID
    pub order_id: String,
    /// Client order ID (if available)
    pub client_order_id: Option<String>,
    /// Order side (BUY/SELL)
    pub side: OrderSide,
    /// Order type (LIMIT/MARKET/etc)
    pub order_type: String,
    /// Order status
    pub status: OrderStatus,
    /// Original quantity
    pub quantity: Decimal,
    /// Filled quantity
    pub filled_quantity: Decimal,
    /// Average fill price
    pub avg_price: Decimal,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Trade/Fill event from any exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Trade ID
    pub trade_id: String,
    /// Symbol
    pub symbol: String,
    /// Related order ID
    pub order_id: String,
    /// Trade side (BUY/SELL)
    pub side: OrderSide,
    /// Trade price
    pub price: Decimal,
    /// Trade quantity
    pub quantity: Decimal,
    /// Commission paid
    pub commission: Decimal,
    /// Commission asset
    pub commission_asset: String,
    /// Realized profit (if any)
    pub realized_pnl: Option<Decimal>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Account log event (funding, deposits, withdrawals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLogEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Log entry type
    pub log_type: AccountLogType,
    /// Asset
    pub asset: String,
    /// Amount
    pub amount: Decimal,
    /// New balance after operation
    pub balance: Option<Decimal>,
    /// Description/info
    pub description: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Connection state change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStateEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// New state
    pub state: WsConnectionState,
    /// Reason (if disconnected/error)
    pub reason: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Exchange identifier
    pub exchange: Exchange,
    /// Error message
    pub message: String,
    /// Error code (if available)
    pub code: Option<String>,
    /// Is recoverable
    pub recoverable: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Exchange identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exchange {
    Binance,
    Kraken,
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Binance => write!(f, "Binance"),
            Exchange::Kraken => write!(f, "Kraken"),
        }
    }
}

/// Position side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
    Both, // For one-way mode
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

/// Account log entry type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountLogType {
    Deposit,
    Withdrawal,
    FundingFee,
    Commission,
    RealizedPnL,
    Transfer,
    Other(String),
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WsConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Reconnecting,
    Failed,
}

/// Configuration for the WebSocket Manager
#[derive(Debug, Clone)]
pub struct WsManagerConfig {
    /// Enable Binance connection
    pub enable_binance: bool,
    /// Enable Kraken connection
    pub enable_kraken: bool,
    /// Channel buffer size
    pub channel_buffer_size: usize,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for WsManagerConfig {
    fn default() -> Self {
        Self {
            enable_binance: true,
            enable_kraken: true,
            channel_buffer_size: 1000,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// WebSocket Manager state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsManagerState {
    pub binance_state: WsConnectionState,
    pub kraken_state: WsConnectionState,
    pub is_running: bool,
}

/// WebSocket Manager handle for controlling the manager
pub struct WsManagerHandle {
    /// Command sender
    cmd_tx: mpsc::Sender<WsManagerCommand>,
    /// Event receiver
    event_rx: mpsc::Receiver<NormalizedEvent>,
}

impl WsManagerHandle {
    /// Receive next event
    pub async fn recv(&mut self) -> Option<NormalizedEvent> {
        self.event_rx.recv().await
    }

    /// Try to receive event without blocking
    pub fn try_recv(&mut self) -> Result<NormalizedEvent, mpsc::error::TryRecvError> {
        self.event_rx.try_recv()
    }

    /// Stop the manager
    pub async fn stop(&self) -> Result<(), String> {
        self.cmd_tx
            .send(WsManagerCommand::Stop)
            .await
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    /// Reconnect a specific exchange
    pub async fn reconnect(&self, exchange: Exchange) -> Result<(), String> {
        self.cmd_tx
            .send(WsManagerCommand::Reconnect(exchange))
            .await
            .map_err(|e| format!("Failed to send reconnect command: {}", e))
    }

    /// Get current state
    pub async fn get_state(&self) -> Result<WsManagerState, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.cmd_tx
            .send(WsManagerCommand::GetState(tx))
            .await
            .map_err(|e| format!("Failed to send get state command: {}", e))?;
        rx.await.map_err(|e| format!("Failed to receive state: {}", e))
    }
}

/// Commands for controlling the WebSocket Manager
enum WsManagerCommand {
    Stop,
    Reconnect(Exchange),
    GetState(tokio::sync::oneshot::Sender<WsManagerState>),
}

/// WebSocket Manager
///
/// Orchestrates multiple exchange WebSocket connections and provides
/// a unified stream of normalized events.
pub struct WsManager {
    config: WsManagerConfig,
}

impl WsManager {
    /// Create a new WebSocket Manager
    pub fn new(config: WsManagerConfig) -> Self {
        Self { config }
    }

    /// Start the WebSocket Manager
    ///
    /// Returns a handle for controlling the manager and receiving events.
    pub fn start(
        self,
        binance_client: Option<robotrade_exchange_gateways::binance::BinanceFuturesClient>,
        kraken_credentials: Option<(String, String)>, // (api_key, api_secret)
    ) -> WsManagerHandle {
        let (event_tx, event_rx) = mpsc::channel(self.config.channel_buffer_size);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        // Spawn the manager task
        tokio::spawn(async move {
            run_manager(self.config, binance_client, kraken_credentials, event_tx, cmd_rx).await;
        });

        WsManagerHandle { cmd_tx, event_rx }
    }
}

/// Main manager loop
async fn run_manager(
    config: WsManagerConfig,
    binance_client: Option<robotrade_exchange_gateways::binance::BinanceFuturesClient>,
    kraken_credentials: Option<(String, String)>,
    event_tx: mpsc::Sender<NormalizedEvent>,
    mut cmd_rx: mpsc::Receiver<WsManagerCommand>,
) {
    info!("WebSocket Manager starting...");

    let mut binance_state = WsConnectionState::Disconnected;
    let mut kraken_state = WsConnectionState::Disconnected;
    let mut is_running = true;

    // Create WebSocket clients
    let mut binance_ws = if config.enable_binance {
        if let Some(client) = binance_client {
            let ws_config = robotrade_exchange_gateways::binance::BinanceWsConfig::default();
            Some(robotrade_exchange_gateways::binance::BinanceWsClient::new(client, ws_config))
        } else {
            warn!("Binance enabled but no client provided");
            None
        }
    } else {
        None
    };

    let mut kraken_ws = if config.enable_kraken {
        if let Some((api_key, api_secret)) = kraken_credentials {
            let ws_config = robotrade_exchange_gateways::kraken::KrakenWsConfig::default();
            Some(robotrade_exchange_gateways::kraken::KrakenWsClient::new(
                api_key, api_secret, ws_config,
            ))
        } else {
            warn!("Kraken enabled but no credentials provided");
            None
        }
    } else {
        None
    };

    // Connect to exchanges
    if let Some(ref mut ws) = binance_ws {
        match ws.connect().await {
            Ok(()) => {
                binance_state = WsConnectionState::Connected;
                send_connection_event(&event_tx, Exchange::Binance, WsConnectionState::Connected, None).await;
            }
            Err(e) => {
                error!("Failed to connect to Binance: {}", e);
                binance_state = WsConnectionState::Failed;
                send_connection_event(&event_tx, Exchange::Binance, WsConnectionState::Failed, Some(e)).await;
            }
        }
    }

    if let Some(ref mut ws) = kraken_ws {
        match ws.connect().await {
            Ok(()) => {
                kraken_state = WsConnectionState::Connected;
                send_connection_event(&event_tx, Exchange::Kraken, WsConnectionState::Connected, None).await;

                // Authenticate
                if let Err(e) = ws.authenticate().await {
                    error!("Failed to authenticate Kraken: {}", e);
                    send_error_event(&event_tx, Exchange::Kraken, e, true).await;
                }
            }
            Err(e) => {
                error!("Failed to connect to Kraken: {}", e);
                kraken_state = WsConnectionState::Failed;
                send_connection_event(&event_tx, Exchange::Kraken, WsConnectionState::Failed, Some(e)).await;
            }
        }
    }

    // Health check timer
    let mut health_check = interval(config.health_check_interval);

    info!("WebSocket Manager running");

    // Main event loop
    while is_running {
        tokio::select! {
            // Handle commands
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    WsManagerCommand::Stop => {
                        info!("WebSocket Manager stopping...");
                        is_running = false;
                    }
                    WsManagerCommand::Reconnect(exchange) => {
                        match exchange {
                            Exchange::Binance => {
                                if let Some(ref mut ws) = binance_ws {
                                    info!("Reconnecting to Binance...");
                                    let _ = ws.reconnect().await;
                                }
                            }
                            Exchange::Kraken => {
                                if let Some(ref mut ws) = kraken_ws {
                                    info!("Reconnecting to Kraken...");
                                    let _ = ws.reconnect().await;
                                }
                            }
                        }
                    }
                    WsManagerCommand::GetState(tx) => {
                        let state = WsManagerState {
                            binance_state,
                            kraken_state,
                            is_running,
                        };
                        let _ = tx.send(state);
                    }
                }
            }

            // Handle Binance events
            Some(event) = async {
                if let Some(ref mut ws) = binance_ws {
                    ws.recv().await
                } else {
                    std::future::pending::<Option<robotrade_exchange_gateways::binance::BinanceWsEvent>>().await
                }
            } => {
                process_binance_event(event, &event_tx, &mut binance_state).await;
            }

            // Handle Kraken events
            Some(event) = async {
                if let Some(ref mut ws) = kraken_ws {
                    ws.recv().await
                } else {
                    std::future::pending::<Option<robotrade_exchange_gateways::kraken::KrakenWsEvent>>().await
                }
            } => {
                process_kraken_event(event, &event_tx, &mut kraken_state).await;
            }

            // Health check / keepalive
            _ = health_check.tick() => {
                // Binance keepalive
                if let Some(ref mut ws) = binance_ws {
                    if ws.needs_keepalive() {
                        if let Err(e) = ws.keepalive().await {
                            warn!("Binance keepalive failed: {}", e);
                        }
                    }
                }

                // Kraken heartbeat
                if let Some(ref mut ws) = kraken_ws {
                    if ws.needs_heartbeat() {
                        if let Err(e) = ws.heartbeat().await {
                            warn!("Kraken heartbeat failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    if let Some(ref mut ws) = binance_ws {
        ws.disconnect().await;
    }
    if let Some(ref mut ws) = kraken_ws {
        ws.disconnect().await;
    }

    info!("WebSocket Manager stopped");
}

/// Send connection state event
async fn send_connection_event(
    tx: &mpsc::Sender<NormalizedEvent>,
    exchange: Exchange,
    state: WsConnectionState,
    reason: Option<String>,
) {
    let event = NormalizedEvent::ConnectionStateChanged(ConnectionStateEvent {
        exchange,
        state,
        reason,
        timestamp: Utc::now(),
    });
    let _ = tx.send(event).await;
}

/// Send error event
async fn send_error_event(
    tx: &mpsc::Sender<NormalizedEvent>,
    exchange: Exchange,
    message: String,
    recoverable: bool,
) {
    let event = NormalizedEvent::Error(ErrorEvent {
        exchange,
        message,
        code: None,
        recoverable,
        timestamp: Utc::now(),
    });
    let _ = tx.send(event).await;
}

/// Process Binance WebSocket event
async fn process_binance_event(
    event: robotrade_exchange_gateways::binance::BinanceWsEvent,
    tx: &mpsc::Sender<NormalizedEvent>,
    state: &mut WsConnectionState,
) {
    use robotrade_exchange_gateways::binance::BinanceWsEvent;

    match event {
        BinanceWsEvent::AccountUpdate(update) => {
            let timestamp = timestamp_to_datetime(update.event_time);

            // Process balance updates
            for balance in update.balances {
                let event = NormalizedEvent::BalanceUpdate(BalanceUpdateEvent {
                    exchange: Exchange::Binance,
                    asset: balance.asset,
                    total: balance.wallet_balance,
                    available: balance.cross_wallet_balance,
                    change: Some(balance.balance_change),
                    timestamp,
                });
                let _ = tx.send(event).await;
            }

            // Process position updates
            for position in update.positions {
                let side = match position.position_side.as_str() {
                    "LONG" => PositionSide::Long,
                    "SHORT" => PositionSide::Short,
                    _ => PositionSide::Both,
                };

                let event = NormalizedEvent::PositionUpdate(PositionUpdateEvent {
                    exchange: Exchange::Binance,
                    symbol: position.symbol,
                    side,
                    size: position.position_amount.abs(),
                    entry_price: position.entry_price,
                    unrealized_pnl: position.unrealized_pnl,
                    timestamp,
                });
                let _ = tx.send(event).await;
            }
        }

        BinanceWsEvent::OrderUpdate(update) => {
            let side = match update.side.as_str() {
                "BUY" => OrderSide::Buy,
                _ => OrderSide::Sell,
            };

            let status = match update.order_status.as_str() {
                "NEW" => OrderStatus::New,
                "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
                "FILLED" => OrderStatus::Filled,
                "CANCELED" => OrderStatus::Canceled,
                "REJECTED" => OrderStatus::Rejected,
                "EXPIRED" => OrderStatus::Expired,
                _ => OrderStatus::New,
            };

            // Send order update
            let order_event = NormalizedEvent::OrderUpdate(OrderUpdateEvent {
                exchange: Exchange::Binance,
                symbol: update.symbol.clone(),
                order_id: update.order_id.to_string(),
                client_order_id: Some(update.client_order_id.clone()),
                side,
                order_type: update.order_type.clone(),
                status,
                quantity: update.original_quantity,
                filled_quantity: update.cumulative_filled_quantity,
                avg_price: update.average_price,
                timestamp: timestamp_to_datetime(update.event_time),
            });
            let _ = tx.send(order_event).await;

            // If this is a trade execution, also send trade event
            if update.execution_type == "TRADE" && update.last_filled_quantity > Decimal::ZERO {
                let trade_event = NormalizedEvent::TradeExecuted(TradeEvent {
                    exchange: Exchange::Binance,
                    trade_id: update.trade_id.to_string(),
                    symbol: update.symbol,
                    order_id: update.order_id.to_string(),
                    side,
                    price: update.last_filled_price,
                    quantity: update.last_filled_quantity,
                    commission: update.commission,
                    commission_asset: update.commission_asset.unwrap_or_else(|| "USDT".to_string()),
                    realized_pnl: if update.realized_profit != Decimal::ZERO {
                        Some(update.realized_profit)
                    } else {
                        None
                    },
                    timestamp: timestamp_to_datetime(update.order_trade_time),
                });
                let _ = tx.send(trade_event).await;
            }
        }

        BinanceWsEvent::ListenKeyExpired => {
            warn!("Binance listen key expired, need to reconnect");
            *state = WsConnectionState::Disconnected;
            send_connection_event(tx, Exchange::Binance, WsConnectionState::Disconnected, Some("Listen key expired".to_string())).await;
        }

        BinanceWsEvent::Disconnected { reason } => {
            *state = WsConnectionState::Disconnected;
            send_connection_event(tx, Exchange::Binance, WsConnectionState::Disconnected, Some(reason)).await;
        }

        BinanceWsEvent::Error { message } => {
            send_error_event(tx, Exchange::Binance, message, true).await;
        }

        _ => {
            debug!("Unhandled Binance event");
        }
    }
}

/// Process Kraken WebSocket event
async fn process_kraken_event(
    event: robotrade_exchange_gateways::kraken::KrakenWsEvent,
    tx: &mpsc::Sender<NormalizedEvent>,
    state: &mut WsConnectionState,
) {
    use robotrade_exchange_gateways::kraken::KrakenWsEvent;

    match event {
        KrakenWsEvent::Subscribed { feed } => {
            if feed == "fills" {
                *state = WsConnectionState::Authenticated;
                send_connection_event(tx, Exchange::Kraken, WsConnectionState::Authenticated, None).await;
            }
        }

        KrakenWsEvent::Fills(fills) => {
            for fill in fills {
                let side = match fill.side.to_uppercase().as_str() {
                    "BUY" => OrderSide::Buy,
                    _ => OrderSide::Sell,
                };

                let event = NormalizedEvent::TradeExecuted(TradeEvent {
                    exchange: Exchange::Kraken,
                    trade_id: fill.fill_id,
                    symbol: fill.symbol,
                    order_id: fill.order_id,
                    side,
                    price: fill.price,
                    quantity: fill.size,
                    commission: Decimal::ZERO, // Kraken doesn't include commission in fills
                    commission_asset: "USD".to_string(),
                    realized_pnl: None,
                    timestamp: timestamp_to_datetime(fill.fill_time),
                });
                let _ = tx.send(event).await;
            }
        }

        KrakenWsEvent::Orders(orders) => {
            for order in orders {
                let side = match order.side.to_uppercase().as_str() {
                    "BUY" => OrderSide::Buy,
                    _ => OrderSide::Sell,
                };

                let status = match order.status.to_lowercase().as_str() {
                    "open" | "untouched" => OrderStatus::New,
                    "partiallyfilled" => OrderStatus::PartiallyFilled,
                    "filled" => OrderStatus::Filled,
                    "cancelled" => OrderStatus::Canceled,
                    _ => OrderStatus::New,
                };

                let event = NormalizedEvent::OrderUpdate(OrderUpdateEvent {
                    exchange: Exchange::Kraken,
                    symbol: order.symbol,
                    order_id: order.order_id,
                    client_order_id: None,
                    side,
                    order_type: order.order_type,
                    status,
                    quantity: order.quantity,
                    filled_quantity: order.filled_quantity,
                    avg_price: order.limit_price.unwrap_or(Decimal::ZERO),
                    timestamp: timestamp_to_datetime(order.timestamp),
                });
                let _ = tx.send(event).await;
            }
        }

        KrakenWsEvent::Positions(positions) => {
            for position in positions {
                let side = match position.side.to_uppercase().as_str() {
                    "LONG" => PositionSide::Long,
                    "SHORT" => PositionSide::Short,
                    _ => PositionSide::Both,
                };

                let event = NormalizedEvent::PositionUpdate(PositionUpdateEvent {
                    exchange: Exchange::Kraken,
                    symbol: position.symbol,
                    side,
                    size: position.size.abs(),
                    entry_price: position.entry_price,
                    unrealized_pnl: position.unrealized_pnl,
                    timestamp: Utc::now(),
                });
                let _ = tx.send(event).await;
            }
        }

        KrakenWsEvent::Balances(balances) => {
            for balance in balances {
                let event = NormalizedEvent::BalanceUpdate(BalanceUpdateEvent {
                    exchange: Exchange::Kraken,
                    asset: balance.currency,
                    total: balance.quantity,
                    available: balance.available,
                    change: None,
                    timestamp: Utc::now(),
                });
                let _ = tx.send(event).await;
            }
        }

        KrakenWsEvent::AccountLog(entries) => {
            for entry in entries {
                let log_type = match entry.log_type.to_lowercase().as_str() {
                    "deposit" => AccountLogType::Deposit,
                    "withdrawal" => AccountLogType::Withdrawal,
                    "funding" | "funding fee" => AccountLogType::FundingFee,
                    "trade" | "commission" => AccountLogType::Commission,
                    "realized pnl" | "pnl" => AccountLogType::RealizedPnL,
                    "transfer" => AccountLogType::Transfer,
                    other => AccountLogType::Other(other.to_string()),
                };

                let event = NormalizedEvent::AccountLogEntry(AccountLogEvent {
                    exchange: Exchange::Kraken,
                    log_type,
                    asset: entry.asset,
                    amount: entry.amount,
                    balance: Some(entry.balance),
                    description: Some(entry.log_type),
                    timestamp: timestamp_to_datetime(entry.timestamp),
                });
                let _ = tx.send(event).await;
            }
        }

        KrakenWsEvent::Disconnected { reason } => {
            *state = WsConnectionState::Disconnected;
            send_connection_event(tx, Exchange::Kraken, WsConnectionState::Disconnected, Some(reason)).await;
        }

        KrakenWsEvent::Error { message } => {
            send_error_event(tx, Exchange::Kraken, message, true).await;
        }

        _ => {
            debug!("Unhandled Kraken event");
        }
    }
}

/// Convert millisecond timestamp to DateTime<Utc>
fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(timestamp_ms).unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_display() {
        assert_eq!(format!("{}", Exchange::Binance), "Binance");
        assert_eq!(format!("{}", Exchange::Kraken), "Kraken");
    }

    #[test]
    fn test_timestamp_conversion() {
        let ts = 1700000000000i64;
        let dt = timestamp_to_datetime(ts);
        assert_eq!(dt.timestamp_millis(), ts);
    }

    #[test]
    fn test_config_default() {
        let config = WsManagerConfig::default();
        assert!(config.enable_binance);
        assert!(config.enable_kraken);
        assert_eq!(config.channel_buffer_size, 1000);
    }
}
