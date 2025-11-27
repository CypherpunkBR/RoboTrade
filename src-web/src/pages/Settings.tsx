import { useEffect, useState } from 'react';
import type { AppConfig, TradingMode } from '@/types';
import { getConfig, saveConfig, getTradingMode, setTradingMode } from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

export function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [tradingMode, setTradingModeState] = useState<TradingMode>('paper');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Local form state
  const [formData, setFormData] = useState({
    // Trading
    defaultLeverage: 10,
    maxLeverage: 50,
    defaultOrderSizePct: 5,
    trailingStopEnabled: false,
    // Risk
    maxDailyLossPct: 5,
    maxPositionSizePct: 20,
    maxPositions: 5,
    circuitBreakerEnabled: true,
    // Exchange
    binanceTestnet: true,
    krakenDemo: true,
  });

  useEffect(() => {
    async function loadConfig() {
      try {
        const [configData, mode] = await Promise.all([
          getConfig(),
          getTradingMode(),
        ]);
        setConfig(configData);
        setTradingModeState(mode);

        // Populate form
        setFormData({
          defaultLeverage: configData.trading.default_leverage,
          maxLeverage: configData.trading.max_leverage,
          defaultOrderSizePct: configData.trading.default_order_size_pct,
          trailingStopEnabled: configData.trading.trailing_stop_enabled,
          maxDailyLossPct: configData.risk.max_daily_loss_pct,
          maxPositionSizePct: configData.risk.max_position_size_pct,
          maxPositions: configData.risk.max_positions,
          circuitBreakerEnabled: configData.risk.circuit_breaker_enabled,
          binanceTestnet: configData.exchange.binance_testnet,
          krakenDemo: configData.exchange.kraken_demo,
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Erro ao carregar configuracoes');
      } finally {
        setLoading(false);
      }
    }
    loadConfig();
  }, []);

  const handleSave = async () => {
    if (!config) return;

    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      const updatedConfig: AppConfig = {
        trading: {
          default_leverage: formData.defaultLeverage,
          max_leverage: formData.maxLeverage,
          default_order_size_pct: formData.defaultOrderSizePct,
          trailing_stop_enabled: formData.trailingStopEnabled,
        },
        risk: {
          max_daily_loss_pct: formData.maxDailyLossPct,
          max_position_size_pct: formData.maxPositionSizePct,
          max_positions: formData.maxPositions,
          circuit_breaker_enabled: formData.circuitBreakerEnabled,
        },
        exchange: {
          binance_testnet: formData.binanceTestnet,
          kraken_demo: formData.krakenDemo,
        },
      };

      await saveConfig(updatedConfig);
      setConfig(updatedConfig);
      setSuccess('Configuracoes salvas com sucesso!');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao salvar configuracoes');
    } finally {
      setSaving(false);
    }
  };

  const handleModeChange = async (mode: TradingMode) => {
    try {
      await setTradingMode(mode);
      setTradingModeState(mode);
      setSuccess(`Modo alterado para ${mode === 'paper' ? 'Paper Trading' : 'Trading Real'}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao alterar modo');
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
    <div className="space-y-6 max-w-4xl mx-auto">
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

      {/* Trading Mode */}
      <Card>
        <CardHeader>
          <CardTitle>Modo de Trading</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={() => handleModeChange('paper')}
              className={`p-4 rounded-lg border-2 text-left transition-all ${
                tradingMode === 'paper'
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'
              }`}
            >
              <div className="font-semibold">Paper Trading</div>
              <div className="text-sm text-muted-foreground mt-1">
                Simulacao sem dinheiro real. Ideal para testes.
              </div>
            </button>
            <button
              onClick={() => handleModeChange('live')}
              className={`p-4 rounded-lg border-2 text-left transition-all ${
                tradingMode === 'live'
                  ? 'border-yellow-500 bg-yellow-500/10'
                  : 'border-border hover:border-yellow-500/50'
              }`}
            >
              <div className="font-semibold text-yellow-600">Trading Real</div>
              <div className="text-sm text-muted-foreground mt-1">
                Operacoes com dinheiro real. Use com cautela!
              </div>
            </button>
          </div>
        </CardContent>
      </Card>

      {/* Exchange Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>Exchanges</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-3 bg-muted/50 rounded-md">
            <div>
              <div className="font-medium">Binance Futures</div>
              <div className="text-sm text-muted-foreground">
                {formData.binanceTestnet ? 'Testnet (simulacao)' : 'Producao (real)'}
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={formData.binanceTestnet}
                onChange={(e) => setFormData({ ...formData, binanceTestnet: e.target.checked })}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-muted peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all" />
              <span className="ml-2 text-sm">Testnet</span>
            </label>
          </div>

          <div className="flex items-center justify-between p-3 bg-muted/50 rounded-md">
            <div>
              <div className="font-medium">Kraken Futures</div>
              <div className="text-sm text-muted-foreground">
                {formData.krakenDemo ? 'Demo (simulacao)' : 'Producao (real)'}
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={formData.krakenDemo}
                onChange={(e) => setFormData({ ...formData, krakenDemo: e.target.checked })}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-muted peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all" />
              <span className="ml-2 text-sm">Demo</span>
            </label>
          </div>
        </CardContent>
      </Card>

      {/* Trading Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Configuracoes de Trading</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-1">
                Alavancagem Padrao
              </label>
              <input
                type="number"
                min="1"
                max="125"
                value={formData.defaultLeverage}
                onChange={(e) => setFormData({ ...formData, defaultLeverage: parseInt(e.target.value) || 1 })}
                className="w-full px-3 py-2 bg-background border border-border rounded-md"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">
                Alavancagem Maxima
              </label>
              <input
                type="number"
                min="1"
                max="125"
                value={formData.maxLeverage}
                onChange={(e) => setFormData({ ...formData, maxLeverage: parseInt(e.target.value) || 1 })}
                className="w-full px-3 py-2 bg-background border border-border rounded-md"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">
              Tamanho Padrao da Ordem (% do saldo)
            </label>
            <input
              type="number"
              min="0.1"
              max="100"
              step="0.1"
              value={formData.defaultOrderSizePct}
              onChange={(e) => setFormData({ ...formData, defaultOrderSizePct: parseFloat(e.target.value) || 1 })}
              className="w-full px-3 py-2 bg-background border border-border rounded-md"
            />
          </div>

          <div className="flex items-center justify-between p-3 bg-muted/50 rounded-md">
            <div>
              <div className="font-medium">Trailing Stop</div>
              <div className="text-sm text-muted-foreground">
                Stop loss que acompanha o preco
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={formData.trailingStopEnabled}
                onChange={(e) => setFormData({ ...formData, trailingStopEnabled: e.target.checked })}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-muted peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all" />
            </label>
          </div>
        </CardContent>
      </Card>

      {/* Risk Management */}
      <Card>
        <CardHeader>
          <CardTitle>Gestao de Risco</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-1">
                Perda Diaria Maxima (%)
              </label>
              <input
                type="number"
                min="0.1"
                max="100"
                step="0.1"
                value={formData.maxDailyLossPct}
                onChange={(e) => setFormData({ ...formData, maxDailyLossPct: parseFloat(e.target.value) || 1 })}
                className="w-full px-3 py-2 bg-background border border-border rounded-md"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">
                Tamanho Maximo por Posicao (%)
              </label>
              <input
                type="number"
                min="1"
                max="100"
                value={formData.maxPositionSizePct}
                onChange={(e) => setFormData({ ...formData, maxPositionSizePct: parseFloat(e.target.value) || 1 })}
                className="w-full px-3 py-2 bg-background border border-border rounded-md"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">
              Numero Maximo de Posicoes
            </label>
            <input
              type="number"
              min="1"
              max="50"
              value={formData.maxPositions}
              onChange={(e) => setFormData({ ...formData, maxPositions: parseInt(e.target.value) || 1 })}
              className="w-full px-3 py-2 bg-background border border-border rounded-md"
            />
          </div>

          <div className="flex items-center justify-between p-3 bg-muted/50 rounded-md">
            <div>
              <div className="font-medium">Circuit Breaker</div>
              <div className="text-sm text-muted-foreground">
                Parar trading automaticamente ao atingir perda maxima
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={formData.circuitBreakerEnabled}
                onChange={(e) => setFormData({ ...formData, circuitBreakerEnabled: e.target.checked })}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-muted peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all" />
            </label>
          </div>
        </CardContent>
      </Card>

      {/* Save Button */}
      <div className="flex justify-end">
        <Button onClick={handleSave} disabled={saving} className="px-8">
          {saving ? 'Salvando...' : 'Salvar Configuracoes'}
        </Button>
      </div>
    </div>
  );
}
