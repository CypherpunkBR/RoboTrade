# API Documentation

Documentação da API do RoboTrade, incluindo comandos Tauri, DTOs e eventos.

## Índice

| Documento | Descrição |
|-----------|-----------|
| [Tauri Commands](./tauri-commands.md) | Comandos IPC expostos ao frontend |
| [DTOs](./dtos.md) | Data Transfer Objects |
| [Events](./events.md) | Sistema de eventos assíncronos |
| [Error Codes](./error-codes.md) | Códigos de erro e tratamento |

## Visão Geral

O RoboTrade usa **Tauri 2.0** para comunicação entre o backend Rust e o frontend React. A comunicação ocorre via:

1. **Commands**: Chamadas síncronas/assíncronas do frontend para o backend
2. **Events**: Notificações do backend para o frontend

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                        │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   invoke()  │  │   listen()  │  │  useQuery/Mutation  │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
└─────────┼────────────────┼─────────────────────┼────────────┘
          │                │                     │
          │   IPC Bridge   │                     │
          │                │                     │
┌─────────┼────────────────┼─────────────────────┼────────────┐
│         ▼                ▼                     ▼            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Commands   │  │   Events    │  │    State Manager    │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
│         └────────────────┴─────────────────────┘            │
│                          │                                  │
│                          ▼                                  │
│                   ┌─────────────┐                          │
│                   │   Services  │                          │
│                   └─────────────┘                          │
│                                                             │
│                     Backend (Rust)                          │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Chamando um Command (Frontend)

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Buscar candlesticks
const candles = await invoke<Candlestick[]>('get_candles', {
  symbol: 'BTCUSDT',
  timeframe: '1h',
  limit: 100,
});

// Colocar ordem
const order = await invoke<OrderResponse>('place_order', {
  request: {
    symbol: 'BTCUSDT',
    side: 'Buy',
    orderType: { type: 'Market' },
    quantity: '0.001',
  },
});
```

### Ouvindo Eventos (Frontend)

```typescript
import { listen } from '@tauri-apps/api/event';

// Escutar atualizações de preço
const unlisten = await listen<PriceUpdate>('price_update', (event) => {
  console.log(`${event.payload.symbol}: ${event.payload.price}`);
});

// Limpar listener
unlisten();
```

### Definindo um Command (Backend)

```rust
#[tauri::command]
pub async fn get_candles(
    state: State<'_, AppState>,
    symbol: String,
    timeframe: String,
    limit: u32,
) -> Result<Vec<Candlestick>, String> {
    state.market_data
        .get_candles(&symbol, &timeframe, limit)
        .await
        .map_err(|e| e.to_string())
}
```

### Emitindo Eventos (Backend)

```rust
use tauri::Manager;

fn emit_price_update(app: &AppHandle, symbol: &str, price: Decimal) {
    app.emit_all("price_update", PriceUpdate {
        symbol: symbol.to_string(),
        price: price.to_string(),
        timestamp: Utc::now(),
    }).ok();
}
```

## Type Safety com specta

Usamos **specta** para gerar tipos TypeScript automaticamente:

```rust
// Backend
use specta::Type;

#[derive(Serialize, Type)]
pub struct Candlestick {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

// Gera bindings TypeScript em build time
#[tauri::command]
#[specta::specta]
pub async fn get_candles(...) -> Result<Vec<Candlestick>, String> { ... }
```

```typescript
// Frontend - tipos gerados automaticamente
import type { Candlestick } from '../bindings';
```

## Convenções

### Nomenclatura

- **Commands**: `snake_case` (Rust) → `snake_case` (TypeScript invoke)
- **DTOs**: `PascalCase` em ambos
- **Events**: `snake_case` (ex: `price_update`, `order_filled`)

### Erro Handling

```typescript
try {
  const result = await invoke('risky_operation');
} catch (error) {
  // error é string do Rust
  console.error('Operation failed:', error);
}
```

### Async vs Sync

- Prefira commands `async` para operações I/O
- Use commands sync apenas para operações triviais em memória

## Seções da Documentação

1. **[Tauri Commands](./tauri-commands.md)**: Lista completa de comandos disponíveis
2. **[DTOs](./dtos.md)**: Estruturas de dados trocadas entre frontend/backend
3. **[Events](./events.md)**: Eventos emitidos pelo backend
4. **[Error Codes](./error-codes.md)**: Tipos de erro e como tratá-los
