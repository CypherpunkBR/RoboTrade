//! Repositório SQLite para Orders
//!
//! Persistência de ordens de trading.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, SqlitePool};
use tracing::debug;
use uuid::Uuid;

use robotrade_core::{ExchangeId, Order, OrderId, OrderSource, OrderStatus};

/// Trait para repositório de ordens
#[async_trait]
pub trait OrderRepository: Send + Sync {
    /// Salva uma ordem
    async fn save(&self, order: &Order) -> Result<(), String>;

    /// Busca ordem por ID
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, String>;

    /// Lista ordens abertas
    async fn find_open_orders(
        &self,
        exchange: Option<ExchangeId>,
        symbol: Option<&str>,
    ) -> Result<Vec<Order>, String>;

    /// Lista ordens recentes
    async fn find_recent(&self, limit: usize) -> Result<Vec<Order>, String>;

    /// Atualiza status de uma ordem
    async fn update_status(
        &self,
        id: &OrderId,
        status: OrderStatus,
        filled_quantity: Decimal,
        average_fill_price: Option<Decimal>,
    ) -> Result<(), String>;

    /// Deleta ordens antigas (cleanup)
    async fn delete_old_orders(&self, before: DateTime<Utc>) -> Result<u64, String>;
}

/// Implementação SQLite do repositório de ordens
pub struct SqliteOrderRepository {
    pool: SqlitePool,
}

impl SqliteOrderRepository {
    /// Cria um novo repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inicializa a tabela
    pub async fn init(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                client_order_id TEXT NOT NULL,
                exchange_order_id TEXT,
                exchange TEXT NOT NULL,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                quantity TEXT NOT NULL,
                price TEXT,
                stop_price TEXT,
                stop_loss TEXT,
                take_profit TEXT,
                time_in_force TEXT NOT NULL,
                status TEXT NOT NULL,
                filled_quantity TEXT NOT NULL,
                average_fill_price TEXT,
                source_type TEXT NOT NULL,
                source_data TEXT,
                error_message TEXT,
                created_at TEXT NOT NULL,
                submitted_at TEXT,
                filled_at TEXT,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create orders table: {}", e))?;

        // Índices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_exchange ON orders(exchange)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_symbol ON orders(symbol)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at)")
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }
}

#[derive(FromRow)]
struct OrderRow {
    id: String,
    client_order_id: String,
    exchange_order_id: Option<String>,
    exchange: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: Option<String>,
    stop_price: Option<String>,
    stop_loss: Option<String>,
    take_profit: Option<String>,
    time_in_force: String,
    status: String,
    filled_quantity: String,
    average_fill_price: Option<String>,
    source_type: String,
    source_data: Option<String>,
    error_message: Option<String>,
    created_at: String,
    submitted_at: Option<String>,
    filled_at: Option<String>,
    updated_at: String,
}

fn parse_exchange_id(s: &str) -> Result<ExchangeId, String> {
    match s {
        "binance_futures" => Ok(ExchangeId::BinanceFutures),
        "binance_spot" => Ok(ExchangeId::BinanceSpot),
        "kraken_futures" => Ok(ExchangeId::KrakenFutures),
        "paper" => Ok(ExchangeId::Paper),
        "okx" => Ok(ExchangeId::Okx),
        "bybit" => Ok(ExchangeId::Bybit),
        _ => Err(format!("Unknown exchange: {}", s)),
    }
}

fn exchange_id_to_string(id: ExchangeId) -> String {
    match id {
        ExchangeId::BinanceFutures => "binance_futures".to_string(),
        ExchangeId::BinanceSpot => "binance_spot".to_string(),
        ExchangeId::KrakenFutures => "kraken_futures".to_string(),
        ExchangeId::Paper => "paper".to_string(),
        ExchangeId::Okx => "okx".to_string(),
        ExchangeId::Bybit => "bybit".to_string(),
    }
}

fn parse_order_side(s: &str) -> Result<robotrade_core::OrderSide, String> {
    match s {
        "buy" => Ok(robotrade_core::OrderSide::Buy),
        "sell" => Ok(robotrade_core::OrderSide::Sell),
        _ => Err(format!("Invalid order side: {}", s)),
    }
}

fn parse_order_type(s: &str) -> Result<robotrade_core::OrderType, String> {
    match s {
        "market" => Ok(robotrade_core::OrderType::Market),
        "limit" => Ok(robotrade_core::OrderType::Limit),
        "stop_loss" | "stoploss" => Ok(robotrade_core::OrderType::StopLoss),
        "stop_loss_limit" | "stoplosslimit" => Ok(robotrade_core::OrderType::StopLossLimit),
        "take_profit" | "takeprofit" => Ok(robotrade_core::OrderType::TakeProfit),
        "take_profit_limit" | "takeprofitlimit" => Ok(robotrade_core::OrderType::TakeProfitLimit),
        "trailing_stop" | "trailingstop" => Ok(robotrade_core::OrderType::TrailingStop),
        _ => Err(format!("Invalid order type: {}", s)),
    }
}

fn parse_time_in_force(s: &str) -> Result<robotrade_core::TimeInForce, String> {
    match s {
        "GTC" => Ok(robotrade_core::TimeInForce::GTC),
        "IOC" => Ok(robotrade_core::TimeInForce::IOC),
        "FOK" => Ok(robotrade_core::TimeInForce::FOK),
        "GTD" => Ok(robotrade_core::TimeInForce::GTD),
        _ => Err(format!("Invalid time in force: {}", s)),
    }
}

fn parse_order_status(s: &str) -> Result<OrderStatus, String> {
    match s {
        "pending" => Ok(OrderStatus::Pending),
        "submitted" => Ok(OrderStatus::Submitted),
        "partially_filled" | "partiallyfilled" => Ok(OrderStatus::PartiallyFilled),
        "filled" => Ok(OrderStatus::Filled),
        "cancelled" => Ok(OrderStatus::Cancelled),
        "rejected" => Ok(OrderStatus::Rejected),
        "expired" => Ok(OrderStatus::Expired),
        "failed" => Ok(OrderStatus::Failed),
        _ => Err(format!("Invalid order status: {}", s)),
    }
}

fn parse_order_source(source_type: &str, source_data: Option<&str>) -> Result<OrderSource, String> {
    match source_type {
        "signal" => {
            let data = source_data.ok_or("Signal source requires data")?;
            let parts: Vec<&str> = data.split('|').collect();
            if parts.len() != 2 {
                return Err("Invalid signal source data format".to_string());
            }
            let signal_id =
                Uuid::parse_str(parts[0]).map_err(|e| format!("Invalid signal_id: {}", e))?;
            Ok(OrderSource::Signal {
                signal_id,
                strategy_id: parts[1].to_string(),
            })
        }
        "manual" => Ok(OrderSource::Manual {
            nota: source_data.map(|s| s.to_string()),
        }),
        "stop_loss" => {
            let data = source_data.ok_or("StopLoss source requires position_id")?;
            let position_id =
                Uuid::parse_str(data).map_err(|e| format!("Invalid position_id: {}", e))?;
            Ok(OrderSource::StopLoss { position_id })
        }
        "take_profit" => {
            let data = source_data.ok_or("TakeProfit source requires position_id")?;
            let position_id =
                Uuid::parse_str(data).map_err(|e| format!("Invalid position_id: {}", e))?;
            Ok(OrderSource::TakeProfit { position_id })
        }
        _ => Err(format!("Invalid source type: {}", source_type)),
    }
}

fn order_source_to_strings(source: &OrderSource) -> (String, Option<String>) {
    match source {
        OrderSource::Signal {
            signal_id,
            strategy_id,
        } => (
            "signal".to_string(),
            Some(format!("{}|{}", signal_id, strategy_id)),
        ),
        OrderSource::Manual { nota } => ("manual".to_string(), nota.clone()),
        OrderSource::StopLoss { position_id } => {
            ("stop_loss".to_string(), Some(position_id.to_string()))
        }
        OrderSource::TakeProfit { position_id } => {
            ("take_profit".to_string(), Some(position_id.to_string()))
        }
    }
}

fn order_type_to_string(ot: robotrade_core::OrderType) -> String {
    match ot {
        robotrade_core::OrderType::Market => "market".to_string(),
        robotrade_core::OrderType::Limit => "limit".to_string(),
        robotrade_core::OrderType::StopLoss => "stop_loss".to_string(),
        robotrade_core::OrderType::StopLossLimit => "stop_loss_limit".to_string(),
        robotrade_core::OrderType::TakeProfit => "take_profit".to_string(),
        robotrade_core::OrderType::TakeProfitLimit => "take_profit_limit".to_string(),
        robotrade_core::OrderType::TrailingStop => "trailing_stop".to_string(),
    }
}

fn order_status_to_string(status: OrderStatus) -> String {
    match status {
        OrderStatus::Pending => "pending".to_string(),
        OrderStatus::Submitted => "submitted".to_string(),
        OrderStatus::PartiallyFilled => "partially_filled".to_string(),
        OrderStatus::Filled => "filled".to_string(),
        OrderStatus::Cancelled => "cancelled".to_string(),
        OrderStatus::Rejected => "rejected".to_string(),
        OrderStatus::Expired => "expired".to_string(),
        OrderStatus::Failed => "failed".to_string(),
    }
}

impl TryFrom<OrderRow> for Order {
    type Error = String;

    fn try_from(row: OrderRow) -> Result<Self, Self::Error> {
        Ok(Order {
            id: OrderId::from_string(&row.id).map_err(|e| format!("Invalid order id: {}", e))?,
            client_order_id: row.client_order_id,
            exchange_order_id: row.exchange_order_id,
            exchange: parse_exchange_id(&row.exchange)?,
            symbol: row.symbol,
            side: parse_order_side(&row.side)?,
            order_type: parse_order_type(&row.order_type)?,
            quantity: row
                .quantity
                .parse()
                .map_err(|e| format!("Invalid quantity: {}", e))?,
            price: row
                .price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid price: {}", e))?,
            stop_price: row
                .stop_price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid stop price: {}", e))?,
            stop_loss: row
                .stop_loss
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid stop loss: {}", e))?,
            take_profit: row
                .take_profit
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid take profit: {}", e))?,
            time_in_force: parse_time_in_force(&row.time_in_force)?,
            status: parse_order_status(&row.status)?,
            filled_quantity: row
                .filled_quantity
                .parse()
                .map_err(|e| format!("Invalid filled quantity: {}", e))?,
            average_fill_price: row
                .average_fill_price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid average fill price: {}", e))?,
            source: parse_order_source(&row.source_type, row.source_data.as_deref())?,
            error_message: row.error_message,
            created_at: row
                .created_at
                .parse()
                .map_err(|e| format!("Invalid created_at: {}", e))?,
            submitted_at: row
                .submitted_at
                .map(|t| t.parse())
                .transpose()
                .map_err(|e| format!("Invalid submitted_at: {}", e))?,
            filled_at: row
                .filled_at
                .map(|t| t.parse())
                .transpose()
                .map_err(|e| format!("Invalid filled_at: {}", e))?,
            updated_at: row
                .updated_at
                .parse()
                .map_err(|e| format!("Invalid updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl OrderRepository for SqliteOrderRepository {
    async fn save(&self, order: &Order) -> Result<(), String> {
        let (source_type, source_data) = order_source_to_strings(&order.source);

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO orders (
                id, client_order_id, exchange_order_id, exchange, symbol, side, order_type,
                quantity, price, stop_price, stop_loss, take_profit, time_in_force,
                status, filled_quantity, average_fill_price, source_type, source_data,
                error_message, created_at, submitted_at, filled_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(order.id.to_string())
        .bind(&order.client_order_id)
        .bind(&order.exchange_order_id)
        .bind(exchange_id_to_string(order.exchange))
        .bind(&order.symbol)
        .bind(order.side.to_string())
        .bind(order_type_to_string(order.order_type))
        .bind(order.quantity.to_string())
        .bind(order.price.map(|p| p.to_string()))
        .bind(order.stop_price.map(|p| p.to_string()))
        .bind(order.stop_loss.map(|p| p.to_string()))
        .bind(order.take_profit.map(|p| p.to_string()))
        .bind(format!("{:?}", order.time_in_force))
        .bind(order_status_to_string(order.status))
        .bind(order.filled_quantity.to_string())
        .bind(order.average_fill_price.map(|p| p.to_string()))
        .bind(&source_type)
        .bind(&source_data)
        .bind(&order.error_message)
        .bind(order.created_at.to_rfc3339())
        .bind(order.submitted_at.map(|t| t.to_rfc3339()))
        .bind(order.filled_at.map(|t| t.to_rfc3339()))
        .bind(order.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save order: {}", e))?;

        debug!(order_id = %order.id, "Order saved");
        Ok(())
    }

    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, String> {
        let row: Option<OrderRow> = sqlx::query_as("SELECT * FROM orders WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find order: {}", e))?;

        row.map(Order::try_from).transpose()
    }

    async fn find_open_orders(
        &self,
        exchange: Option<ExchangeId>,
        symbol: Option<&str>,
    ) -> Result<Vec<Order>, String> {
        let mut query = String::from(
            "SELECT * FROM orders WHERE status IN ('pending', 'submitted', 'partially_filled')",
        );

        if exchange.is_some() {
            query.push_str(" AND exchange = ?");
        }
        if symbol.is_some() {
            query.push_str(" AND symbol = ?");
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<_, OrderRow>(&query);

        if let Some(ex) = exchange {
            q = q.bind(exchange_id_to_string(ex));
        }
        if let Some(sym) = symbol {
            q = q.bind(sym);
        }

        let rows: Vec<OrderRow> = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find open orders: {}", e))?;

        rows.into_iter().map(Order::try_from).collect()
    }

    async fn find_recent(&self, limit: usize) -> Result<Vec<Order>, String> {
        let rows: Vec<OrderRow> =
            sqlx::query_as("SELECT * FROM orders ORDER BY created_at DESC LIMIT ?")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Failed to find recent orders: {}", e))?;

        rows.into_iter().map(Order::try_from).collect()
    }

    async fn update_status(
        &self,
        id: &OrderId,
        status: OrderStatus,
        filled_quantity: Decimal,
        average_fill_price: Option<Decimal>,
    ) -> Result<(), String> {
        let now = Utc::now();
        let filled_at = if status == OrderStatus::Filled {
            Some(now)
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE orders
            SET status = ?, filled_quantity = ?, average_fill_price = ?,
                filled_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(order_status_to_string(status))
        .bind(filled_quantity.to_string())
        .bind(average_fill_price.map(|p| p.to_string()))
        .bind(filled_at.map(|t| t.to_rfc3339()))
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update order status: {}", e))?;

        debug!(order_id = %id, status = ?status, "Order status updated");
        Ok(())
    }

    async fn delete_old_orders(&self, before: DateTime<Utc>) -> Result<u64, String> {
        let result = sqlx::query(
            "DELETE FROM orders WHERE created_at < ? AND status IN ('filled', 'cancelled', 'rejected', 'expired', 'failed')",
        )
        .bind(before.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old orders: {}", e))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robotrade_core::{OrderRequest, OrderSide, OrderType};
    use rust_decimal_macros::dec;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let repo = SqliteOrderRepository::new(pool.clone());
        repo.init().await.unwrap();
        pool
    }

    fn create_test_order() -> Order {
        let request = OrderRequest::market("BTCUSDT", OrderSide::Buy, dec!(0.1));
        Order::from_request(
            request,
            ExchangeId::BinanceFutures,
            OrderSource::Manual { nota: None },
        )
    }

    #[tokio::test]
    async fn test_save_and_find_order() {
        let pool = setup_test_db().await;
        let repo = SqliteOrderRepository::new(pool);

        let order = create_test_order();
        repo.save(&order).await.unwrap();

        let found = repo.find_by_id(&order.id).await.unwrap();
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.symbol, "BTCUSDT");
        assert_eq!(found.side, OrderSide::Buy);
        assert_eq!(found.order_type, OrderType::Market);
    }

    #[tokio::test]
    async fn test_find_open_orders() {
        let pool = setup_test_db().await;
        let repo = SqliteOrderRepository::new(pool);

        let order = create_test_order();
        repo.save(&order).await.unwrap();

        let open = repo.find_open_orders(None, None).await.unwrap();
        assert_eq!(open.len(), 1);

        let open = repo
            .find_open_orders(Some(ExchangeId::BinanceFutures), Some("BTCUSDT"))
            .await
            .unwrap();
        assert_eq!(open.len(), 1);
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = setup_test_db().await;
        let repo = SqliteOrderRepository::new(pool);

        let order = create_test_order();
        repo.save(&order).await.unwrap();

        repo.update_status(&order.id, OrderStatus::Filled, dec!(0.1), Some(dec!(50000)))
            .await
            .unwrap();

        let found = repo.find_by_id(&order.id).await.unwrap().unwrap();
        assert_eq!(found.status, OrderStatus::Filled);
        assert_eq!(found.filled_quantity, dec!(0.1));
        assert_eq!(found.average_fill_price, Some(dec!(50000)));
    }
}
