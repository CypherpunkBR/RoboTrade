//! Repositório SQLite para Positions
//!
//! Persistência de posições abertas.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, SqlitePool};
use tracing::debug;

use robotrade_core::{ExchangeId, OrderId, Position, PositionId, PositionSide, PositionStatus};

/// Trait para repositório de posições
#[async_trait]
pub trait PositionRepository: Send + Sync {
    /// Salva uma posição
    async fn save(&self, position: &Position) -> Result<(), String>;

    /// Busca posição por ID
    async fn find_by_id(&self, id: &PositionId) -> Result<Option<Position>, String>;

    /// Busca posição aberta por símbolo
    async fn find_open_by_symbol(
        &self,
        exchange: ExchangeId,
        symbol: &str,
    ) -> Result<Option<Position>, String>;

    /// Lista todas as posições abertas
    async fn find_all_open(&self, exchange: Option<ExchangeId>) -> Result<Vec<Position>, String>;

    /// Atualiza preço atual e P&L
    async fn update_price(
        &self,
        id: &PositionId,
        current_price: Decimal,
        unrealized_pnl: Decimal,
        unrealized_pnl_pct: Decimal,
    ) -> Result<(), String>;

    /// Fecha uma posição
    async fn close_position(
        &self,
        id: &PositionId,
        realized_pnl: Decimal,
        closed_at: DateTime<Utc>,
    ) -> Result<(), String>;

    /// Deleta posições fechadas antigas (cleanup)
    async fn delete_old_positions(&self, before: DateTime<Utc>) -> Result<u64, String>;
}

/// Implementação SQLite do repositório de posições
pub struct SqlitePositionRepository {
    pool: SqlitePool,
}

impl SqlitePositionRepository {
    /// Cria um novo repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inicializa a tabela
    pub async fn init(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS positions (
                id TEXT PRIMARY KEY,
                exchange TEXT NOT NULL,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity TEXT NOT NULL,
                entry_price TEXT NOT NULL,
                current_price TEXT NOT NULL,
                leverage INTEGER NOT NULL,
                margin TEXT NOT NULL,
                unrealized_pnl TEXT NOT NULL,
                unrealized_pnl_pct TEXT NOT NULL,
                realized_pnl TEXT NOT NULL,
                stop_loss_order_id TEXT,
                stop_loss_price TEXT,
                take_profit_order_id TEXT,
                take_profit_price TEXT,
                entry_order_id TEXT NOT NULL,
                status TEXT NOT NULL,
                liquidation_price TEXT,
                opened_at TEXT NOT NULL,
                closed_at TEXT,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create positions table: {}", e))?;

        // Índices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_positions_exchange ON positions(exchange)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status)")
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }
}

#[derive(FromRow)]
struct PositionRow {
    id: String,
    exchange: String,
    symbol: String,
    side: String,
    quantity: String,
    entry_price: String,
    current_price: String,
    leverage: i64,
    margin: String,
    unrealized_pnl: String,
    unrealized_pnl_pct: String,
    realized_pnl: String,
    stop_loss_order_id: Option<String>,
    stop_loss_price: Option<String>,
    take_profit_order_id: Option<String>,
    take_profit_price: Option<String>,
    entry_order_id: String,
    status: String,
    liquidation_price: Option<String>,
    opened_at: String,
    closed_at: Option<String>,
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

fn parse_position_side(s: &str) -> Result<PositionSide, String> {
    match s {
        "long" => Ok(PositionSide::Long),
        "short" => Ok(PositionSide::Short),
        _ => Err(format!("Invalid position side: {}", s)),
    }
}

fn parse_position_status(s: &str) -> Result<PositionStatus, String> {
    match s {
        "open" => Ok(PositionStatus::Open),
        "closed" => Ok(PositionStatus::Closed),
        "liquidated" => Ok(PositionStatus::Liquidated),
        _ => Err(format!("Invalid position status: {}", s)),
    }
}

fn position_status_to_string(status: PositionStatus) -> String {
    match status {
        PositionStatus::Open => "open".to_string(),
        PositionStatus::Closed => "closed".to_string(),
        PositionStatus::Liquidated => "liquidated".to_string(),
    }
}

fn parse_position_id(s: &str) -> Result<PositionId, String> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid position id: {}", e))?;
    Ok(PositionId(uuid))
}

fn parse_order_id(s: &str) -> Result<OrderId, String> {
    OrderId::from_string(s).map_err(|e| format!("Invalid order id: {}", e))
}

impl TryFrom<PositionRow> for Position {
    type Error = String;

    fn try_from(row: PositionRow) -> Result<Self, Self::Error> {
        Ok(Position {
            id: parse_position_id(&row.id)?,
            exchange: parse_exchange_id(&row.exchange)?,
            symbol: row.symbol,
            side: parse_position_side(&row.side)?,
            quantity: row
                .quantity
                .parse()
                .map_err(|e| format!("Invalid quantity: {}", e))?,
            entry_price: row
                .entry_price
                .parse()
                .map_err(|e| format!("Invalid entry price: {}", e))?,
            current_price: row
                .current_price
                .parse()
                .map_err(|e| format!("Invalid current price: {}", e))?,
            leverage: row.leverage as u32,
            margin: row
                .margin
                .parse()
                .map_err(|e| format!("Invalid margin: {}", e))?,
            unrealized_pnl: row
                .unrealized_pnl
                .parse()
                .map_err(|e| format!("Invalid unrealized pnl: {}", e))?,
            unrealized_pnl_pct: row
                .unrealized_pnl_pct
                .parse()
                .map_err(|e| format!("Invalid unrealized pnl pct: {}", e))?,
            realized_pnl: row
                .realized_pnl
                .parse()
                .map_err(|e| format!("Invalid realized pnl: {}", e))?,
            stop_loss_order_id: row
                .stop_loss_order_id
                .map(|s| parse_order_id(&s))
                .transpose()?,
            stop_loss_price: row
                .stop_loss_price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid stop loss price: {}", e))?,
            take_profit_order_id: row
                .take_profit_order_id
                .map(|s| parse_order_id(&s))
                .transpose()?,
            take_profit_price: row
                .take_profit_price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid take profit price: {}", e))?,
            entry_order_id: parse_order_id(&row.entry_order_id)?,
            status: parse_position_status(&row.status)?,
            liquidation_price: row
                .liquidation_price
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid liquidation price: {}", e))?,
            opened_at: row
                .opened_at
                .parse()
                .map_err(|e| format!("Invalid opened_at: {}", e))?,
            closed_at: row
                .closed_at
                .map(|t| t.parse())
                .transpose()
                .map_err(|e| format!("Invalid closed_at: {}", e))?,
            updated_at: row
                .updated_at
                .parse()
                .map_err(|e| format!("Invalid updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl PositionRepository for SqlitePositionRepository {
    async fn save(&self, position: &Position) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO positions (
                id, exchange, symbol, side, quantity, entry_price, current_price,
                leverage, margin, unrealized_pnl, unrealized_pnl_pct, realized_pnl,
                stop_loss_order_id, stop_loss_price, take_profit_order_id, take_profit_price,
                entry_order_id, status, liquidation_price, opened_at, closed_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(position.id.to_string())
        .bind(exchange_id_to_string(position.exchange))
        .bind(&position.symbol)
        .bind(position.side.to_string())
        .bind(position.quantity.to_string())
        .bind(position.entry_price.to_string())
        .bind(position.current_price.to_string())
        .bind(position.leverage as i64)
        .bind(position.margin.to_string())
        .bind(position.unrealized_pnl.to_string())
        .bind(position.unrealized_pnl_pct.to_string())
        .bind(position.realized_pnl.to_string())
        .bind(position.stop_loss_order_id.as_ref().map(|id| id.to_string()))
        .bind(position.stop_loss_price.map(|p| p.to_string()))
        .bind(position.take_profit_order_id.as_ref().map(|id| id.to_string()))
        .bind(position.take_profit_price.map(|p| p.to_string()))
        .bind(position.entry_order_id.to_string())
        .bind(position_status_to_string(position.status))
        .bind(position.liquidation_price.map(|p| p.to_string()))
        .bind(position.opened_at.to_rfc3339())
        .bind(position.closed_at.map(|t| t.to_rfc3339()))
        .bind(position.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save position: {}", e))?;

        debug!(position_id = %position.id, "Position saved");
        Ok(())
    }

    async fn find_by_id(&self, id: &PositionId) -> Result<Option<Position>, String> {
        let row: Option<PositionRow> = sqlx::query_as("SELECT * FROM positions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find position: {}", e))?;

        row.map(Position::try_from).transpose()
    }

    async fn find_open_by_symbol(
        &self,
        exchange: ExchangeId,
        symbol: &str,
    ) -> Result<Option<Position>, String> {
        let row: Option<PositionRow> = sqlx::query_as(
            "SELECT * FROM positions WHERE exchange = ? AND symbol = ? AND status = 'open'",
        )
        .bind(exchange_id_to_string(exchange))
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find open position: {}", e))?;

        row.map(Position::try_from).transpose()
    }

    async fn find_all_open(&self, exchange: Option<ExchangeId>) -> Result<Vec<Position>, String> {
        let mut query = String::from("SELECT * FROM positions WHERE status = 'open'");

        if exchange.is_some() {
            query.push_str(" AND exchange = ?");
        }

        query.push_str(" ORDER BY opened_at DESC");

        let mut q = sqlx::query_as::<_, PositionRow>(&query);

        if let Some(ex) = exchange {
            q = q.bind(exchange_id_to_string(ex));
        }

        let rows: Vec<PositionRow> = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find open positions: {}", e))?;

        rows.into_iter().map(Position::try_from).collect()
    }

    async fn update_price(
        &self,
        id: &PositionId,
        current_price: Decimal,
        unrealized_pnl: Decimal,
        unrealized_pnl_pct: Decimal,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE positions SET current_price = ?, unrealized_pnl = ?, unrealized_pnl_pct = ?, updated_at = ? WHERE id = ?",
        )
        .bind(current_price.to_string())
        .bind(unrealized_pnl.to_string())
        .bind(unrealized_pnl_pct.to_string())
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update position price: {}", e))?;

        debug!(position_id = %id, "Position price updated");
        Ok(())
    }

    async fn close_position(
        &self,
        id: &PositionId,
        realized_pnl: Decimal,
        closed_at: DateTime<Utc>,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE positions SET status = 'closed', realized_pnl = ?, closed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(realized_pnl.to_string())
        .bind(closed_at.to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to close position: {}", e))?;

        debug!(position_id = %id, "Position closed");
        Ok(())
    }

    async fn delete_old_positions(&self, before: DateTime<Utc>) -> Result<u64, String> {
        let result = sqlx::query(
            "DELETE FROM positions WHERE closed_at < ? AND status IN ('closed', 'liquidated')",
        )
        .bind(before.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old positions: {}", e))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let repo = SqlitePositionRepository::new(pool.clone());
        repo.init().await.unwrap();
        pool
    }

    fn create_test_position() -> Position {
        Position::from_entry_order(
            ExchangeId::BinanceFutures,
            "BTCUSDT".to_string(),
            PositionSide::Long,
            dec!(0.1),
            dec!(50000),
            5,
            OrderId::new(),
        )
    }

    #[tokio::test]
    async fn test_save_and_find_position() {
        let pool = setup_test_db().await;
        let repo = SqlitePositionRepository::new(pool);

        let position = create_test_position();
        repo.save(&position).await.unwrap();

        let found = repo.find_by_id(&position.id).await.unwrap();
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.symbol, "BTCUSDT");
        assert_eq!(found.side, PositionSide::Long);
        assert_eq!(found.quantity, dec!(0.1));
    }

    #[tokio::test]
    async fn test_find_open_by_symbol() {
        let pool = setup_test_db().await;
        let repo = SqlitePositionRepository::new(pool);

        let position = create_test_position();
        repo.save(&position).await.unwrap();

        let found = repo
            .find_open_by_symbol(ExchangeId::BinanceFutures, "BTCUSDT")
            .await
            .unwrap();
        assert!(found.is_some());

        let found = repo
            .find_open_by_symbol(ExchangeId::BinanceFutures, "ETHUSDT")
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_price() {
        let pool = setup_test_db().await;
        let repo = SqlitePositionRepository::new(pool);

        let position = create_test_position();
        repo.save(&position).await.unwrap();

        repo.update_price(&position.id, dec!(55000), dec!(500), dec!(10))
            .await
            .unwrap();

        let found = repo.find_by_id(&position.id).await.unwrap().unwrap();
        assert_eq!(found.current_price, dec!(55000));
        assert_eq!(found.unrealized_pnl, dec!(500));
        assert_eq!(found.unrealized_pnl_pct, dec!(10));
    }

    #[tokio::test]
    async fn test_close_position() {
        let pool = setup_test_db().await;
        let repo = SqlitePositionRepository::new(pool);

        let position = create_test_position();
        repo.save(&position).await.unwrap();

        let closed_at = Utc::now();
        repo.close_position(&position.id, dec!(100), closed_at)
            .await
            .unwrap();

        let found = repo.find_by_id(&position.id).await.unwrap().unwrap();
        assert_eq!(found.status, PositionStatus::Closed);
        assert_eq!(found.realized_pnl, dec!(100));
        assert!(found.closed_at.is_some());
    }
}
