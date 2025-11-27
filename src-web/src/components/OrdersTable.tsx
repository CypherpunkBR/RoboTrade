import type { OrderDto } from '@/types';
import { formatCurrency } from '@/lib/tauri';
import { Badge } from './ui/badge';

interface OrdersTableProps {
  orders: OrderDto[];
  onCancelOrder?: (id: string, symbol: string) => void;
  onSymbolClick?: (symbol: string) => void;
}

const orderTypeLabels: Record<string, string> = {
  market: 'Mercado',
  limit: 'Limite',
  stop_market: 'Stop',
  stop_limit: 'Stop Limite',
  take_profit_market: 'Take Profit',
  take_profit_limit: 'TP Limite',
};

const orderStatusVariant: Record<string, 'default' | 'secondary' | 'success' | 'warning' | 'destructive'> = {
  pending: 'secondary',
  open: 'default',
  partially_filled: 'warning',
  filled: 'success',
  cancelled: 'secondary',
  rejected: 'destructive',
  expired: 'secondary',
};

const exchangeLabels: Record<string, string> = {
  paper: 'Paper',
  binance_futures: 'Binance',
  kraken_futures: 'Kraken',
};

export function OrdersTable({ orders, onCancelOrder, onSymbolClick }: OrdersTableProps) {
  if (orders.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        Nenhuma ordem ativa
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-border">
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Exchange</th>
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Par</th>
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Tipo</th>
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Lado</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Qtd</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Preço</th>
            <th className="text-center py-3 px-2 font-medium text-muted-foreground">Status</th>
            <th className="text-center py-3 px-2 font-medium text-muted-foreground">Ações</th>
          </tr>
        </thead>
        <tbody>
          {orders.map((order) => (
            <tr
              key={order.id}
              className={`border-b border-border/50 hover:bg-secondary/30 ${onSymbolClick ? 'cursor-pointer' : ''}`}
              onClick={() => onSymbolClick?.(order.symbol)}
            >
              <td className="py-3 px-2 text-muted-foreground">{exchangeLabels[order.exchange] || order.exchange}</td>
              <td className="py-3 px-2 font-medium">{order.symbol}</td>
              <td className="py-3 px-2">{orderTypeLabels[order.order_type] || order.order_type}</td>
              <td className="py-3 px-2">
                <Badge variant={order.side === 'buy' ? 'success' : 'destructive'}>
                  {order.side === 'buy' ? 'COMPRA' : 'VENDA'}
                </Badge>
              </td>
              <td className="text-right py-3 px-2">{formatCurrency(order.quantity, '')}</td>
              <td className="text-right py-3 px-2">
                {order.price ? formatCurrency(order.price) : 'Mercado'}
              </td>
              <td className="text-center py-3 px-2">
                <Badge variant={orderStatusVariant[order.status]}>
                  {order.status}
                </Badge>
              </td>
              <td className="text-center py-3 px-2">
                {onCancelOrder && (order.status === 'open' || order.status === 'pending') && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onCancelOrder(order.id, order.symbol);
                    }}
                    className="text-xs text-destructive hover:text-destructive/80 font-medium"
                  >
                    Cancelar
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
