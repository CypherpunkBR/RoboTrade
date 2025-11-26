# Error Codes

Referência de códigos de erro e tratamento no RoboTrade.

## Visão Geral

Erros são retornados como strings formatadas que podem ser parseadas no frontend:

```typescript
try {
  await invoke('place_order', { request });
} catch (error) {
  // error é uma string do backend
  const parsedError = parseError(error as string);
  handleError(parsedError);
}
```

## Estrutura de Erros

### Backend (Rust)

```rust
#[derive(Debug, thiserror::Error)]
pub enum RoboTradeError {
    // Market Data Errors (1xxx)
    #[error("MARKET_1001: Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("MARKET_1002: Invalid timeframe: {0}")]
    InvalidTimeframe(String),

    #[error("MARKET_1003: Data fetch failed: {0}")]
    DataFetchFailed(String),

    // Order Errors (2xxx)
    #[error("ORDER_2001: Insufficient balance for {symbol}: required {required}, available {available}")]
    InsufficientBalance {
        symbol: String,
        required: String,
        available: String,
    },

    #[error("ORDER_2002: Order rejected by exchange: {0}")]
    OrderRejected(String),

    // ... etc
}
```

### Frontend (TypeScript)

```typescript
interface ParsedError {
  code: string;
  category: ErrorCategory;
  message: string;
  details?: Record<string, unknown>;
}

type ErrorCategory =
  | 'Market'
  | 'Order'
  | 'Position'
  | 'Strategy'
  | 'Config'
  | 'Auth'
  | 'System';

function parseError(errorString: string): ParsedError {
  const match = errorString.match(/^([A-Z]+)_(\d+): (.+)$/);

  if (!match) {
    return {
      code: 'UNKNOWN',
      category: 'System',
      message: errorString,
    };
  }

  const [, categoryCode, numericCode, message] = match;
  const code = `${categoryCode}_${numericCode}`;

  return {
    code,
    category: categoryCode as ErrorCategory,
    message,
  };
}
```

## Categorias de Erro

### Market Data Errors (1xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `MARKET_1001` | Símbolo não encontrado | Verificar se símbolo existe |
| `MARKET_1002` | Timeframe inválido | Usar timeframe suportado |
| `MARKET_1003` | Falha ao buscar dados | Retry ou verificar conexão |
| `MARKET_1004` | Rate limit excedido | Aguardar e retry |
| `MARKET_1005` | Dados insuficientes | Ampliar período de busca |
| `MARKET_1006` | Provider indisponível | Usar provider alternativo |

**Exemplo de tratamento:**

```typescript
async function fetchCandles(symbol: string, timeframe: string) {
  try {
    return await invoke('get_candles', { symbol, timeframe, limit: 100 });
  } catch (error) {
    const parsed = parseError(error as string);

    switch (parsed.code) {
      case 'MARKET_1001':
        throw new Error(`Símbolo "${symbol}" não existe`);

      case 'MARKET_1004':
        // Rate limit - aguardar e retry
        await sleep(5000);
        return fetchCandles(symbol, timeframe);

      default:
        throw new Error(parsed.message);
    }
  }
}
```

---

### Order Errors (2xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `ORDER_2001` | Saldo insuficiente | Reduzir quantidade ou depositar |
| `ORDER_2002` | Ordem rejeitada pela exchange | Verificar parâmetros |
| `ORDER_2003` | Quantidade abaixo do mínimo | Aumentar quantidade |
| `ORDER_2004` | Quantidade acima do máximo | Reduzir quantidade |
| `ORDER_2005` | Preço fora do range | Ajustar preço |
| `ORDER_2006` | Valor notional muito baixo | Aumentar valor da ordem |
| `ORDER_2007` | Ordem não encontrada | Verificar ID |
| `ORDER_2008` | Ordem já cancelada | Nenhuma ação necessária |
| `ORDER_2009` | Ordem já executada | Nenhuma ação necessária |
| `ORDER_2010` | Modo live requer confirmação | Enviar token de confirmação |
| `ORDER_2011` | Limite de ordens atingido | Aguardar ou cancelar ordens |
| `ORDER_2012` | Símbolo em manutenção | Aguardar |

**Validação no Frontend:**

```typescript
interface OrderValidation {
  valid: boolean;
  errors: ValidationError[];
}

interface ValidationError {
  field: string;
  code: string;
  message: string;
}

function validateOrderRequest(request: OrderRequest, symbolInfo: SymbolInfo): OrderValidation {
  const errors: ValidationError[] = [];

  // Validar quantidade mínima
  const quantity = new Decimal(request.quantity);
  const minQty = new Decimal(symbolInfo.minQuantity);

  if (quantity.lt(minQty)) {
    errors.push({
      field: 'quantity',
      code: 'ORDER_2003',
      message: `Quantidade mínima é ${symbolInfo.minQuantity}`,
    });
  }

  // Validar valor notional
  if (request.orderType.type === 'Limit') {
    const price = new Decimal(request.orderType.price);
    const notional = quantity.times(price);
    const minNotional = new Decimal(symbolInfo.minNotional);

    if (notional.lt(minNotional)) {
      errors.push({
        field: 'quantity',
        code: 'ORDER_2006',
        message: `Valor mínimo da ordem é ${symbolInfo.minNotional} USDT`,
      });
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
```

---

### Position Errors (3xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `POSITION_3001` | Posição não encontrada | Verificar ID |
| `POSITION_3002` | Posição já fechada | Nenhuma ação |
| `POSITION_3003` | Quantidade excede posição | Reduzir quantidade |
| `POSITION_3004` | Stop loss inválido | Ajustar preço do stop |
| `POSITION_3005` | Take profit inválido | Ajustar preço do TP |
| `POSITION_3006` | Máximo de posições atingido | Fechar posições existentes |

---

### Strategy Errors (4xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `STRATEGY_4001` | Estratégia não encontrada | Verificar ID |
| `STRATEGY_4002` | Configuração inválida | Corrigir configuração |
| `STRATEGY_4003` | Indicador não suportado | Usar indicador válido |
| `STRATEGY_4004` | Regra inválida | Corrigir regras |
| `STRATEGY_4005` | Versão não encontrada | Verificar ID da versão |
| `STRATEGY_4006` | Estratégia já ativa | Nenhuma ação |
| `STRATEGY_4007` | Estratégia já inativa | Nenhuma ação |
| `STRATEGY_4008` | Erro de execução | Ver logs para detalhes |

---

### Configuration Errors (5xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `CONFIG_5001` | Arquivo de config não encontrado | Criar config padrão |
| `CONFIG_5002` | Config inválida | Corrigir formato |
| `CONFIG_5003` | Valor fora do range | Ajustar valor |
| `CONFIG_5004` | Campo obrigatório ausente | Preencher campo |
| `CONFIG_5005` | Tipo inválido | Corrigir tipo |

---

### Authentication Errors (6xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `AUTH_6001` | API key inválida | Verificar credenciais |
| `AUTH_6002` | Secret key inválida | Verificar credenciais |
| `AUTH_6003` | Credenciais expiradas | Renovar API key |
| `AUTH_6004` | Permissões insuficientes | Habilitar permissões na exchange |
| `AUTH_6005` | IP não autorizado | Adicionar IP ao whitelist |
| `AUTH_6006` | Assinatura inválida | Verificar secret key |

---

### System Errors (9xxx)

| Código | Descrição | Ação Recomendada |
|--------|-----------|------------------|
| `SYSTEM_9001` | Erro de banco de dados | Verificar integridade do DB |
| `SYSTEM_9002` | Erro de conexão | Verificar internet |
| `SYSTEM_9003` | Timeout | Retry |
| `SYSTEM_9004` | Erro interno | Reportar bug |
| `SYSTEM_9005` | Serviço indisponível | Aguardar |
| `SYSTEM_9006` | Versão incompatível | Atualizar aplicação |
| `SYSTEM_9007` | Recurso não encontrado | Verificar caminho |
| `SYSTEM_9008` | Permissão negada | Verificar permissões do SO |

---

## Tratamento Global de Erros

### React Error Boundary

```typescript
import React from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: ParsedError;
}

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  ErrorBoundaryState
> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error: parseError(error.message),
    };
  }

  render() {
    if (this.state.hasError) {
      return <ErrorDisplay error={this.state.error!} />;
    }

    return this.props.children;
  }
}
```

### Hook de Erro Global

```typescript
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

function useGlobalErrorHandler() {
  useEffect(() => {
    const unlisten = listen<SystemAlertEvent>('system_alert', (event) => {
      const { level, title, message } = event.payload;

      if (level === 'Error' || level === 'Critical') {
        showErrorNotification(title, message);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);
}
```

### Retry Logic

```typescript
async function withRetry<T>(
  fn: () => Promise<T>,
  options: {
    maxRetries?: number;
    retryableErrors?: string[];
    backoffMs?: number;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    retryableErrors = ['MARKET_1004', 'SYSTEM_9003'],
    backoffMs = 1000,
  } = options;

  let lastError: unknown;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      const parsed = parseError(error as string);

      if (!retryableErrors.includes(parsed.code)) {
        throw error;
      }

      if (attempt < maxRetries - 1) {
        await sleep(backoffMs * Math.pow(2, attempt));
      }
    }
  }

  throw lastError;
}

// Uso
const candles = await withRetry(() =>
  invoke('get_candles', { symbol, timeframe, limit })
);
```

## Logging de Erros

### Frontend

```typescript
function logError(error: ParsedError, context: Record<string, unknown> = {}) {
  console.error('[RoboTrade Error]', {
    code: error.code,
    category: error.category,
    message: error.message,
    ...context,
    timestamp: new Date().toISOString(),
  });

  // Opcional: enviar para serviço de monitoramento
  // errorReporter.capture(error);
}
```

### Backend

```rust
fn handle_error(error: RoboTradeError, context: &str) {
    tracing::error!(
        error_code = %error.code(),
        error_message = %error,
        context = %context,
        "Operation failed"
    );
}
```

## Mensagens Amigáveis

```typescript
const errorMessages: Record<string, string> = {
  'MARKET_1001': 'O par de trading não foi encontrado. Verifique se o símbolo está correto.',
  'ORDER_2001': 'Saldo insuficiente para realizar esta operação.',
  'ORDER_2003': 'A quantidade é muito pequena. Aumente a quantidade e tente novamente.',
  'AUTH_6001': 'Credenciais inválidas. Por favor, verifique sua API key.',
  'SYSTEM_9002': 'Não foi possível conectar ao servidor. Verifique sua conexão com a internet.',
  // ... etc
};

function getErrorMessage(code: string, fallback: string): string {
  return errorMessages[code] || fallback;
}
```
