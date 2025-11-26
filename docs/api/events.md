# Events

Sistema de eventos assíncronos do RoboTrade.

## Visão Geral

O backend emite eventos via Tauri para notificar o frontend sobre mudanças de estado, atualizações de dados e alertas. O frontend pode registrar listeners para reagir a esses eventos.

```typescript
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// Registrar listener
const unlisten: UnlistenFn = await listen<PriceUpdate>('price_update', (event) => {
  console.log('Price update:', event.payload);
});

// Remover listener quando não mais necessário
unlisten();
```

## Eventos de Market Data

### price_update

Emitido quando o preço de um símbolo é atualizado.

**Payload:**

```typescript
interface PriceUpdateEvent {
  symbol: string;
  price: string;
  bidPrice: string;
  askPrice: string;
  change24h: string;
  volume24h: string;
  timestamp: string;
}
```

**Frequência:** A cada tick de preço (via WebSocket) ou polling interval

**Exemplo:**

```typescript
await listen<PriceUpdateEvent>('price_update', (event) => {
  const { symbol, price, change24h } = event.payload;
  updatePriceDisplay(symbol, price, change24h);
});
```

---

### candle_update

Emitido quando um novo candlestick é fechado.

**Payload:**

```typescript
interface CandleUpdateEvent {
  symbol: string;
  timeframe: string;
  candle: Candlestick;
  isComplete: boolean;  // true quando candle fechou
}
```

**Frequência:** A cada fechamento de candle do timeframe

---

### fear_greed_update

Emitido quando o índice Fear & Greed é atualizado.

**Payload:**

```typescript
interface FearGreedUpdateEvent {
  value: number;
  classification: string;
  previousValue: number;
  change: number;
  timestamp: string;
}
```

**Frequência:** Uma vez por dia (ou quando forçado refresh)

---

## Eventos de Orders

### order_created

Emitido quando uma ordem é criada.

**Payload:**

```typescript
interface OrderCreatedEvent {
  orderId: string;
  clientOrderId: string;
  symbol: string;
  side: OrderSide;
  orderType: string;
  quantity: string;
  price?: string;
  timestamp: string;
}
```

---

### order_updated

Emitido quando o status de uma ordem muda.

**Payload:**

```typescript
interface OrderUpdatedEvent {
  orderId: string;
  symbol: string;
  previousStatus: OrderStatus;
  newStatus: OrderStatus;
  filledQuantity: string;
  averagePrice?: string;
  timestamp: string;
}
```

---

### order_filled

Emitido quando uma ordem é completamente executada.

**Payload:**

```typescript
interface OrderFilledEvent {
  orderId: string;
  clientOrderId: string;
  symbol: string;
  side: OrderSide;
  quantity: string;
  averagePrice: string;
  commission: string;
  commissionAsset: string;
  pnl?: string;           // Se fechou posição
  timestamp: string;
}
```

---

### order_cancelled

Emitido quando uma ordem é cancelada.

**Payload:**

```typescript
interface OrderCancelledEvent {
  orderId: string;
  symbol: string;
  reason: CancelReason;
  timestamp: string;
}

type CancelReason = 'UserRequested' | 'Expired' | 'InsufficientBalance' | 'RiskLimit' | 'SystemShutdown';
```

---

### order_rejected

Emitido quando uma ordem é rejeitada.

**Payload:**

```typescript
interface OrderRejectedEvent {
  orderId?: string;
  clientOrderId: string;
  symbol: string;
  reason: string;
  errorCode: string;
  timestamp: string;
}
```

---

## Eventos de Positions

### position_opened

Emitido quando uma nova posição é aberta.

**Payload:**

```typescript
interface PositionOpenedEvent {
  positionId: string;
  symbol: string;
  side: PositionSide;
  quantity: string;
  entryPrice: string;
  strategyId?: string;
  strategyName?: string;
  timestamp: string;
}
```

---

### position_updated

Emitido quando uma posição existente é modificada.

**Payload:**

```typescript
interface PositionUpdatedEvent {
  positionId: string;
  symbol: string;
  previousQuantity: string;
  newQuantity: string;
  averageEntryPrice: string;
  unrealizedPnl: string;
  timestamp: string;
}
```

---

### position_closed

Emitido quando uma posição é fechada.

**Payload:**

```typescript
interface PositionClosedEvent {
  positionId: string;
  symbol: string;
  side: PositionSide;
  entryPrice: string;
  exitPrice: string;
  quantity: string;
  realizedPnl: string;
  realizedPnlPercent: string;
  holdingPeriodHours: number;
  closeReason: CloseReason;
  timestamp: string;
}

type CloseReason =
  | 'Manual'
  | 'StopLoss'
  | 'TakeProfit'
  | 'TrailingStop'
  | 'StrategyExit'
  | 'Liquidation'
  | 'RiskLimit';
```

---

### pnl_update

Emitido periodicamente com P&L atualizado de posições abertas.

**Payload:**

```typescript
interface PnlUpdateEvent {
  positions: PositionPnl[];
  totalUnrealizedPnl: string;
  totalRealizedPnlToday: string;
  timestamp: string;
}

interface PositionPnl {
  positionId: string;
  symbol: string;
  unrealizedPnl: string;
  unrealizedPnlPercent: string;
  currentPrice: string;
}
```

**Frequência:** A cada 1 segundo (configurável)

---

## Eventos de Strategy

### strategy_signal

Emitido quando uma estratégia gera um sinal.

**Payload:**

```typescript
interface StrategySignalEvent {
  strategyId: string;
  strategyName: string;
  symbol: string;
  signal: Signal;
  indicators: IndicatorValues;
  timestamp: string;
}

interface Signal {
  type: 'Entry' | 'Exit' | 'Hold';
  side?: OrderSide;
  strength: number;      // 0-1
  reason: string;
}

interface IndicatorValues {
  [indicatorName: string]: number | Record<string, number>;
}
```

---

### strategy_execution

Emitido quando uma estratégia executa uma ação.

**Payload:**

```typescript
interface StrategyExecutionEvent {
  strategyId: string;
  strategyName: string;
  executionId: string;
  action: StrategyAction;
  result: ExecutionResult;
  timestamp: string;
}

type StrategyAction = 'PlaceOrder' | 'ModifyPosition' | 'ClosePosition' | 'SetStopLoss' | 'SetTakeProfit';

interface ExecutionResult {
  success: boolean;
  orderId?: string;
  error?: string;
}
```

---

### strategy_error

Emitido quando uma estratégia encontra um erro.

**Payload:**

```typescript
interface StrategyErrorEvent {
  strategyId: string;
  strategyName: string;
  error: string;
  errorCode: string;
  willRetry: boolean;
  retryCount: number;
  timestamp: string;
}
```

---

## Eventos de Backtest

### backtest_progress

Emitido durante a execução de um backtest.

**Payload:**

```typescript
interface BacktestProgressEvent {
  jobId: string;
  progress: number;         // 0-100
  currentDate: string;
  tradesProcessed: number;
  currentEquity: string;
  estimatedTimeRemaining?: number;  // segundos
}
```

**Frequência:** A cada 1% de progresso ou 1 segundo

---

### backtest_completed

Emitido quando um backtest é concluído.

**Payload:**

```typescript
interface BacktestCompletedEvent {
  jobId: string;
  success: boolean;
  summary?: BacktestSummary;
  error?: string;
  duration: number;  // ms
  timestamp: string;
}
```

---

## Eventos de System

### connection_status

Emitido quando o status de conexão muda.

**Payload:**

```typescript
interface ConnectionStatusEvent {
  service: 'Exchange' | 'Database' | 'WebSocket';
  status: 'Connected' | 'Disconnected' | 'Reconnecting' | 'Error';
  error?: string;
  timestamp: string;
}
```

---

### system_alert

Emitido para alertas importantes do sistema.

**Payload:**

```typescript
interface SystemAlertEvent {
  level: 'Info' | 'Warning' | 'Error' | 'Critical';
  title: string;
  message: string;
  action?: AlertAction;
  timestamp: string;
}

interface AlertAction {
  label: string;
  command: string;
  params?: Record<string, unknown>;
}
```

---

### rate_limit_warning

Emitido quando se aproxima do rate limit da API.

**Payload:**

```typescript
interface RateLimitWarningEvent {
  service: string;
  currentUsage: number;
  limit: number;
  resetAt: string;
  timestamp: string;
}
```

---

### risk_alert

Emitido quando limites de risco são atingidos.

**Payload:**

```typescript
interface RiskAlertEvent {
  type: RiskAlertType;
  level: 'Warning' | 'Critical';
  currentValue: string;
  threshold: string;
  message: string;
  action?: string;
  timestamp: string;
}

type RiskAlertType =
  | 'DailyLossLimit'
  | 'MaxDrawdown'
  | 'MaxOpenPositions'
  | 'MaxOrderSize'
  | 'MarginLevel';
```

---

### job_status

Emitido quando o status de um job muda.

**Payload:**

```typescript
interface JobStatusEvent {
  jobId: string;
  jobType: string;
  previousStatus: JobStatus;
  newStatus: JobStatus;
  progress?: number;
  error?: string;
  timestamp: string;
}
```

---

## Gerenciamento de Eventos no Frontend

### Hook React para Eventos

```typescript
import { useEffect, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

function useEvent<T>(eventName: string): T | null {
  const [data, setData] = useState<T | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn;

    const setupListener = async () => {
      unlisten = await listen<T>(eventName, (event) => {
        setData(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [eventName]);

  return data;
}

// Uso
function PriceDisplay({ symbol }: { symbol: string }) {
  const priceUpdate = useEvent<PriceUpdateEvent>('price_update');

  if (!priceUpdate || priceUpdate.symbol !== symbol) {
    return null;
  }

  return <span>{priceUpdate.price}</span>;
}
```

### Múltiplos Listeners

```typescript
import { listen } from '@tauri-apps/api/event';

async function setupEventListeners() {
  const unlisteners: UnlistenFn[] = [];

  // Price updates
  unlisteners.push(
    await listen<PriceUpdateEvent>('price_update', handlePriceUpdate)
  );

  // Order events
  unlisteners.push(
    await listen<OrderFilledEvent>('order_filled', handleOrderFilled)
  );
  unlisteners.push(
    await listen<OrderRejectedEvent>('order_rejected', handleOrderRejected)
  );

  // System alerts
  unlisteners.push(
    await listen<SystemAlertEvent>('system_alert', handleSystemAlert)
  );

  // Cleanup function
  return () => {
    unlisteners.forEach((unlisten) => unlisten());
  };
}
```

### Filtro de Eventos

```typescript
// Hook com filtro
function useFilteredEvent<T>(
  eventName: string,
  filter: (payload: T) => boolean
): T | null {
  const [data, setData] = useState<T | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn;

    const setupListener = async () => {
      unlisten = await listen<T>(eventName, (event) => {
        if (filter(event.payload)) {
          setData(event.payload);
        }
      });
    };

    setupListener();
    return () => unlisten?.();
  }, [eventName, filter]);

  return data;
}

// Uso: apenas eventos do símbolo específico
const btcPrice = useFilteredEvent<PriceUpdateEvent>(
  'price_update',
  (p) => p.symbol === 'BTCUSDT'
);
```

## Emitindo Eventos (Backend)

```rust
use tauri::Manager;

// Emit para todas as janelas
fn emit_price_update(app: &AppHandle, update: PriceUpdate) {
    app.emit_all("price_update", update)
        .expect("Failed to emit price update");
}

// Emit para janela específica
fn emit_to_window(app: &AppHandle, window_label: &str, event: &str, payload: impl Serialize) {
    if let Some(window) = app.get_window(window_label) {
        window.emit(event, payload).ok();
    }
}

// Com logging de erros
fn emit_with_logging<T: Serialize + Clone>(
    app: &AppHandle,
    event: &str,
    payload: T,
) {
    if let Err(e) = app.emit_all(event, payload) {
        tracing::error!(event = event, error = %e, "Failed to emit event");
    }
}
```
