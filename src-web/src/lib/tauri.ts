import { invoke } from '@tauri-apps/api/core';
import type {
  DashboardSummary,
  FearGreedDto,
  PositionDto,
  OrderDto,
  BalanceDto,
  TradeDto,
  TradeStatsDto,
  TradingMode,
  ConnectionStatus,
  CreateOrderRequest,
  RiskStats,
  AppConfig,
  CandleDto,
  IndicatorsDto,
  PriceAlertDto,
  CreatePriceAlertRequest,
} from '@/types';

// Dashboard
export async function getDashboardSummary(): Promise<DashboardSummary> {
  return invoke('get_dashboard_summary');
}

export async function getConnectionStatus(): Promise<ConnectionStatus> {
  return invoke('get_connection_status');
}

// Fear & Greed
export async function getFearGreedCurrent(): Promise<FearGreedDto | null> {
  return invoke('get_fear_greed_current');
}

export async function getFearGreedHistory(days: number): Promise<FearGreedDto[]> {
  return invoke('get_fear_greed_history', { days });
}

// Posições e Ordens
export async function getPositions(): Promise<PositionDto[]> {
  return invoke('get_positions');
}

export async function getOpenOrders(): Promise<OrderDto[]> {
  return invoke('get_open_orders');
}

export async function getBalances(): Promise<BalanceDto[]> {
  return invoke('get_balances');
}

// Trading
export async function createOrder(request: CreateOrderRequest): Promise<OrderDto> {
  return invoke('create_order', { request });
}

export async function cancelOrder(symbol: string, orderId: string): Promise<boolean> {
  return invoke('cancel_order', { symbol, orderId });
}

// Histórico
export async function getTradeHistory(symbol?: string, limit?: number): Promise<TradeDto[]> {
  return invoke('get_trade_history', { symbol, limit });
}

export async function getTradeStats(): Promise<TradeStatsDto> {
  return invoke('get_trade_stats');
}

// Configuração
export async function getTradingMode(): Promise<TradingMode> {
  return invoke('get_trading_mode');
}

export async function setTradingMode(mode: TradingMode): Promise<void> {
  return invoke('set_trading_mode', { mode });
}

// Utilitários
export function formatDecimal(value: string | number, decimals = 2): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  return num.toLocaleString('pt-BR', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

export function formatCurrency(value: string | number, symbol = '$'): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  return `${symbol}${formatDecimal(num)}`;
}

export function formatPercent(value: string | number): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  const sign = num >= 0 ? '+' : '';
  return `${sign}${formatDecimal(num)}%`;
}

export function getFearGreedColor(value: number): string {
  if (value <= 24) return 'var(--color-extreme-fear)';
  if (value <= 44) return 'var(--color-fear)';
  if (value <= 55) return 'var(--color-neutral)';
  if (value <= 74) return 'var(--color-greed)';
  return 'var(--color-extreme-greed)';
}

export function getPnlColor(value: string | number): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  return num >= 0 ? 'var(--color-profit)' : 'var(--color-loss)';
}

// Configuração
export async function getConfig(): Promise<AppConfig> {
  return invoke('get_config');
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config });
}

// Trading Worker
export async function startTradingWorker(): Promise<void> {
  return invoke('start_trading_worker');
}

export async function stopTradingWorker(): Promise<void> {
  return invoke('stop_trading_worker');
}

export async function isWorkerRunning(): Promise<boolean> {
  return invoke('is_worker_running');
}

export async function getRiskStats(): Promise<RiskStats> {
  return invoke('get_risk_stats');
}

export async function setTradingEnabled(enabled: boolean): Promise<void> {
  return invoke('set_trading_enabled', { enabled });
}

export async function resetDailyLosses(): Promise<void> {
  return invoke('reset_daily_losses');
}

// Market Data - Klines
export async function getKlines(
  symbol: string,
  interval: string,
  limit?: number
): Promise<CandleDto[]> {
  return invoke('get_klines', { symbol, interval, limit });
}

export async function getKlinesRange(
  symbol: string,
  interval: string,
  startTime?: number,
  endTime?: number,
  limit?: number
): Promise<CandleDto[]> {
  return invoke('get_klines_range', {
    symbol,
    interval,
    startTime,
    endTime,
    limit
  });
}

export async function getKlinesHistory(
  symbol: string,
  interval: string,
  startTime: number,
  endTime?: number
): Promise<CandleDto[]> {
  return invoke('get_klines_history', { symbol, interval, startTime, endTime });
}

export async function calculateIndicators(
  symbol: string,
  interval: string,
  limit?: number,
  smaPeriods?: number[],
  emaPeriods?: number[]
): Promise<IndicatorsDto> {
  return invoke('calculate_indicators', {
    symbol,
    interval,
    limit,
    smaPeriods,
    emaPeriods,
  });
}

// Price Alerts
export async function listPriceAlerts(): Promise<PriceAlertDto[]> {
  return invoke('list_price_alerts');
}

export async function createPriceAlert(
  request: CreatePriceAlertRequest
): Promise<PriceAlertDto> {
  return invoke('create_price_alert', { request });
}

export async function deletePriceAlert(id: string): Promise<boolean> {
  return invoke('delete_price_alert', { id });
}

export async function disablePriceAlert(id: string): Promise<boolean> {
  return invoke('disable_price_alert', { id });
}

export async function enablePriceAlert(id: string): Promise<boolean> {
  return invoke('enable_price_alert', { id });
}
