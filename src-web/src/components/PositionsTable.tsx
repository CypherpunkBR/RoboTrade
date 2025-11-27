import type { PositionDto } from '@/types';
import { formatCurrency, formatPercent, getPnlColor } from '@/lib/tauri';
import { Badge } from './ui/badge';

interface PositionsTableProps {
  positions: PositionDto[];
  onClosePosition?: (id: string) => void;
}

export function PositionsTable({ positions, onClosePosition }: PositionsTableProps) {
  if (positions.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        Nenhuma posição aberta
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-border">
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Par</th>
            <th className="text-left py-3 px-2 font-medium text-muted-foreground">Lado</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Tamanho</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Entrada</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Atual</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">PnL</th>
            <th className="text-right py-3 px-2 font-medium text-muted-foreground">Alav.</th>
            <th className="text-center py-3 px-2 font-medium text-muted-foreground">Ações</th>
          </tr>
        </thead>
        <tbody>
          {positions.map((position) => (
            <tr key={position.id} className="border-b border-border/50 hover:bg-secondary/30">
              <td className="py-3 px-2 font-medium">{position.symbol}</td>
              <td className="py-3 px-2">
                <Badge variant={position.side === 'long' ? 'success' : 'destructive'}>
                  {position.side === 'long' ? 'LONG' : 'SHORT'}
                </Badge>
              </td>
              <td className="text-right py-3 px-2">{formatCurrency(position.quantity, '')}</td>
              <td className="text-right py-3 px-2">{formatCurrency(position.entry_price)}</td>
              <td className="text-right py-3 px-2">{formatCurrency(position.current_price)}</td>
              <td className="text-right py-3 px-2">
                <div className="flex flex-col items-end">
                  <span style={{ color: getPnlColor(position.unrealized_pnl) }}>
                    {formatCurrency(position.unrealized_pnl)}
                  </span>
                  <span
                    className="text-xs"
                    style={{ color: getPnlColor(position.unrealized_pnl_pct) }}
                  >
                    {formatPercent(position.unrealized_pnl_pct)}
                  </span>
                </div>
              </td>
              <td className="text-right py-3 px-2">{position.leverage}x</td>
              <td className="text-center py-3 px-2">
                {onClosePosition && (
                  <button
                    onClick={() => onClosePosition(position.id)}
                    className="text-xs text-destructive hover:text-destructive/80 font-medium"
                  >
                    Fechar
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
