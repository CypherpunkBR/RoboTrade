import { useGlobalFilter, type ExchangeFilter } from '@/contexts/GlobalFilterContext';

const EXCHANGE_OPTIONS: { value: ExchangeFilter; label: string }[] = [
  { value: 'all', label: 'Todas' },
  { value: 'binance', label: 'Binance' },
  { value: 'kraken', label: 'Kraken' },
  { value: 'paper', label: 'Paper' },
];

export function GlobalFilterDropdown() {
  const { filter, setFilter, availableCurrencies, loading } = useGlobalFilter();

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <div className="animate-pulse w-24 h-8 bg-muted rounded" />
        <div className="animate-pulse w-20 h-8 bg-muted rounded" />
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      {/* Exchange Filter */}
      <select
        value={filter.exchange}
        onChange={(e) => setFilter({ exchange: e.target.value as ExchangeFilter })}
        className="h-8 px-2 text-sm bg-muted border-0 rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
        title="Filtrar por corretora"
      >
        {EXCHANGE_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>

      {/* Currency Filter */}
      <select
        value={filter.currency}
        onChange={(e) => setFilter({ currency: e.target.value })}
        className="h-8 px-2 text-sm bg-muted border-0 rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
        title="Filtrar por moeda"
      >
        <option value="all">Todas moedas</option>
        {availableCurrencies.map((currency) => (
          <option key={currency} value={currency}>
            {currency}
          </option>
        ))}
      </select>
    </div>
  );
}
