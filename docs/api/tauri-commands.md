# Tauri Commands

Referência completa dos comandos IPC disponíveis no RoboTrade.

## Market Data

### get_candles

Busca candlesticks históricos.

```rust
#[tauri::command]
pub async fn get_candles(
    state: State<'_, AppState>,
    symbol: String,
    timeframe: String,
    limit: u32,
) -> Result<Vec<Candlestick>, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `symbol` | `string` | Par de trading (ex: "BTCUSDT") |
| `timeframe` | `string` | Intervalo ("1m", "5m", "15m", "1h", "4h", "1d") |
| `limit` | `number` | Número de candles (max: 1000) |

**Retorno:** `Candlestick[]`

**Exemplo:**

```typescript
const candles = await invoke<Candlestick[]>('get_candles', {
  symbol: 'BTCUSDT',
  timeframe: '1h',
  limit: 100,
});
```

---

### get_current_price

Obtém preço atual de um símbolo.

```rust
#[tauri::command]
pub async fn get_current_price(
    state: State<'_, AppState>,
    symbol: String,
) -> Result<PriceInfo, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `symbol` | `string` | Par de trading |

**Retorno:** `PriceInfo`

```typescript
interface PriceInfo {
  symbol: string;
  price: string;      // Decimal como string
  change24h: string;  // Variação 24h em %
  volume24h: string;  // Volume 24h
  timestamp: string;  // ISO 8601
}
```

---

### get_symbols

Lista símbolos disponíveis para trading.

```rust
#[tauri::command]
pub async fn get_symbols(
    state: State<'_, AppState>,
    quote_asset: Option<String>,
) -> Result<Vec<SymbolInfo>, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `quote_asset` | `string?` | Filtrar por moeda de cotação (ex: "USDT") |

**Retorno:** `SymbolInfo[]`

---

### get_fear_greed_index

Obtém o índice Fear & Greed atual.

```rust
#[tauri::command]
pub async fn get_fear_greed_index(
    state: State<'_, AppState>,
) -> Result<FearGreedData, String>
```

**Retorno:** `FearGreedData`

```typescript
interface FearGreedData {
  value: number;           // 0-100
  classification: string;  // "Extreme Fear", "Fear", "Neutral", "Greed", "Extreme Greed"
  timestamp: string;
  previousValue: number;
  previousClassification: string;
}
```

---

## Orders

### place_order

Envia uma ordem para a exchange.

```rust
#[tauri::command]
pub async fn place_order(
    state: State<'_, AppState>,
    request: OrderRequest,
    mode_confirmation: Option<String>,
) -> Result<OrderResponse, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `request` | `OrderRequest` | Detalhes da ordem |
| `mode_confirmation` | `string?` | "CONFIRM_LIVE_ORDER" para modo live |

**Tipos:**

```typescript
interface OrderRequest {
  symbol: string;
  side: 'Buy' | 'Sell';
  orderType: OrderType;
  quantity: string;
  clientOrderId?: string;
}

type OrderType =
  | { type: 'Market' }
  | { type: 'Limit'; price: string; timeInForce: TimeInForce }
  | { type: 'StopLoss'; stopPrice: string }
  | { type: 'TakeProfit'; stopPrice: string }
  | { type: 'StopLossLimit'; stopPrice: string; price: string; timeInForce: TimeInForce };

type TimeInForce = 'GTC' | 'IOC' | 'FOK';

interface OrderResponse {
  orderId: string;
  clientOrderId: string;
  symbol: string;
  status: OrderStatus;
  side: 'Buy' | 'Sell';
  orderType: string;
  quantity: string;
  filledQuantity: string;
  price: string;
  averagePrice: string;
  createdAt: string;
  updatedAt: string;
}

type OrderStatus = 'Pending' | 'PartiallyFilled' | 'Filled' | 'Cancelled' | 'Rejected' | 'Expired';
```

**Exemplo:**

```typescript
// Ordem de mercado
const marketOrder = await invoke<OrderResponse>('place_order', {
  request: {
    symbol: 'BTCUSDT',
    side: 'Buy',
    orderType: { type: 'Market' },
    quantity: '0.001',
  },
});

// Ordem limitada
const limitOrder = await invoke<OrderResponse>('place_order', {
  request: {
    symbol: 'BTCUSDT',
    side: 'Buy',
    orderType: {
      type: 'Limit',
      price: '50000.00',
      timeInForce: 'GTC',
    },
    quantity: '0.001',
  },
});
```

---

### cancel_order

Cancela uma ordem aberta.

```rust
#[tauri::command]
pub async fn cancel_order(
    state: State<'_, AppState>,
    symbol: String,
    order_id: String,
) -> Result<OrderResponse, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `symbol` | `string` | Par de trading |
| `order_id` | `string` | ID da ordem a cancelar |

---

### get_open_orders

Lista ordens abertas.

```rust
#[tauri::command]
pub async fn get_open_orders(
    state: State<'_, AppState>,
    symbol: Option<String>,
) -> Result<Vec<OrderResponse>, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `symbol` | `string?` | Filtrar por símbolo (opcional) |

---

### get_order_history

Busca histórico de ordens.

```rust
#[tauri::command]
pub async fn get_order_history(
    state: State<'_, AppState>,
    filter: OrderHistoryFilter,
) -> Result<Vec<OrderResponse>, String>
```

**Parâmetros:**

```typescript
interface OrderHistoryFilter {
  symbol?: string;
  status?: OrderStatus;
  side?: 'Buy' | 'Sell';
  startTime?: string;  // ISO 8601
  endTime?: string;    // ISO 8601
  limit?: number;
}
```

---

## Positions

### get_positions

Lista posições atuais.

```rust
#[tauri::command]
pub async fn get_positions(
    state: State<'_, AppState>,
) -> Result<Vec<Position>, String>
```

**Retorno:** `Position[]`

```typescript
interface Position {
  id: string;
  symbol: string;
  side: 'Long' | 'Short';
  quantity: string;
  entryPrice: string;
  currentPrice: string;
  unrealizedPnl: string;
  unrealizedPnlPercent: string;
  openedAt: string;
  strategyId?: string;
}
```

---

### close_position

Fecha uma posição aberta.

```rust
#[tauri::command]
pub async fn close_position(
    state: State<'_, AppState>,
    position_id: String,
    percentage: Option<f64>,
) -> Result<OrderResponse, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `position_id` | `string` | ID da posição |
| `percentage` | `number?` | Percentual a fechar (default: 100%) |

---

## Strategies

### get_strategies

Lista estratégias configuradas.

```rust
#[tauri::command]
pub async fn get_strategies(
    state: State<'_, AppState>,
) -> Result<Vec<StrategySummary>, String>
```

**Retorno:** `StrategySummary[]`

```typescript
interface StrategySummary {
  id: string;
  name: string;
  description: string;
  isActive: boolean;
  symbols: string[];
  version: string;
  performance?: PerformanceMetrics;
}

interface PerformanceMetrics {
  totalReturn: string;
  winRate: string;
  profitFactor: string;
  totalTrades: number;
  sharpeRatio: string;
}
```

---

### get_strategy_details

Obtém detalhes completos de uma estratégia.

```rust
#[tauri::command]
pub async fn get_strategy_details(
    state: State<'_, AppState>,
    strategy_id: String,
) -> Result<StrategyDetails, String>
```

---

### activate_strategy

Ativa uma estratégia para execução.

```rust
#[tauri::command]
pub async fn activate_strategy(
    state: State<'_, AppState>,
    strategy_id: String,
) -> Result<(), String>
```

---

### deactivate_strategy

Desativa uma estratégia.

```rust
#[tauri::command]
pub async fn deactivate_strategy(
    state: State<'_, AppState>,
    strategy_id: String,
) -> Result<(), String>
```

---

### update_strategy_config

Atualiza configuração de uma estratégia.

```rust
#[tauri::command]
pub async fn update_strategy_config(
    state: State<'_, AppState>,
    strategy_id: String,
    config: StrategyConfig,
    version_bump: String,
    description: String,
) -> Result<StrategyVersion, String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `strategy_id` | `string` | ID da estratégia |
| `config` | `StrategyConfig` | Nova configuração |
| `version_bump` | `string` | "major", "minor", ou "patch" |
| `description` | `string` | Descrição da mudança |

---

## Backtesting

### run_backtest

Executa um backtest.

```rust
#[tauri::command]
pub async fn run_backtest(
    state: State<'_, AppState>,
    config: BacktestConfig,
) -> Result<String, String>  // Retorna job_id
```

**Parâmetros:**

```typescript
interface BacktestConfig {
  strategyId: string;
  symbol: string;
  timeframe: string;
  startDate: string;     // ISO 8601
  endDate: string;       // ISO 8601
  initialCapital: string;
  commissionRate: string;
  slippageBps: number;
}
```

**Retorno:** `string` (job_id para acompanhamento)

---

### get_backtest_status

Obtém status de um backtest em execução.

```rust
#[tauri::command]
pub async fn get_backtest_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<BacktestStatus, String>
```

**Retorno:**

```typescript
interface BacktestStatus {
  jobId: string;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed';
  progress: number;      // 0-100
  currentDate?: string;
  tradesProcessed?: number;
  error?: string;
}
```

---

### get_backtest_results

Obtém resultados de um backtest concluído.

```rust
#[tauri::command]
pub async fn get_backtest_results(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<BacktestResults, String>
```

---

## Account

### get_balance

Obtém saldo da conta.

```rust
#[tauri::command]
pub async fn get_balance(
    state: State<'_, AppState>,
) -> Result<AccountBalance, String>
```

**Retorno:**

```typescript
interface AccountBalance {
  totalValue: string;      // Em USDT
  availableBalance: string;
  lockedBalance: string;
  assets: AssetBalance[];
}

interface AssetBalance {
  asset: string;
  free: string;
  locked: string;
  usdValue: string;
}
```

---

### get_trading_mode

Retorna o modo de trading atual.

```rust
#[tauri::command]
pub async fn get_trading_mode(
    state: State<'_, AppState>,
) -> Result<TradingMode, String>
```

**Retorno:**

```typescript
type TradingMode = 'Paper' | 'Live';
```

---

### set_trading_mode

Altera o modo de trading.

```rust
#[tauri::command]
pub async fn set_trading_mode(
    state: State<'_, AppState>,
    mode: String,
    confirmation: Option<String>,
) -> Result<(), String>
```

**Parâmetros:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `mode` | `string` | "Paper" ou "Live" |
| `confirmation` | `string?` | "CONFIRM_LIVE_MODE" para ativar live |

---

## Configuration

### get_config

Obtém configuração atual.

```rust
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<AppConfig, String>
```

---

### update_config

Atualiza configuração.

```rust
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String>
```

---

### validate_api_credentials

Valida credenciais de API.

```rust
#[tauri::command]
pub async fn validate_api_credentials(
    state: State<'_, AppState>,
    api_key: String,
    secret_key: String,
) -> Result<CredentialValidation, String>
```

**Retorno:**

```typescript
interface CredentialValidation {
  valid: boolean;
  permissions: string[];  // ["SPOT", "MARGIN", etc]
  ipRestriction: boolean;
  error?: string;
}
```

---

## System

### get_system_status

Obtém status do sistema.

```rust
#[tauri::command]
pub async fn get_system_status(
    state: State<'_, AppState>,
) -> Result<SystemStatus, String>
```

**Retorno:**

```typescript
interface SystemStatus {
  version: string;
  mode: TradingMode;
  databaseStatus: 'Ok' | 'Error';
  exchangeStatus: 'Connected' | 'Disconnected' | 'RateLimited';
  workerStatus: 'Running' | 'Stopped' | 'Error';
  activeStrategies: number;
  openPositions: number;
  pendingOrders: number;
  uptimeSeconds: number;
  lastSync: string;
}
```

---

### get_jobs

Lista jobs em execução/pendentes.

```rust
#[tauri::command]
pub async fn get_jobs(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<JobInfo>, String>
```

---

### cancel_job

Cancela um job.

```rust
#[tauri::command]
pub async fn cancel_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String>
```

---

## Telemetry

### get_logs

Busca logs do sistema.

```rust
#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    filter: LogFilter,
) -> Result<Vec<LogEntry>, String>
```

**Parâmetros:**

```typescript
interface LogFilter {
  level?: 'debug' | 'info' | 'warn' | 'error';
  target?: string;       // Ex: "robotrade_trading"
  startTime?: string;
  endTime?: string;
  search?: string;       // Busca em mensagem
  limit?: number;
}

interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  fields: Record<string, unknown>;
}
```

---

### get_metrics

Obtém métricas do sistema.

```rust
#[tauri::command]
pub async fn get_metrics(
    state: State<'_, AppState>,
) -> Result<Metrics, String>
```

**Retorno:**

```typescript
interface Metrics {
  ordersPlaced: number;
  ordersFilled: number;
  ordersCancelled: number;
  ordersRejected: number;
  apiCalls: number;
  apiErrors: number;
  avgLatencyMs: number;
  p99LatencyMs: number;
}
```
