# Data Transfer Objects (DTOs)

Estruturas de dados usadas na comunicação entre frontend e backend.

## Market Data

### Candlestick

Representa um candlestick (OHLCV).

```typescript
interface Candlestick {
  openTime: number;       // Unix timestamp ms
  open: string;           // Decimal como string
  high: string;
  low: string;
  close: string;
  volume: string;
  closeTime: number;
  quoteVolume: string;
  tradeCount: number;
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Candlestick {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_volume: String,
    pub trade_count: i64,
}
```

### SymbolInfo

Informações sobre um par de trading.

```typescript
interface SymbolInfo {
  symbol: string;           // "BTCUSDT"
  baseAsset: string;        // "BTC"
  quoteAsset: string;       // "USDT"
  status: SymbolStatus;
  pricePrecision: number;   // Casas decimais do preço
  quantityPrecision: number;// Casas decimais da quantidade
  minQuantity: string;
  maxQuantity: string;
  minNotional: string;      // Valor mínimo da ordem
  tickSize: string;         // Incremento mínimo de preço
  stepSize: string;         // Incremento mínimo de quantidade
}

type SymbolStatus = 'Trading' | 'Halt' | 'Break';
```

### PriceInfo

Preço atual com estatísticas 24h.

```typescript
interface PriceInfo {
  symbol: string;
  price: string;
  bidPrice: string;
  askPrice: string;
  change24h: string;        // Variação absoluta
  changePercent24h: string; // Variação percentual
  high24h: string;
  low24h: string;
  volume24h: string;        // Volume em base asset
  quoteVolume24h: string;   // Volume em quote asset
  timestamp: string;        // ISO 8601
}
```

### FearGreedData

Índice Fear & Greed.

```typescript
interface FearGreedData {
  value: number;            // 0-100
  classification: FearGreedClassification;
  timestamp: string;
  previousValue: number;
  previousClassification: FearGreedClassification;
  trend: 'Up' | 'Down' | 'Stable';
}

type FearGreedClassification =
  | 'ExtremeFear'    // 0-24
  | 'Fear'           // 25-44
  | 'Neutral'        // 45-55
  | 'Greed'          // 56-74
  | 'ExtremeGreed';  // 75-100
```

## Orders

### OrderRequest

Requisição para criar uma ordem.

```typescript
interface OrderRequest {
  symbol: string;
  side: OrderSide;
  orderType: OrderType;
  quantity: string;
  clientOrderId?: string;   // ID customizado (opcional)
  reduceOnly?: boolean;     // Apenas reduzir posição
}

type OrderSide = 'Buy' | 'Sell';

type OrderType =
  | { type: 'Market' }
  | { type: 'Limit'; price: string; timeInForce: TimeInForce }
  | { type: 'StopLoss'; stopPrice: string }
  | { type: 'TakeProfit'; stopPrice: string }
  | { type: 'StopLossLimit'; stopPrice: string; price: string; timeInForce: TimeInForce }
  | { type: 'TakeProfitLimit'; stopPrice: string; price: string; timeInForce: TimeInForce }
  | { type: 'TrailingStop'; callbackRate: string };  // % de callback

type TimeInForce =
  | 'GTC'  // Good Till Cancelled
  | 'IOC'  // Immediate Or Cancel
  | 'FOK'; // Fill Or Kill
```

### OrderResponse

Resposta de uma ordem.

```typescript
interface OrderResponse {
  orderId: string;
  clientOrderId: string;
  symbol: string;
  side: OrderSide;
  orderType: string;
  status: OrderStatus;
  quantity: string;
  executedQuantity: string;
  price: string;            // Preço limite (se aplicável)
  averagePrice: string;     // Preço médio de execução
  stopPrice?: string;
  commission: string;
  commissionAsset: string;
  createdAt: string;
  updatedAt: string;
  fills: OrderFill[];
}

type OrderStatus =
  | 'Pending'
  | 'New'
  | 'PartiallyFilled'
  | 'Filled'
  | 'Cancelled'
  | 'Rejected'
  | 'Expired';

interface OrderFill {
  price: string;
  quantity: string;
  commission: string;
  commissionAsset: string;
  tradeId: string;
  timestamp: string;
}
```

## Positions

### Position

Posição aberta.

```typescript
interface Position {
  id: string;
  symbol: string;
  side: PositionSide;
  quantity: string;
  entryPrice: string;
  currentPrice: string;
  liquidationPrice?: string;
  marginType?: MarginType;
  leverage?: number;
  unrealizedPnl: string;
  unrealizedPnlPercent: string;
  realizedPnl: string;
  commission: string;
  openedAt: string;
  updatedAt: string;
  strategyId?: string;
  strategyName?: string;
  stopLoss?: StopLossConfig;
  takeProfit?: TakeProfitConfig;
}

type PositionSide = 'Long' | 'Short';
type MarginType = 'Cross' | 'Isolated';

interface StopLossConfig {
  type: 'Fixed' | 'Trailing';
  price?: string;
  percentage?: string;
  activated: boolean;
}

interface TakeProfitConfig {
  price: string;
  percentage?: string;
}
```

## Strategies

### StrategySummary

Resumo de uma estratégia.

```typescript
interface StrategySummary {
  id: string;
  name: string;
  description: string;
  type: StrategyType;
  isActive: boolean;
  symbols: string[];
  timeframe: string;
  version: string;
  createdAt: string;
  lastExecutedAt?: string;
  performance?: PerformanceSummary;
}

type StrategyType =
  | 'FearGreed'
  | 'SmaCrossover'
  | 'RsiOversold'
  | 'Custom';

interface PerformanceSummary {
  totalReturn: string;
  winRate: string;
  profitFactor: string;
  totalTrades: number;
  winningTrades: number;
  losingTrades: number;
}
```

### StrategyDetails

Detalhes completos de uma estratégia.

```typescript
interface StrategyDetails {
  id: string;
  name: string;
  description: string;
  type: StrategyType;
  isActive: boolean;
  config: StrategyConfig;
  versions: StrategyVersionInfo[];
  activeVersion: string;
  performance: PerformanceMetrics;
  recentTrades: Trade[];
  signals: SignalHistory[];
}

interface StrategyConfig {
  symbols: string[];
  timeframe: string;
  indicators: IndicatorConfig[];
  entryRules: Rule[];
  exitRules: Rule[];
  riskParams: RiskParameters;
  metadata: Record<string, unknown>;
}

interface IndicatorConfig {
  type: IndicatorType;
  params: Record<string, number>;
  name?: string;
}

type IndicatorType =
  | 'SMA'
  | 'EMA'
  | 'RSI'
  | 'MACD'
  | 'BollingerBands'
  | 'ATR'
  | 'FearGreedIndex';

interface Rule {
  id: string;
  description: string;
  conditions: Condition[];
  operator: 'AND' | 'OR';
}

interface Condition {
  left: Operand;
  operator: ComparisonOperator;
  right: Operand;
}

type Operand =
  | { type: 'Indicator'; name: string; field?: string }
  | { type: 'Price'; field: 'open' | 'high' | 'low' | 'close' }
  | { type: 'Constant'; value: number };

type ComparisonOperator = 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq' | 'crosses_above' | 'crosses_below';

interface RiskParameters {
  maxPositionSize: string;
  maxDrawdownPercent: string;
  stopLossPercent?: string;
  takeProfitPercent?: string;
  trailingStopPercent?: string;
  maxDailyLossPercent?: string;
  maxOpenPositions: number;
}

interface StrategyVersionInfo {
  id: string;
  version: string;
  createdAt: string;
  changeDescription: string;
  isActive: boolean;
}
```

## Backtesting

### BacktestConfig

Configuração de backtest.

```typescript
interface BacktestConfig {
  strategyId: string;
  strategyVersionId?: string;  // Usa versão específica
  symbol: string;
  timeframe: string;
  startDate: string;           // ISO 8601
  endDate: string;
  initialCapital: string;
  commissionRate: string;      // Ex: "0.001" para 0.1%
  slippageBps: number;         // Basis points
  riskFreeRate?: string;       // Para cálculo de Sharpe
}
```

### BacktestResults

Resultados de um backtest.

```typescript
interface BacktestResults {
  jobId: string;
  config: BacktestConfig;
  summary: BacktestSummary;
  trades: BacktestTrade[];
  equityCurve: EquityPoint[];
  drawdownCurve: DrawdownPoint[];
  monthlyReturns: MonthlyReturn[];
}

interface BacktestSummary {
  initialCapital: string;
  finalCapital: string;
  totalReturn: string;
  totalReturnPercent: string;
  annualizedReturn: string;
  sharpeRatio: string;
  sortinoRatio: string;
  maxDrawdown: string;
  maxDrawdownPercent: string;
  maxDrawdownDuration: number;  // Em dias
  totalTrades: number;
  winningTrades: number;
  losingTrades: number;
  winRate: string;
  profitFactor: string;
  averageWin: string;
  averageLoss: string;
  largestWin: string;
  largestLoss: string;
  averageHoldingPeriod: number; // Em horas
  totalCommission: string;
  startDate: string;
  endDate: string;
  tradingDays: number;
}

interface BacktestTrade {
  id: number;
  entryTime: string;
  exitTime: string;
  side: PositionSide;
  entryPrice: string;
  exitPrice: string;
  quantity: string;
  pnl: string;
  pnlPercent: string;
  commission: string;
  holdingPeriodHours: number;
  entryReason: string;
  exitReason: string;
}

interface EquityPoint {
  timestamp: string;
  equity: string;
  drawdown: string;
}

interface DrawdownPoint {
  timestamp: string;
  drawdown: string;
  drawdownPercent: string;
}

interface MonthlyReturn {
  year: number;
  month: number;
  return: string;
  returnPercent: string;
  trades: number;
}
```

## Account

### AccountBalance

Saldo da conta.

```typescript
interface AccountBalance {
  totalValueUsdt: string;
  availableBalance: string;
  lockedInOrders: string;
  lockedInPositions: string;
  unrealizedPnl: string;
  marginLevel?: string;
  assets: AssetBalance[];
  lastUpdated: string;
}

interface AssetBalance {
  asset: string;
  free: string;
  locked: string;
  total: string;
  usdtValue: string;
  btcValue: string;
}
```

## System

### SystemStatus

Status do sistema.

```typescript
interface SystemStatus {
  version: string;
  buildDate: string;
  mode: TradingMode;
  components: ComponentStatus[];
  stats: SystemStats;
}

type TradingMode = 'Paper' | 'Live';

interface ComponentStatus {
  name: string;
  status: 'Ok' | 'Warning' | 'Error';
  message?: string;
  lastCheck: string;
}

interface SystemStats {
  uptimeSeconds: number;
  activeStrategies: number;
  openPositions: number;
  pendingOrders: number;
  jobsInQueue: number;
  memoryUsageMb: number;
  cpuUsagePercent: number;
}
```

### JobInfo

Informações sobre um job.

```typescript
interface JobInfo {
  id: string;
  type: JobType;
  status: JobStatus;
  priority: JobPriority;
  progress?: number;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  retryCount: number;
  error?: string;
  result?: unknown;
}

type JobType =
  | 'FetchCandles'
  | 'FetchFearGreedIndex'
  | 'ExecuteStrategy'
  | 'PlaceOrder'
  | 'CancelOrder'
  | 'RunBacktest'
  | 'CleanupOldData'
  | 'VacuumDatabase'
  | 'HealthCheck';

type JobStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled' | 'Retrying';

type JobPriority = 'Critical' | 'High' | 'Normal' | 'Low';
```

## Configuration

### AppConfig

Configuração da aplicação.

```typescript
interface AppConfig {
  general: GeneralConfig;
  trading: TradingConfig;
  marketData: MarketDataConfig;
  notifications: NotificationConfig;
  display: DisplayConfig;
}

interface GeneralConfig {
  language: 'en' | 'pt-BR';
  theme: 'light' | 'dark' | 'system';
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  dataRetentionDays: number;
}

interface TradingConfig {
  mode: TradingMode;
  defaultTimeframe: string;
  confirmOrders: boolean;
  paperInitialBalance: string;
  maxDailyLossPercent: string;
  maxOpenPositions: number;
}

interface MarketDataConfig {
  refreshIntervalMs: number;
  defaultSymbols: string[];
  enableWebsocket: boolean;
}

interface NotificationConfig {
  enabled: boolean;
  onOrderFilled: boolean;
  onPositionClosed: boolean;
  onStrategySignal: boolean;
  onError: boolean;
  sound: boolean;
}

interface DisplayConfig {
  decimalsPrice: number;
  decimalsQuantity: number;
  decimalsPercent: number;
  timezone: string;
  dateFormat: string;
}
```

## Tipos Utilitários

### Pagination

```typescript
interface PaginatedRequest {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}
```

### TimeRange

```typescript
interface TimeRange {
  start: string;  // ISO 8601
  end: string;    // ISO 8601
}
```

### Decimal Handling

Todos os valores monetários são transmitidos como `string` para preservar precisão:

```typescript
// Frontend
const price = new Decimal(response.price);
const quantity = new Decimal(response.quantity);
const total = price.times(quantity);

// Para enviar ao backend
const request = {
  price: price.toString(),
  quantity: quantity.toString(),
};
```
