// Tipos que espelham os DTOs do Rust

export type TradingMode = 'paper' | 'live';

export type ServiceStatus = 'connected' | 'disconnected' | 'connecting' | 'error';

export type PositionSide = 'long' | 'short';

export type OrderSide = 'buy' | 'sell';

export type OrderType = 'market' | 'limit' | 'stop_market' | 'stop_limit' | 'take_profit_market' | 'take_profit_limit';

export type OrderStatus = 'pending' | 'open' | 'partially_filled' | 'filled' | 'cancelled' | 'rejected' | 'expired';

export type FearGreedClassification = 'extreme_fear' | 'fear' | 'neutral' | 'greed' | 'extreme_greed';

export type SignalStrength = 'weak' | 'moderate' | 'strong';

export type TradeDirection = 'long' | 'short';

export interface ConnectionStatus {
  binance: ServiceStatus;
  fear_greed_api: ServiceStatus;
  database: ServiceStatus;
}

export interface DashboardSummary {
  trading_mode: TradingMode;
  total_balance_usdt: string;
  daily_pnl: string;
  daily_pnl_pct: string;
  open_positions_count: number;
  active_orders_count: number;
  active_signals_count: number;
  fear_greed_value: number | null;
  fear_greed_classification: FearGreedClassification | null;
  connection_status: ConnectionStatus;
  updated_at: string;
}

export interface FearGreedDto {
  value: number;
  classification: FearGreedClassification;
  classification_text: string;
  avg_7d: number | null;
  avg_30d: number | null;
  trend: string | null;
  updated_at: string;
}

export interface PositionDto {
  id: string;
  exchange: string;
  symbol: string;
  side: PositionSide;
  quantity: string;
  entry_price: string;
  current_price: string;
  leverage: number;
  unrealized_pnl: string;
  unrealized_pnl_pct: string;
  stop_loss_price: string | null;
  take_profit_price: string | null;
  liquidation_price: string | null;
  status: string;
  opened_at: string;
  duration: string;
}

export interface OrderDto {
  id: string;
  exchange_order_id: string | null;
  exchange: string;
  symbol: string;
  side: OrderSide;
  order_type: OrderType;
  quantity: string;
  price: string | null;
  status: OrderStatus;
  filled_quantity: string;
  average_fill_price: string | null;
  created_at: string;
}

export interface TradeDto {
  id: string;
  exchange: string;
  symbol: string;
  side: PositionSide;
  quantity: string;
  entry_price: string;
  exit_price: string;
  net_pnl: string;
  pnl_pct: string;
  roi_pct: string;
  leverage: number;
  close_reason: string;
  duration: string;
  entered_at: string;
  exited_at: string;
  is_profitable: boolean;
}

export interface BalanceDto {
  asset: string;
  free: string;
  locked: string;
  total: string;
}

export interface TradeStatsDto {
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate_pct: string;
  total_pnl: string;
  gross_profit: string;
  gross_loss: string;
  profit_factor: string;
  avg_win: string;
  avg_loss: string;
  largest_win: string;
  largest_loss: string;
  total_fees: string;
}

export interface FillDto {
  id: string;
  exchange: string;
  symbol: string;
  side: string;
  price: string;
  quantity: string;
  fee: string;
  fee_asset: string;
  realized_pnl: string | null;
  timestamp: string;
}

export interface CreateOrderRequest {
  symbol: string;
  side: OrderSide;
  order_type: OrderType;
  quantity: string;
  price?: string;
  stop_loss?: string;
  take_profit?: string;
  leverage?: number;
}

export interface BacktestResultDto {
  id: string;
  strategy_id: string;
  symbol: string;
  period: string;
  initial_capital: string;
  final_capital: string;
  total_return_pct: string;
  total_trades: number;
  winning_trades: number;
  win_rate_pct: string;
  profit_factor: string;
  sharpe_ratio: string;
  max_drawdown_pct: string;
  trades: BacktestTradeDto[];
  equity_curve: EquityPoint[];
}

export interface BacktestTradeDto {
  entry_date: string;
  exit_date: string;
  side: PositionSide;
  entry_price: string;
  exit_price: string;
  pnl_pct: string;
}

export interface EquityPoint {
  date: string;
  equity: string;
  drawdown: string;
}

// Trading Worker types
export interface RiskStats {
  trading_enabled: boolean;
  daily_pnl: string;
  total_exposure: string;
  position_count: number;
  circuit_state: string;
}

export interface AppConfig {
  trading: TradingConfig;
  risk: RiskLimitsConfig;
  exchange: ExchangeConfig;
}

export interface TradingConfig {
  default_leverage: number;
  max_leverage: number;
  default_order_size_pct: number;
  trailing_stop_enabled: boolean;
}

export interface RiskLimitsConfig {
  max_daily_loss_pct: number;
  max_position_size_pct: number;
  max_positions: number;
  circuit_breaker_enabled: boolean;
}

export interface ExchangeConfig {
  binance_testnet: boolean;
  kraken_demo: boolean;
}

// Market Data types
export interface CandleDto {
  time: number;
  open: string | number;
  high: string | number;
  low: string | number;
  close: string | number;
  volume: string | number;
}

export type TimeFrame = '1m' | '5m' | '15m' | '30m' | '1h' | '4h' | '1d' | '1w';

export interface IndicatorsDto {
  symbol: string;
  interval: string;
  candles: CandleDto[];
  sma: { [period: string]: (string | null)[] };
  ema: { [period: string]: (string | null)[] };
}

// Price Alerts types
export type AlertCondition = 'above' | 'below' | 'cross_above' | 'cross_below' | 'percent_up' | 'percent_down';

export type AlertStatus = 'active' | 'triggered' | 'disabled' | 'expired';

export interface PriceAlertDto {
  id: string;
  symbol: string;
  condition: AlertCondition;
  target_price: string;
  percent: string | null;
  status: AlertStatus;
  message: string | null;
  recurring: boolean;
  trigger_count: number;
  created_at: string;
  triggered_at: string | null;
  last_triggered_at: string | null;
}

export interface CreatePriceAlertRequest {
  symbol: string;
  condition: AlertCondition;
  target_price: string;
  percent?: string;
  message?: string;
  recurring: boolean;
}

// Report types
export type ReportPeriod = 'day' | 'week' | 'month' | 'all';

export interface PnLDataPoint {
  time: number;
  value: number;
  cumulative: number;
}

export interface SymbolPnL {
  symbol: string;
  totalPnl: number;
  tradeCount: number;
  winCount: number;
  lossCount: number;
  winRate: number;
}

// EquityPoint for reports (numeric)
export interface ReportEquityPoint {
  time: number;
  equity: number;
}

// User Preferences
export interface UserPreferences {
  default_exchange: string;
  default_currency: string;
}

// Extended AppConfig with all sections
export interface FullAppConfig {
  general: GeneralConfig;
  trading: TradingConfig;
  data_collection: DataCollectionConfig;
  notifications: NotificationConfig;
  logging: LoggingConfig;
  exchange: ExchangeConfig;
  user_preferences?: UserPreferences;
}

export interface GeneralConfig {
  theme: string;
  language: string;
  start_minimized: boolean;
  start_with_system: boolean;
}

export interface DataCollectionConfig {
  fear_greed_interval_mins: number;
  candle_interval_secs: number;
  symbols: string[];
  timeframes: string[];
  candle_history_days: number;
}

export interface NotificationConfig {
  enabled: boolean;
  sounds_enabled: boolean;
  notify_signals: boolean;
  notify_orders: boolean;
  notify_positions: boolean;
  notify_errors: boolean;
}

export interface LoggingConfig {
  level: string;
  file_logging: boolean;
  retention_days: number;
  json_format: boolean;
}

// Filter types
export type ExchangeFilter = 'all' | 'binance' | 'kraken' | 'paper';
export type CurrencyFilter = 'all' | string;
