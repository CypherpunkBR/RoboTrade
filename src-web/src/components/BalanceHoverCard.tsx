import { HoverCard } from '@/components/ui/popover';
import type { BalanceDto } from '@/types';
import { formatCurrency } from '@/lib/tauri';
import { Card, CardContent } from '@/components/ui/card';

interface BalanceHoverCardProps {
  totalBalance: number;
  balances: BalanceDto[];
}

export function BalanceHoverCard({ totalBalance, balances }: BalanceHoverCardProps) {
  // Sort balances by value (largest first)
  const sortedBalances = [...balances]
    .filter(b => parseFloat(b.total) > 0)
    .sort((a, b) => parseFloat(b.total) - parseFloat(a.total));

  const content = (
    <Card className="shadow-sm border-0 bg-transparent">
      <CardContent className="pt-4">
        <div className="text-sm text-muted-foreground">Saldo Total</div>
        <div className="text-2xl font-bold">{formatCurrency(totalBalance)}</div>
      </CardContent>
    </Card>
  );

  if (sortedBalances.length === 0) {
    return content;
  }

  return (
    <HoverCard
      trigger={<div className="cursor-pointer">{content}</div>}
      className="min-w-[260px]"
    >
      <div className="space-y-3">
        <h4 className="font-semibold text-sm border-b border-border pb-2">
          Detalhes do Saldo
        </h4>

        <div className="space-y-2">
          {sortedBalances.slice(0, 10).map((balance) => (
            <div key={balance.asset} className="flex items-center justify-between">
              <span className="font-medium text-sm">{balance.asset}</span>
              <div className="text-right">
                <div className="text-sm font-semibold">
                  {formatCurrency(balance.total, '')}
                </div>
                {parseFloat(balance.locked) > 0 && (
                  <div className="text-xs text-muted-foreground">
                    Bloqueado: {formatCurrency(balance.locked, '')}
                  </div>
                )}
              </div>
            </div>
          ))}
          {sortedBalances.length > 10 && (
            <div className="text-xs text-muted-foreground italic pt-1">
              +{sortedBalances.length - 10} outros ativos
            </div>
          )}
        </div>
      </div>
    </HoverCard>
  );
}
