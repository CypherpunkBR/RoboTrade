import { useState, useEffect, useCallback } from 'react';
import type { PriceAlertDto, AlertCondition, CreatePriceAlertRequest } from '@/types';
import {
  listPriceAlerts,
  createPriceAlert,
  deletePriceAlert,
  disablePriceAlert,
  enablePriceAlert,
} from '@/lib/tauri';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface PriceAlertsProps {
  symbol?: string;
  prefilledPrice?: number;
  onAlertCreated?: () => void;
}

const CONDITION_LABELS: Record<AlertCondition, string> = {
  above: 'Acima de',
  below: 'Abaixo de',
  cross_above: 'Cruza acima',
  cross_below: 'Cruza abaixo',
  percent_up: '% para cima',
  percent_down: '% para baixo',
};

const STATUS_COLORS: Record<string, string> = {
  active: 'bg-green-500/20 text-green-400',
  triggered: 'bg-yellow-500/20 text-yellow-400',
  disabled: 'bg-gray-500/20 text-gray-400',
  expired: 'bg-red-500/20 text-red-400',
};

export function PriceAlerts({ symbol = 'BTCUSDT', prefilledPrice, onAlertCreated }: PriceAlertsProps) {
  const [alerts, setAlerts] = useState<PriceAlertDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);

  // Form state
  const [newAlert, setNewAlert] = useState<CreatePriceAlertRequest>({
    symbol,
    condition: 'above',
    target_price: prefilledPrice?.toString() || '',
    recurring: false,
  });

  // Update form when prefilledPrice changes
  useEffect(() => {
    if (prefilledPrice) {
      setNewAlert((prev) => ({
        ...prev,
        target_price: prefilledPrice.toFixed(2),
      }));
      setShowForm(true);
    }
  }, [prefilledPrice]);

  // Fetch alerts
  const fetchAlerts = useCallback(async () => {
    try {
      const data = await listPriceAlerts();
      // Filter by symbol if specified
      const filtered = symbol ? data.filter((a) => a.symbol === symbol) : data;
      setAlerts(filtered);
      setError(null);
    } catch (err) {
      console.error('Error fetching alerts:', err);
      setError(err instanceof Error ? err.message : 'Erro ao carregar alertas');
    } finally {
      setLoading(false);
    }
  }, [symbol]);

  useEffect(() => {
    fetchAlerts();
  }, [fetchAlerts]);

  const handleCreateAlert = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!newAlert.target_price) {
      setError('Preco alvo e obrigatorio');
      return;
    }

    try {
      await createPriceAlert(newAlert);
      await fetchAlerts();
      setNewAlert({
        symbol,
        condition: 'above',
        target_price: '',
        recurring: false,
      });
      setShowForm(false);
      onAlertCreated?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao criar alerta');
    }
  };

  const handleDeleteAlert = async (id: string) => {
    try {
      await deletePriceAlert(id);
      await fetchAlerts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao excluir alerta');
    }
  };

  const handleToggleAlert = async (alert: PriceAlertDto) => {
    try {
      if (alert.status === 'active') {
        await disablePriceAlert(alert.id);
      } else if (alert.status === 'disabled') {
        await enablePriceAlert(alert.id);
      }
      await fetchAlerts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erro ao alternar alerta');
    }
  };

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const needsPercent = (condition: AlertCondition) =>
    condition === 'percent_up' || condition === 'percent_down';

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Alertas de Preco</CardTitle>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-3 py-1 text-sm bg-primary text-white rounded-md hover:bg-primary/90 transition-colors"
        >
          {showForm ? 'Cancelar' : '+ Novo Alerta'}
        </button>
      </CardHeader>
      <CardContent>
        {error && (
          <div className="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
            {error}
          </div>
        )}

        {/* Create Alert Form */}
        {showForm && (
          <form onSubmit={handleCreateAlert} className="mb-4 p-4 bg-muted rounded-lg space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium mb-1">Condicao</label>
                <select
                  value={newAlert.condition}
                  onChange={(e) =>
                    setNewAlert((prev) => ({
                      ...prev,
                      condition: e.target.value as AlertCondition,
                    }))
                  }
                  className="w-full px-3 py-2 bg-card border border-border rounded-md text-sm"
                >
                  {Object.entries(CONDITION_LABELS).map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">
                  {needsPercent(newAlert.condition) ? 'Porcentagem (%)' : 'Preco Alvo ($)'}
                </label>
                <input
                  type="number"
                  step={needsPercent(newAlert.condition) ? '0.1' : '0.01'}
                  value={
                    needsPercent(newAlert.condition)
                      ? newAlert.percent || ''
                      : newAlert.target_price
                  }
                  onChange={(e) =>
                    setNewAlert((prev) =>
                      needsPercent(newAlert.condition)
                        ? { ...prev, percent: e.target.value }
                        : { ...prev, target_price: e.target.value }
                    )
                  }
                  placeholder={needsPercent(newAlert.condition) ? '5.0' : '50000.00'}
                  className="w-full px-3 py-2 bg-card border border-border rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Mensagem (opcional)</label>
              <input
                type="text"
                value={newAlert.message || ''}
                onChange={(e) =>
                  setNewAlert((prev) => ({ ...prev, message: e.target.value || undefined }))
                }
                placeholder="Lembrete: verificar posicao..."
                className="w-full px-3 py-2 bg-card border border-border rounded-md text-sm"
              />
            </div>

            <div className="flex items-center gap-4">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={newAlert.recurring}
                  onChange={(e) =>
                    setNewAlert((prev) => ({ ...prev, recurring: e.target.checked }))
                  }
                  className="rounded"
                />
                <span className="text-sm">Alerta recorrente</span>
              </label>

              <button
                type="submit"
                className="ml-auto px-4 py-2 bg-primary text-white rounded-md hover:bg-primary/90 transition-colors text-sm font-medium"
              >
                Criar Alerta
              </button>
            </div>
          </form>
        )}

        {/* Alerts List */}
        {loading ? (
          <div className="flex justify-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
          </div>
        ) : alerts.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            Nenhum alerta configurado
          </div>
        ) : (
          <div className="space-y-2">
            {alerts.map((alert) => (
              <div
                key={alert.id}
                className="flex items-center justify-between p-3 bg-muted rounded-lg"
              >
                <div className="flex items-center gap-3">
                  <Badge className={STATUS_COLORS[alert.status] || STATUS_COLORS.active}>
                    {alert.status}
                  </Badge>
                  <div>
                    <div className="font-medium">
                      {CONDITION_LABELS[alert.condition]}{' '}
                      {alert.condition.includes('percent')
                        ? `${alert.percent}%`
                        : `$${parseFloat(alert.target_price).toLocaleString()}`}
                    </div>
                    {alert.message && (
                      <div className="text-sm text-muted-foreground">{alert.message}</div>
                    )}
                    <div className="text-xs text-muted-foreground">
                      Criado: {formatDate(alert.created_at)}
                      {alert.trigger_count > 0 && ` | Disparos: ${alert.trigger_count}`}
                      {alert.recurring && ' | Recorrente'}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  {(alert.status === 'active' || alert.status === 'disabled') && (
                    <button
                      onClick={() => handleToggleAlert(alert)}
                      className="px-2 py-1 text-xs bg-card hover:bg-muted-foreground/20 rounded transition-colors"
                    >
                      {alert.status === 'active' ? 'Desativar' : 'Ativar'}
                    </button>
                  )}
                  <button
                    onClick={() => handleDeleteAlert(alert.id)}
                    className="px-2 py-1 text-xs text-destructive hover:bg-destructive/20 rounded transition-colors"
                  >
                    Excluir
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
