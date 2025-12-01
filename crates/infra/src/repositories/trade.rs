//! Repositório SQLite para Trades
//!
//! Persistência de histórico de trades executados.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use tracing::debug;

use robotrade_core::{
    ExchangeId, OrderId, PositionId, PositionSide, SignalId, Trade, TradeCloseReason, TradeId,
    TradeStats,
};

/// Trait para repositório de trades
#[async_trait]
pub trait TradeRepository: Send + Sync {
    /// Salva um trade
    async fn save(&self, trade: &Trade) -> Result<(), String>;

    /// Busca trade por ID
    async fn find_by_id(&self, id: &TradeId) -> Result<Option<Trade>, String>;

    /// Lista trades por período
    async fn find_by_period(
        &self,
        exchange: Option<ExchangeId>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>, String>;

    /// Lista trades por símbolo
    async fn find_by_symbol(
        &self,
        exchange: ExchangeId,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Trade>, String>;

    /// Calcula estatísticas de trades
    async fn get_stats(
        &self,
        exchange: Option<ExchangeId>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TradeStats, String>;

    /// Deleta trades antigos (cleanup)
    async fn delete_old_trades(&self, before: DateTime<Utc>) -> Result<u64, String>;
}

/// Implementação SQLite do repositório de trades
pub struct SqliteTradeRepository {
    pool: SqlitePool,
}

impl SqliteTradeRepository {
    /// Cria um novo repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inicializa a tabela
    pub async fn init(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                exchange TEXT NOT NULL,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                position_id TEXT NOT NULL,
                signal_id TEXT,
                entry_order_id TEXT NOT NULL,
                exit_order_id TEXT NOT NULL,
                quantity TEXT NOT NULL,
                entry_price TEXT NOT NULL,
                exit_price TEXT NOT NULL,
                gross_pnl TEXT NOT NULL,
                total_fees TEXT NOT NULL,
                net_pnl TEXT NOT NULL,
                pnl_pct TEXT NOT NULL,
                roi_pct TEXT NOT NULL,
                leverage INTEGER NOT NULL,
                close_reason TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                entered_at TEXT NOT NULL,
                exited_at TEXT NOT NULL,
                metadata TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create trades table: {}", e))?;

        // Índices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_exchange ON trades(exchange)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_exited_at ON trades(exited_at)")
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }
}

#[derive(FromRow)]
struct TradeRow {
    id: String,
    exchange: String,
    symbol: String,
    side: String,
    position_id: String,
    signal_id: Option<String>,
    entry_order_id: String,
    exit_order_id: String,
    quantity: String,
    entry_price: String,
    exit_price: String,
    gross_pnl: String,
    total_fees: String,
    net_pnl: String,
    pnl_pct: String,
    roi_pct: String,
    leverage: i64,
    close_reason: String,
    duration_seconds: i64,
    entered_at: String,
    exited_at: String,
    metadata: Option<String>,
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

fn parse_position_side(s: &str) -> Result<PositionSide, String> {
    match s {
        "long" => Ok(PositionSide::Long),
        "short" => Ok(PositionSide::Short),
        _ => Err(format!("Invalid position side: {}", s)),
    }
}

fn parse_trade_id(s: &str) -> Result<TradeId, String> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid trade id: {}", e))?;
    Ok(TradeId(uuid))
}

fn parse_position_id(s: &str) -> Result<PositionId, String> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid position id: {}", e))?;
    Ok(PositionId(uuid))
}

fn parse_signal_id(s: &str) -> Result<SignalId, String> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid signal id: {}", e))?;
    Ok(SignalId(uuid))
}

fn parse_order_id(s: &str) -> Result<OrderId, String> {
    OrderId::from_string(s).map_err(|e| format!("Invalid order id: {}", e))
}

fn parse_close_reason(s: &str) -> Result<TradeCloseReason, String> {
    // TradeCloseReason is a complex enum, we serialize it as JSON
    serde_json::from_str(s).map_err(|e| format!("Invalid close reason: {}", e))
}

fn close_reason_to_string(reason: &TradeCloseReason) -> Result<String, String> {
    serde_json::to_string(reason).map_err(|e| format!("Failed to serialize close reason: {}", e))
}

impl TryFrom<TradeRow> for Trade {
    type Error = String;

    fn try_from(row: TradeRow) -> Result<Self, Self::Error> {
        Ok(Trade {
            id: parse_trade_id(&row.id)?,
            exchange: parse_exchange_id(&row.exchange)?,
            symbol: row.symbol,
            side: parse_position_side(&row.side)?,
            position_id: parse_position_id(&row.position_id)?,
            signal_id: row.signal_id.map(|s| parse_signal_id(&s)).transpose()?,
            entry_order_id: parse_order_id(&row.entry_order_id)?,
            exit_order_id: parse_order_id(&row.exit_order_id)?,
            quantity: row
                .quantity
                .parse()
                .map_err(|e| format!("Invalid quantity: {}", e))?,
            entry_price: row
                .entry_price
                .parse()
                .map_err(|e| format!("Invalid entry price: {}", e))?,
            exit_price: row
                .exit_price
                .parse()
                .map_err(|e| format!("Invalid exit price: {}", e))?,
            gross_pnl: row
                .gross_pnl
                .parse()
                .map_err(|e| format!("Invalid gross pnl: {}", e))?,
            total_fees: row
                .total_fees
                .parse()
                .map_err(|e| format!("Invalid total fees: {}", e))?,
            net_pnl: row
                .net_pnl
                .parse()
                .map_err(|e| format!("Invalid net pnl: {}", e))?,
            pnl_pct: row
                .pnl_pct
                .parse()
                .map_err(|e| format!("Invalid pnl pct: {}", e))?,
            roi_pct: row
                .roi_pct
                .parse()
                .map_err(|e| format!("Invalid roi pct: {}", e))?,
            leverage: row.leverage as u32,
            close_reason: parse_close_reason(&row.close_reason)?,
            duration_seconds: row.duration_seconds,
            entered_at: row
                .entered_at
                .parse()
                .map_err(|e| format!("Invalid entered_at: {}", e))?,
            exited_at: row
                .exited_at
                .parse()
                .map_err(|e| format!("Invalid exited_at: {}", e))?,
            metadata: row
                .metadata
                .map(|m| serde_json::from_str(&m))
                .transpose()
                .map_err(|e| format!("Invalid metadata: {}", e))?,
        })
    }
}

#[async_trait]
impl TradeRepository for SqliteTradeRepository {
    async fn save(&self, trade: &Trade) -> Result<(), String> {
        let close_reason_json = close_reason_to_string(&trade.close_reason)?;
        let metadata = trade
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO trades (
                id, exchange, symbol, side, position_id, signal_id,
                entry_order_id, exit_order_id, quantity, entry_price, exit_price,
                gross_pnl, total_fees, net_pnl, pnl_pct, roi_pct,
                leverage, close_reason, duration_seconds, entered_at, exited_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(trade.id.to_string())
        .bind(exchange_id_to_string(trade.exchange))
        .bind(&trade.symbol)
        .bind(trade.side.to_string())
        .bind(trade.position_id.to_string())
        .bind(trade.signal_id.as_ref().map(|id| id.to_string()))
        .bind(trade.entry_order_id.to_string())
        .bind(trade.exit_order_id.to_string())
        .bind(trade.quantity.to_string())
        .bind(trade.entry_price.to_string())
        .bind(trade.exit_price.to_string())
        .bind(trade.gross_pnl.to_string())
        .bind(trade.total_fees.to_string())
        .bind(trade.net_pnl.to_string())
        .bind(trade.pnl_pct.to_string())
        .bind(trade.roi_pct.to_string())
        .bind(trade.leverage as i64)
        .bind(&close_reason_json)
        .bind(trade.duration_seconds)
        .bind(trade.entered_at.to_rfc3339())
        .bind(trade.exited_at.to_rfc3339())
        .bind(&metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save trade: {}", e))?;

        debug!(trade_id = %trade.id, "Trade saved");
        Ok(())
    }

    async fn find_by_id(&self, id: &TradeId) -> Result<Option<Trade>, String> {
        let row: Option<TradeRow> = sqlx::query_as("SELECT * FROM trades WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find trade: {}", e))?;

        row.map(Trade::try_from).transpose()
    }

    async fn find_by_period(
        &self,
        exchange: Option<ExchangeId>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>, String> {
        let mut query =
            String::from("SELECT * FROM trades WHERE exited_at >= ? AND exited_at <= ?");

        if exchange.is_some() {
            query.push_str(" AND exchange = ?");
        }

        query.push_str(" ORDER BY exited_at DESC");

        let mut q = sqlx::query_as::<_, TradeRow>(&query)
            .bind(start.to_rfc3339())
            .bind(end.to_rfc3339());

        if let Some(ex) = exchange {
            q = q.bind(exchange_id_to_string(ex));
        }

        let rows: Vec<TradeRow> = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find trades by period: {}", e))?;

        rows.into_iter().map(Trade::try_from).collect()
    }

    async fn find_by_symbol(
        &self,
        exchange: ExchangeId,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Trade>, String> {
        let rows: Vec<TradeRow> = sqlx::query_as(
            "SELECT * FROM trades WHERE exchange = ? AND symbol = ? ORDER BY exited_at DESC LIMIT ?",
        )
        .bind(exchange_id_to_string(exchange))
        .bind(symbol)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find trades by symbol: {}", e))?;

        rows.into_iter().map(Trade::try_from).collect()
    }

    async fn get_stats(
        &self,
        exchange: Option<ExchangeId>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TradeStats, String> {
        let trades = self.find_by_period(exchange, start, end).await?;
        Ok(TradeStats::from_trades(&trades))
    }

    async fn delete_old_trades(&self, before: DateTime<Utc>) -> Result<u64, String> {
        let result = sqlx::query("DELETE FROM trades WHERE exited_at < ?")
            .bind(before.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete old trades: {}", e))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let repo = SqliteTradeRepository::new(pool.clone());
        repo.init().await.unwrap();
        pool
    }

    fn create_test_trade(net_pnl: Decimal) -> Trade {
        Trade::from_closed_position(
            ExchangeId::BinanceFutures,
            "BTCUSDT".to_string(),
            PositionSide::Long,
            PositionId::new(),
            None,
            OrderId::new(),
            OrderId::new(),
            dec!(0.1),
            dec!(50000),
            if net_pnl > Decimal::ZERO {
                dec!(51000)
            } else {
                dec!(49000)
            },
            dec!(5),
            5,
            TradeCloseReason::TakeProfit,
            Utc::now() - chrono::Duration::hours(1),
        )
    }

    #[tokio::test]
    async fn test_save_and_find_trade() {
        let pool = setup_test_db().await;
        let repo = SqliteTradeRepository::new(pool);

        let trade = create_test_trade(dec!(100));
        repo.save(&trade).await.unwrap();

        let found = repo.find_by_id(&trade.id).await.unwrap();
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.symbol, "BTCUSDT");
        assert_eq!(found.side, PositionSide::Long);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let pool = setup_test_db().await;
        let repo = SqliteTradeRepository::new(pool);

        // 3 winning trades, 2 losing trades
        repo.save(&create_test_trade(dec!(100))).await.unwrap();
        repo.save(&create_test_trade(dec!(50))).await.unwrap();
        repo.save(&create_test_trade(dec!(75))).await.unwrap();
        repo.save(&create_test_trade(dec!(-30))).await.unwrap();
        repo.save(&create_test_trade(dec!(-20))).await.unwrap();

        let stats = repo
            .get_stats(
                None,
                Utc::now() - chrono::Duration::days(1),
                Utc::now() + chrono::Duration::days(1),
            )
            .await
            .unwrap();

        assert_eq!(stats.total_trades, 5);
        assert_eq!(stats.winning_trades, 3);
        assert_eq!(stats.losing_trades, 2);
    }
}
