//! Repositório SQLite para Fear & Greed Index

use async_trait::async_trait;
use chrono::NaiveDate;
use robotrade_core::entities::{FearGreedClassification, FearGreedData};
use robotrade_core::error::{InfraError, InfraResult};
use robotrade_core::traits::FearGreedRepository;
use sqlx::SqlitePool;
use tracing::debug;

/// Implementação SQLite do repositório de Fear & Greed
pub struct SqliteFearGreedRepository {
  pool: SqlitePool,
}

impl SqliteFearGreedRepository {
  /// Cria uma nova instância do repositório
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl FearGreedRepository for SqliteFearGreedRepository {
  async fn find_history(&self, days: u32) -> InfraResult<Vec<FearGreedData>> {
    debug!(days = %days, "Buscando histórico Fear & Greed");

    let rows = sqlx::query_as::<_, FearGreedRow>(
      r#"
            SELECT value, classification, date, collected_at
            FROM fear_greed_data
            ORDER BY date DESC
            LIMIT ?
            "#,
    )
    .bind(days as i64)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar histórico: {}", e)))?;

    let data = rows
      .into_iter()
      .filter_map(|row| row.try_into().ok())
      .collect();

    Ok(data)
  }

  async fn find_by_date(&self, date: NaiveDate) -> InfraResult<Option<FearGreedData>> {
    debug!(date = %date, "Buscando Fear & Greed por data");

    let date_str = date.format("%Y-%m-%d").to_string();

    let row = sqlx::query_as::<_, FearGreedRow>(
      r#"
            SELECT value, classification, date, collected_at
            FROM fear_greed_data
            WHERE date = ?
            "#,
    )
    .bind(&date_str)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar por data: {}", e)))?;

    match row {
      Some(r) => Ok(Some(r.try_into().map_err(|e: String| {
        InfraError::Database(format!("Erro ao converter: {}", e))
      })?)),
      None => Ok(None),
    }
  }

  async fn save(&self, data: &FearGreedData) -> InfraResult<()> {
    debug!(
        value = %data.value,
        date = %data.date,
        "Salvando Fear & Greed"
    );

    let date_str = data.date.format("%Y-%m-%d").to_string();
    let collected_at_str = data.collected_at.to_rfc3339();
    let classification_str = classification_to_string(&data.classification);

    sqlx::query(
      r#"
            INSERT INTO fear_greed_data (value, classification, date, collected_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(date) DO UPDATE SET
                value = excluded.value,
                classification = excluded.classification,
                collected_at = excluded.collected_at
            "#,
    )
    .bind(data.value as i32)
    .bind(&classification_str)
    .bind(&date_str)
    .bind(&collected_at_str)
    .execute(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao salvar: {}", e)))?;

    Ok(())
  }

  async fn get_latest(&self) -> InfraResult<Option<FearGreedData>> {
    debug!("Buscando Fear & Greed mais recente");

    let row = sqlx::query_as::<_, FearGreedRow>(
      r#"
            SELECT value, classification, date, collected_at
            FROM fear_greed_data
            ORDER BY date DESC
            LIMIT 1
            "#,
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| InfraError::Database(format!("Erro ao buscar mais recente: {}", e)))?;

    match row {
      Some(r) => Ok(Some(r.try_into().map_err(|e: String| {
        InfraError::Database(format!("Erro ao converter: {}", e))
      })?)),
      None => Ok(None),
    }
  }
}

/// Row do banco de dados
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct FearGreedRow {
  value: i32,
  classification: String,
  date: String,
  collected_at: String,
}

impl TryFrom<FearGreedRow> for FearGreedData {
  type Error = String;

  fn try_from(row: FearGreedRow) -> Result<Self, Self::Error> {
    let date = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
      .map_err(|e| format!("Data inválida: {}", e))?;

    let collected_at = chrono::DateTime::parse_from_rfc3339(&row.collected_at)
      .map_err(|e| format!("Timestamp inválido: {}", e))?
      .with_timezone(&chrono::Utc);

    Ok(FearGreedData {
      value: row.value as u8,
      classification: FearGreedClassification::from_value(row.value as u8),
      date,
      collected_at,
    })
  }
}

fn classification_to_string(classification: &FearGreedClassification) -> String {
  match classification {
    FearGreedClassification::ExtremeFear => "extreme_fear".into(),
    FearGreedClassification::Fear => "fear".into(),
    FearGreedClassification::Neutral => "neutral".into(),
    FearGreedClassification::Greed => "greed".into(),
    FearGreedClassification::ExtremeGreed => "extreme_greed".into(),
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
  async fn test_save_and_find() {
    let pool = setup_test_db().await;
    let repo = SqliteFearGreedRepository::new(pool);

    let data = FearGreedData::new(25, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

    repo.save(&data).await.unwrap();

    let found = repo
      .find_by_date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
      .await
      .unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().value, 25);
  }

  #[tokio::test]
  async fn test_get_latest() {
    let pool = setup_test_db().await;
    let repo = SqliteFearGreedRepository::new(pool);

    // Sem dados
    let latest = repo.get_latest().await.unwrap();
    assert!(latest.is_none());

    // Adiciona dados
    let data1 = FearGreedData::new(25, NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
    let data2 = FearGreedData::new(30, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

    repo.save(&data1).await.unwrap();
    repo.save(&data2).await.unwrap();

    let latest = repo.get_latest().await.unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().value, 30);
  }
}
