# Arquitetura do RoboTrade - Detalhada

## Workspace de 7 Crates

```
┌─────────────────────────────────────────────────────┐
│         src-tauri (Tauri App - Integration)         │
│  - Comandos IPC (60+)                               │
│  - State management (AppState)                      │
│  - Services (exchange, market_data, worker)         │
└─────────────────────────────────────────────────────┘
                         │ depende de ↓
    ┌────────────────────┼────────────────────┐
    │                    │                    │
┌───▼────┐  ┌───────────▼──┐  ┌──────────────▼───────┐  ┌───────────────┐
│trading_│  │ analytics    │  │exchange_gateways     │  │market_data    │
│worker  │  │              │  │                      │  │               │
│        │  │- indicators  │  │- binance (complete)  │  │- fear_greed   │
│- risk  │  │- strategies  │  │- kraken (complete)   │  │- providers    │
│- queue │  │- backtest    │  │- websocket           │  │               │
└────┬───┘  └───────┬──────┘  └──────────┬───────────┘  └───────┬───────┘
     │              │                    │                      │
     │ depende de   │ depende de         │ depende de          │
     └──────────────┼────────────────────┼─────────────────────┘
                    │                    │
                ┌───▼────────────────────▼───┐
                │     infra                  │
                │  - config (TOML)           │
                │  - database (SQLite)       │
                │  - logging (tracing)       │
                │  - repositories            │
                └───────────┬────────────────┘
                            │ depende de
                      ┌─────▼──────┐
                      │    core    │
                      │            │
                      │- entities  │
                      │- traits    │
                      │- errors    │
                      │- dto       │
                      │            │
                      │NO INTERNAL │
                      │DEPENDENCIES│
                      └────────────┘
```

## Fluxo de Dados

### Trading Manual (usuário cria ordem)

```
Frontend React
    ↓ invoke('create_order')
Tauri Command (commands.rs)
    ↓ valida input
ExchangeService
    ↓ roteia por exchange
BinanceFuturesClient / KrakenFuturesClient
    ↓ assina request (HMAC)
Exchange API (Binance/Kraken)
    ↓ executa ordem
Response
    ↓ parse
Order entity (core)
    ↓ salva
Database (SQLite)
    ↓ atualiza estado
Frontend (via refresh)
```

### Trading Automático (worker)

```
TradingWorker (loop a cada 1min)
    ↓
Collector → busca candles
    ↓
Strategy.evaluate(candles) → gera Signal
    ↓
RiskManager.can_open_position(signal)?
    ├─ NÃO → descarta sinal
    └─ SIM ↓
JobQueue.enqueue(signal)
    ↓
Worker processa job
    ↓
ExchangeGateway.place_order()
    ↓
Order executada
    ↓
PositionManager.track(position)
    ↓
Loop continua...
```

### Backtest

```
Usuario seleciona estratégia + período
    ↓
CandleRepository.fetch_history()
    ↓
BacktestEngine.run(candles, strategy)
    ↓
Loop por cada candle:
    ├─ Strategy.evaluate() → signal?
    ├─ Se sinal → simula entrada
    ├─ Calcula PnL
    └─ Checa SL/TP
    ↓
Calcula métricas finais:
    - Total return
    - Sharpe ratio
    - Max drawdown
    - Win rate
    - Equity curve
    ↓
Retorna BacktestResult
```

## Traits e Implementações

### ExchangeGateway

**Definido em:** `core/src/traits.rs`

**Implementado por:**
- `BinanceFuturesClient` (exchange_gateways)
- `KrakenFuturesClient` (exchange_gateways)

**Métodos principais:**
```rust
trait ExchangeGateway {
    async fn get_balance() -> Vec<Balance>;
    async fn get_positions() -> Vec<Position>;
    async fn place_order(order: Order) -> Order;
    async fn cancel_order(symbol, order_id);
    async fn set_leverage(symbol, leverage);
    // ... 15+ métodos
}
```

### Strategy

**Definido em:** `core/src/traits.rs`

**Implementado por:**
- `FearGreedStrategy` (analytics)
- (outros que tu criar)

**Métodos:**
```rust
trait Strategy {
    async fn evaluate(candles: &[Candle]) -> Option<Signal>;
    async fn should_close_position(position, price) -> bool;
    fn name() -> &str;
}
```

### Repository<T, Id>

**Definido em:** `core/src/traits.rs`

**Implementado por:**
- `SqliteCandleRepository` (infra)
- `SqliteFearGreedRepository` (infra)
- `SqliteExchangeRepository` (infra)
- `SqliteSymbolRepository` (infra)
- `SqliteLedgerRepository` (infra)
- ... (mais 4 planejados)

**Métodos:**
```rust
trait Repository<T, Id> {
    async fn save(entity: &T) -> Result<T>;
    async fn find_by_id(id: Id) -> Result<Option<T>>;
    async fn find_all() -> Result<Vec<T>>;
    async fn delete(id: Id) -> Result<()>;
}
```

## State Management

### AppState (Backend)

Thread-safe, compartilhado entre todos comandos Tauri:

```rust
pub struct AppState {
    pub trading_mode: RwLock<TradingMode>,         // Paper/Live
    pub fear_greed_cache: RwLock<Option<FearGreedData>>,
    pub positions_cache: RwLock<Vec<Position>>,
    pub daily_pnl: RwLock<Decimal>,
    pub connection_status: RwLock<ConnectionStatus>,
    pub config: RwLock<Config>,
    // ... mais campos
}
```

Acesso:
```rust
#[tauri::command]
async fn meu_comando(state: tauri::State<'_, AppState>) -> Result<String> {
    let mode = state.trading_mode.read().await;
    // usa mode
}
```

### Frontend State

**Não usa Redux/Zustand!**

Usa **React state local** + **contexts**:
- `GlobalFilterContext` - Filtros globais
- Props drilling onde necessário
- Fetch via `invoke()` quando precisa

**Atualização:** Polling a cada 3-5 segundos.

## Database Schema

### Principais Tabelas

```sql
-- Configurações
settings (key TEXT PRIMARY KEY, value TEXT)

-- Market Data
candles (symbol, interval, open_time, OHLCV, volume)
fear_greed_data (value, classification, timestamp)

-- Trading
orders (id, exchange, symbol, side, type, status, ...)
positions (id, exchange, symbol, side, quantity, entry_price, ...)
trades (id, entry/exit prices, pnl, fees, duration, ...)

-- Signals
signals (strategy_name, signal_type, strength, timestamp)

-- Backtest
backtest_results (strategy, period, metrics_json, trades_json)

-- Ledger (contabilidade)
ledger_entries (type, asset, quantity, price, timestamp)
cost_basis_lots (asset, quantity, cost_basis, acquisition_date)
reconciliation_snapshots (exchange, ledger_balances, exchange_balances)
sync_state (exchange, last_sync, status)

-- Auditoria
audit_log (action, user, timestamp, details)
```

### Índices

Todos otimizados:
- `idx_candles_symbol_interval_time`
- `idx_orders_symbol_status`
- `idx_positions_symbol_status`
- `idx_trades_exit_time`
- ... (20+ índices)

## Performance

### Async Todo o Caminho

- Frontend → Tauri → Rust backend → Exchange API
- Tudo non-blocking
- Requests paralelos onde possível

### Caching

- Fear & Greed: Cache de 30min (atualiza em background)
- Positions/Orders: Cache com refresh a cada 5s
- Candles: Persistidos no SQLite

### Rate Limiting

- Binance: 1200 req/min = governor com 20 req/s
- Kraken: Variável por endpoint
- WebSocket: Sem limite (subscriptions)

---

**Arquitetura robusta e escalável! 🏗️**
