//! Repositório de Exchange
//!
//! Implementação SQLite para persistência de exchanges.

use async_trait::async_trait;
use robotrade_core::{
  error::{InfraError, InfraResult},
  Exchange, ExchangeId, ExchangeStatus, ExchangeType,
};
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::debug;

use crate::database::ExchangeRow;

/// Trait para repositório de exchanges
#[async_trait]
pub trait ExchangeRepository: Send + Sync {
  /// Busca exchange por ID
  async fn find_by_id(&self, id: &str) -> InfraResult<Option<Exchange>>;

  /// Lista todas as exchanges
  async fn find_all(&self) -> InfraResult<Vec<Exchange>>;

  /// Lista exchanges ativas
  async fn find_active(&self) -> InfraResult<Vec<Exchange>>;

  /// Salva ou atualiza exchange
  async fn save(&self, exchange: &Exchange) -> InfraResult<()>;

  /// Atualiza status de uma exchange
  async fn update_status(&self, id: &str, status: ExchangeStatus) -> InfraResult<bool>;
}

/// Implementação SQLite do repositório de exchanges
pub struct SqliteExchangeRepository {
  pool: SqlitePool,
}

impl SqliteExchangeRepository {
  /// Cria novo repositório
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }

  /// Converte row do banco para entidade
  fn row_to_entity(row: ExchangeRow) -> InfraResult<Exchange> {
    let exchange_type = ExchangeType::from_str(&row.exchange_type)
      .map_err(|e| InfraError::Database(format!("ExchangeType inválido: {}", e)))?;
    let status = ExchangeStatus::from_str(&row.status)
      .map_err(|e| InfraError::Database(format!("ExchangeStatus inválido: {}", e)))?;

    // Parse supported features from JSON
    let supported_features: Vec<String> = row
      .supported_features
      .and_then(|s| serde_json::from_str(&s).ok())
      .unwrap_or_default();

    Ok(Exchange {
      id: parse_exchange_id(&row.id)?,
      name: row.name,
      exchange_type,
      api_base_url: row.api_base_url,
      ws_base_url: row.ws_base_url,
      status,
      rate_limit_requests: row.rate_limit_requests as u32,
      rate_limit_orders: row.rate_limit_orders as u32,
      supported_features,
      metadata: row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
      created_at: parse_datetime(&row.created_at)?,
      updated_at: parse_datetime(&row.updated_at)?,
    })
  }
}

#[async_trait]
impl ExchangeRepository for SqliteExchangeRepository {
  async fn find_by_id(&self, id: &str) -> InfraResult<Option<Exchange>> {
    let row: Option<ExchangeRow> = sqlx::query_as("SELECT * FROM exchanges WHERE id = ?")
      .bind(id)
      .fetch_optional(&self.pool)
      .await
      .map_err(|e| InfraError::Database(e.to_string()))?;

    row.map(Self::row_to_entity).transpose()
  }

  async fn find_all(&self) -> InfraResult<Vec<Exchange>> {
    let rows: Vec<ExchangeRow> = sqlx::query_as("SELECT * FROM exchanges ORDER BY name")
      .fetch_all(&self.pool)
      .await
      .map_err(|e| InfraError::Database(e.to_string()))?;

    rows.into_iter().map(Self::row_to_entity).collect()
  }

  async fn find_active(&self) -> InfraResult<Vec<Exchange>> {
    let rows: Vec<ExchangeRow> =
      sqlx::query_as("SELECT * FROM exchanges WHERE status = 'active' ORDER BY name")
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

    rows.into_iter().map(Self::row_to_entity).collect()
  }

  async fn save(&self, exchange: &Exchange) -> InfraResult<()> {
    debug!(exchange_id = %exchange.id, "Salvando exchange");

    let metadata_json = exchange
      .metadata
      .as_ref()
      .and_then(|m| serde_json::to_string(m).ok());

    let supported_features_json = serde_json::to_string(&exchange.supported_features).ok();

    sqlx::query(
      r#"
            INSERT INTO exchanges (
                id, name, exchange_type, api_base_url, ws_base_url,
                status, rate_limit_requests, rate_limit_orders,
                supported_features, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                exchange_type = excluded.exchange_type,
                api_base_url = excluded.api_base_url,
                ws_base_url = excluded.ws_base_url,
                status = excluded.status,
                rate_limit_requests = excluded.rate_limit_requests,
                rate_limit_orders = excluded.rate_limit_orders,
                supported_features = excluded.supported_features,
                metadata = excluded.metadata,
                updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(exchange.id.to_string())
    .bind(&exchange.name)
    .bind(exchange.exchange_type.to_string())
    .bind(&exchange.api_base_url)
    .bind(&exchange.ws_base_url)
    .bind(exchange.status.to_string())
    .bind(exchange.rate_limit_requests as i64)
    .bind(exchange.rate_limit_orders as i64)
    .bind(supported_features_json)
    .bind(metadata_json)
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(e.to_string()))?;

    Ok(())
  }

  async fn update_status(&self, id: &str, status: ExchangeStatus) -> InfraResult<bool> {
    let result =
      sqlx::query("UPDATE exchanges SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(status.to_string())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

    Ok(result.rows_affected() > 0)
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

fn parse_datetime(s: &str) -> InfraResult<chrono::DateTime<chrono::Utc>> {
  chrono::DateTime::parse_from_rfc3339(s)
    .map(|dt| dt.with_timezone(&chrono::Utc))
    .or_else(|_| {
      chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc())
    })
    .map_err(|e| InfraError::Database(format!("Data inválida: {}", e)))
}
