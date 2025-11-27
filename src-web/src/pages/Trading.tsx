import { useEffect, useState, useCallback } from 'react';
import type {
  CreateOrderRequest,
  OrderSide,
  OrderType,
  BalanceDto,
  RiskStats,
  OrderDto,
  PositionDto,
} from '@/types';
import {
  createOrder,
  getBalances,
  getRiskStats,
  startTradingWorker,
  stopTradingWorker,
  isWorkerRunning,
  setTradingEnabled,
  resetDailyLosses,
  formatCurrency,
  getOpenOrders,
  getPositions,
  cancelOrder,
} from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { TradingChart } from '@/components/TradingChart';
import { PositionsTable } from '@/components/PositionsTable';
import { OrdersTable } from '@/components/OrdersTable';

const SYMBOLS = [
  'BTCUSDT',
  'ETHUSDT',
  'BNBUSDT',
  'SOLUSDT',
  // 'XRPUSDT',
  // 'ADAUSDT',
  // 'DOGEUSDT',
  // 'AVAXUSDT',
];

interface TradingProps {
  initialSymbol?: string | null;
  onSymbolUsed?: () => void;
}

export function Trading({ initialSymbol, onSymbolUsed }: TradingProps) {
  const [balances, setBalances] = useState<BalanceDto[]>([]);
  const [riskStats, setRiskStats] = useState<RiskStats | null>(null);
  const [workerRunning, setWorkerRunning] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [orders, setOrders] = useState<OrderDto[]>([]);
  const [positions, setPositions] = useState<PositionDto[]>([]);

  // Order form state
  const [symbol, setSymbol] = useState(initialSymbol || 'BTCUSDT');
  const [side, setSide] = useState<OrderSide>('buy');
  const [orderType, setOrderType] = useState<OrderType>('market');
  const [quantity, setQuantity] = useState('0.001');
  const [price, setPrice] = useState('');
  const [stopLoss, setStopLoss] = useState('');
  const [takeProfit, setTakeProfit] = useState('');
  const [leverage, setLeverage] = useState(10);
  const [submitting, setSubmitting] = useState(false);

  const fetchData = useCallback(async () => {
    try {
      const [balancesData, riskData, running, ordersData, positionsData] = await Promise.all([
        getBalances(),
        getRiskStats(),
        isWorkerRunning(),
        getOpenOrders(),
        getPositions(),
      ]);
      setBalances(balancesData);
      setRiskStats(riskData);
      setWorkerRunning(running);
      setOrders(ordersData);
      setPositions(positionsData);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 3000);
    return () => clearInterval(interval);
  }, [fetchData]);

  // Handle initial symbol from navigation
  useEffect(() => {
    if (initialSymbol) {
      setSymbol(initialSymbol);
      onSymbolUsed?.();
    }
  }, [initialSymbol, onSymbolUsed]);

  const handleSubmitOrder = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    setSuccess(null);

    try {
      const request: CreateOrderRequest = {
        symbol,
        side,
        order_type: orderType,
        quantity,
        price: orderType !== 'market' && price ? price : undefined,
        stop_loss: stopLoss || undefined,
        take_profit: takeProfit || undefined,
        leverage,
      };

      const order = await createOrder(request);
      setSuccess(`Ordem criada com sucesso! ID: ${order.id}`);

      // Reset form
      setQuantity('0.001');
      setPrice('');
      setStopLoss('');
      setTakeProfit('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao criar ordem');
    } finally {
      setSubmitting(false);
    }
  };

  const handleToggleWorker = async () => {
    try {
      if (workerRunning) {
        await stopTradingWorker();
      } else {
        await startTradingWorker();
      }
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao alterar worker');
    }
  };

  const handleToggleTrading = async () => {
    if (!riskStats) return;
    try {
      await setTradingEnabled(!riskStats.trading_enabled);
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao alterar trading');
    }
  };

  const handleResetLosses = async () => {
    try {
      await resetDailyLosses();
      await fetchData();
      setSuccess('Perdas diárias resetadas');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao resetar perdas');
    }
  };

  const handleCancelOrder = async (orderId: string) => {
    try {
      await cancelOrder(symbol, orderId);
      setSuccess('Ordem cancelada');
      fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao cancelar ordem');
    }
  };

  const handlePriceClick = (clickedPrice: number) => {
    if (orderType !== 'market') {
      setPrice(clickedPrice.toFixed(2));
    }
  };

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
      {success && (
        <div className="p-3 bg-green-500/10 border border-green-500/20 rounded-md text-green-600 text-sm">
          {success}
        </div>
      )}

      {/* Chart Section */}
      <Card>
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle>Grafico - {symbol}</CardTitle>
            {/* Symbol selector */}
            <div className="flex gap-1 bg-muted rounded-md p-0.5">
              {SYMBOLS.map((sym) => (
                <button
                  key={sym}
                  onClick={() => setSymbol(sym)}
                  className={`px-2 py-1 text-xs font-medium rounded transition-colors ${
                    symbol === sym
                      ? 'bg-primary text-white'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                >
                  {sym.replace('USDT', '')}
                </button>
              ))}
            </div>
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          <TradingChart
            symbol={symbol}
            interval="1h"
            height={400}
            orders={orders}
            positions={positions}
            onPriceClick={handlePriceClick}
          />
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Order Form */}
        <Card>
          <CardHeader>
            <CardTitle>Nova Ordem - {symbol}</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmitOrder} className="space-y-4">

              {/* Side */}
              <div>
                <label className="block text-sm font-medium mb-1">Lado</label>
                <div className="grid grid-cols-2 gap-2">
                  <Button
                    type="button"
                    variant={side === 'buy' ? 'default' : 'outline'}
                    onClick={() => setSide('buy')}
                    className={side === 'buy' ? 'bg-green-600 hover:bg-green-700' : ''}
                  >
                    Compra (Long)
                  </Button>
                  <Button
                    type="button"
                    variant={side === 'sell' ? 'default' : 'outline'}
                    onClick={() => setSide('sell')}
                    className={side === 'sell' ? 'bg-red-600 hover:bg-red-700' : ''}
                  >
                    Venda (Short)
                  </Button>
                </div>
              </div>

              {/* Order Type */}
              <div>
                <label className="block text-sm font-medium mb-1">Tipo</label>
                <select
                  value={orderType}
                  onChange={e => setOrderType(e.target.value as OrderType)}
                  className="w-full px-3 py-2 bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                >
                  <option value="market">Market</option>
                  <option value="limit">Limit</option>
                  <option value="stop_market">Stop Market</option>
                  <option value="stop_limit">Stop Limit</option>
                </select>
              </div>

              {/* Price (for limit orders) */}
              {orderType !== 'market' && (
                <div>
                  <label className="block text-sm font-medium mb-1">Preco</label>
                  <input
                    type="text"
                    value={price}
                    onChange={e => setPrice(e.target.value)}
                    placeholder="0.00"
                    className="w-full px-3 py-2 bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
              )}

              {/* Quantity */}
              <div>
                <label className="block text-sm font-medium mb-1">Quantidade</label>
                <input
                  type="text"
                  value={quantity}
                  onChange={e => setQuantity(e.target.value)}
                  placeholder="0.001"
                  className="w-full px-3 py-2 bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>

              {/* Leverage */}
              <div>
                <label className="block text-sm font-medium mb-1">Alavancagem: {leverage}x</label>
                <input
                  type="range"
                  min="1"
                  max="125"
                  value={leverage}
                  onChange={e => setLeverage(parseInt(e.target.value))}
                  className="w-full"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>1x</span>
                  <span>125x</span>
                </div>
              </div>

              {/* Stop Loss / Take Profit */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Stop Loss</label>
                  <input
                    type="text"
                    value={stopLoss}
                    onChange={e => setStopLoss(e.target.value)}
                    placeholder="Opcional"
                    className="w-full px-3 py-2 bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Take Profit</label>
                  <input
                    type="text"
                    value={takeProfit}
                    onChange={e => setTakeProfit(e.target.value)}
                    placeholder="Opcional"
                    className="w-full px-3 py-2 bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
              </div>

              {/* Submit */}
              <Button
                type="submit"
                disabled={submitting}
                className={`w-full ${side === 'buy' ? 'bg-green-600 hover:bg-green-700' : 'bg-red-600 hover:bg-red-700'}`}
              >
                {submitting ? 'Enviando...' : side === 'buy' ? 'Comprar' : 'Vender'}
              </Button>
            </form>
          </CardContent>
        </Card>

        {/* Risk Management & Balances */}
        <div className="space-y-6">
          {/* Balances */}
          <Card>
            <CardHeader>
              <CardTitle>Saldos</CardTitle>
            </CardHeader>
            <CardContent>
              {balances.length > 0 ? (
                <div className="space-y-2">
                  {balances.map(balance => (
                    <div key={balance.asset} className="flex justify-between items-center py-2 border-b border-border last:border-0">
                      <span className="font-medium">{balance.asset}</span>
                      <div className="text-right">
                        <div>{formatCurrency(balance.total, '')}</div>
                        <div className="text-xs text-muted-foreground">
                          Livre: {formatCurrency(balance.free, '')} | Bloqueado: {formatCurrency(balance.locked, '')}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-muted-foreground text-center py-4">Nenhum saldo disponivel</p>
              )}
            </CardContent>
          </Card>

          {/* Risk Stats */}
          <Card>
            <CardHeader>
              <CardTitle>Gestao de Risco</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {riskStats && (
                <>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-muted/50 rounded-md">
                      <div className="text-sm text-muted-foreground">Trading</div>
                      <div className={`font-bold ${riskStats.trading_enabled ? 'text-green-500' : 'text-red-500'}`}>
                        {riskStats.trading_enabled ? 'Habilitado' : 'Desabilitado'}
                      </div>
                    </div>
                    <div className="p-3 bg-muted/50 rounded-md">
                      <div className="text-sm text-muted-foreground">Circuit Breaker</div>
                      <div className="font-bold">{riskStats.circuit_state}</div>
                    </div>
                    <div className="p-3 bg-muted/50 rounded-md">
                      <div className="text-sm text-muted-foreground">PnL Diario</div>
                      <div className={`font-bold ${parseFloat(riskStats.daily_pnl) >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                        {formatCurrency(riskStats.daily_pnl)}
                      </div>
                    </div>
                    <div className="p-3 bg-muted/50 rounded-md">
                      <div className="text-sm text-muted-foreground">Exposicao</div>
                      <div className="font-bold">{formatCurrency(riskStats.total_exposure)}</div>
                    </div>
                    <div className="p-3 bg-muted/50 rounded-md col-span-2">
                      <div className="text-sm text-muted-foreground">Posicoes Abertas</div>
                      <div className="font-bold">{riskStats.position_count}</div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Button
                      variant={workerRunning ? 'destructive' : 'default'}
                      onClick={handleToggleWorker}
                      className="w-full"
                    >
                      {workerRunning ? 'Parar Worker' : 'Iniciar Worker'}
                    </Button>
                    <Button
                      variant={riskStats.trading_enabled ? 'destructive' : 'default'}
                      onClick={handleToggleTrading}
                      className="w-full"
                    >
                      {riskStats.trading_enabled ? 'Desabilitar Trading' : 'Habilitar Trading'}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={handleResetLosses}
                      className="w-full"
                    >
                      Resetar Perdas Diarias
                    </Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Positions and Orders Tables */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Open Positions */}
        <Card>
          <CardHeader>
            <CardTitle>Posicoes Abertas</CardTitle>
          </CardHeader>
          <CardContent>
            <PositionsTable positions={positions} />
          </CardContent>
        </Card>

        {/* Open Orders */}
        <Card>
          <CardHeader>
            <CardTitle>Ordens Abertas</CardTitle>
          </CardHeader>
          <CardContent>
            <OrdersTable orders={orders} onCancelOrder={handleCancelOrder} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
