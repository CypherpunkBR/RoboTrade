//! Repositório SQLite para Signals
//!
//! Persistência de sinais de trading gerados por estratégias.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use tracing::debug;

use robotrade_core::{Signal, SignalId, SignalStatus, SignalStrength, SignalType, TradeDirection};

/// Trait para repositório de sinais
#[async_trait]
pub trait SignalRepository: Send + Sync {
    /// Salva um sinal
    async fn save(&self, signal: &Signal) -> Result<(), String>;

    /// Busca sinal por ID
    async fn find_by_id(&self, id: &SignalId) -> Result<Option<Signal>, String>;

    /// Lista sinais ativos (que podem ser executados)
    async fn find_active(&self, symbol: Option<&str>) -> Result<Vec<Signal>, String>;

    /// Lista sinais recentes
    async fn find_recent(&self, limit: usize) -> Result<Vec<Signal>, String>;

    /// Atualiza status de um sinal
    async fn update_status(&self, id: &SignalId, status: SignalStatus) -> Result<(), String>;

    /// Marca sinal como executado
    async fn mark_executed(&self, id: &SignalId) -> Result<(), String>;

    /// Deleta sinais antigos (cleanup)
    async fn delete_old_signals(&self, before: DateTime<Utc>) -> Result<u64, String>;
}

/// Implementação SQLite do repositório de sinais
pub struct SqliteSignalRepository {
    pool: SqlitePool,
}

impl SqliteSignalRepository {
    /// Cria um novo repositório
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inicializa a tabela
    pub async fn init(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS signals (
                id TEXT PRIMARY KEY,
                strategy_id TEXT NOT NULL,
                strategy_name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                signal_type TEXT NOT NULL,
                direction TEXT NOT NULL,
                strength TEXT NOT NULL,
                trigger_price TEXT NOT NULL,
                suggested_entry TEXT,
                suggested_stop_loss TEXT,
                suggested_take_profit TEXT,
                risk_reward_ratio TEXT,
                reason TEXT NOT NULL,
                confidence INTEGER NOT NULL,
                status TEXT NOT NULL,
                requires_confirmation INTEGER NOT NULL,
                metadata TEXT,
                generated_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                confirmed_at TEXT,
                executed_at TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create signals table: {}", e))?;

        // Índices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_signals_status ON signals(status)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_signals_symbol ON signals(symbol)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_signals_generated_at ON signals(generated_at)")
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }
}

#[derive(FromRow)]
struct SignalRow {
    id: String,
    strategy_id: String,
    strategy_name: String,
    symbol: String,
    signal_type: String,
    direction: String,
    strength: String,
    trigger_price: String,
    suggested_entry: Option<String>,
    suggested_stop_loss: Option<String>,
    suggested_take_profit: Option<String>,
    risk_reward_ratio: Option<String>,
    reason: String,
    confidence: i64,
    status: String,
    requires_confirmation: i64,
    metadata: Option<String>,
    generated_at: String,
    expires_at: String,
    confirmed_at: Option<String>,
    executed_at: Option<String>,
}

fn parse_signal_id(s: &str) -> Result<SignalId, String> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid signal id: {}", e))?;
    Ok(SignalId(uuid))
}

fn parse_direction(s: &str) -> Result<TradeDirection, String> {
    match s {
        "long" => Ok(TradeDirection::Long),
        "short" => Ok(TradeDirection::Short),
        _ => Err(format!("Invalid direction: {}", s)),
    }
}

fn parse_strength(s: &str) -> Result<SignalStrength, String> {
    match s {
        "weak" => Ok(SignalStrength::Weak),
        "moderate" => Ok(SignalStrength::Moderate),
        "strong" => Ok(SignalStrength::Strong),
        _ => Err(format!("Invalid strength: {}", s)),
    }
}

fn parse_status(s: &str) -> Result<SignalStatus, String> {
    match s {
        "active" => Ok(SignalStatus::Active),
        "awaiting_confirmation" => Ok(SignalStatus::AwaitingConfirmation),
        "confirmed" => Ok(SignalStatus::Confirmed),
        "executed" => Ok(SignalStatus::Executed),
        "expired" => Ok(SignalStatus::Expired),
        "cancelled" => Ok(SignalStatus::Cancelled),
        _ => Err(format!("Invalid status: {}", s)),
    }
}

fn status_to_string(status: SignalStatus) -> String {
    match status {
        SignalStatus::Active => "active".to_string(),
        SignalStatus::AwaitingConfirmation => "awaiting_confirmation".to_string(),
        SignalStatus::Confirmed => "confirmed".to_string(),
        SignalStatus::Executed => "executed".to_string(),
        SignalStatus::Expired => "expired".to_string(),
        SignalStatus::Cancelled => "cancelled".to_string(),
    }
}

fn strength_to_string(strength: SignalStrength) -> String {
    match strength {
        SignalStrength::Weak => "weak".to_string(),
        SignalStrength::Moderate => "moderate".to_string(),
        SignalStrength::Strong => "strong".to_string(),
    }
}

fn parse_signal_type(s: &str) -> Result<SignalType, String> {
    // SignalType is a complex enum, we serialize it as JSON
    serde_json::from_str(s).map_err(|e| format!("Invalid signal type: {}", e))
}

fn signal_type_to_string(st: &SignalType) -> Result<String, String> {
    serde_json::to_string(st).map_err(|e| format!("Failed to serialize signal type: {}", e))
}

impl TryFrom<SignalRow> for Signal {
    type Error = String;

    fn try_from(row: SignalRow) -> Result<Self, Self::Error> {
        Ok(Signal {
            id: parse_signal_id(&row.id)?,
            strategy_id: row.strategy_id,
            strategy_name: row.strategy_name,
            symbol: row.symbol,
            signal_type: parse_signal_type(&row.signal_type)?,
            direction: parse_direction(&row.direction)?,
            strength: parse_strength(&row.strength)?,
            trigger_price: row
                .trigger_price
                .parse()
                .map_err(|e| format!("Invalid trigger price: {}", e))?,
            suggested_entry: row
                .suggested_entry
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid suggested entry: {}", e))?,
            suggested_stop_loss: row
                .suggested_stop_loss
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid suggested stop loss: {}", e))?,
            suggested_take_profit: row
                .suggested_take_profit
                .map(|p| p.parse())
                .transpose()
                .map_err(|e| format!("Invalid suggested take profit: {}", e))?,
            risk_reward_ratio: row
                .risk_reward_ratio
                .map(|r| r.parse())
                .transpose()
                .map_err(|e| format!("Invalid risk reward ratio: {}", e))?,
            reason: row.reason,
            confidence: row.confidence as u8,
            status: parse_status(&row.status)?,
            requires_confirmation: row.requires_confirmation != 0,
            metadata: row
                .metadata
                .map(|m| serde_json::from_str(&m))
                .transpose()
                .map_err(|e| format!("Invalid metadata: {}", e))?,
            generated_at: row
                .generated_at
                .parse()
                .map_err(|e| format!("Invalid generated_at: {}", e))?,
            expires_at: row
                .expires_at
                .parse()
                .map_err(|e| format!("Invalid expires_at: {}", e))?,
            confirmed_at: row
                .confirmed_at
                .map(|t| t.parse())
                .transpose()
                .map_err(|e| format!("Invalid confirmed_at: {}", e))?,
            executed_at: row
                .executed_at
                .map(|t| t.parse())
                .transpose()
                .map_err(|e| format!("Invalid executed_at: {}", e))?,
        })
    }
}

#[async_trait]
impl SignalRepository for SqliteSignalRepository {
    async fn save(&self, signal: &Signal) -> Result<(), String> {
        let signal_type_json = signal_type_to_string(&signal.signal_type)?;
        let metadata = signal
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO signals (
                id, strategy_id, strategy_name, symbol, signal_type, direction, strength,
                trigger_price, suggested_entry, suggested_stop_loss, suggested_take_profit,
                risk_reward_ratio, reason, confidence, status, requires_confirmation,
                metadata, generated_at, expires_at, confirmed_at, executed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(signal.id.to_string())
        .bind(&signal.strategy_id)
        .bind(&signal.strategy_name)
        .bind(&signal.symbol)
        .bind(&signal_type_json)
        .bind(signal.direction.to_string())
        .bind(strength_to_string(signal.strength))
        .bind(signal.trigger_price.to_string())
        .bind(signal.suggested_entry.map(|p| p.to_string()))
        .bind(signal.suggested_stop_loss.map(|p| p.to_string()))
        .bind(signal.suggested_take_profit.map(|p| p.to_string()))
        .bind(signal.risk_reward_ratio.map(|r| r.to_string()))
        .bind(&signal.reason)
        .bind(signal.confidence as i64)
        .bind(status_to_string(signal.status))
        .bind(if signal.requires_confirmation { 1i64 } else { 0i64 })
        .bind(&metadata)
        .bind(signal.generated_at.to_rfc3339())
        .bind(signal.expires_at.to_rfc3339())
        .bind(signal.confirmed_at.map(|t| t.to_rfc3339()))
        .bind(signal.executed_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save signal: {}", e))?;

        debug!(signal_id = %signal.id, "Signal saved");
        Ok(())
    }

    async fn find_by_id(&self, id: &SignalId) -> Result<Option<Signal>, String> {
        let row: Option<SignalRow> = sqlx::query_as("SELECT * FROM signals WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find signal: {}", e))?;

        row.map(Signal::try_from).transpose()
    }

    async fn find_active(&self, symbol: Option<&str>) -> Result<Vec<Signal>, String> {
        let mut query =
            String::from("SELECT * FROM signals WHERE status IN ('active', 'confirmed')");

        if symbol.is_some() {
            query.push_str(" AND symbol = ?");
        }

        query.push_str(" ORDER BY generated_at DESC");

        let mut q = sqlx::query_as::<_, SignalRow>(&query);

        if let Some(sym) = symbol {
            q = q.bind(sym);
        }

        let rows: Vec<SignalRow> = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find active signals: {}", e))?;

        rows.into_iter().map(Signal::try_from).collect()
    }

    async fn find_recent(&self, limit: usize) -> Result<Vec<Signal>, String> {
        let rows: Vec<SignalRow> =
            sqlx::query_as("SELECT * FROM signals ORDER BY generated_at DESC LIMIT ?")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Failed to find recent signals: {}", e))?;

        rows.into_iter().map(Signal::try_from).collect()
    }

    async fn update_status(&self, id: &SignalId, status: SignalStatus) -> Result<(), String> {
        sqlx::query("UPDATE signals SET status = ? WHERE id = ?")
            .bind(status_to_string(status))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update signal status: {}", e))?;

        debug!(signal_id = %id, status = ?status, "Signal status updated");
        Ok(())
    }

    async fn mark_executed(&self, id: &SignalId) -> Result<(), String> {
        sqlx::query("UPDATE signals SET status = 'executed', executed_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to mark signal executed: {}", e))?;

        debug!(signal_id = %id, "Signal marked as executed");
        Ok(())
    }

    async fn delete_old_signals(&self, before: DateTime<Utc>) -> Result<u64, String> {
        let result = sqlx::query(
            "DELETE FROM signals WHERE generated_at < ? AND status IN ('executed', 'cancelled', 'expired')",
        )
        .bind(before.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old signals: {}", e))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let repo = SqliteSignalRepository::new(pool.clone());
        repo.init().await.unwrap();
        pool
    }

    fn create_test_signal() -> Signal {
        Signal::new(
            "test_strategy",
            "Test Strategy",
            "BTCUSDT",
            SignalType::Custom {
                name: "test".to_string(),
                description: "test signal".to_string(),
            },
            TradeDirection::Long,
            SignalStrength::Strong,
            dec!(50000),
            "Test reason",
        )
    }

    #[tokio::test]
    async fn test_save_and_find_signal() {
        let pool = setup_test_db().await;
        let repo = SqliteSignalRepository::new(pool);

        let signal = create_test_signal();
        repo.save(&signal).await.unwrap();

        let found = repo.find_by_id(&signal.id).await.unwrap();
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.symbol, "BTCUSDT");
        assert_eq!(found.direction, TradeDirection::Long);
        assert_eq!(found.strength, SignalStrength::Strong);
    }

    #[tokio::test]
    async fn test_find_active_signals() {
        let pool = setup_test_db().await;
        let repo = SqliteSignalRepository::new(pool);

        let signal = create_test_signal();
        repo.save(&signal).await.unwrap();

        let active = repo.find_active(None).await.unwrap();
        assert_eq!(active.len(), 1);

        let active = repo.find_active(Some("BTCUSDT")).await.unwrap();
        assert_eq!(active.len(), 1);

        let active = repo.find_active(Some("ETHUSDT")).await.unwrap();
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_mark_executed() {
        let pool = setup_test_db().await;
        let repo = SqliteSignalRepository::new(pool);

        let signal = create_test_signal();
        repo.save(&signal).await.unwrap();

        repo.mark_executed(&signal.id).await.unwrap();

        let found = repo.find_by_id(&signal.id).await.unwrap().unwrap();
        assert_eq!(found.status, SignalStatus::Executed);
        assert!(found.executed_at.is_some());
    }
}
