//! Repositório SQLite para Cost Basis Lots

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::entities::{
    AcquisitionType, CostBasisLot, CostBasisLotId, CostBasisMethod,
};
use robotrade_core::error::{InfraError, InfraResult};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::debug;

/// Trait para operações de repositório de Cost Basis
#[async_trait]
pub trait CostBasisRepository: Send + Sync {
    /// Insere um novo lote
    async fn insert(&self, lot: &CostBasisLot) -> InfraResult<()>;

    /// Insere múltiplos lotes em batch
    async fn insert_batch(&self, lots: &[CostBasisLot]) -> InfraResult<()>;

    /// Busca lote por ID
    async fn find_by_id(&self, id: &CostBasisLotId) -> InfraResult<Option<CostBasisLot>>;

    /// Busca lotes abertos para um ativo (ordenados por data de aquisição)
    async fn find_open_lots(
        &self,
        exchange_id: &str,
        asset: &str,
        method: CostBasisMethod,
    ) -> InfraResult<Vec<CostBasisLot>>;

    /// Busca todos os lotes de um ativo
    async fn find_by_asset(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Vec<CostBasisLot>>;

    /// Atualiza um lote (após consumo parcial)
    async fn update(&self, lot: &CostBasisLot) -> InfraResult<()>;

    /// Fecha um lote (quando totalmente consumido)
    async fn close_lot(&self, id: &CostBasisLotId, disposal_ref: Option<String>) -> InfraResult<()>;

    /// Calcula custo médio ponderado de um ativo
    async fn get_average_cost(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Option<Decimal>>;

    /// Retorna quantidade total em lotes abertos
    async fn get_total_open_quantity(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Decimal>;

    /// Retorna custo total dos lotes abertos
    async fn get_total_open_cost(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Decimal>;

    /// Deleta todos os lotes de uma exchange
    async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64>;

    /// Conta lotes abertos
    async fn count_open(&self, exchange_id: &str, asset: Option<&str>) -> InfraResult<u64>;
}

/// Implementação SQLite do repositório de Cost Basis
pub struct SqliteCostBasisRepository {
    pool: SqlitePool,
}

impl SqliteCostBasisRepository {
    /// Cria uma nova instância do repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CostBasisRepository for SqliteCostBasisRepository {
    async fn insert(&self, lot: &CostBasisLot) -> InfraResult<()> {
        debug!(
            lot_id = %lot.id,
            exchange_id = %lot.exchange_id,
            asset = %lot.asset,
            quantity = %lot.quantity,
            cost_per_unit = %lot.cost_per_unit,
            "Inserindo lote de cost basis"
        );

        sqlx::query(
            r#"
            INSERT INTO cost_basis_lots (
                id, exchange_id, asset, symbol, quantity, remaining_quantity,
                cost_per_unit, total_cost, fee_included, cost_basis_method,
                acquisition_type, acquisition_date, acquisition_timestamp,
                reference_id, ledger_entry_id, is_closed, closed_at,
                disposal_reference_id, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(lot.id.0.as_str())
        .bind(&lot.exchange_id)
        .bind(&lot.asset)
        .bind(&lot.symbol)
        .bind(lot.quantity.to_string())
        .bind(lot.remaining_quantity.to_string())
        .bind(lot.cost_per_unit.to_string())
        .bind(lot.total_cost.to_string())
        .bind(lot.fee_included.to_string())
        .bind(lot.cost_basis_method.to_string())
        .bind(lot.acquisition_type.to_string())
        .bind(lot.acquisition_date.timestamp())
        .bind(lot.acquisition_date.to_rfc3339())
        .bind(&lot.reference_id)
        .bind(&lot.ledger_entry_id)
        .bind(lot.is_closed as i32)
        .bind(lot.closed_at.map(|d| d.timestamp()))
        .bind(&lot.disposal_reference_id)
        .bind(lot.created_at.timestamp())
        .bind(lot.updated_at.timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao inserir cost basis lot: {}", e)))?;

        Ok(())
    }

    async fn insert_batch(&self, lots: &[CostBasisLot]) -> InfraResult<()> {
        if lots.is_empty() {
            return Ok(());
        }

        debug!(count = lots.len(), "Inserindo batch de cost basis lots");

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao iniciar transação: {}", e)))?;

        for lot in lots {
            sqlx::query(
                r#"
                INSERT INTO cost_basis_lots (
                    id, exchange_id, asset, symbol, quantity, remaining_quantity,
                    cost_per_unit, total_cost, fee_included, cost_basis_method,
                    acquisition_type, acquisition_date, acquisition_timestamp,
                    reference_id, ledger_entry_id, is_closed, closed_at,
                    disposal_reference_id, created_at, updated_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(lot.id.0.as_str())
            .bind(&lot.exchange_id)
            .bind(&lot.asset)
            .bind(&lot.symbol)
            .bind(lot.quantity.to_string())
            .bind(lot.remaining_quantity.to_string())
            .bind(lot.cost_per_unit.to_string())
            .bind(lot.total_cost.to_string())
            .bind(lot.fee_included.to_string())
            .bind(lot.cost_basis_method.to_string())
            .bind(lot.acquisition_type.to_string())
            .bind(lot.acquisition_date.timestamp())
            .bind(lot.acquisition_date.to_rfc3339())
            .bind(&lot.reference_id)
            .bind(&lot.ledger_entry_id)
            .bind(lot.is_closed as i32)
            .bind(lot.closed_at.map(|d| d.timestamp()))
            .bind(&lot.disposal_reference_id)
            .bind(lot.created_at.timestamp())
            .bind(lot.updated_at.timestamp())
            .execute(&mut *tx)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao inserir lot: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao commitar transação: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &CostBasisLotId) -> InfraResult<Option<CostBasisLot>> {
        let row = sqlx::query_as::<_, CostBasisLotRow>(
            r#"
            SELECT id, exchange_id, asset, symbol, quantity, remaining_quantity,
                   cost_per_unit, total_cost, fee_included, cost_basis_method,
                   acquisition_type, acquisition_date, reference_id, ledger_entry_id,
                   is_closed, closed_at, disposal_reference_id, created_at, updated_at
            FROM cost_basis_lots
            WHERE id = ?
            "#,
        )
        .bind(id.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar cost basis lot: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_open_lots(
        &self,
        exchange_id: &str,
        asset: &str,
        method: CostBasisMethod,
    ) -> InfraResult<Vec<CostBasisLot>> {
        // FIFO = mais antigo primeiro, LIFO = mais recente primeiro
        let order = match method {
            CostBasisMethod::Fifo => "ASC",
            CostBasisMethod::Lifo => "DESC",
            CostBasisMethod::Avg => "ASC", // Ordem não importa para average
        };

        let query = format!(
            r#"
            SELECT id, exchange_id, asset, symbol, quantity, remaining_quantity,
                   cost_per_unit, total_cost, fee_included, cost_basis_method,
                   acquisition_type, acquisition_date, reference_id, ledger_entry_id,
                   is_closed, closed_at, disposal_reference_id, created_at, updated_at
            FROM cost_basis_lots
            WHERE exchange_id = ? AND asset = ? AND is_closed = 0
            ORDER BY acquisition_date {}
            "#,
            order
        );

        let rows = sqlx::query_as::<_, CostBasisLotRow>(&query)
            .bind(exchange_id)
            .bind(asset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao buscar open lots: {}", e)))?;

        let lots: Result<Vec<CostBasisLot>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        lots
    }

    async fn find_by_asset(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Vec<CostBasisLot>> {
        let rows = sqlx::query_as::<_, CostBasisLotRow>(
            r#"
            SELECT id, exchange_id, asset, symbol, quantity, remaining_quantity,
                   cost_per_unit, total_cost, fee_included, cost_basis_method,
                   acquisition_type, acquisition_date, reference_id, ledger_entry_id,
                   is_closed, closed_at, disposal_reference_id, created_at, updated_at
            FROM cost_basis_lots
            WHERE exchange_id = ? AND asset = ?
            ORDER BY acquisition_date ASC
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar lots: {}", e)))?;

        let lots: Result<Vec<CostBasisLot>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        lots
    }

    async fn update(&self, lot: &CostBasisLot) -> InfraResult<()> {
        debug!(
            lot_id = %lot.id,
            remaining = %lot.remaining_quantity,
            is_closed = lot.is_closed,
            "Atualizando cost basis lot"
        );

        sqlx::query(
            r#"
            UPDATE cost_basis_lots
            SET remaining_quantity = ?,
                is_closed = ?,
                closed_at = ?,
                disposal_reference_id = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(lot.remaining_quantity.to_string())
        .bind(lot.is_closed as i32)
        .bind(lot.closed_at.map(|d| d.timestamp()))
        .bind(&lot.disposal_reference_id)
        .bind(Utc::now().timestamp())
        .bind(lot.id.0.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao atualizar lot: {}", e)))?;

        Ok(())
    }

    async fn close_lot(&self, id: &CostBasisLotId, disposal_ref: Option<String>) -> InfraResult<()> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE cost_basis_lots
            SET is_closed = 1,
                closed_at = ?,
                remaining_quantity = '0',
                disposal_reference_id = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(&disposal_ref)
        .bind(now)
        .bind(id.0.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao fechar lot: {}", e)))?;

        Ok(())
    }

    async fn get_average_cost(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Option<Decimal>> {
        // Calcula custo médio ponderado: SUM(remaining_qty * cost_per_unit) / SUM(remaining_qty)
        let row: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT
                CAST(SUM(CAST(remaining_quantity AS REAL) * CAST(cost_per_unit AS REAL)) AS TEXT) as total_cost,
                CAST(SUM(CAST(remaining_quantity AS REAL)) AS TEXT) as total_qty
            FROM cost_basis_lots
            WHERE exchange_id = ? AND asset = ? AND is_closed = 0
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao calcular custo médio: {}", e)))?;

        match row {
            Some((total_cost, total_qty)) => {
                let cost = Decimal::from_str(&total_cost).unwrap_or(Decimal::ZERO);
                let qty = Decimal::from_str(&total_qty).unwrap_or(Decimal::ZERO);

                if qty.is_zero() {
                    Ok(None)
                } else {
                    Ok(Some(cost / qty))
                }
            }
            None => Ok(None),
        }
    }

    async fn get_total_open_quantity(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Decimal> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT CAST(SUM(CAST(remaining_quantity AS REAL)) AS TEXT) as total
            FROM cost_basis_lots
            WHERE exchange_id = ? AND asset = ? AND is_closed = 0
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao somar quantidade: {}", e)))?;

        match row {
            Some((total,)) => Ok(Decimal::from_str(&total).unwrap_or(Decimal::ZERO)),
            None => Ok(Decimal::ZERO),
        }
    }

    async fn get_total_open_cost(
        &self,
        exchange_id: &str,
        asset: &str,
    ) -> InfraResult<Decimal> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT CAST(SUM(CAST(remaining_quantity AS REAL) * CAST(cost_per_unit AS REAL)) AS TEXT) as total
            FROM cost_basis_lots
            WHERE exchange_id = ? AND asset = ? AND is_closed = 0
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao somar custo: {}", e)))?;

        match row {
            Some((total,)) => Ok(Decimal::from_str(&total).unwrap_or(Decimal::ZERO)),
            None => Ok(Decimal::ZERO),
        }
    }

    async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64> {
        let result = sqlx::query("DELETE FROM cost_basis_lots WHERE exchange_id = ?")
            .bind(exchange_id)
            .execute(&self.pool)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao deletar lots: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn count_open(&self, exchange_id: &str, asset: Option<&str>) -> InfraResult<u64> {
        let (query, params): (String, Vec<String>) = match asset {
            Some(a) => (
                "SELECT COUNT(*) FROM cost_basis_lots WHERE exchange_id = ? AND asset = ? AND is_closed = 0".to_string(),
                vec![exchange_id.to_string(), a.to_string()],
            ),
            None => (
                "SELECT COUNT(*) FROM cost_basis_lots WHERE exchange_id = ? AND is_closed = 0".to_string(),
                vec![exchange_id.to_string()],
            ),
        };

        let mut sql_query = sqlx::query_as::<_, (i64,)>(&query);
        for param in &params {
            sql_query = sql_query.bind(param);
        }

        let row = sql_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao contar lots: {}", e)))?;

        Ok(row.0 as u64)
    }
}

/// Row do banco de dados para CostBasisLot
#[derive(sqlx::FromRow)]
struct CostBasisLotRow {
    id: String,
    exchange_id: String,
    asset: String,
    symbol: Option<String>,
    quantity: String,
    remaining_quantity: String,
    cost_per_unit: String,
    total_cost: String,
    fee_included: String,
    cost_basis_method: String,
    acquisition_type: String,
    acquisition_date: i64,
    reference_id: Option<String>,
    ledger_entry_id: Option<String>,
    is_closed: i32,
    closed_at: Option<i64>,
    disposal_reference_id: Option<String>,
    created_at: i64,
    updated_at: i64,
}

impl TryFrom<CostBasisLotRow> for CostBasisLot {
    type Error = InfraError;

    fn try_from(row: CostBasisLotRow) -> Result<Self, Self::Error> {
        let cost_basis_method = CostBasisMethod::from_str(&row.cost_basis_method)
            .map_err(|e| InfraError::Database(format!("cost_basis_method inválido: {}", e)))?;

        let acquisition_type = AcquisitionType::from_str(&row.acquisition_type)
            .map_err(|e| InfraError::Database(format!("acquisition_type inválido: {}", e)))?;

        let quantity = Decimal::from_str(&row.quantity)
            .map_err(|e| InfraError::Database(format!("quantity inválido: {}", e)))?;

        let remaining_quantity = Decimal::from_str(&row.remaining_quantity)
            .map_err(|e| InfraError::Database(format!("remaining_quantity inválido: {}", e)))?;

        let cost_per_unit = Decimal::from_str(&row.cost_per_unit)
            .map_err(|e| InfraError::Database(format!("cost_per_unit inválido: {}", e)))?;

        let total_cost = Decimal::from_str(&row.total_cost)
            .map_err(|e| InfraError::Database(format!("total_cost inválido: {}", e)))?;

        let fee_included = Decimal::from_str(&row.fee_included)
            .map_err(|e| InfraError::Database(format!("fee_included inválido: {}", e)))?;

        let acquisition_date = Utc.timestamp_opt(row.acquisition_date, 0).single().ok_or_else(|| {
            InfraError::Database("acquisition_date inválido".to_string())
        })?;

        let closed_at = row
            .closed_at
            .map(|ts| Utc.timestamp_opt(ts, 0).single())
            .flatten();

        let created_at = Utc.timestamp_opt(row.created_at, 0).single().ok_or_else(|| {
            InfraError::Database("created_at inválido".to_string())
        })?;

        let updated_at = Utc.timestamp_opt(row.updated_at, 0).single().ok_or_else(|| {
            InfraError::Database("updated_at inválido".to_string())
        })?;

        Ok(CostBasisLot {
            id: CostBasisLotId::from_string(row.id),
            exchange_id: row.exchange_id,
            asset: row.asset,
            symbol: row.symbol,
            quantity,
            remaining_quantity,
            cost_per_unit,
            total_cost,
            fee_included,
            cost_basis_method,
            acquisition_type,
            acquisition_date,
            reference_id: row.reference_id,
            ledger_entry_id: row.ledger_entry_id,
            is_closed: row.is_closed != 0,
            closed_at,
            disposal_reference_id: row.disposal_reference_id,
            created_at,
            updated_at,
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
    async fn test_insert_and_find_lot() {
        let pool = setup_test_db().await;
        let repo = SqliteCostBasisRepository::new(pool);

        let lot = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(50000),
            CostBasisMethod::Fifo,
            AcquisitionType::Buy,
            Utc::now(),
        );

        repo.insert(&lot).await.unwrap();

        let found = repo.find_by_id(&lot.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.quantity, dec!(1));
        assert_eq!(found.cost_per_unit, dec!(50000));
    }

    #[tokio::test]
    async fn test_fifo_ordering() {
        let pool = setup_test_db().await;
        let repo = SqliteCostBasisRepository::new(pool);

        // Insere lotes em ordem diferente
        let lot1 = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(40000),
            CostBasisMethod::Fifo,
            AcquisitionType::Buy,
            Utc::now() - chrono::Duration::days(10),
        );

        let lot2 = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(50000),
            CostBasisMethod::Fifo,
            AcquisitionType::Buy,
            Utc::now() - chrono::Duration::days(5),
        );

        let lot3 = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(45000),
            CostBasisMethod::Fifo,
            AcquisitionType::Buy,
            Utc::now(),
        );

        // Insere fora de ordem
        repo.insert(&lot2).await.unwrap();
        repo.insert(&lot1).await.unwrap();
        repo.insert(&lot3).await.unwrap();

        // FIFO deve retornar mais antigo primeiro
        let lots = repo
            .find_open_lots("binance", "BTC", CostBasisMethod::Fifo)
            .await
            .unwrap();

        assert_eq!(lots.len(), 3);
        assert_eq!(lots[0].cost_per_unit, dec!(40000)); // Mais antigo
        assert_eq!(lots[1].cost_per_unit, dec!(50000));
        assert_eq!(lots[2].cost_per_unit, dec!(45000)); // Mais recente
    }

    #[tokio::test]
    async fn test_average_cost_calculation() {
        let pool = setup_test_db().await;
        let repo = SqliteCostBasisRepository::new(pool);

        let lot1 = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(40000),
            CostBasisMethod::Avg,
            AcquisitionType::Buy,
            Utc::now(),
        );

        let lot2 = CostBasisLot::new(
            "binance".to_string(),
            "BTC".to_string(),
            dec!(1),
            dec!(50000),
            CostBasisMethod::Avg,
            AcquisitionType::Buy,
            Utc::now(),
        );

        repo.insert(&lot1).await.unwrap();
        repo.insert(&lot2).await.unwrap();

        let avg = repo.get_average_cost("binance", "BTC").await.unwrap();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), dec!(45000)); // (40000 + 50000) / 2
    }
}
