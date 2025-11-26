//! Banco de dados SQLite do RoboTrade
//!
//! Gerenciamento de conexão e migrations.

mod schema;

pub use schema::*;

use robotrade_core::error::{InfraError, InfraResult};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tokio::fs;
use tracing::{debug, info};

use crate::config::database_path;

/// Pool de conexões com o banco de dados
pub type DbPool = SqlitePool;

/// Opções de configuração do banco de dados
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Caminho para o arquivo do banco
    pub path: String,
    /// Número máximo de conexões
    pub max_connections: u32,
    /// Timeout de conexão (segundos)
    pub connect_timeout_secs: u64,
    /// Criar banco se não existir
    pub create_if_missing: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: database_path().to_string_lossy().to_string(),
            max_connections: 5,
            connect_timeout_secs: 30,
            create_if_missing: true,
        }
    }
}

/// Inicializa o banco de dados
pub async fn init_database(config: &DatabaseConfig) -> InfraResult<DbPool> {
    let db_path = Path::new(&config.path);

    // Cria diretório se não existir
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| InfraError::Database(format!("Erro ao criar diretório: {}", e)))?;
        }
    }

    debug!("Conectando ao banco de dados: {:?}", db_path);

    // Configura opções de conexão
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.path))
        .map_err(|e| InfraError::Database(format!("URL inválida: {}", e)))?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(config.create_if_missing)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(config.connect_timeout_secs));

    // Cria pool de conexões
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .connect_with(options)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao conectar: {}", e)))?;

    info!("Conexão com banco de dados estabelecida");

    // Executa migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

/// Executa migrations do banco de dados
async fn run_migrations(pool: &DbPool) -> InfraResult<()> {
    debug!("Executando migrations...");

    // Cria tabelas se não existirem
    sqlx::query(SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro nas migrations: {}", e)))?;

    // Verifica versão do schema
    let version = get_schema_version(pool).await?;
    info!(version = %version, "Schema do banco de dados atualizado");

    Ok(())
}

/// Obtém a versão atual do schema
async fn get_schema_version(pool: &DbPool) -> InfraResult<i32> {
    let row: (i32,) = sqlx::query_as("SELECT version FROM schema_version ORDER BY version DESC LIMIT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| InfraError::Database(format!("Erro ao obter versão: {}", e)))?;

    Ok(row.0)
}

/// Verifica se o banco de dados está saudável
pub async fn health_check(pool: &DbPool) -> InfraResult<bool> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| InfraError::Database(format!("Health check falhou: {}", e)))?;

    Ok(true)
}

/// Fecha o pool de conexões
pub async fn close_database(pool: DbPool) {
    pool.close().await;
    info!("Conexão com banco de dados encerrada");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let config = DatabaseConfig {
            path: db_path.to_string_lossy().to_string(),
            max_connections: 1,
            connect_timeout_secs: 5,
            create_if_missing: true,
        };

        let pool = init_database(&config).await.unwrap();
        assert!(health_check(&pool).await.unwrap());

        close_database(pool).await;
    }
}
