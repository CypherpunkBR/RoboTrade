import { useState, useEffect, useCallback } from 'react';
import { Dashboard } from './pages/Dashboard';
import { Trading } from './pages/Trading';
import { History } from './pages/History';
import { Charts } from './pages/Charts';
import { Settings } from './pages/Settings';
import { Reports } from './pages/Reports';
import type { TradingMode, ConnectionStatus } from './types';
import { getTradingMode, getConnectionStatus, setTradingMode } from './lib/tauri';
import { StatusIndicator } from './components/StatusIndicator';
import { TradingModeSwitch } from './components/TradingModeSwitch';
import { GlobalFilterDropdown } from './components/GlobalFilterDropdown';

type Page = 'dashboard' | 'trading' | 'charts' | 'history' | 'reports' | 'settings';

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('dashboard');
  const [tradingMode, setTradingModeState] = useState<TradingMode>('paper');
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus | null>(null);
  const [selectedTradingSymbol, setSelectedTradingSymbol] = useState<string | null>(null);

  // Navigate to Trading page with a specific symbol
  const navigateToTrading = (symbol: string) => {
    setSelectedTradingSymbol(symbol);
    setCurrentPage('trading');
  };

  const fetchStatus = useCallback(async () => {
    try {
      const [mode, status] = await Promise.all([
        getTradingMode(),
        getConnectionStatus(),
      ]);
      setTradingModeState(mode);
      setConnectionStatus(status);
    } catch (err) {
      console.error('Error fetching status:', err);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleModeChange = async (mode: TradingMode) => {
    try {
      await setTradingMode(mode);
      setTradingModeState(mode);
    } catch (err) {
      console.error('Error changing mode:', err);
    }
  };

  const navItems: { key: Page; label: string }[] = [
    { key: 'dashboard', label: 'Dashboard' },
    { key: 'charts', label: 'Graficos' },
    { key: 'trading', label: 'Trading' },
    { key: 'history', label: 'Historico' },
    { key: 'reports', label: 'Relatorios' },
    { key: 'settings', label: 'Config' },
  ];

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card sticky top-0 z-50">
        <div className="container mx-auto px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <h1 className="text-xl font-bold text-primary">RoboTrade</h1>

              {/* Navigation */}
              <nav className="flex gap-1">
                {navItems.map(item => (
                  <button
                    key={item.key}
                    onClick={() => setCurrentPage(item.key)}
                    className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                      currentPage === item.key
                        ? 'bg-primary text-white'
                        : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                    }`}
                  >
                    {item.label}
                  </button>
                ))}
              </nav>
            </div>

            <div className="flex items-center gap-4">
              <GlobalFilterDropdown />
              <TradingModeSwitch
                mode={tradingMode}
                onModeChange={handleModeChange}
              />
              {connectionStatus && (
                <div className="flex items-center gap-3">
                  <StatusIndicator
                    status={connectionStatus.binance}
                    label="Binance"
                  />
                  <StatusIndicator
                    status={connectionStatus.database}
                    label="DB"
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="container mx-auto px-4 py-6">
        {currentPage === 'dashboard' && <Dashboard onNavigateToTrading={navigateToTrading} />}
        {currentPage === 'charts' && <Charts />}
        {currentPage === 'trading' && (
          <Trading
            initialSymbol={selectedTradingSymbol}
            onSymbolUsed={() => setSelectedTradingSymbol(null)}
          />
        )}
        {currentPage === 'history' && <History />}
        {currentPage === 'reports' && <Reports />}
        {currentPage === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;
