import { useEffect, useState, useCallback } from 'react';
import type { TradeDto, TradeStatsDto } from '@/types';
import {
  getTradeHistory,
  getTradeStats,
  formatCurrency,
  formatPercent,
  getPnlColor,
} from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { StatsCard } from '@/components/StatsCard';

export function History() {
  const [trades, setTrades] = useState<TradeDto[]>([]);
  const [stats, setStats] = useState<TradeStatsDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const [tradesData, statsData] = await Promise.all([
        getTradeHistory(undefined, 100),
        getTradeStats(),
      ]);
      setTrades(tradesData);
      setStats(statsData);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

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

      {/* Stats Summary */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
          <StatsCard
            title="Total de Trades"
            value={stats.total_trades.toString()}
          />
          <StatsCard
            title="Trades Vencedores"
            value={stats.winning_trades.toString()}
          />
          <StatsCard
            title="Trades Perdedores"
            value={stats.losing_trades.toString()}
          />
          <StatsCard
            title="Win Rate"
            value={formatPercent(stats.win_rate_pct).replace('+', '')}
          />
          <StatsCard
            title="Lucro Total"
            value={formatCurrency(stats.total_pnl)}
            isPnl
          />
          <StatsCard
            title="Profit Factor"
            value={parseFloat(stats.profit_factor).toFixed(2)}
          />
        </div>
      )}

      {/* Detailed Stats */}
      {stats && stats.total_trades > 0 && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Lucro Bruto</div>
              <div className="text-xl font-bold text-green-500">
                {formatCurrency(stats.gross_profit)}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Perda Bruta</div>
              <div className="text-xl font-bold text-red-500">
                {formatCurrency(stats.gross_loss)}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Media Ganho</div>
              <div className="text-xl font-bold text-green-500">
                {formatCurrency(stats.avg_win)}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Media Perda</div>
              <div className="text-xl font-bold text-red-500">
                {formatCurrency(stats.avg_loss)}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Maior Ganho</div>
              <div className="text-xl font-bold text-green-500">
                {formatCurrency(stats.largest_win)}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Maior Perda</div>
              <div className="text-xl font-bold text-red-500">
                {formatCurrency(stats.largest_loss)}
              </div>
            </CardContent>
          </Card>
          <Card className="col-span-2">
            <CardContent className="pt-6">
              <div className="text-sm text-muted-foreground">Taxas Totais</div>
              <div className="text-xl font-bold">
                {formatCurrency(stats.total_fees)}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Trade History Table */}
      <Card>
        <CardHeader>
          <CardTitle>Historico de Trades</CardTitle>
        </CardHeader>
        <CardContent>
          {trades.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border">
                    <th className="text-left py-3 px-2 font-medium">Par</th>
                    <th className="text-left py-3 px-2 font-medium">Lado</th>
                    <th className="text-right py-3 px-2 font-medium">Entrada</th>
                    <th className="text-right py-3 px-2 font-medium">Saida</th>
                    <th className="text-right py-3 px-2 font-medium">Qtde</th>
                    <th className="text-right py-3 px-2 font-medium">PnL</th>
                    <th className="text-right py-3 px-2 font-medium">ROI</th>
                    <th className="text-left py-3 px-2 font-medium">Motivo</th>
                    <th className="text-left py-3 px-2 font-medium">Duracao</th>
                    <th className="text-left py-3 px-2 font-medium">Data</th>
                  </tr>
                </thead>
                <tbody>
                  {trades.map(trade => (
                    <tr key={trade.id} className="border-b border-border/50 hover:bg-muted/50">
                      <td className="py-3 px-2 font-medium">{trade.symbol}</td>
                      <td className="py-3 px-2">
                        <span className={trade.side === 'long' ? 'text-green-500' : 'text-red-500'}>
                          {trade.side === 'long' ? 'Long' : 'Short'}
                        </span>
                      </td>
                      <td className="py-3 px-2 text-right">{formatCurrency(trade.entry_price, '')}</td>
                      <td className="py-3 px-2 text-right">{formatCurrency(trade.exit_price, '')}</td>
                      <td className="py-3 px-2 text-right">{trade.quantity}</td>
                      <td className="py-3 px-2 text-right" style={{ color: getPnlColor(trade.net_pnl) }}>
                        {formatCurrency(trade.net_pnl)}
                      </td>
                      <td className="py-3 px-2 text-right" style={{ color: getPnlColor(trade.roi_pct) }}>
                        {formatPercent(trade.roi_pct)}
                      </td>
                      <td className="py-3 px-2 text-muted-foreground">{trade.close_reason}</td>
                      <td className="py-3 px-2 text-muted-foreground">{trade.duration}</td>
                      <td className="py-3 px-2 text-muted-foreground">
                        {new Date(trade.exited_at).toLocaleDateString('pt-BR')}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <p className="text-muted-foreground text-center py-8">
              Nenhum trade encontrado no historico
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
