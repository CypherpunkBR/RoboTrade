import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from 'react';
import { getUserPreferences, saveUserPreferences, getBalances } from '@/lib/tauri';
import type { BalanceDto } from '@/types';

export type ExchangeFilter = 'all' | 'binance' | 'kraken' | 'paper';
export type CurrencyFilter = 'all' | string;

export interface GlobalFilter {
  exchange: ExchangeFilter;
  currency: CurrencyFilter;
}

export interface UserPreferences {
  default_exchange: string;
  default_currency: string;
}

interface GlobalFilterContextValue {
  filter: GlobalFilter;
  setFilter: (filter: Partial<GlobalFilter>) => void;
  availableExchanges: ExchangeFilter[];
  availableCurrencies: string[];
  loading: boolean;
}

const defaultFilter: GlobalFilter = {
  exchange: 'all',
  currency: 'all',
};

const GlobalFilterContext = createContext<GlobalFilterContextValue | null>(null);

interface GlobalFilterProviderProps {
  children: ReactNode;
}

export function GlobalFilterProvider({ children }: GlobalFilterProviderProps) {
  const [filter, setFilterState] = useState<GlobalFilter>(defaultFilter);
  const [availableCurrencies, setAvailableCurrencies] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  const availableExchanges: ExchangeFilter[] = ['all', 'binance', 'kraken', 'paper'];

  // Load user preferences on mount
  useEffect(() => {
    const loadPreferences = async () => {
      try {
        const prefs = await getUserPreferences();
        if (prefs) {
          setFilterState({
            exchange: (prefs.default_exchange as ExchangeFilter) || 'all',
            currency: prefs.default_currency || 'all',
          });
        }
      } catch (err) {
        console.error('Error loading user preferences:', err);
      } finally {
        setLoading(false);
      }
    };

    loadPreferences();
  }, []);

  // Load available currencies from balances
  useEffect(() => {
    const loadCurrencies = async () => {
      try {
        const balances: BalanceDto[] = await getBalances();
        const currencies = [...new Set(balances.map((b) => b.asset))].sort();
        setAvailableCurrencies(currencies);
      } catch (err) {
        console.error('Error loading currencies:', err);
      }
    };

    loadCurrencies();
    const interval = setInterval(loadCurrencies, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, []);

  const setFilter = useCallback(async (newFilter: Partial<GlobalFilter>) => {
    setFilterState((prev) => {
      const updated = { ...prev, ...newFilter };

      // Save to backend
      saveUserPreferences({
        default_exchange: updated.exchange,
        default_currency: updated.currency,
      }).catch((err) => console.error('Error saving preferences:', err));

      return updated;
    });
  }, []);

  return (
    <GlobalFilterContext.Provider
      value={{
        filter,
        setFilter,
        availableExchanges,
        availableCurrencies,
        loading,
      }}
    >
      {children}
    </GlobalFilterContext.Provider>
  );
}

export function useGlobalFilter() {
  const context = useContext(GlobalFilterContext);
  if (!context) {
    throw new Error('useGlobalFilter must be used within a GlobalFilterProvider');
  }
  return context;
}

// Helper to filter data by current global filter
export function filterByGlobalFilter<T extends { exchange?: string; symbol?: string }>(
  data: T[],
  filter: GlobalFilter
): T[] {
  return data.filter((item) => {
    // Filter by exchange
    if (filter.exchange !== 'all' && item.exchange) {
      if (item.exchange.toLowerCase() !== filter.exchange) {
        return false;
      }
    }

    // Filter by currency (symbol contains currency)
    if (filter.currency !== 'all' && item.symbol) {
      if (!item.symbol.includes(filter.currency)) {
        return false;
      }
    }

    return true;
  });
}
