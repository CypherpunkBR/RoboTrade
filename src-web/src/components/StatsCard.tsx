import { Card, CardContent } from './ui/card';
import { getPnlColor } from '@/lib/tauri';

interface StatsCardProps {
  title: string;
  value: string;
  subtitle?: string;
  isPnl?: boolean;
  icon?: React.ReactNode;
}

export function StatsCard({ title, value, subtitle, isPnl, icon }: StatsCardProps) {
  const valueColor = isPnl ? getPnlColor(value) : undefined;

  return (
    <Card>
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{title}</p>
            <p
              className="text-2xl font-bold mt-1"
              style={valueColor ? { color: valueColor } : undefined}
            >
              {value}
            </p>
            {subtitle && (
              <p
                className="text-xs mt-1"
                style={isPnl && valueColor ? { color: valueColor } : undefined}
              >
                {subtitle}
              </p>
            )}
          </div>
          {icon && (
            <div className="text-muted-foreground">{icon}</div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
