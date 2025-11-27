//! Repositório de Symbol
//!
//! Implementação SQLite para persistência de símbolos de trading.

use async_trait::async_trait;
use robotrade_core::{
    error::{InfraError, InfraResult},
    ExchangeId, Symbol, SymbolStatus, SymbolType,
};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::debug;

use crate::database::SymbolRow;

/// Trait para repositório de símbolos
#[async_trait]
pub trait SymbolRepository: Send + Sync {
    /// Busca símbolo por ID
    async fn find_by_id(&self, id: i64) -> InfraResult<Option<Symbol>>;

    /// Busca símbolo por exchange e nome
    async fn find_by_exchange_and_name(
        &self,
        exchange_id: ExchangeId,
        symbol: &str,
    ) -> InfraResult<Option<Symbol>>;

    /// Lista todos os símbolos de uma exchange
    async fn find_by_exchange(&self, exchange_id: ExchangeId) -> InfraResult<Vec<Symbol>>;

    /// Lista símbolos ativos
    async fn find_active(&self) -> InfraResult<Vec<Symbol>>;

    /// Salva ou atualiza símbolo
    async fn save(&self, symbol: &Symbol) -> InfraResult<i64>;

    /// Remove símbolo
    async fn delete(&self, id: i64) -> InfraResult<bool>;

    /// Conta símbolos por exchange
    async fn count_by_exchange(&self, exchange_id: ExchangeId) -> InfraResult<u64>;
}

/// Implementação SQLite do repositório de símbolos
pub struct SqliteSymbolRepository {
    pool: SqlitePool,
}

impl SqliteSymbolRepository {
    /// Cria novo repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Converte row do banco para entidade
    fn row_to_entity(row: SymbolRow) -> InfraResult<Symbol> {
        let exchange_id = parse_exchange_id(&row.exchange_id)?;
        let symbol_type = SymbolType::from_str(&row.symbol_type)
            .map_err(|e| InfraError::Database(format!("SymbolType inválido: {}", e)))?;
        let status = SymbolStatus::from_str(&row.status)
            .map_err(|e| InfraError::Database(format!("SymbolStatus inválido: {}", e)))?;

        Ok(Symbol {
            id: row.id,
            exchange_id,
            symbol: row.symbol,
            base_asset: row.base_asset,
            quote_asset: row.quote_asset,
            symbol_type,
            status,
            price_precision: row.price_precision as u8,
            quantity_precision: row.quantity_precision as u8,
            quote_precision: row.quote_precision as u8,
            min_quantity: parse_decimal(&row.min_quantity)?,
            max_quantity: row.max_quantity.map(|s| parse_decimal(&s)).transpose()?,
            min_notional: parse_decimal(&row.min_notional)?,
            max_notional: row.max_notional.map(|s| parse_decimal(&s)).transpose()?,
            tick_size: parse_decimal(&row.tick_size)?,
            step_size: parse_decimal(&row.step_size)?,
            contract_type: row.contract_type,
            contract_size: row.contract_size.map(|s| parse_decimal(&s)).transpose()?,
            margin_asset: row.margin_asset,
            maintenance_margin_rate: row.maintenance_margin_rate.map(|s| parse_decimal(&s)).transpose()?,
            max_leverage: row.max_leverage.map(|v| v as u32),
            maker_fee: parse_decimal(&row.maker_fee)?,
            taker_fee: parse_decimal(&row.taker_fee)?,
            metadata: row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
            created_at: parse_datetime(&row.created_at)?,
            updated_at: parse_datetime(&row.updated_at)?,
        })
    }
}

#[async_trait]
impl SymbolRepository for SqliteSymbolRepository {
    async fn find_by_id(&self, id: i64) -> InfraResult<Option<Symbol>> {
        let row: Option<SymbolRow> = sqlx::query_as(
            "SELECT * FROM symbols WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        row.map(Self::row_to_entity).transpose()
    }

    async fn find_by_exchange_and_name(
        &self,
        exchange_id: ExchangeId,
        symbol: &str,
    ) -> InfraResult<Option<Symbol>> {
        let row: Option<SymbolRow> = sqlx::query_as(
            "SELECT * FROM symbols WHERE exchange_id = ? AND symbol = ?"
        )
        .bind(exchange_id.to_string())
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        row.map(Self::row_to_entity).transpose()
    }

    async fn find_by_exchange(&self, exchange_id: ExchangeId) -> InfraResult<Vec<Symbol>> {
        let rows: Vec<SymbolRow> = sqlx::query_as(
            "SELECT * FROM symbols WHERE exchange_id = ? ORDER BY symbol"
        )
        .bind(exchange_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    async fn find_active(&self) -> InfraResult<Vec<Symbol>> {
        let rows: Vec<SymbolRow> = sqlx::query_as(
            "SELECT * FROM symbols WHERE status = 'active' ORDER BY exchange_id, symbol"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    async fn save(&self, symbol: &Symbol) -> InfraResult<i64> {
        debug!(symbol = %symbol.symbol, exchange = %symbol.exchange_id, "Salvando símbolo");

        let metadata_json = symbol.metadata.as_ref()
            .map(|m| serde_json::to_string(m).ok())
            .flatten();

        if symbol.id == 0 {
            // Insert
            let result = sqlx::query(r#"
                INSERT INTO symbols (
                    exchange_id, symbol, base_asset, quote_asset, symbol_type, status,
                    price_precision, quantity_precision, quote_precision,
                    min_quantity, max_quantity, min_notional, max_notional,
                    tick_size, step_size, contract_type, contract_size,
                    margin_asset, maintenance_margin_rate, max_leverage,
                    maker_fee, taker_fee, metadata
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(symbol.exchange_id.to_string())
            .bind(&symbol.symbol)
            .bind(&symbol.base_asset)
            .bind(&symbol.quote_asset)
            .bind(symbol.symbol_type.to_string())
            .bind(symbol.status.to_string())
            .bind(symbol.price_precision as i64)
            .bind(symbol.quantity_precision as i64)
            .bind(symbol.quote_precision as i64)
            .bind(symbol.min_quantity.to_string())
            .bind(symbol.max_quantity.map(|v| v.to_string()))
            .bind(symbol.min_notional.to_string())
            .bind(symbol.max_notional.map(|v| v.to_string()))
            .bind(symbol.tick_size.to_string())
            .bind(symbol.step_size.to_string())
            .bind(&symbol.contract_type)
            .bind(symbol.contract_size.map(|v| v.to_string()))
            .bind(&symbol.margin_asset)
            .bind(symbol.maintenance_margin_rate.map(|v| v.to_string()))
            .bind(symbol.max_leverage.map(|v| v as i64))
            .bind(symbol.maker_fee.to_string())
            .bind(symbol.taker_fee.to_string())
            .bind(metadata_json)
            .execute(&self.pool)
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

            Ok(result.last_insert_rowid())
        } else {
            // Update
            sqlx::query(r#"
                UPDATE symbols SET
                    exchange_id = ?, symbol = ?, base_asset = ?, quote_asset = ?,
                    symbol_type = ?, status = ?, price_precision = ?,
                    quantity_precision = ?, quote_precision = ?,
                    min_quantity = ?, max_quantity = ?, min_notional = ?, max_notional = ?,
                    tick_size = ?, step_size = ?, contract_type = ?, contract_size = ?,
                    margin_asset = ?, maintenance_margin_rate = ?, max_leverage = ?,
                    maker_fee = ?, taker_fee = ?, metadata = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
            "#)
            .bind(symbol.exchange_id.to_string())
            .bind(&symbol.symbol)
            .bind(&symbol.base_asset)
            .bind(&symbol.quote_asset)
            .bind(symbol.symbol_type.to_string())
            .bind(symbol.status.to_string())
            .bind(symbol.price_precision as i64)
            .bind(symbol.quantity_precision as i64)
            .bind(symbol.quote_precision as i64)
            .bind(symbol.min_quantity.to_string())
            .bind(symbol.max_quantity.map(|v| v.to_string()))
            .bind(symbol.min_notional.to_string())
            .bind(symbol.max_notional.map(|v| v.to_string()))
            .bind(symbol.tick_size.to_string())
            .bind(symbol.step_size.to_string())
            .bind(&symbol.contract_type)
            .bind(symbol.contract_size.map(|v| v.to_string()))
            .bind(&symbol.margin_asset)
            .bind(symbol.maintenance_margin_rate.map(|v| v.to_string()))
            .bind(symbol.max_leverage.map(|v| v as i64))
            .bind(symbol.maker_fee.to_string())
            .bind(symbol.taker_fee.to_string())
            .bind(metadata_json)
            .bind(symbol.id)
            .execute(&self.pool)
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

            Ok(symbol.id)
        }
    }

    async fn delete(&self, id: i64) -> InfraResult<bool> {
        let result = sqlx::query("DELETE FROM symbols WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_exchange(&self, exchange_id: ExchangeId) -> InfraResult<u64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM symbols WHERE exchange_id = ?"
        )
        .bind(exchange_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        Ok(row.0 as u64)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_exchange_id(s: &str) -> InfraResult<ExchangeId> {
    match s {
        "binance_futures" => Ok(ExchangeId::BinanceFutures),
        "binance_spot" => Ok(ExchangeId::BinanceSpot),
        "okx" => Ok(ExchangeId::Okx),
        "bybit" => Ok(ExchangeId::Bybit),
        "paper" => Ok(ExchangeId::Paper),
        _ => Err(InfraError::Database(format!("Exchange ID inválido: {}", s))),
    }
}

fn parse_decimal(s: &str) -> InfraResult<Decimal> {
    Decimal::from_str(s).map_err(|e| InfraError::Database(format!("Decimal inválido: {}", e)))
}

fn parse_datetime(s: &str) -> InfraResult<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
        })
        .map_err(|e| InfraError::Database(format!("Data inválida: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_exchange_id() {
        assert_eq!(parse_exchange_id("binance_futures").unwrap(), ExchangeId::BinanceFutures);
        assert_eq!(parse_exchange_id("paper").unwrap(), ExchangeId::Paper);
        assert!(parse_exchange_id("invalid").is_err());
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_decimal("123.456").unwrap(), dec!(123.456));
        assert!(parse_decimal("not_a_number").is_err());
    }
}
