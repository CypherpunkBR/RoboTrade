import { useEffect, useState, useMemo } from 'react';
import type { TradeDto, TradeStatsDto, ReportPeriod } from '@/types';
import { getTradeHistory, getTradeStats, formatCurrency } from '@/lib/tauri';
import {
  aggregatePnLByPeriod,
  aggregatePnLBySymbol,
  calculateEquityCurve,
  filterTradesByPeriod,
  calculateSummaryStats,
} from '@/lib/report-utils';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { PnLChart } from '@/components/reports/PnLChart';
import { SymbolPnLChart } from '@/components/reports/SymbolPnLChart';
import { EquityCurve } from '@/components/reports/EquityCurve';
import { WinLossChart } from '@/components/reports/WinLossChart';

const PERIOD_OPTIONS: { value: ReportPeriod; label: string }[] = [
  { value: 'day', label: 'Hoje' },
  { value: 'week', label: 'Esta Semana' },
  { value: 'month', label: 'Este Mes' },
  { value: 'all', label: 'Todo Periodo' },
];

export function Reports() {
  const [trades, setTrades] = useState<TradeDto[]>([]);
  const [stats, setStats] = useState<TradeStatsDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [period, setPeriod] = useState<ReportPeriod>('month');

  useEffect(() => {
    async function loadData() {
      try {
        const [tradesData, statsData] = await Promise.all([
          getTradeHistory(undefined, 1000),
          getTradeStats(),
        ]);
        setTrades(tradesData);
        setStats(statsData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
      } finally {
        setLoading(false);
      }
    }
    loadData();
  }, []);

  // Filter trades by selected period
  const filteredTrades = useMemo(() => {
    return filterTradesByPeriod(trades, period);
  }, [trades, period]);

  // Calculate charts data
  const pnlData = useMemo(() => {
    return aggregatePnLByPeriod(filteredTrades, period);
  }, [filteredTrades, period]);

  const symbolPnlData = useMemo(() => {
    return aggregatePnLBySymbol(filteredTrades);
  }, [filteredTrades]);

  const equityData = useMemo(() => {
    return calculateEquityCurve(filteredTrades, 10000); // Assume 10k initial
  }, [filteredTrades]);

  const periodStats = useMemo(() => {
    return calculateSummaryStats(filteredTrades);
  }, [filteredTrades]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
          {error}
        </div>
      )}

      {/* Period Selector */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Relatorios</h1>
        <div className="flex gap-1 bg-muted rounded-md p-0.5">
          {PERIOD_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setPeriod(opt.value)}
              className={`px-3 py-1.5 text-sm font-medium rounded transition-colors ${
                period === opt.value
                  ? 'bg-primary text-white'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-4">
            <div className="text-sm text-muted-foreground">Total Trades</div>
            <div className="text-2xl font-bold">{periodStats.totalTrades}</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-sm text-muted-foreground">PnL Total</div>
            <div className={`text-2xl font-bold ${periodStats.totalPnl >= 0 ? 'text-green-500' : 'text-red-500'}`}>
              {formatCurrency(periodStats.totalPnl)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-sm text-muted-foreground">Win Rate</div>
            <div className="text-2xl font-bold">
              {periodStats.winRate.toFixed(1)}%
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-sm text-muted-foreground">Profit Factor</div>
            <div className="text-2xl font-bold">
              {periodStats.profitFactor === Infinity ? '∞' : periodStats.profitFactor.toFixed(2)}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Charts Row 1: PnL over time and Win/Loss */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>PnL ao Longo do Tempo</CardTitle>
          </CardHeader>
          <CardContent>
            <PnLChart data={pnlData} height={300} showCumulative />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Distribuicao de Resultados</CardTitle>
          </CardHeader>
          <CardContent className="flex justify-center">
            {stats && <WinLossChart stats={stats} size={200} />}
          </CardContent>
        </Card>
      </div>

      {/* Charts Row 2: Equity Curve */}
      <Card>
        <CardHeader>
          <CardTitle>Curva de Capital</CardTitle>
        </CardHeader>
        <CardContent>
          <EquityCurve data={equityData} height={300} />
        </CardContent>
      </Card>

      {/* Charts Row 3: PnL by Symbol */}
      <Card>
        <CardHeader>
          <CardTitle>PnL por Moeda</CardTitle>
        </CardHeader>
        <CardContent>
          <SymbolPnLChart data={symbolPnlData} height={300} />
        </CardContent>
      </Card>

      {/* Detailed Stats Table */}
      <Card>
        <CardHeader>
          <CardTitle>Estatisticas Detalhadas</CardTitle>
        </CardHeader>
        <CardContent>
          {stats && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Trades Totais</div>
                <div className="font-bold">{stats.total_trades}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Trades Vencedores</div>
                <div className="font-bold text-green-500">{stats.winning_trades}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Trades Perdedores</div>
                <div className="font-bold text-red-500">{stats.losing_trades}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Win Rate</div>
                <div className="font-bold">{stats.win_rate_pct}%</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Lucro Bruto</div>
                <div className="font-bold text-green-500">{formatCurrency(stats.gross_profit)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Perda Bruta</div>
                <div className="font-bold text-red-500">{formatCurrency(stats.gross_loss)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Media Ganho</div>
                <div className="font-bold text-green-500">{formatCurrency(stats.avg_win)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Media Perda</div>
                <div className="font-bold text-red-500">{formatCurrency(stats.avg_loss)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Maior Ganho</div>
                <div className="font-bold text-green-500">{formatCurrency(stats.largest_win)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Maior Perda</div>
                <div className="font-bold text-red-500">{formatCurrency(stats.largest_loss)}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Profit Factor</div>
                <div className="font-bold">{stats.profit_factor}</div>
              </div>
              <div className="p-3 bg-muted/50 rounded-md">
                <div className="text-sm text-muted-foreground">Total Taxas</div>
                <div className="font-bold">{formatCurrency(stats.total_fees)}</div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
