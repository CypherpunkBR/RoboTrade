import { useState, useEffect, useCallback } from 'react';
import { TradingChart } from '@/components/TradingChart';
import { PriceAlerts } from '@/components/PriceAlerts';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import type { OrderDto, PositionDto } from '@/types';
import { getOpenOrders, getPositions } from '@/lib/tauri';

const SYMBOLS = [
  'BTCUSDT',
  'ETHUSDT',
  'BNBUSDT',
  'SOLUSDT',
  'XRPUSDT',
  'ADAUSDT',
  'DOGEUSDT',
  'AVAXUSDT',
];

export function Charts() {
  const [selectedSymbol, setSelectedSymbol] = useState('BTCUSDT');
  const [alertPrice, setAlertPrice] = useState<number | undefined>();
  const [orders, setOrders] = useState<OrderDto[]>([]);
  const [positions, setPositions] = useState<PositionDto[]>([]);

  // Fetch orders and positions
  const fetchOrdersAndPositions = useCallback(async () => {
    try {
      const [ordersData, positionsData] = await Promise.all([
        getOpenOrders(),
        getPositions(),
      ]);
      setOrders(ordersData);
      setPositions(positionsData);
    } catch (err) {
      console.error('Error fetching orders/positions:', err);
    }
  }, []);

  // Fetch on mount and periodically
  useEffect(() => {
    fetchOrdersAndPositions();
    const interval = setInterval(fetchOrdersAndPositions, 10000); // Refresh every 10 seconds
    return () => clearInterval(interval);
  }, [fetchOrdersAndPositions]);

  // Called when user clicks on chart to set alert price
  const handlePriceClick = (price: number) => {
    setAlertPrice(price);
  };

  return (
    <div className="space-y-6">
      {/* Main content */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Chart Card - takes 2 columns on xl screens */}
        <Card className="xl:col-span-2">
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle>Grafico de Precos</CardTitle>
              {/* Symbol selector */}
              <div className="flex gap-1 bg-muted rounded-md p-0.5">
                {SYMBOLS.map((sym) => (
                  <button
                    key={sym}
                    onClick={() => {
                      setSelectedSymbol(sym);
                      setAlertPrice(undefined);
                    }}
                    className={`px-2 py-1 text-xs font-medium rounded transition-colors ${
                      selectedSymbol === sym
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
              symbol={selectedSymbol}
              interval="1h"
              height={550}
              orders={orders}
              positions={positions}
              onPriceClick={handlePriceClick}
            />
            <p className="mt-2 text-xs text-muted-foreground">
              Dica: Clique no grafico para definir um alerta de preco. Use scroll para zoom e arraste para navegar.
            </p>
          </CardContent>
        </Card>

        {/* Price Alerts - component already includes Card wrapper */}
        <div className="xl:col-span-1">
          <PriceAlerts
            symbol={selectedSymbol}
            prefilledPrice={alertPrice}
            onAlertCreated={() => setAlertPrice(undefined)}
          />
        </div>
      </div>
    </div>
  );
}
