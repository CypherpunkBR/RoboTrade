import { useEffect, useState, useCallback } from 'react';
import type { DashboardSummary, PositionDto, OrderDto, BalanceDto } from '@/types';
import {
  getDashboardSummary,
  getPositions,
  getOpenOrders,
  getBalances,
  cancelOrder,
  formatCurrency,
  formatPercent,
} from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { FearGreedGauge } from '@/components/FearGreedGauge';
import { PositionsTable } from '@/components/PositionsTable';
import { OrdersTable } from '@/components/OrdersTable';
import { StatsCard } from '@/components/StatsCard';
import { BalanceHoverCard } from '@/components/BalanceHoverCard';

interface DashboardProps {
  onNavigateToTrading?: (symbol: string) => void;
}

export function Dashboard({ onNavigateToTrading }: DashboardProps) {
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [positions, setPositions] = useState<PositionDto[]>([]);
  const [orders, setOrders] = useState<OrderDto[]>([]);
  const [balances, setBalances] = useState<BalanceDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const [summaryData, positionsData, ordersData, balancesData] = await Promise.all([
        getDashboardSummary(),
        getPositions(),
        getOpenOrders(),
        getBalances(),
      ]);
      setSummary(summaryData);
      setPositions(positionsData);
      setOrders(ordersData);
      setBalances(balancesData);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, [fetchData]);

  const handleClosePosition = async (id: string) => {
    console.log('Close position:', id);
    // TODO: Implementar fechamento de posicao
  };

  const handleCancelOrder = async (id: string, symbol: string) => {
    try {
      await cancelOrder(symbol, id);
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao cancelar ordem');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto" />
          <p className="mt-4 text-muted-foreground">Carregando...</p>
        </div>
      </div>
    );
  }

  if (error && !summary) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-destructive text-lg">{error}</p>
          <button
            onClick={fetchData}
            className="mt-4 px-4 py-2 bg-primary text-white rounded-md hover:bg-primary/90"
          >
            Tentar novamente
          </button>
        </div>
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

      {/* Stats row */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <BalanceHoverCard
          totalBalance={summary ? parseFloat(summary.total_balance_usdt) : 0}
          balances={balances}
        />
        <StatsCard
          title="PnL Diario"
          value={summary ? formatCurrency(summary.daily_pnl) : '-'}
          subtitle={summary ? formatPercent(summary.daily_pnl_pct) : undefined}
          isPnl
        />
        <StatsCard
          title="Posicoes Abertas"
          value={summary?.open_positions_count.toString() || '0'}
        />
        <StatsCard
          title="Ordens Ativas"
          value={summary?.active_orders_count.toString() || '0'}
        />
      </div>

      {/* Fear & Greed and Positions row */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Fear & Greed Card */}
        <Card>
          <CardHeader>
            <CardTitle>Fear & Greed Index</CardTitle>
          </CardHeader>
          <CardContent className="flex justify-center">
            <FearGreedGauge
              value={summary?.fear_greed_value ?? null}
              classification={summary?.fear_greed_classification ?? null}
              size="lg"
            />
          </CardContent>
        </Card>

        {/* Positions Card */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Posicoes Abertas</CardTitle>
          </CardHeader>
          <CardContent>
            <PositionsTable
              positions={positions}
              onClosePosition={handleClosePosition}
              onSymbolClick={onNavigateToTrading}
            />
          </CardContent>
        </Card>
      </div>

      {/* Orders */}
      <Card>
        <CardHeader>
          <CardTitle>Ordens Ativas</CardTitle>
        </CardHeader>
        <CardContent>
          <OrdersTable
            orders={orders}
            onCancelOrder={handleCancelOrder}
            onSymbolClick={onNavigateToTrading}
          />
        </CardContent>
      </Card>
    </div>
  );
}
