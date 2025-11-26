//! Repositório SQLite para Candles (OHLCV)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use robotrade_core::entities::{Candle, TimeFrame};
use robotrade_core::error::{InfraError, InfraResult};
use robotrade_core::traits::CandleRepository;
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::debug;

/// Implementação SQLite do repositório de candles
pub struct SqliteCandleRepository {
    pool: SqlitePool,
}

impl SqliteCandleRepository {
    /// Cria uma nova instância do repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CandleRepository for SqliteCandleRepository {
    async fn find_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        limit: usize,
    ) -> InfraResult<Vec<Candle>> {
        debug!(
            symbol = %symbol,
            timeframe = %timeframe,
            limit = %limit,
            "Buscando candles"
        );

        let tf_str = timeframe.to_binance_interval();

        let rows = sqlx::query_as::<_, CandleRow>(
            r#"
            SELECT open_time, close_time, open, high, low, close, volume, quote_volume, trade_count
            FROM candles
            WHERE symbol = ? AND timeframe = ?
            ORDER BY open_time DESC
            LIMIT ?
            "#,
        )
        .bind(symbol)
        .bind(tf_str)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar candles: {}", e)))?;

        let candles: Vec<Candle> = rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect();

        // Inverte para ordem cronológica
        let mut candles = candles;
        candles.reverse();

        Ok(candles)
    }

    async fn find_candles_in_period(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> InfraResult<Vec<Candle>> {
        debug!(
            symbol = %symbol,
            timeframe = %timeframe,
            start = %start,
            end = %end,
            "Buscando candles no período"
        );

        let tf_str = timeframe.to_binance_interval();
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let rows = sqlx::query_as::<_, CandleRow>(
            r#"
            SELECT open_time, close_time, open, high, low, close, volume, quote_volume, trade_count
            FROM candles
            WHERE symbol = ? AND timeframe = ?
              AND open_time >= ? AND open_time <= ?
            ORDER BY open_time ASC
            "#,
        )
        .bind(symbol)
        .bind(tf_str)
        .bind(&start_str)
        .bind(&end_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar candles no período: {}", e)))?;

        let candles: Vec<Candle> = rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect();

        Ok(candles)
    }

    async fn save_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        candles: &[Candle],
    ) -> InfraResult<()> {
        if candles.is_empty() {
            return Ok(());
        }

        debug!(
            symbol = %symbol,
            timeframe = %timeframe,
            count = %candles.len(),
            "Salvando candles"
        );

        let tf_str = timeframe.to_binance_interval();

        // Usa transação para inserção em batch
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao iniciar transação: {}", e)))?;

        for candle in candles {
            let open_time_str = candle.open_time.to_rfc3339();
            let close_time_str = candle.close_time.to_rfc3339();

            sqlx::query(
                r#"
                INSERT INTO candles (symbol, timeframe, open_time, close_time, open, high, low, close, volume, quote_volume, trade_count)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(symbol, timeframe, open_time) DO UPDATE SET
                    close_time = excluded.close_time,
                    open = excluded.open,
                    high = excluded.high,
                    low = excluded.low,
                    close = excluded.close,
                    volume = excluded.volume,
                    quote_volume = excluded.quote_volume,
                    trade_count = excluded.trade_count
                "#,
            )
            .bind(symbol)
            .bind(tf_str)
            .bind(&open_time_str)
            .bind(&close_time_str)
            .bind(candle.open.to_string().parse::<f64>().unwrap_or(0.0))
            .bind(candle.high.to_string().parse::<f64>().unwrap_or(0.0))
            .bind(candle.low.to_string().parse::<f64>().unwrap_or(0.0))
            .bind(candle.close.to_string().parse::<f64>().unwrap_or(0.0))
            .bind(candle.volume.to_string().parse::<f64>().unwrap_or(0.0))
            .bind(candle.quote_volume.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)))
            .bind(candle.trade_count.map(|c| c as i32))
            .execute(&mut *tx)
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao inserir candle: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| InfraError::Database(format!("Erro ao commitar transação: {}", e)))?;

        Ok(())
    }

    async fn get_last_candle(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
    ) -> InfraResult<Option<Candle>> {
        debug!(
            symbol = %symbol,
            timeframe = %timeframe,
            "Buscando último candle"
        );

        let tf_str = timeframe.to_binance_interval();

        let row = sqlx::query_as::<_, CandleRow>(
            r#"
            SELECT open_time, close_time, open, high, low, close, volume, quote_volume, trade_count
            FROM candles
            WHERE symbol = ? AND timeframe = ?
            ORDER BY open_time DESC
            LIMIT 1
            "#,
        )
        .bind(symbol)
        .bind(tf_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao buscar último candle: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.try_into().map_err(|e: String| {
                InfraError::Database(format!("Erro ao converter candle: {}", e))
            })?)),
            None => Ok(None),
        }
    }
}

/// Row do banco de dados
#[derive(sqlx::FromRow)]
struct CandleRow {
    open_time: String,
    close_time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    quote_volume: Option<f64>,
    trade_count: Option<i32>,
}

impl TryFrom<CandleRow> for Candle {
    type Error = String;

    fn try_from(row: CandleRow) -> Result<Self, Self::Error> {
        let open_time = DateTime::parse_from_rfc3339(&row.open_time)
            .map_err(|e| format!("open_time inválido: {}", e))?
            .with_timezone(&Utc);

        let close_time = DateTime::parse_from_rfc3339(&row.close_time)
            .map_err(|e| format!("close_time inválido: {}", e))?
            .with_timezone(&Utc);

        Ok(Candle {
            open: Decimal::from_str(&row.open.to_string()).unwrap_or(Decimal::ZERO),
            high: Decimal::from_str(&row.high.to_string()).unwrap_or(Decimal::ZERO),
            low: Decimal::from_str(&row.low.to_string()).unwrap_or(Decimal::ZERO),
            close: Decimal::from_str(&row.close.to_string()).unwrap_or(Decimal::ZERO),
            volume: Decimal::from_str(&row.volume.to_string()).unwrap_or(Decimal::ZERO),
            quote_volume: row
                .quote_volume
                .map(|v| Decimal::from_str(&v.to_string()).unwrap_or(Decimal::ZERO)),
            trade_count: row.trade_count.map(|c| c as u32),
            open_time,
            close_time,
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
    async fn test_save_and_find_candles() {
        let pool = setup_test_db().await;
        let repo = SqliteCandleRepository::new(pool);

        let now = Utc::now();
        let candles = vec![
            Candle::new(
                dec!(100),
                dec!(110),
                dec!(95),
                dec!(105),
                dec!(1000),
                now - chrono::Duration::hours(2),
                now - chrono::Duration::hours(1),
            ),
            Candle::new(
                dec!(105),
                dec!(115),
                dec!(100),
                dec!(110),
                dec!(1200),
                now - chrono::Duration::hours(1),
                now,
            ),
        ];

        repo.save_candles("BTCUSDT", TimeFrame::H1, &candles)
            .await
            .unwrap();

        let found = repo
            .find_candles("BTCUSDT", TimeFrame::H1, 10)
            .await
            .unwrap();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].open, dec!(100));
        assert_eq!(found[1].open, dec!(105));
    }

    #[tokio::test]
    async fn test_get_last_candle() {
        let pool = setup_test_db().await;
        let repo = SqliteCandleRepository::new(pool);

        // Sem candles
        let last = repo
            .get_last_candle("BTCUSDT", TimeFrame::H1)
            .await
            .unwrap();
        assert!(last.is_none());

        // Adiciona candles
        let now = Utc::now();
        let candles = vec![Candle::new(
            dec!(100),
            dec!(110),
            dec!(95),
            dec!(105),
            dec!(1000),
            now - chrono::Duration::hours(1),
            now,
        )];

        repo.save_candles("BTCUSDT", TimeFrame::H1, &candles)
            .await
            .unwrap();

        let last = repo
            .get_last_candle("BTCUSDT", TimeFrame::H1)
            .await
            .unwrap();

        assert!(last.is_some());
        assert_eq!(last.unwrap().open, dec!(100));
    }
}
