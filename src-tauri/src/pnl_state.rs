//! P&L State - Simplified Implementation
//!
//! This module provides the P&L state for calculating realized and unrealized
//! gains/losses. The actual PnL calculator integration will be implemented
//! incrementally.

use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::entities::{
    AcquisitionType as CoreAcquisitionType, CostBasisMethod as CoreCostBasisMethod,
};
use robotrade_infra::database::DbPool;
use robotrade_trading_worker::{
    pnl_calculator::{
        AcquisitionType, CostBasisMethod, PositionSide,
    },
    ws_manager::Exchange,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Tax lot representing acquired assets
#[derive(Debug, Clone)]
pub struct TaxLot {
    pub id: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub side: PositionSide,
    pub acquired_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub cost_per_unit: Decimal,
    pub total_cost: Decimal,
    pub acquired_at: DateTime<Utc>,
    pub acquisition_type: AcquisitionType,
    pub trade_id: String,
    pub is_closed: bool,
}

/// Realized P&L entry
#[derive(Debug, Clone)]
pub struct RealizedPnL {
    pub id: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub cost_basis: Decimal,
    pub gain_loss: Decimal,
    pub is_short_term: bool,
    pub is_long_term: bool,
    pub acquired_at: DateTime<Utc>,
    pub disposed_at: DateTime<Utc>,
    pub method: CostBasisMethod,
}

/// Unrealized P&L
#[derive(Debug, Clone)]
pub struct UnrealizedPnL {
    pub exchange: Exchange,
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: Decimal,
    pub cost_basis: Decimal,
    pub avg_cost_per_unit: Decimal,
    pub current_price: Decimal,
    pub current_value: Decimal,
    pub unrealized_gain_loss: Decimal,
    pub unrealized_pct: Decimal,
    pub oldest_lot_date: Option<DateTime<Utc>>,
    pub short_term_quantity: Decimal,
    pub long_term_quantity: Decimal,
}

/// P&L Summary
#[derive(Debug, Clone)]
pub struct PnLSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_realized: Decimal,
    pub short_term_gains: Decimal,
    pub short_term_losses: Decimal,
    pub long_term_gains: Decimal,
    pub long_term_losses: Decimal,
    pub net_short_term: Decimal,
    pub net_long_term: Decimal,
    pub total_proceeds: Decimal,
    pub total_cost_basis: Decimal,
    pub num_trades: u64,
}

/// State container for P&L services
pub struct PnLState {
    inner: Arc<RwLock<PnLStateInner>>,
}

struct PnLStateInner {
    pool: Option<DbPool>,
    method: CostBasisMethod,
}

impl PnLState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PnLStateInner {
                pool: None,
                method: CostBasisMethod::FIFO,
            })),
        }
    }

    /// Initialize with dependencies
    pub fn initialize(&self, pool: DbPool) {
        debug!("Initializing PnLState");
        let mut inner = futures::executor::block_on(self.inner.write());
        inner.pool = Some(pool);
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        if let Ok(inner) = self.inner.try_read() {
            inner.pool.is_some()
        } else {
            false
        }
    }

    /// Get open tax lots
    pub async fn get_open_lots(
        &self,
        exchange: Exchange,
        symbol: &str,
        side: PositionSide,
    ) -> Vec<TaxLot> {
        let inner = self.inner.read().await;
        let pool = match inner.pool.as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let exchange_id = match exchange {
            Exchange::Binance => "binance_futures",
            Exchange::Kraken => "kraken_futures",
        };
        let side_str = format!("{:?}", side);

        let rows: Vec<TaxLotRow> = sqlx::query_as(
            r#"
            SELECT id, exchange_id, symbol, side, acquired_quantity, remaining_quantity,
                   cost_per_unit, total_cost, acquired_at, acquisition_type, trade_id, is_closed
            FROM tax_lots
            WHERE exchange_id = ? AND symbol = ? AND side = ? AND is_closed = 0
            ORDER BY acquired_at ASC
            "#,
        )
        .bind(exchange_id)
        .bind(symbol)
        .bind(&side_str)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|r| r.into_tax_lot(exchange, side))
            .collect()
    }

    /// Get realized P&L
    pub async fn get_realized_pnl(
        &self,
        exchange: Option<Exchange>,
        symbol: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<RealizedPnL> {
        let inner = self.inner.read().await;
        let pool = match inner.pool.as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut query = String::from(
            "SELECT id, exchange_id, symbol, quantity, proceeds, cost_basis, gain_loss, is_short_term, is_long_term, acquired_at, disposed_at, method FROM realized_pnl WHERE disposed_at >= ? AND disposed_at <= ?"
        );

        if let Some(ex) = &exchange {
            let exchange_id = match ex {
                Exchange::Binance => "binance_futures",
                Exchange::Kraken => "kraken_futures",
            };
            query.push_str(&format!(" AND exchange_id = '{}'", exchange_id));
        }

        if let Some(sym) = symbol {
            query.push_str(&format!(" AND symbol = '{}'", sym));
        }

        query.push_str(" ORDER BY disposed_at DESC");

        let rows: Vec<RealizedPnLRow> = sqlx::query_as(&query)
            .bind(start_time.timestamp())
            .bind(end_time.timestamp())
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        rows.into_iter()
            .filter_map(|r| r.try_into_realized().ok())
            .collect()
    }

    /// Calculate unrealized P&L
    pub async fn calculate_unrealized(
        &self,
        exchange: Exchange,
        symbol: &str,
        side: PositionSide,
        current_price: Decimal,
    ) -> Option<UnrealizedPnL> {
        let lots = self.get_open_lots(exchange, symbol, side).await;

        if lots.is_empty() {
            return None;
        }

        let mut total_quantity = Decimal::ZERO;
        let mut total_cost = Decimal::ZERO;
        let mut oldest_date: Option<DateTime<Utc>> = None;
        let one_year_ago = Utc::now() - chrono::Duration::days(365);
        let mut short_term_qty = Decimal::ZERO;
        let mut long_term_qty = Decimal::ZERO;

        for lot in &lots {
            total_quantity += lot.remaining_quantity;
            total_cost += lot.cost_per_unit * lot.remaining_quantity;

            if oldest_date.is_none() || lot.acquired_at < oldest_date.unwrap() {
                oldest_date = Some(lot.acquired_at);
            }

            if lot.acquired_at < one_year_ago {
                long_term_qty += lot.remaining_quantity;
            } else {
                short_term_qty += lot.remaining_quantity;
            }
        }

        let current_value = total_quantity * current_price;
        let unrealized = if side == PositionSide::Long {
            current_value - total_cost
        } else {
            total_cost - current_value
        };

        let unrealized_pct = if total_cost != Decimal::ZERO {
            (unrealized / total_cost) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Some(UnrealizedPnL {
            exchange,
            symbol: symbol.to_string(),
            side,
            quantity: total_quantity,
            cost_basis: total_cost,
            avg_cost_per_unit: if total_quantity != Decimal::ZERO {
                total_cost / total_quantity
            } else {
                Decimal::ZERO
            },
            current_price,
            current_value,
            unrealized_gain_loss: unrealized,
            unrealized_pct,
            oldest_lot_date: oldest_date,
            short_term_quantity: short_term_qty,
            long_term_quantity: long_term_qty,
        })
    }

    /// Get P&L summary
    pub async fn get_summary(
        &self,
        exchange: Option<Exchange>,
        symbol: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> PnLSummary {
        let pnl_entries = self.get_realized_pnl(exchange, symbol, start_time, end_time).await;

        let mut summary = PnLSummary {
            period_start: start_time,
            period_end: end_time,
            total_realized: Decimal::ZERO,
            short_term_gains: Decimal::ZERO,
            short_term_losses: Decimal::ZERO,
            long_term_gains: Decimal::ZERO,
            long_term_losses: Decimal::ZERO,
            net_short_term: Decimal::ZERO,
            net_long_term: Decimal::ZERO,
            total_proceeds: Decimal::ZERO,
            total_cost_basis: Decimal::ZERO,
            num_trades: pnl_entries.len() as u64,
        };

        for entry in pnl_entries {
            summary.total_realized += entry.gain_loss;
            summary.total_proceeds += entry.proceeds;
            summary.total_cost_basis += entry.cost_basis;

            if entry.is_short_term {
                if entry.gain_loss > Decimal::ZERO {
                    summary.short_term_gains += entry.gain_loss;
                } else {
                    summary.short_term_losses += entry.gain_loss.abs();
                }
            } else {
                if entry.gain_loss > Decimal::ZERO {
                    summary.long_term_gains += entry.gain_loss;
                } else {
                    summary.long_term_losses += entry.gain_loss.abs();
                }
            }
        }

        summary.net_short_term = summary.short_term_gains - summary.short_term_losses;
        summary.net_long_term = summary.long_term_gains - summary.long_term_losses;

        summary
    }

    /// Set cost basis method
    pub async fn set_method(&self, method: CostBasisMethod) {
        let mut inner = self.inner.write().await;
        inner.method = method;
    }
}

impl Default for PnLState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PnLState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaxLotRow {
    id: String,
    exchange_id: String,
    symbol: String,
    side: String,
    acquired_quantity: String,
    remaining_quantity: String,
    cost_per_unit: String,
    total_cost: String,
    acquired_at: i64,
    acquisition_type: String,
    trade_id: String,
    is_closed: bool,
}

impl TaxLotRow {
    fn into_tax_lot(self, exchange: Exchange, side: PositionSide) -> TaxLot {
        use std::str::FromStr;

        let acquisition_type = match self.acquisition_type.as_str() {
            "Trade" => AcquisitionType::Trade,
            "Transfer" => AcquisitionType::Transfer,
            "Airdrop" => AcquisitionType::Airdrop,
            _ => AcquisitionType::Other,
        };

        TaxLot {
            id: self.id,
            exchange,
            symbol: self.symbol,
            side,
            acquired_quantity: Decimal::from_str(&self.acquired_quantity).unwrap_or(Decimal::ZERO),
            remaining_quantity: Decimal::from_str(&self.remaining_quantity).unwrap_or(Decimal::ZERO),
            cost_per_unit: Decimal::from_str(&self.cost_per_unit).unwrap_or(Decimal::ZERO),
            total_cost: Decimal::from_str(&self.total_cost).unwrap_or(Decimal::ZERO),
            acquired_at: Utc.timestamp_opt(self.acquired_at, 0).single().unwrap_or_else(Utc::now),
            acquisition_type,
            trade_id: self.trade_id,
            is_closed: self.is_closed,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RealizedPnLRow {
    id: String,
    exchange_id: String,
    symbol: String,
    quantity: String,
    proceeds: String,
    cost_basis: String,
    gain_loss: String,
    is_short_term: bool,
    is_long_term: bool,
    acquired_at: i64,
    disposed_at: i64,
    method: String,
}

impl RealizedPnLRow {
    fn try_into_realized(self) -> Result<RealizedPnL, String> {
        use std::str::FromStr;

        let exchange = match self.exchange_id.as_str() {
            "binance_futures" | "binance" => Exchange::Binance,
            "kraken_futures" | "kraken" => Exchange::Kraken,
            _ => Exchange::Binance,
        };

        let method = match self.method.as_str() {
            "FIFO" => CostBasisMethod::FIFO,
            "LIFO" => CostBasisMethod::LIFO,
            "AverageCost" | "Average" => CostBasisMethod::AverageCost,
            _ => CostBasisMethod::FIFO,
        };

        Ok(RealizedPnL {
            id: self.id,
            exchange,
            symbol: self.symbol,
            quantity: Decimal::from_str(&self.quantity).unwrap_or(Decimal::ZERO),
            proceeds: Decimal::from_str(&self.proceeds).unwrap_or(Decimal::ZERO),
            cost_basis: Decimal::from_str(&self.cost_basis).unwrap_or(Decimal::ZERO),
            gain_loss: Decimal::from_str(&self.gain_loss).unwrap_or(Decimal::ZERO),
            is_short_term: self.is_short_term,
            is_long_term: self.is_long_term,
            acquired_at: Utc.timestamp_opt(self.acquired_at, 0).single().unwrap_or_else(Utc::now),
            disposed_at: Utc.timestamp_opt(self.disposed_at, 0).single().unwrap_or_else(Utc::now),
            method,
        })
    }
}
