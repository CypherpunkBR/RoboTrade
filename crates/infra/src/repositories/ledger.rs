//! Repositório SQLite para Ledger Entries

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::entities::{
    AssetBalanceSummary, LedgerEntry, LedgerEntryId, LedgerEntryType, LedgerFilters, ReferenceType,
};
use robotrade_core::error::{InfraError, InfraResult};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::debug;

/// Trait para operações de repositório do Ledger
#[async_trait]
pub trait LedgerRepository: Send + Sync {
    /// Insere uma nova entrada no ledger
    async fn insert(&self, entry: &LedgerEntry) -> InfraResult<()>;

    /// Insere múltiplas entradas em batch
    async fn insert_batch(&self, entries: &[LedgerEntry]) -> InfraResult<()>;

    /// Busca entrada por ID
    async fn find_by_id(&self, id: &LedgerEntryId) -> InfraResult<Option<LedgerEntry>>;

    /// Busca entrada por ID externo (para deduplicação)
    async fn find_by_external_id(
        &self,
        exchange_id: &str,
        external_id: &str,
    ) -> InfraResult<Option<LedgerEntry>>;

    /// Lista entradas com filtros
    async fn find_with_filters(
        &self,
        exchange_id: &str,
        filters: &LedgerFilters,
    ) -> InfraResult<Vec<LedgerEntry>>;

    /// Calcula saldo atual de um ativo
    async fn get_balance(&self, exchange_id: &str, asset: &str) -> InfraResult<Decimal>;

    /// Retorna todos os saldos de uma exchange
    async fn get_all_balances(&self, exchange_id: &str) -> InfraResult<HashMap<String, Decimal>>;

    /// Retorna resumo dos saldos
    async fn get_balance_summaries(&self, exchange_id: &str) -> InfraResult<Vec<AssetBalanceSummary>>;

    /// Conta entradas de uma exchange
    async fn count(&self, exchange_id: &str) -> InfraResult<u64>;

    /// Deleta todas as entradas de uma exchange (para rebuild)
    async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64>;

    /// Busca última entrada por ativo
    async fn get_last_entry(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Option<LedgerEntry>>;
}

/// Implementação SQLite do repositório de Ledger
pub struct SqliteLedgerRepository {
    pool: SqlitePool,
}

impl SqliteLedgerRepository {
    /// Cria uma nova instância do repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LedgerRepository for SqliteLedgerRepository {
    async fn insert(&self, entry: &LedgerEntry) -> InfraResult<()> {
        debug!(
            entry_id = %entry.id,
            exchange_id = %entry.exchange_id,
            entry_type = %entry.entry_type,
            asset = %entry.asset,
            amount = %entry.amount,
            "Inserindo entrada no ledger"
        );

        sqlx::query(
            r#"
            INSERT INTO ledger_entries (
                id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                reference_type, reference_id, external_id, description, metadata,
                timestamp, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.0.as_str())
        .bind(&entry.exchange_id)
        .bind(&entry.account_id)
        .bind(entry.entry_type.to_string())
        .bind(&entry.asset)
        .bind(entry.amount.to_string())
        .bind(entry.balance_after.to_string())
        .bind(entry.reference_type.map(|r| r.to_string()))
        .bind(&entry.reference_id)
        .bind(&entry.external_id)
        .bind(&entry.description)
        .bind(&entry.metadata)
        .bind(entry.timestamp.timestamp())
        .bind(entry.created_at.timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao inserir ledger entry: {}", e)))?;

        Ok(())
    }

    async fn insert_batch(&self, entries: &[LedgerEntry]) -> InfraResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        debug!(
            count = entries.len(),
            "Inserindo batch de entradas no ledger"
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao iniciar transação: {}", e)))?;

        for entry in entries {
            sqlx::query(
                r#"
                INSERT INTO ledger_entries (
                    id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                    reference_type, reference_id, external_id, description, metadata,
                    timestamp, created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(external_id) DO NOTHING
                "#,
            )
            .bind(entry.id.0.as_str())
            .bind(&entry.exchange_id)
            .bind(&entry.account_id)
            .bind(entry.entry_type.to_string())
            .bind(&entry.asset)
            .bind(entry.amount.to_string())
            .bind(entry.balance_after.to_string())
            .bind(entry.reference_type.map(|r| r.to_string()))
            .bind(&entry.reference_id)
            .bind(&entry.external_id)
            .bind(&entry.description)
            .bind(&entry.metadata)
            .bind(entry.timestamp.timestamp())
            .bind(entry.created_at.timestamp())
            .execute(&mut *tx)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao inserir ledger entry: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao commitar transação: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &LedgerEntryId) -> InfraResult<Option<LedgerEntry>> {
        let row = sqlx::query_as::<_, LedgerEntryRow>(
            r#"
            SELECT id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                   reference_type, reference_id, external_id, description, metadata,
                   timestamp, created_at
            FROM ledger_entries
            WHERE id = ?
            "#,
        )
        .bind(id.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar ledger entry: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_by_external_id(
        &self,
        exchange_id: &str,
        external_id: &str,
    ) -> InfraResult<Option<LedgerEntry>> {
        let row = sqlx::query_as::<_, LedgerEntryRow>(
            r#"
            SELECT id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                   reference_type, reference_id, external_id, description, metadata,
                   timestamp, created_at
            FROM ledger_entries
            WHERE exchange_id = ? AND external_id = ?
            "#,
        )
        .bind(exchange_id)
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar ledger entry: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_with_filters(
        &self,
        exchange_id: &str,
        filters: &LedgerFilters,
    ) -> InfraResult<Vec<LedgerEntry>> {
        // Constrói query dinâmica
        let mut query = String::from(
            r#"
            SELECT id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                   reference_type, reference_id, external_id, description, metadata,
                   timestamp, created_at
            FROM ledger_entries
            WHERE exchange_id = ?
            "#,
        );

        let mut params: Vec<String> = vec![exchange_id.to_string()];

        if let Some(ref asset) = filters.asset {
            query.push_str(" AND asset = ?");
            params.push(asset.clone());
        }

        if let Some(ref entry_types) = filters.entry_types {
            if !entry_types.is_empty() {
                let placeholders: Vec<&str> = entry_types.iter().map(|_| "?").collect();
                query.push_str(&format!(" AND entry_type IN ({})", placeholders.join(",")));
                for et in entry_types {
                    params.push(et.to_string());
                }
            }
        }

        if let Some(ref ref_type) = filters.reference_type {
            query.push_str(" AND reference_type = ?");
            params.push(ref_type.to_string());
        }

        if let Some(start) = filters.start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(start.timestamp().to_string());
        }

        if let Some(end) = filters.end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(end.timestamp().to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Executa query construída dinamicamente
        let mut sql_query = sqlx::query_as::<_, LedgerEntryRow>(&query);
        for param in &params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao buscar ledger entries: {}", e)))?;

        let entries: Result<Vec<LedgerEntry>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        entries
    }

    async fn get_balance(&self, exchange_id: &str, asset: &str) -> InfraResult<Decimal> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT CAST(SUM(CAST(amount AS REAL)) AS TEXT) as balance
            FROM ledger_entries
            WHERE exchange_id = ? AND asset = ?
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao calcular saldo: {}", e)))?;

        match row {
            Some((balance,)) => {
                Decimal::from_str(&balance).map_err(|e| {
                    InfraError::Database(format!("Erro ao parsear saldo: {}", e))
                })
            }
            None => Ok(Decimal::ZERO),
        }
    }

    async fn get_all_balances(&self, exchange_id: &str) -> InfraResult<HashMap<String, Decimal>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT asset, CAST(SUM(CAST(amount AS REAL)) AS TEXT) as balance
            FROM ledger_entries
            WHERE exchange_id = ?
            GROUP BY asset
            HAVING balance != 0
            "#,
        )
        .bind(exchange_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar saldos: {}", e)))?;

        let mut balances = HashMap::new();
        for (asset, balance) in rows {
            let dec = Decimal::from_str(&balance).unwrap_or(Decimal::ZERO);
            if !dec.is_zero() {
                balances.insert(asset, dec);
            }
        }

        Ok(balances)
    }

    async fn get_balance_summaries(&self, exchange_id: &str) -> InfraResult<Vec<AssetBalanceSummary>> {
        let rows: Vec<(String, String, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                asset,
                CAST(SUM(CAST(amount AS REAL)) AS TEXT) as balance,
                COUNT(*) as entry_count,
                MAX(timestamp) as last_update
            FROM ledger_entries
            WHERE exchange_id = ?
            GROUP BY asset
            HAVING balance != 0
            "#,
        )
        .bind(exchange_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar resumo de saldos: {}", e)))?;

        let summaries = rows
            .into_iter()
            .map(|(asset, balance, entry_count, last_update)| {
                AssetBalanceSummary {
                    asset,
                    balance: Decimal::from_str(&balance).unwrap_or(Decimal::ZERO),
                    entry_count: entry_count as u64,
                    last_update: Utc.timestamp_opt(last_update, 0).unwrap(),
                }
            })
            .collect();

        Ok(summaries)
    }

    async fn count(&self, exchange_id: &str) -> InfraResult<u64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM ledger_entries WHERE exchange_id = ?",
        )
        .bind(exchange_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao contar entradas: {}", e)))?;

        Ok(row.0 as u64)
    }

    async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64> {
        let result = sqlx::query("DELETE FROM ledger_entries WHERE exchange_id = ?")
            .bind(exchange_id)
            .execute(&self.pool)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao deletar entradas: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn get_last_entry(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Option<LedgerEntry>> {
        let row = sqlx::query_as::<_, LedgerEntryRow>(
            r#"
            SELECT id, exchange_id, account_id, entry_type, asset, amount, balance_after,
                   reference_type, reference_id, external_id, description, metadata,
                   timestamp, created_at
            FROM ledger_entries
            WHERE exchange_id = ? AND asset = ?
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar última entrada: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }
}

/// Row do banco de dados para LedgerEntry
#[derive(sqlx::FromRow)]
struct LedgerEntryRow {
    id: String,
    exchange_id: String,
    account_id: Option<String>,
    entry_type: String,
    asset: String,
    amount: String,
    balance_after: String,
    reference_type: Option<String>,
    reference_id: Option<String>,
    external_id: Option<String>,
    description: Option<String>,
    metadata: Option<String>,
    timestamp: i64,
    created_at: i64,
}

impl TryFrom<LedgerEntryRow> for LedgerEntry {
    type Error = InfraError;

    fn try_from(row: LedgerEntryRow) -> Result<Self, Self::Error> {
        let entry_type = LedgerEntryType::from_str(&row.entry_type)
            .map_err(|e| InfraError::Database(format!("entry_type inválido: {}", e)))?;

        let reference_type = row
            .reference_type
            .map(|s| ReferenceType::from_str(&s))
            .transpose()
            .map_err(|e| InfraError::Database(format!("reference_type inválido: {}", e)))?;

        let amount = Decimal::from_str(&row.amount)
            .map_err(|e| InfraError::Database(format!("amount inválido: {}", e)))?;

        let balance_after = Decimal::from_str(&row.balance_after)
            .map_err(|e| InfraError::Database(format!("balance_after inválido: {}", e)))?;

        let timestamp = Utc.timestamp_opt(row.timestamp, 0).single().ok_or_else(|| {
            InfraError::Database("timestamp inválido".to_string())
        })?;

        let created_at = Utc.timestamp_opt(row.created_at, 0).single().ok_or_else(|| {
            InfraError::Database("created_at inválido".to_string())
        })?;

        Ok(LedgerEntry {
            id: LedgerEntryId::from_string(row.id),
            exchange_id: row.exchange_id,
            account_id: row.account_id,
            entry_type,
            asset: row.asset,
            amount,
            balance_after,
            reference_type,
            reference_id: row.reference_id,
            external_id: row.external_id,
            description: row.description,
            metadata: row.metadata,
            timestamp,
            created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{init_database, DatabaseConfig};
    use rust_decimal_macros::dec;
    use tempfile::tempdir;

    async fn setup_test_db() -> SqlitePool {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let config = DatabaseConfig {
            path: db_path.to_string_lossy().to_string(),
            max_connections: 1,
            connect_timeout_secs: 5,
            create_if_missing: true,
        };

        init_database(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_insert_and_find_entry() {
        let pool = setup_test_db().await;
        let repo = SqliteLedgerRepository::new(pool);

        let entry = LedgerEntry::new(
            "binance".to_string(),
            LedgerEntryType::Deposit,
            "USDT".to_string(),
            dec!(1000),
            dec!(1000),
            Utc::now(),
        )
        .with_external_id("dep123".to_string());

        repo.insert(&entry).await.unwrap();

        let found = repo.find_by_id(&entry.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.amount, dec!(1000));
        assert_eq!(found.asset, "USDT");
    }

    #[tokio::test]
    async fn test_calculate_balance() {
        let pool = setup_test_db().await;
        let repo = SqliteLedgerRepository::new(pool);

        // Depósito
        let entry1 = LedgerEntry::new(
            "binance".to_string(),
            LedgerEntryType::Deposit,
            "USDT".to_string(),
            dec!(1000),
            dec!(1000),
            Utc::now(),
        )
        .with_external_id("dep1".to_string());

        // Trade loss
        let entry2 = LedgerEntry::new(
            "binance".to_string(),
            LedgerEntryType::TradePnl,
            "USDT".to_string(),
            dec!(-100),
            dec!(900),
            Utc::now(),
        )
        .with_external_id("trade1".to_string());

        // Fee
        let entry3 = LedgerEntry::new(
            "binance".to_string(),
            LedgerEntryType::Fee,
            "USDT".to_string(),
            dec!(-10),
            dec!(890),
            Utc::now(),
        )
        .with_external_id("fee1".to_string());

        repo.insert(&entry1).await.unwrap();
        repo.insert(&entry2).await.unwrap();
        repo.insert(&entry3).await.unwrap();

        let balance = repo.get_balance("binance", "USDT").await.unwrap();
        assert_eq!(balance, dec!(890));
    }

    #[tokio::test]
    async fn test_deduplication() {
        let pool = setup_test_db().await;
        let repo = SqliteLedgerRepository::new(pool);

        let entry = LedgerEntry::new(
            "binance".to_string(),
            LedgerEntryType::Deposit,
            "USDT".to_string(),
            dec!(1000),
            dec!(1000),
            Utc::now(),
        )
        .with_external_id("dep123".to_string());

        repo.insert(&entry).await.unwrap();

        // Tenta inserir de novo via batch (com ON CONFLICT DO NOTHING)
        repo.insert_batch(&[entry.clone()]).await.unwrap();

        let count = repo.count("binance").await.unwrap();
        assert_eq!(count, 1);
    }
}
