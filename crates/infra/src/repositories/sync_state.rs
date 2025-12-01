//! Repositório SQLite para Sync State

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use robotrade_core::entities::{SyncDataType, SyncState, SyncStateId, SyncStatus};
use robotrade_core::error::{InfraError, InfraResult};
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::debug;

/// Trait para operações de repositório de Sync State
#[async_trait]
pub trait SyncStateRepository: Send + Sync {
  /// Insere ou atualiza estado de sync (upsert)
  async fn upsert(&self, state: &SyncState) -> InfraResult<()>;

  /// Busca estado por exchange e tipo de dados
  async fn find(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
  ) -> InfraResult<Option<SyncState>>;

  /// Busca todos os estados de uma exchange
  async fn find_all_for_exchange(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>>;

  /// Atualiza status
  async fn update_status(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    status: SyncStatus,
    error_message: Option<String>,
  ) -> InfraResult<()>;

  /// Atualiza progresso
  async fn update_progress(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    processed: u64,
    total: Option<u64>,
  ) -> InfraResult<()>;

  /// Atualiza cursor de paginação
  async fn update_cursor(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    cursor: Option<String>,
    last_id: Option<String>,
    last_timestamp: Option<DateTime<Utc>>,
  ) -> InfraResult<()>;

  /// Marca sync como completo
  async fn mark_completed(&self, exchange_id: &str, data_type: SyncDataType) -> InfraResult<()>;

  /// Reseta estado para idle
  async fn reset(&self, exchange_id: &str, data_type: SyncDataType) -> InfraResult<()>;

  /// Deleta todos os estados de uma exchange
  async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64>;

  /// Busca estados com erro
  async fn find_with_errors(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>>;

  /// Busca estados sincronizando
  async fn find_syncing(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>>;
}

/// Implementação SQLite do repositório de Sync State
pub struct SqliteSyncStateRepository {
  pool: SqlitePool,
}

impl SqliteSyncStateRepository {
  /// Cria uma nova instância do repositório
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl SyncStateRepository for SqliteSyncStateRepository {
  async fn upsert(&self, state: &SyncState) -> InfraResult<()> {
    debug!(
        exchange_id = %state.exchange_id,
        data_type = %state.data_type,
        status = %state.status,
        "Upserting sync state"
    );

    sqlx::query(
      r#"
            INSERT INTO sync_state (
                id, exchange_id, data_type, last_sync_id, last_sync_timestamp,
                cursor, sync_start_time, sync_end_time, status, error_message,
                retry_count, total_records, processed_records, last_success_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(exchange_id, data_type) DO UPDATE SET
                last_sync_id = excluded.last_sync_id,
                last_sync_timestamp = excluded.last_sync_timestamp,
                cursor = excluded.cursor,
                sync_start_time = excluded.sync_start_time,
                sync_end_time = excluded.sync_end_time,
                status = excluded.status,
                error_message = excluded.error_message,
                retry_count = excluded.retry_count,
                total_records = excluded.total_records,
                processed_records = excluded.processed_records,
                last_success_at = excluded.last_success_at,
                updated_at = excluded.updated_at
            "#,
    )
    .bind(state.id.0.as_str())
    .bind(&state.exchange_id)
    .bind(state.data_type.to_string())
    .bind(&state.last_sync_id)
    .bind(state.last_sync_timestamp.map(|d| d.timestamp()))
    .bind(&state.cursor)
    .bind(state.sync_start_time.map(|d| d.timestamp()))
    .bind(state.sync_end_time.map(|d| d.timestamp()))
    .bind(state.status.to_string())
    .bind(&state.error_message)
    .bind(state.retry_count as i32)
    .bind(state.total_records.map(|v| v as i64))
    .bind(state.processed_records.map(|v| v as i64))
    .bind(state.last_success_at.map(|d| d.timestamp()))
    .bind(state.updated_at.timestamp())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao upsert sync state: {}", e)))?;

    Ok(())
  }

  async fn find(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
  ) -> InfraResult<Option<SyncState>> {
    let row = sqlx::query_as::<_, SyncStateRow>(
      r#"
            SELECT id, exchange_id, data_type, last_sync_id, last_sync_timestamp,
                   cursor, sync_start_time, sync_end_time, status, error_message,
                   retry_count, total_records, processed_records, last_success_at, updated_at
            FROM sync_state
            WHERE exchange_id = ? AND data_type = ?
            "#,
    )
    .bind(exchange_id)
    .bind(data_type.to_string())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar sync state: {}", e)))?;

    match row {
      Some(r) => Ok(Some(r.try_into()?)),
      None => Ok(None),
    }
  }

  async fn find_all_for_exchange(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>> {
    let rows = sqlx::query_as::<_, SyncStateRow>(
      r#"
            SELECT id, exchange_id, data_type, last_sync_id, last_sync_timestamp,
                   cursor, sync_start_time, sync_end_time, status, error_message,
                   retry_count, total_records, processed_records, last_success_at, updated_at
            FROM sync_state
            WHERE exchange_id = ?
            ORDER BY data_type
            "#,
    )
    .bind(exchange_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar sync states: {}", e)))?;

    let states: Result<Vec<SyncState>, _> = rows.into_iter().map(|r| r.try_into()).collect();
    states
  }

  async fn update_status(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    status: SyncStatus,
    error_message: Option<String>,
  ) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    // Se status é ERROR, incrementa retry_count
    let retry_increment = if status == SyncStatus::Error { 1 } else { 0 };

    sqlx::query(
      r#"
            UPDATE sync_state
            SET status = ?,
                error_message = ?,
                retry_count = retry_count + ?,
                updated_at = ?
            WHERE exchange_id = ? AND data_type = ?
            "#,
    )
    .bind(status.to_string())
    .bind(&error_message)
    .bind(retry_increment)
    .bind(now)
    .bind(exchange_id)
    .bind(data_type.to_string())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao atualizar status: {}", e)))?;

    Ok(())
  }

  async fn update_progress(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    processed: u64,
    total: Option<u64>,
  ) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    match total {
      Some(t) => {
        sqlx::query(
          r#"
                    UPDATE sync_state
                    SET processed_records = ?,
                        total_records = ?,
                        updated_at = ?
                    WHERE exchange_id = ? AND data_type = ?
                    "#,
        )
        .bind(processed as i64)
        .bind(t as i64)
        .bind(now)
        .bind(exchange_id)
        .bind(data_type.to_string())
        .execute(&self.pool)
        .await
      }
      None => {
        sqlx::query(
          r#"
                    UPDATE sync_state
                    SET processed_records = ?,
                        updated_at = ?
                    WHERE exchange_id = ? AND data_type = ?
                    "#,
        )
        .bind(processed as i64)
        .bind(now)
        .bind(exchange_id)
        .bind(data_type.to_string())
        .execute(&self.pool)
        .await
      }
    }
    .map_err(|e| InfraError::Database(format!("Erro ao atualizar progresso: {}", e)))?;

    Ok(())
  }

  async fn update_cursor(
    &self,
    exchange_id: &str,
    data_type: SyncDataType,
    cursor: Option<String>,
    last_id: Option<String>,
    last_timestamp: Option<DateTime<Utc>>,
  ) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    sqlx::query(
      r#"
            UPDATE sync_state
            SET cursor = ?,
                last_sync_id = COALESCE(?, last_sync_id),
                last_sync_timestamp = COALESCE(?, last_sync_timestamp),
                updated_at = ?
            WHERE exchange_id = ? AND data_type = ?
            "#,
    )
    .bind(&cursor)
    .bind(&last_id)
    .bind(last_timestamp.map(|d| d.timestamp()))
    .bind(now)
    .bind(exchange_id)
    .bind(data_type.to_string())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao atualizar cursor: {}", e)))?;

    Ok(())
  }

  async fn mark_completed(&self, exchange_id: &str, data_type: SyncDataType) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    sqlx::query(
      r#"
            UPDATE sync_state
            SET status = 'COMPLETED',
                error_message = NULL,
                retry_count = 0,
                cursor = NULL,
                last_success_at = ?,
                updated_at = ?
            WHERE exchange_id = ? AND data_type = ?
            "#,
    )
    .bind(now)
    .bind(now)
    .bind(exchange_id)
    .bind(data_type.to_string())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao marcar como completo: {}", e)))?;

    Ok(())
  }

  async fn reset(&self, exchange_id: &str, data_type: SyncDataType) -> InfraResult<()> {
    let now = Utc::now().timestamp();

    sqlx::query(
      r#"
            UPDATE sync_state
            SET status = 'IDLE',
                error_message = NULL,
                retry_count = 0,
                cursor = NULL,
                processed_records = NULL,
                total_records = NULL,
                updated_at = ?
            WHERE exchange_id = ? AND data_type = ?
            "#,
    )
    .bind(now)
    .bind(exchange_id)
    .bind(data_type.to_string())
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao resetar sync state: {}", e)))?;

    Ok(())
  }

  async fn delete_all(&self, exchange_id: &str) -> InfraResult<u64> {
    let result = sqlx::query("DELETE FROM sync_state WHERE exchange_id = ?")
      .bind(exchange_id)
      .execute(&self.pool)
      .await
      .map_err(|e| InfraError::Database(format!("Erro ao deletar sync states: {}", e)))?;

    Ok(result.rows_affected())
  }

  async fn find_with_errors(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>> {
    let rows = sqlx::query_as::<_, SyncStateRow>(
      r#"
            SELECT id, exchange_id, data_type, last_sync_id, last_sync_timestamp,
                   cursor, sync_start_time, sync_end_time, status, error_message,
                   retry_count, total_records, processed_records, last_success_at, updated_at
            FROM sync_state
            WHERE exchange_id = ? AND status = 'ERROR'
            "#,
    )
    .bind(exchange_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar sync states com erro: {}", e)))?;

    let states: Result<Vec<SyncState>, _> = rows.into_iter().map(|r| r.try_into()).collect();
    states
  }

  async fn find_syncing(&self, exchange_id: &str) -> InfraResult<Vec<SyncState>> {
    let rows = sqlx::query_as::<_, SyncStateRow>(
      r#"
            SELECT id, exchange_id, data_type, last_sync_id, last_sync_timestamp,
                   cursor, sync_start_time, sync_end_time, status, error_message,
                   retry_count, total_records, processed_records, last_success_at, updated_at
            FROM sync_state
            WHERE exchange_id = ? AND status = 'SYNCING'
            "#,
    )
    .bind(exchange_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar sync states: {}", e)))?;

    let states: Result<Vec<SyncState>, _> = rows.into_iter().map(|r| r.try_into()).collect();
    states
  }
}

/// Row do banco de dados para SyncState
#[derive(sqlx::FromRow)]
struct SyncStateRow {
  id: String,
  exchange_id: String,
  data_type: String,
  last_sync_id: Option<String>,
  last_sync_timestamp: Option<i64>,
  cursor: Option<String>,
  sync_start_time: Option<i64>,
  sync_end_time: Option<i64>,
  status: String,
  error_message: Option<String>,
  retry_count: i32,
  total_records: Option<i64>,
  processed_records: Option<i64>,
  last_success_at: Option<i64>,
  updated_at: i64,
}

impl TryFrom<SyncStateRow> for SyncState {
  type Error = InfraError;

  fn try_from(row: SyncStateRow) -> Result<Self, Self::Error> {
    let data_type = SyncDataType::from_str(&row.data_type)
      .map_err(|e| InfraError::Database(format!("data_type inválido: {}", e)))?;

    let status = SyncStatus::from_str(&row.status)
      .map_err(|e| InfraError::Database(format!("status inválido: {}", e)))?;

    let last_sync_timestamp = row
      .last_sync_timestamp
      .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let sync_start_time = row
      .sync_start_time
      .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let sync_end_time = row
      .sync_end_time
      .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let last_success_at = row
      .last_success_at
      .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let updated_at = Utc
      .timestamp_opt(row.updated_at, 0)
      .single()
      .ok_or_else(|| InfraError::Database("updated_at inválido".to_string()))?;

    Ok(SyncState {
      id: SyncStateId::from_string(row.id),
      exchange_id: row.exchange_id,
      data_type,
      last_sync_id: row.last_sync_id,
      last_sync_timestamp,
      cursor: row.cursor,
      sync_start_time,
      sync_end_time,
      status,
      error_message: row.error_message,
      retry_count: row.retry_count as u32,
      total_records: row.total_records.map(|v| v as u64),
      processed_records: row.processed_records.map(|v| v as u64),
      last_success_at,
      updated_at,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::database::{init_database, DatabaseConfig};
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
  async fn test_upsert_and_find() {
    let pool = setup_test_db().await;
    let repo = SqliteSyncStateRepository::new(pool);

    let state = SyncState::new("binance".to_string(), SyncDataType::Trades);

    repo.upsert(&state).await.unwrap();

    let found = repo.find("binance", SyncDataType::Trades).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().status, SyncStatus::Idle);
  }

  #[tokio::test]
  async fn test_status_updates() {
    let pool = setup_test_db().await;
    let repo = SqliteSyncStateRepository::new(pool);

    let state = SyncState::new("binance".to_string(), SyncDataType::Orders);
    repo.upsert(&state).await.unwrap();

    // Start syncing
    repo
      .update_status("binance", SyncDataType::Orders, SyncStatus::Syncing, None)
      .await
      .unwrap();

    let found = repo
      .find("binance", SyncDataType::Orders)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(found.status, SyncStatus::Syncing);

    // Mark error
    repo
      .update_status(
        "binance",
        SyncDataType::Orders,
        SyncStatus::Error,
        Some("Connection failed".to_string()),
      )
      .await
      .unwrap();

    let found = repo
      .find("binance", SyncDataType::Orders)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(found.status, SyncStatus::Error);
    assert_eq!(found.retry_count, 1);
    assert_eq!(found.error_message, Some("Connection failed".to_string()));
  }

  #[tokio::test]
  async fn test_progress_tracking() {
    let pool = setup_test_db().await;
    let repo = SqliteSyncStateRepository::new(pool);

    let state = SyncState::new("binance".to_string(), SyncDataType::Funding);
    repo.upsert(&state).await.unwrap();

    repo
      .update_progress("binance", SyncDataType::Funding, 50, Some(100))
      .await
      .unwrap();

    let found = repo
      .find("binance", SyncDataType::Funding)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(found.processed_records, Some(50));
    assert_eq!(found.total_records, Some(100));
    assert_eq!(found.progress_percentage(), Some(50.0));
  }

  #[tokio::test]
  async fn test_mark_completed() {
    let pool = setup_test_db().await;
    let repo = SqliteSyncStateRepository::new(pool);

    let mut state = SyncState::new("binance".to_string(), SyncDataType::Deposits);
    state.status = SyncStatus::Syncing;
    state.retry_count = 2;
    repo.upsert(&state).await.unwrap();

    repo
      .mark_completed("binance", SyncDataType::Deposits)
      .await
      .unwrap();

    let found = repo
      .find("binance", SyncDataType::Deposits)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(found.status, SyncStatus::Completed);
    assert_eq!(found.retry_count, 0);
    assert!(found.last_success_at.is_some());
  }
}
