import { useState } from 'react';
import { TradingChart } from '@/components/TradingChart';
import { PriceAlerts } from '@/components/PriceAlerts';

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

  // Called when user clicks on chart to set alert price
  const handlePriceClick = (price: number) => {
    setAlertPrice(price);
  };

  return (
    <div className="space-y-6">
      {/* Symbol selector */}
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium text-muted-foreground">Simbolo:</span>
        <div className="flex gap-1 bg-muted rounded-md p-0.5">
          {SYMBOLS.map((sym) => (
            <button
              key={sym}
              onClick={() => {
                setSelectedSymbol(sym);
                setAlertPrice(undefined);
              }}
              className={`px-3 py-1.5 text-sm font-medium rounded transition-colors ${
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

      {/* Main content */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Chart - takes 2 columns on xl screens */}
        <div className="xl:col-span-2">
          <TradingChart
            symbol={selectedSymbol}
            interval="1h"
            height={600}
            onPriceClick={handlePriceClick}
          />
          <p className="mt-2 text-xs text-muted-foreground">
            Dica: Clique no grafico para definir um alerta de preco. Use scroll para zoom e arraste para navegar.
          </p>
        </div>

        {/* Price Alerts */}
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
