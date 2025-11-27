import { useEffect, useState, useCallback } from 'react';
import type { TradeDto, TradeStatsDto, FillDto } from '@/types';
import {
  getTradeHistory,
  getTradeStats,
  getFillHistory,
  formatCurrency,
  formatPercent,
  getPnlColor,
} from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { StatsCard } from '@/components/StatsCard';

type Tab = 'trades' | 'fills';

const exchangeLabels: Record<string, string> = {
  paper: 'Paper',
  binance_futures: 'Binance',
  kraken_futures: 'Kraken',
};

export function History() {
  const [activeTab, setActiveTab] = useState<Tab>('fills');
  const [trades, setTrades] = useState<TradeDto[]>([]);
  const [fills, setFills] = useState<FillDto[]>([]);
  const [stats, setStats] = useState<TradeStatsDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // Fetch each independently to not fail all if one fails
      const [tradesResult, statsResult, fillsResult] = await Promise.allSettled([
        getTradeHistory(undefined, 100),
        getTradeStats(),
        getFillHistory(undefined, 100),
      ]);

      if (tradesResult.status === 'fulfilled') {
        setTrades(tradesResult.value);
      } else {
        console.error('Erro ao buscar trades:', tradesResult.reason);
      }

      if (statsResult.status === 'fulfilled') {
        setStats(statsResult.value);
      } else {
        console.error('Erro ao buscar stats:', statsResult.reason);
      }

      if (fillsResult.status === 'fulfilled') {
        setFills(fillsResult.value);
        console.log('Fills carregados:', fillsResult.value.length);
      } else {
        console.error('Erro ao buscar fills:', fillsResult.reason);
        setError('Erro ao buscar historico de execucoes: ' + String(fillsResult.reason));
      }
    } catch (err) {
      console.error('Erro geral ao carregar dados:', err);
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

      {/* Tabs */}
      <div className="flex gap-2 border-b border-border">
        <button
          onClick={() => setActiveTab('fills')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${
            activeTab === 'fills'
              ? 'border-b-2 border-primary text-primary'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          Execucoes ({fills.length})
        </button>
        <button
          onClick={() => setActiveTab('trades')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${
            activeTab === 'trades'
              ? 'border-b-2 border-primary text-primary'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          Trades Completos ({trades.length})
        </button>
      </div>

      {/* Fills Table */}
      {activeTab === 'fills' && (
        <Card>
          <CardHeader>
            <CardTitle>Historico de Execucoes</CardTitle>
          </CardHeader>
          <CardContent>
            {fills.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="text-left py-3 px-2 font-medium">Exchange</th>
                      <th className="text-left py-3 px-2 font-medium">Par</th>
                      <th className="text-left py-3 px-2 font-medium">Lado</th>
                      <th className="text-right py-3 px-2 font-medium">Preco</th>
                      <th className="text-right py-3 px-2 font-medium">Qtde</th>
                      <th className="text-right py-3 px-2 font-medium">Taxa</th>
                      <th className="text-right py-3 px-2 font-medium">PnL Realizado</th>
                      <th className="text-left py-3 px-2 font-medium">Data</th>
                    </tr>
                  </thead>
                  <tbody>
                    {fills.map(fill => (
                      <tr key={fill.id} className="border-b border-border/50 hover:bg-muted/50">
                        <td className="py-3 px-2 text-muted-foreground">
                          {exchangeLabels[fill.exchange] || fill.exchange}
                        </td>
                        <td className="py-3 px-2 font-medium">{fill.symbol}</td>
                        <td className="py-3 px-2">
                          <span className={fill.side.toLowerCase() === 'buy' ? 'text-green-500' : 'text-red-500'}>
                            {fill.side.toLowerCase() === 'buy' ? 'Compra' : 'Venda'}
                          </span>
                        </td>
                        <td className="py-3 px-2 text-right">{formatCurrency(fill.price, '')}</td>
                        <td className="py-3 px-2 text-right">{fill.quantity}</td>
                        <td className="py-3 px-2 text-right text-muted-foreground">
                          {formatCurrency(fill.fee, '')} {fill.fee_asset}
                        </td>
                        <td className="py-3 px-2 text-right">
                          {fill.realized_pnl ? (
                            <span style={{ color: getPnlColor(fill.realized_pnl) }}>
                              {formatCurrency(fill.realized_pnl)}
                            </span>
                          ) : (
                            <span className="text-muted-foreground">-</span>
                          )}
                        </td>
                        <td className="py-3 px-2 text-muted-foreground">
                          {new Date(fill.timestamp).toLocaleString('pt-BR')}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="text-center py-8">
                <p className="text-muted-foreground">
                  Nenhuma execucao encontrada.
                </p>
                <p className="text-xs text-muted-foreground mt-2">
                  Configure suas credenciais de API no arquivo <code className="bg-muted px-1 rounded">.env</code>:
                </p>
                <pre className="text-xs text-left bg-muted p-3 rounded mt-2 inline-block">
{`# Binance Futures
BINANCE_API_KEY=sua_api_key
BINANCE_API_SECRET=sua_api_secret
BINANCE_TESTNET=true

# Kraken Futures
KRAKEN_FUTURES_API_KEY=sua_api_key
KRAKEN_FUTURES_API_SECRET=sua_api_secret
KRAKEN_FUTURES_DEMO=true`}
                </pre>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Trade History Table */}
      {activeTab === 'trades' && (
        <Card>
          <CardHeader>
            <CardTitle>Historico de Trades Completos</CardTitle>
          </CardHeader>
          <CardContent>
            {trades.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="text-left py-3 px-2 font-medium">Exchange</th>
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
                        <td className="py-3 px-2 text-muted-foreground">
                          {exchangeLabels[trade.exchange] || trade.exchange}
                        </td>
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
                Nenhum trade completo encontrado no cache interno.
                <br />
                <span className="text-xs">
                  Trades completos sao registrados quando posicoes sao fechadas pelo RoboTrade.
                </span>
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {/* Refresh Button */}
      <div className="flex justify-end">
        <button
          onClick={fetchData}
          disabled={loading}
          className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
        >
          {loading ? 'Atualizando...' : 'Atualizar'}
        </button>
      </div>
    </div>
  );
}
