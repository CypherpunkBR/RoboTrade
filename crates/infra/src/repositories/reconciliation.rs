//! Repositório SQLite para Reconciliation Snapshots

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::entities::{
  Discrepancy, ReconciliationSnapshot, ReconciliationSnapshotId, ReconciliationSnapshotType,
  ReconciliationStatus,
};
use robotrade_core::error::{InfraError, InfraResult};
use rust_decimal::Decimal;
use serde_json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::debug;

/// Trait para operações de repositório de Reconciliation
#[async_trait]
pub trait ReconciliationRepository: Send + Sync {
  /// Insere um novo snapshot
  async fn insert(&self, snapshot: &ReconciliationSnapshot) -> InfraResult<()>;

  /// Busca snapshot por ID
  async fn find_by_id(
    &self,
    id: &ReconciliationSnapshotId,
  ) -> InfraResult<Option<ReconciliationSnapshot>>;

  /// Busca último snapshot de uma exchange
  async fn find_latest(&self, exchange_id: &str) -> InfraResult<Option<ReconciliationSnapshot>>;

  /// Lista snapshots de uma exchange
  async fn find_by_exchange(
    &self,
    exchange_id: &str,
    limit: Option<u32>,
  ) -> InfraResult<Vec<ReconciliationSnapshot>>;

  /// Lista snapshots com discrepâncias
  async fn find_with_discrepancies(
    &self,
    exchange_id: &str,
  ) -> InfraResult<Vec<ReconciliationSnapshot>>;

  /// Lista snapshots pendentes de revisão
  async fn find_pending_review(
    &self,
    exchange_id: &str,
  ) -> InfraResult<Vec<ReconciliationSnapshot>>;

  /// Atualiza status de um snapshot
  async fn update_status(
    &self,
    id: &ReconciliationSnapshotId,
    status: ReconciliationStatus,
    reviewed_by: Option<String>,
    notes: Option<String>,
  ) -> InfraResult<()>;

  /// Conta snapshots por status
  async fn count_by_status(
    &self,
    exchange_id: &str,
    status: ReconciliationStatus,
  ) -> InfraResult<u64>;

  /// Deleta snapshots antigos
  async fn delete_old_snapshots(
    &self,
    exchange_id: &str,
    before: DateTime<Utc>,
  ) -> InfraResult<u64>;
}

/// Implementação SQLite do repositório de Reconciliation
pub struct SqliteReconciliationRepository {
  pool: SqlitePool,
}

impl SqliteReconciliationRepository {
  /// Cria uma nova instância do repositório
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }

  /// Serializa HashMap de balances para JSON
  fn serialize_balances(balances: &HashMap<String, Decimal>) -> InfraResult<String> {
    let map: HashMap<String, String> = balances
      .iter()
      .map(|(k, v)| (k.clone(), v.to_string()))
      .collect();
    serde_json::to_string(&map)
      .map_err(|e| InfraError::Database(format!("Erro ao serializar balances: {}", e)))
  }

  /// Deserializa JSON para HashMap de balances
  fn deserialize_balances(json: &str) -> InfraResult<HashMap<String, Decimal>> {
    let map: HashMap<String, String> = serde_json::from_str(json)
      .map_err(|e| InfraError::Database(format!("Erro ao deserializar balances: {}", e)))?;

    let mut result = HashMap::new();
    for (k, v) in map {
      let dec = Decimal::from_str(&v)
        .map_err(|e| InfraError::Database(format!("Decimal inválido: {}", e)))?;
      result.insert(k, dec);
    }
    Ok(result)
  }

  /// Serializa discrepâncias para JSON
  fn serialize_discrepancies(discrepancies: &[Discrepancy]) -> InfraResult<String> {
    serde_json::to_string(discrepancies)
      .map_err(|e| InfraError::Database(format!("Erro ao serializar discrepancies: {}", e)))
  }

  /// Deserializa JSON para discrepâncias
  fn deserialize_discrepancies(json: &str) -> InfraResult<Vec<Discrepancy>> {
    serde_json::from_str(json)
      .map_err(|e| InfraError::Database(format!("Erro ao deserializar discrepancies: {}", e)))
  }
}

#[async_trait]
impl ReconciliationRepository for SqliteReconciliationRepository {
  async fn insert(&self, snapshot: &ReconciliationSnapshot) -> InfraResult<()> {
    debug!(
        snapshot_id = %snapshot.id,
        exchange_id = %snapshot.exchange_id,
        status = %snapshot.status,
        discrepancy_count = snapshot.discrepancies.len(),
        "Inserindo reconciliation snapshot"
    );

    let calculated_json = Self::serialize_balances(&snapshot.calculated_balances)?;
    let reported_json = Self::serialize_balances(&snapshot.reported_balances)?;
    let discrepancies_json = Self::serialize_discrepancies(&snapshot.discrepancies)?;

    sqlx::query(
      r#"
            INSERT INTO reconciliation_snapshots (
                id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                snapshot_timestamp, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
    )
    .bind(snapshot.id.0.as_str())
    .bind(&snapshot.exchange_id)
    .bind(snapshot.snapshot_type.to_string())
    .bind(&calculated_json)
    .bind(&reported_json)
    .bind(&discrepancies_json)
    .bind(snapshot.status.to_string())
    .bind(snapshot.reviewed_at.map(|d| d.timestamp()))
    .bind(&snapshot.reviewed_by)
    .bind(&snapshot.resolution_notes)
    .bind(snapshot.snapshot_timestamp.timestamp())
    .bind(snapshot.created_at.timestamp())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao inserir snapshot: {}", e)))?;

    Ok(())
  }

  async fn find_by_id(
    &self,
    id: &ReconciliationSnapshotId,
  ) -> InfraResult<Option<ReconciliationSnapshot>> {
    let row = sqlx::query_as::<_, ReconciliationSnapshotRow>(
      r#"
            SELECT id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                   discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                   snapshot_timestamp, created_at
            FROM reconciliation_snapshots
            WHERE id = ?
            "#,
    )
    .bind(id.0.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar snapshot: {}", e)))?;

    match row {
      Some(r) => Ok(Some(Self::row_to_snapshot(r)?)),
      None => Ok(None),
    }
  }

  async fn find_latest(&self, exchange_id: &str) -> InfraResult<Option<ReconciliationSnapshot>> {
    let row = sqlx::query_as::<_, ReconciliationSnapshotRow>(
      r#"
            SELECT id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                   discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                   snapshot_timestamp, created_at
            FROM reconciliation_snapshots
            WHERE exchange_id = ?
            ORDER BY snapshot_timestamp DESC
            LIMIT 1
            "#,
    )
    .bind(exchange_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar último snapshot: {}", e)))?;

    match row {
      Some(r) => Ok(Some(Self::row_to_snapshot(r)?)),
      None => Ok(None),
    }
  }

  async fn find_by_exchange(
    &self,
    exchange_id: &str,
    limit: Option<u32>,
  ) -> InfraResult<Vec<ReconciliationSnapshot>> {
    let limit = limit.unwrap_or(100) as i64;

    let rows = sqlx::query_as::<_, ReconciliationSnapshotRow>(
      r#"
            SELECT id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                   discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                   snapshot_timestamp, created_at
            FROM reconciliation_snapshots
            WHERE exchange_id = ?
            ORDER BY snapshot_timestamp DESC
            LIMIT ?
            "#,
    )
    .bind(exchange_id)
    .bind(limit)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar snapshots: {}", e)))?;

    let snapshots: Result<Vec<_>, _> = rows.into_iter().map(Self::row_to_snapshot).collect();
    snapshots
  }

  async fn find_with_discrepancies(
    &self,
    exchange_id: &str,
  ) -> InfraResult<Vec<ReconciliationSnapshot>> {
    let rows = sqlx::query_as::<_, ReconciliationSnapshotRow>(
      r#"
            SELECT id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                   discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                   snapshot_timestamp, created_at
            FROM reconciliation_snapshots
            WHERE exchange_id = ? AND status = 'DISCREPANCY'
            ORDER BY snapshot_timestamp DESC
            "#,
    )
    .bind(exchange_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar snapshots: {}", e)))?;

    let snapshots: Result<Vec<_>, _> = rows.into_iter().map(Self::row_to_snapshot).collect();
    snapshots
  }

  async fn find_pending_review(
    &self,
    exchange_id: &str,
  ) -> InfraResult<Vec<ReconciliationSnapshot>> {
    let rows = sqlx::query_as::<_, ReconciliationSnapshotRow>(
      r#"
            SELECT id, exchange_id, snapshot_type, calculated_balances, reported_balances,
                   discrepancies, status, reviewed_at, reviewed_by, resolution_notes,
                   snapshot_timestamp, created_at
            FROM reconciliation_snapshots
            WHERE exchange_id = ? AND status IN ('DISCREPANCY', 'PENDING_REVIEW')
            ORDER BY snapshot_timestamp DESC
            "#,
    )
    .bind(exchange_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar snapshots: {}", e)))?;

    let snapshots: Result<Vec<_>, _> = rows.into_iter().map(Self::row_to_snapshot).collect();
    snapshots
  }

  async fn update_status(
    &self,
    id: &ReconciliationSnapshotId,
    status: ReconciliationStatus,
    reviewed_by: Option<String>,
    notes: Option<String>,
  ) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    sqlx::query(
      r#"
            UPDATE reconciliation_snapshots
            SET status = ?,
                reviewed_at = ?,
                reviewed_by = COALESCE(?, reviewed_by),
                resolution_notes = COALESCE(?, resolution_notes)
            WHERE id = ?
            "#,
    )
    .bind(status.to_string())
    .bind(now)
    .bind(&reviewed_by)
    .bind(&notes)
    .bind(id.0.as_str())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao atualizar status: {}", e)))?;

    Ok(())
  }

  async fn count_by_status(
    &self,
    exchange_id: &str,
    status: ReconciliationStatus,
  ) -> InfraResult<u64> {
    let row: (i64,) = sqlx::query_as(
      "SELECT COUNT(*) FROM reconciliation_snapshots WHERE exchange_id = ? AND status = ?",
    )
    .bind(exchange_id)
    .bind(status.to_string())
    .fetch_one(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao contar snapshots: {}", e)))?;

    Ok(row.0 as u64)
  }

  async fn delete_old_snapshots(
    &self,
    exchange_id: &str,
    before: DateTime<Utc>,
  ) -> InfraResult<u64> {
    let result = sqlx::query(
            r#"
            DELETE FROM reconciliation_snapshots
            WHERE exchange_id = ? AND snapshot_timestamp < ? AND status IN ('MATCHED', 'RESOLVED', 'IGNORED')
            "#,
        )
        .bind(exchange_id)
        .bind(before.timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao deletar snapshots: {}", e)))?;

    Ok(result.rows_affected())
  }
}

impl SqliteReconciliationRepository {
  fn row_to_snapshot(row: ReconciliationSnapshotRow) -> InfraResult<ReconciliationSnapshot> {
    let snapshot_type = ReconciliationSnapshotType::from_str(&row.snapshot_type)
      .map_err(|e| InfraError::Database(format!("snapshot_type inválido: {}", e)))?;

    let status = ReconciliationStatus::from_str(&row.status)
      .map_err(|e| InfraError::Database(format!("status inválido: {}", e)))?;

    let calculated_balances = Self::deserialize_balances(&row.calculated_balances)?;
    let reported_balances = Self::deserialize_balances(&row.reported_balances)?;
    let discrepancies = row
      .discrepancies
      .as_ref()
      .map(|d| Self::deserialize_discrepancies(d))
      .transpose()?
      .unwrap_or_default();

    let reviewed_at = row
      .reviewed_at
      .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let snapshot_timestamp = Utc
      .timestamp_opt(row.snapshot_timestamp, 0)
      .single()
      .ok_or_else(|| InfraError::Database("snapshot_timestamp inválido".to_string()))?;

    let created_at = Utc
      .timestamp_opt(row.created_at, 0)
      .single()
      .ok_or_else(|| InfraError::Database("created_at inválido".to_string()))?;

    Ok(ReconciliationSnapshot {
      id: ReconciliationSnapshotId::from_string(row.id),
      exchange_id: row.exchange_id,
      snapshot_type,
      calculated_balances,
      reported_balances,
      discrepancies,
      status,
      reviewed_at,
      reviewed_by: row.reviewed_by,
      resolution_notes: row.resolution_notes,
      snapshot_timestamp,
      created_at,
    })
  }
}

/// Row do banco de dados para ReconciliationSnapshot
#[derive(sqlx::FromRow)]
struct ReconciliationSnapshotRow {
  id: String,
  exchange_id: String,
  snapshot_type: String,
  calculated_balances: String,
  reported_balances: String,
  discrepancies: Option<String>,
  status: String,
  reviewed_at: Option<i64>,
  reviewed_by: Option<String>,
  resolution_notes: Option<String>,
  snapshot_timestamp: i64,
  created_at: i64,
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
  async fn test_insert_and_find_snapshot() {
    let pool = setup_test_db().await;
    let repo = SqliteReconciliationRepository::new(pool);

    let mut calculated = HashMap::new();
    calculated.insert("USDT".to_string(), dec!(1000));
    calculated.insert("BTC".to_string(), dec!(0.5));

    let mut reported = HashMap::new();
    reported.insert("USDT".to_string(), dec!(1000));
    reported.insert("BTC".to_string(), dec!(0.5));

    let snapshot = ReconciliationSnapshot::new(
      "binance".to_string(),
      ReconciliationSnapshotType::Manual,
      calculated,
      reported,
      dec!(0.00000001),
    );

    repo.insert(&snapshot).await.unwrap();

    let found = repo.find_by_id(&snapshot.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.status, ReconciliationStatus::Matched);
    assert!(found.discrepancies.is_empty());
  }

  #[tokio::test]
  async fn test_snapshot_with_discrepancy() {
    let pool = setup_test_db().await;
    let repo = SqliteReconciliationRepository::new(pool);

    let mut calculated = HashMap::new();
    calculated.insert("USDT".to_string(), dec!(1000));

    let mut reported = HashMap::new();
    reported.insert("USDT".to_string(), dec!(990));

    let snapshot = ReconciliationSnapshot::new(
      "binance".to_string(),
      ReconciliationSnapshotType::Daily,
      calculated,
      reported,
      dec!(0.00000001),
    );

    repo.insert(&snapshot).await.unwrap();

    let discrepancy_snapshots = repo.find_with_discrepancies("binance").await.unwrap();
    assert_eq!(discrepancy_snapshots.len(), 1);
    assert_eq!(discrepancy_snapshots[0].discrepancies.len(), 1);
    assert_eq!(
      discrepancy_snapshots[0].discrepancies[0].difference,
      dec!(10)
    );
  }

  #[tokio::test]
  async fn test_update_status() {
    let pool = setup_test_db().await;
    let repo = SqliteReconciliationRepository::new(pool);

    let snapshot = ReconciliationSnapshot::new(
      "binance".to_string(),
      ReconciliationSnapshotType::Manual,
      HashMap::new(),
      HashMap::new(),
      dec!(0.00000001),
    );

    repo.insert(&snapshot).await.unwrap();

    repo
      .update_status(
        &snapshot.id,
        ReconciliationStatus::Resolved,
        Some("admin".to_string()),
        Some("All good".to_string()),
      )
      .await
      .unwrap();

    let found = repo.find_by_id(&snapshot.id).await.unwrap().unwrap();
    assert_eq!(found.status, ReconciliationStatus::Resolved);
    assert_eq!(found.reviewed_by, Some("admin".to_string()));
    assert!(found.reviewed_at.is_some());
  }

  #[tokio::test]
  async fn test_find_latest() {
    let pool = setup_test_db().await;
    let repo = SqliteReconciliationRepository::new(pool);

    // Insere dois snapshots
    let snapshot1 = ReconciliationSnapshot::new(
      "binance".to_string(),
      ReconciliationSnapshotType::Daily,
      HashMap::new(),
      HashMap::new(),
      dec!(0.00000001),
    );
    repo.insert(&snapshot1).await.unwrap();

    // Aguarda um pouco para ter timestamps diferentes
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let snapshot2 = ReconciliationSnapshot::new(
      "binance".to_string(),
      ReconciliationSnapshotType::Manual,
      HashMap::new(),
      HashMap::new(),
      dec!(0.00000001),
    );
    repo.insert(&snapshot2).await.unwrap();

    let latest = repo.find_latest("binance").await.unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().id, snapshot2.id);
  }
}
