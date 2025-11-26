# Módulo Infra

O crate `robotrade-infra` fornece a infraestrutura de suporte: configuração, banco de dados, logging e persistência.

## Estrutura de Diretórios

```
crates/infra/src/
├── lib.rs              # Exportações públicas
├── config/             # Sistema de configuração
│   └── mod.rs          # AppConfig, GeneralConfig, etc.
├── database/           # Banco de dados SQLite
│   ├── mod.rs          # Pool, migrations, schema
│   ├── repositories/   # Implementações de repositórios
│   │   ├── candle.rs   # SqliteCandleRepository
│   │   └── fear_greed.rs # SqliteFearGreedRepository
│   └── schema.sql      # Schema do banco
└── logging/            # Logging estruturado
    └── mod.rs          # Configuração do tracing
```

## Configuração

### Estrutura de Configuração

```rust
/// Configuração principal do RoboTrade
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub trading: TradingConfig,
    pub data_collection: DataCollectionConfig,
    pub notifications: NotificationConfig,
    pub logging: LoggingConfig,
    pub exchange: ExchangeConfig,
}
```

### Configurações Específicas

```rust
/// Configurações gerais
pub struct GeneralConfig {
    pub app_name: String,
    pub version: String,
    pub environment: Environment,
}

/// Configurações de trading
pub struct TradingConfig {
    pub mode: TradingMode,           // Paper ou Live
    pub default_leverage: u32,
    pub max_open_positions: u32,
    pub risk_per_trade_pct: Decimal,
    pub daily_loss_limit_pct: Decimal,
}

/// Configurações de coleta de dados
pub struct DataCollectionConfig {
    pub fear_greed_enabled: bool,
    pub candles_symbols: Vec<String>,
    pub candles_timeframes: Vec<String>,
}

/// Configurações de logging
pub struct LoggingConfig {
    pub level: String,              // trace, debug, info, warn, error
    pub format: LogFormat,          // json, pretty
    pub file_enabled: bool,
    pub file_path: Option<PathBuf>,
}

/// Configurações de exchange
pub struct ExchangeConfig {
    pub binance_testnet: bool,
    pub binance_api_key: Option<String>,
    pub binance_api_secret: Option<String>,
}
```

### Carregamento de Configuração

```rust
use robotrade_infra::config::{AppConfig, config_path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carrega configuração do arquivo ou cria padrão
    let config = AppConfig::load().await?;

    println!("Modo: {:?}", config.trading.mode);
    println!("Alavancagem: {}x", config.trading.default_leverage);

    // Salvar configuração
    config.save().await?;

    Ok(())
}
```

### Arquivo de Configuração (TOML)

```toml
# ~/.config/robotrade/config.toml

[general]
app_name = "RoboTrade"
environment = "development"

[trading]
mode = "paper"
default_leverage = 10
max_open_positions = 3
risk_per_trade_pct = "2.0"
daily_loss_limit_pct = "5.0"

[data_collection]
fear_greed_enabled = true
candles_symbols = ["BTCUSDT", "ETHUSDT"]
candles_timeframes = ["1h", "4h", "1d"]

[logging]
level = "info"
format = "pretty"
file_enabled = true

[exchange]
binance_testnet = true
# API keys devem ser configuradas via variáveis de ambiente
```

### Diretórios do Sistema

```rust
use robotrade_infra::config::{config_path, database_path, logs_path};

// Linux: ~/.config/robotrade/config.toml
// macOS: ~/Library/Application Support/br.cypherpunk.robotrade/config.toml
// Windows: C:\Users\<User>\AppData\Roaming\cypherpunk\robotrade\config.toml
let config = config_path();

// Banco de dados
let db = database_path(); // .../robotrade.db

// Logs
let logs = logs_path(); // .../logs/
```

## Banco de Dados

### Inicialização

```rust
use robotrade_infra::database::{init_database, health_check, close_database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa banco de dados (cria se não existir)
    let pool = init_database().await?;

    // Verificar saúde
    if health_check(&pool).await {
        println!("Banco de dados OK!");
    }

    // Usar pool...

    // Fechar conexões
    close_database(&pool).await;

    Ok(())
}
```

### Schema do Banco

```sql
-- Versão do schema
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Configurações key-value
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Dados Fear & Greed
CREATE TABLE IF NOT EXISTS fear_greed_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    value INTEGER NOT NULL,
    classification TEXT NOT NULL,
    timestamp DATETIME NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Candles OHLCV
CREATE TABLE IF NOT EXISTS candles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    open_time DATETIME NOT NULL,
    close_time DATETIME NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume REAL NOT NULL,
    quote_volume REAL,
    trade_count INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(symbol, timeframe, open_time)
);

-- Ordens
CREATE TABLE IF NOT EXISTS orders (
    id TEXT PRIMARY KEY,
    exchange TEXT NOT NULL,
    exchange_order_id TEXT,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    order_type TEXT NOT NULL,
    quantity REAL NOT NULL,
    price REAL,
    stop_price REAL,
    filled_quantity REAL DEFAULT 0,
    avg_fill_price REAL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    position_id TEXT,
    signal_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Posições
CREATE TABLE IF NOT EXISTS positions (
    id TEXT PRIMARY KEY,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity REAL NOT NULL,
    entry_price REAL NOT NULL,
    leverage INTEGER NOT NULL,
    margin REAL NOT NULL,
    unrealized_pnl REAL DEFAULT 0,
    stop_loss REAL,
    take_profit REAL,
    status TEXT NOT NULL,
    entry_order_id TEXT,
    exit_order_id TEXT,
    signal_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    closed_at DATETIME
);

-- Trades (histórico)
CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    position_id TEXT NOT NULL,
    signal_id TEXT,
    entry_order_id TEXT NOT NULL,
    exit_order_id TEXT NOT NULL,
    quantity REAL NOT NULL,
    entry_price REAL NOT NULL,
    exit_price REAL NOT NULL,
    gross_pnl REAL NOT NULL,
    total_fees REAL NOT NULL,
    net_pnl REAL NOT NULL,
    pnl_pct REAL NOT NULL,
    roi_pct REAL NOT NULL,
    leverage INTEGER NOT NULL,
    close_reason TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    entered_at DATETIME NOT NULL,
    exited_at DATETIME NOT NULL,
    metadata TEXT
);

-- Sinais
CREATE TABLE IF NOT EXISTS signals (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    strength TEXT NOT NULL,
    trigger_price REAL NOT NULL,
    confidence REAL,
    stop_loss REAL,
    take_profit REAL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    executed_at DATETIME,
    expired_at DATETIME,
    metadata TEXT
);

-- Índices para performance
CREATE INDEX IF NOT EXISTS idx_candles_symbol_timeframe ON candles(symbol, timeframe);
CREATE INDEX IF NOT EXISTS idx_candles_open_time ON candles(open_time);
CREATE INDEX IF NOT EXISTS idx_orders_symbol ON orders(symbol);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
CREATE INDEX IF NOT EXISTS idx_trades_exited_at ON trades(exited_at);
CREATE INDEX IF NOT EXISTS idx_signals_symbol ON signals(symbol);
CREATE INDEX IF NOT EXISTS idx_signals_status ON signals(status);
CREATE INDEX IF NOT EXISTS idx_fear_greed_timestamp ON fear_greed_data(timestamp);
```

### Repositórios

#### SqliteCandleRepository

```rust
use robotrade_infra::database::SqliteCandleRepository;
use robotrade_core::traits::CandleRepository;

let repo = SqliteCandleRepository::new(pool.clone());

// Salvar candle
repo.save(&candle, "BTCUSDT", TimeFrame::H4).await?;

// Buscar range
let candles = repo.get_range(
    "BTCUSDT",
    TimeFrame::H4,
    start_time,
    end_time,
    1000,
).await?;

// Buscar último candle
let latest = repo.get_latest("BTCUSDT", TimeFrame::H4).await?;
```

#### SqliteFearGreedRepository

```rust
use robotrade_infra::database::SqliteFearGreedRepository;
use robotrade_core::traits::FearGreedRepository;

let repo = SqliteFearGreedRepository::new(pool.clone());

// Salvar dados
repo.save(&fear_greed_data).await?;

// Buscar atual
let current = repo.get_current().await?;

// Buscar histórico
let history = repo.get_history(30).await?;
```

## Logging

### Configuração

```rust
use robotrade_infra::logging::init_logging;

fn main() {
    // Inicializa logging com tracing
    init_logging();

    tracing::info!("Aplicação iniciada");
    tracing::debug!(symbol = "BTCUSDT", "Processando símbolo");
    tracing::warn!(reason = "rate_limit", "API rate limited");
    tracing::error!(error = ?err, "Erro ao conectar");
}
```

### Níveis de Log

| Nível | Uso |
|-------|-----|
| trace | Detalhes de debug muito granulares |
| debug | Informações de desenvolvimento |
| info | Eventos normais de operação |
| warn | Situações anômalas mas tratáveis |
| error | Erros que precisam de atenção |

### Formatação JSON

```json
{
  "timestamp": "2024-11-26T15:30:00Z",
  "level": "INFO",
  "target": "robotrade_app",
  "message": "Ordem executada",
  "fields": {
    "order_id": "abc123",
    "symbol": "BTCUSDT",
    "side": "buy",
    "quantity": "0.001"
  }
}
```

### Arquivo de Log

```rust
// Logs são salvos em:
// ~/.local/share/robotrade/logs/robotrade.log (Linux)
// ~/Library/Application Support/br.cypherpunk.robotrade/logs/robotrade.log (macOS)
```

## Variáveis de Ambiente

| Variável | Descrição | Padrão |
|----------|-----------|--------|
| `RUST_LOG` | Nível de log | `info` |
| `BINANCE_API_KEY` | API key da Binance | - |
| `BINANCE_API_SECRET` | API secret da Binance | - |
| `ROBOTRADE_CONFIG` | Caminho do config | Sistema |
| `ROBOTRADE_DATA` | Caminho dos dados | Sistema |
