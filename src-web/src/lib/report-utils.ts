/**
 * Utilitários para agregação e processamento de dados de relatórios
 */

import type { TradeDto } from '@/types';

// ============================================================================
// TIPOS
// ============================================================================

export type ReportPeriod = 'day' | 'week' | 'month' | 'all';

export interface PnLDataPoint {
  time: number; // Unix timestamp
  value: number; // PnL do período
  cumulative: number; // PnL acumulado
}

export interface SymbolPnL {
  symbol: string;
  totalPnl: number;
  tradeCount: number;
  winCount: number;
  lossCount: number;
  winRate: number;
}

export interface EquityPoint {
  time: number;
  equity: number;
}

// ============================================================================
// FUNÇÕES DE AGREGAÇÃO
// ============================================================================

/**
 * Agrupa PnL por período (dia/semana/mês)
 */
export function aggregatePnLByPeriod(
  trades: TradeDto[],
  period: ReportPeriod
): PnLDataPoint[] {
  if (trades.length === 0) return [];

  // Ordena por data de saída
  const sorted = [...trades].sort(
    (a, b) => new Date(a.exited_at).getTime() - new Date(b.exited_at).getTime()
  );

  const grouped = new Map<string, number>();

  sorted.forEach((trade) => {
    const date = new Date(trade.exited_at);
    const key = getPeriodKey(date, period);
    const pnl = parseFloat(trade.net_pnl);
    grouped.set(key, (grouped.get(key) || 0) + pnl);
  });

  // Converte para array com cumulativo
  let cumulative = 0;
  const result: PnLDataPoint[] = [];

  // Ordena as chaves cronologicamente
  const sortedKeys = [...grouped.keys()].sort();

  sortedKeys.forEach((key) => {
    const value = grouped.get(key) || 0;
    cumulative += value;
    result.push({
      time: getTimestampFromKey(key, period),
      value,
      cumulative,
    });
  });

  return result;
}

/**
 * Agrupa PnL por símbolo
 */
export function aggregatePnLBySymbol(trades: TradeDto[]): SymbolPnL[] {
  const grouped = new Map<string, { pnl: number; count: number; wins: number; losses: number }>();

  trades.forEach((trade) => {
    const existing = grouped.get(trade.symbol) || { pnl: 0, count: 0, wins: 0, losses: 0 };
    const pnl = parseFloat(trade.net_pnl);

    existing.pnl += pnl;
    existing.count += 1;
    if (trade.is_profitable) {
      existing.wins += 1;
    } else {
      existing.losses += 1;
    }

    grouped.set(trade.symbol, existing);
  });

  return [...grouped.entries()]
    .map(([symbol, data]) => ({
      symbol,
      totalPnl: data.pnl,
      tradeCount: data.count,
      winCount: data.wins,
      lossCount: data.losses,
      winRate: data.count > 0 ? (data.wins / data.count) * 100 : 0,
    }))
    .sort((a, b) => b.totalPnl - a.totalPnl);
}

/**
 * Calcula curva de equity
 */
export function calculateEquityCurve(
  trades: TradeDto[],
  initialCapital: number = 10000
): EquityPoint[] {
  if (trades.length === 0) {
    return [{ time: Date.now(), equity: initialCapital }];
  }

  // Ordena por data de saída
  const sorted = [...trades].sort(
    (a, b) => new Date(a.exited_at).getTime() - new Date(b.exited_at).getTime()
  );

  let equity = initialCapital;
  const result: EquityPoint[] = [
    {
      time: new Date(sorted[0].entered_at).getTime(),
      equity: initialCapital,
    },
  ];

  sorted.forEach((trade) => {
    equity += parseFloat(trade.net_pnl);
    result.push({
      time: new Date(trade.exited_at).getTime(),
      equity,
    });
  });

  return result;
}

/**
 * Filtra trades por período de tempo
 */
export function filterTradesByPeriod(
  trades: TradeDto[],
  period: ReportPeriod
): TradeDto[] {
  if (period === 'all') return trades;

  const now = new Date();
  let startDate: Date;

  switch (period) {
    case 'day':
      startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      break;
    case 'week':
      startDate = new Date(now);
      startDate.setDate(now.getDate() - 7);
      break;
    case 'month':
      startDate = new Date(now);
      startDate.setMonth(now.getMonth() - 1);
      break;
    default:
      return trades;
  }

  return trades.filter((trade) => new Date(trade.exited_at) >= startDate);
}

// ============================================================================
// FUNÇÕES AUXILIARES
// ============================================================================

function getPeriodKey(date: Date, period: ReportPeriod): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');

  switch (period) {
    case 'day':
    case 'all':
      return `${year}-${month}-${day}`;
    case 'week': {
      // Usar início da semana (segunda-feira)
      const weekStart = new Date(date);
      const dayOfWeek = weekStart.getDay();
      const diff = dayOfWeek === 0 ? -6 : 1 - dayOfWeek;
      weekStart.setDate(weekStart.getDate() + diff);
      const wYear = weekStart.getFullYear();
      const wMonth = String(weekStart.getMonth() + 1).padStart(2, '0');
      const wDay = String(weekStart.getDate()).padStart(2, '0');
      return `${wYear}-${wMonth}-${wDay}`;
    }
    case 'month':
      return `${year}-${month}`;
    default:
      return `${year}-${month}-${day}`;
  }
}

function getTimestampFromKey(key: string, period: ReportPeriod): number {
  const parts = key.split('-');

  switch (period) {
    case 'month':
      return new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, 1).getTime();
    default:
      return new Date(
        parseInt(parts[0]),
        parseInt(parts[1]) - 1,
        parseInt(parts[2])
      ).getTime();
  }
}

/**
 * Calcula estatísticas resumidas dos trades
 */
export function calculateSummaryStats(trades: TradeDto[]) {
  if (trades.length === 0) {
    return {
      totalTrades: 0,
      winningTrades: 0,
      losingTrades: 0,
      winRate: 0,
      totalPnl: 0,
      avgPnl: 0,
      maxWin: 0,
      maxLoss: 0,
      profitFactor: 0,
    };
  }

  const winningTrades = trades.filter((t) => t.is_profitable);
  const losingTrades = trades.filter((t) => !t.is_profitable);

  const totalPnl = trades.reduce((sum, t) => sum + parseFloat(t.net_pnl), 0);
  const grossProfit = winningTrades.reduce((sum, t) => sum + parseFloat(t.net_pnl), 0);
  const grossLoss = Math.abs(losingTrades.reduce((sum, t) => sum + parseFloat(t.net_pnl), 0));

  const pnls = trades.map((t) => parseFloat(t.net_pnl));
  const maxWin = Math.max(...pnls, 0);
  const maxLoss = Math.min(...pnls, 0);

  return {
    totalTrades: trades.length,
    winningTrades: winningTrades.length,
    losingTrades: losingTrades.length,
    winRate: (winningTrades.length / trades.length) * 100,
    totalPnl,
    avgPnl: totalPnl / trades.length,
    maxWin,
    maxLoss,
    profitFactor: grossLoss > 0 ? grossProfit / grossLoss : grossProfit > 0 ? Infinity : 0,
  };
}
