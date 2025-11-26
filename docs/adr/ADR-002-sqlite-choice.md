# ADR-002: Escolha do SQLite como Banco de Dados

## Status

Aceita

## Contexto

O RoboTrade precisa persistir diversos tipos de dados:

1. **Dados de mercado**: Candlesticks históricos (potencialmente milhões de registros)
2. **Configurações**: Estratégias, pares monitorados, preferências do usuário
3. **Estado operacional**: Ordens, posições, execuções
4. **Métricas**: Resultados de backtests, performance de estratégias
5. **Logs estruturados**: Eventos de trading para auditoria

Requisitos identificados:

- Aplicação desktop que roda localmente
- Não requer acesso remoto ao banco
- Usuário não deve precisar instalar software adicional
- Performance adequada para séries temporais
- Suporte a transações ACID
- Backup simples (arquivo único)

## Decisão

Adotamos **SQLite** como banco de dados principal, utilizando a biblioteca **sqlx** para acesso type-safe e async.

### Configuração

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30))
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}
```

### Otimizações Aplicadas

1. **WAL Mode**: Write-Ahead Logging para melhor concorrência
2. **Synchronous Normal**: Balanço entre performance e durabilidade
3. **Foreign Keys**: Habilitado explicitamente para integridade referencial
4. **Connection Pool**: 5 conexões para paralelismo controlado
5. **Busy Timeout**: 30s para evitar falhas em contenção

### Estrutura de Tabelas Principais

```sql
-- Candlesticks com índice otimizado para range queries
CREATE TABLE candlesticks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    open_time INTEGER NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume REAL NOT NULL,
    close_time INTEGER NOT NULL,
    quote_volume REAL,
    trade_count INTEGER,
    UNIQUE(symbol, timeframe, open_time)
);

CREATE INDEX idx_candles_symbol_time ON candlesticks(symbol, timeframe, open_time DESC);

-- Particionamento lógico por período (via queries)
-- SQLite não suporta particionamento nativo, mas podemos usar
-- múltiplos arquivos se necessário no futuro
```

### Migrations com sqlx

```rust
// Executadas automaticamente no startup
sqlx::migrate!("./migrations").run(&pool).await?;
```

## Consequências

### Positivas

- **Zero configuração**: Funciona out-of-the-box, arquivo único
- **Portabilidade**: Usuário pode copiar/backup o arquivo .db facilmente
- **Performance local**: Sem latência de rede, muito rápido para reads
- **Type-safety**: sqlx verifica queries em compile-time
- **ACID garantido**: Transações confiáveis mesmo com crash
- **Footprint pequeno**: ~1MB de overhead do engine
- **Cross-platform**: Funciona em Windows, macOS, Linux sem mudanças

### Negativas

- **Escalabilidade limitada**: Não adequado para múltiplos usuários simultâneos
- **Sem replicação nativa**: Backup manual ou via ferramentas externas
- **Writes sequenciais**: Apenas um writer por vez (mitigado com WAL)
- **Features limitadas**: Sem stored procedures, triggers limitados
- **Tipos numéricos**: REAL é float64, precisão limitada (mitigado com TEXT para decimais críticos)

### Neutras

- Arquivo de banco cresce com o tempo (vacuum periódico recomendado)
- Queries complexas podem ser mais lentas que PostgreSQL
- Não há servidor para monitorar, mas também não há métricas built-in

## Alternativas Consideradas

### Alternativa 1: PostgreSQL

- **Descrição**: Banco relacional completo client-server
- **Prós**: Features avançadas, melhor para grandes volumes, extensões (TimescaleDB)
- **Contras**: Requer instalação separada, configuração, manutenção
- **Motivo da rejeição**: Complexidade desnecessária para aplicação desktop single-user

### Alternativa 2: DuckDB

- **Descrição**: Banco OLAP embeddable, otimizado para analytics
- **Prós**: Excelente para queries analíticas, columnar storage
- **Contras**: Ecossistema menor em Rust, menos maduro para OLTP
- **Motivo da rejeição**: Nosso caso é mais OLTP (ordens, posições) que OLAP

### Alternativa 3: Arquivos JSON/TOML

- **Descrição**: Persistência simples em arquivos de texto
- **Prós**: Máxima simplicidade, human-readable
- **Contras**: Sem queries, sem transações, não escala para séries temporais
- **Motivo da rejeição**: Inadequado para volume de dados esperado

### Alternativa 4: RocksDB/Sled

- **Descrição**: Key-value stores embeddables de alta performance
- **Prós**: Muito rápido para writes, compressão eficiente
- **Contras**: Sem SQL, model de dados menos flexível
- **Motivo da rejeição**: Preferimos modelo relacional para queries complexas

## Mitigações para Limitações

### Precisão Decimal

```rust
// Armazenamos preços como TEXT e convertemos
#[derive(sqlx::FromRow)]
struct CandleRow {
    open: String,  // "1234.56789012"
    // ...
}

impl From<CandleRow> for Candlestick {
    fn from(row: CandleRow) -> Self {
        Candlestick {
            open: Decimal::from_str(&row.open).unwrap(),
            // ...
        }
    }
}
```

### Volume de Dados

```rust
// Agregação e limpeza periódica
pub async fn cleanup_old_candles(
    pool: &SqlitePool,
    symbol: &str,
    keep_days: i64,
) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now() - Duration::days(keep_days);

    let result = sqlx::query!(
        "DELETE FROM candlesticks WHERE symbol = ? AND open_time < ?",
        symbol,
        cutoff.timestamp_millis()
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
```

## Referências

- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [sqlx - Rust SQL Toolkit](https://github.com/launchbadge/sqlx)
- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [Appropriate Uses For SQLite](https://www.sqlite.org/whentouse.html)
