# Módulo Tauri App

O crate `robotrade-app` (src-tauri) é a aplicação desktop construída com Tauri, fornecendo interface nativa com system tray e comandos IPC.

## Estrutura de Diretórios

```
src-tauri/
├── Cargo.toml          # Dependências do crate
├── tauri.conf.json     # Configuração do Tauri
├── build.rs            # Script de build
├── icons/              # Ícones da aplicação
│   ├── 32x32.png
│   ├── 128x128.png
│   ├── 256x256.png
│   ├── 512x512.png
│   └── icon.png
└── src/
    ├── main.rs         # Entry point (Windows)
    ├── lib.rs          # Entry point principal
    ├── commands.rs     # Comandos Tauri
    ├── state.rs        # Estado da aplicação
    └── tray.rs         # System tray
```

## Comandos Tauri

Os comandos Tauri são funções Rust que podem ser chamadas pelo frontend JavaScript.

### Dashboard

```rust
/// Obtém resumo do dashboard
#[tauri::command]
async fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String>;

/// Obtém status de conexão
#[tauri::command]
async fn get_connection_status(state: State<'_, AppState>) -> Result<ServiceStatus, String>;
```

### Fear & Greed

```rust
/// Obtém valor atual do índice
#[tauri::command]
async fn get_fear_greed_current(state: State<'_, AppState>) -> Result<Option<FearGreedData>, String>;

/// Obtém histórico do índice
#[tauri::command]
async fn get_fear_greed_history(
    state: State<'_, AppState>,
    days: u32,
) -> Result<Vec<FearGreedData>, String>;
```

### Trading

```rust
/// Obtém posições abertas
#[tauri::command]
async fn get_positions(state: State<'_, AppState>) -> Result<Vec<PositionDto>, String>;

/// Obtém ordens abertas
#[tauri::command]
async fn get_open_orders(state: State<'_, AppState>) -> Result<Vec<Order>, String>;

/// Obtém saldos da conta
#[tauri::command]
async fn get_balances(state: State<'_, AppState>) -> Result<HashMap<String, Decimal>, String>;

/// Cria uma nova ordem
#[tauri::command]
async fn create_order(
    state: State<'_, AppState>,
    request: OrderRequest,
) -> Result<Order, String>;

/// Cancela uma ordem
#[tauri::command]
async fn cancel_order(
    state: State<'_, AppState>,
    order_id: String,
) -> Result<bool, String>;
```

### Configuração

```rust
/// Obtém configuração atual
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String>;

/// Salva configuração
#[tauri::command]
async fn save_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String>;

/// Obtém modo de trading
#[tauri::command]
async fn get_trading_mode(state: State<'_, AppState>) -> Result<TradingMode, String>;

/// Define modo de trading
#[tauri::command]
async fn set_trading_mode(
    state: State<'_, AppState>,
    mode: TradingMode,
) -> Result<(), String>;
```

### Histórico

```rust
/// Obtém histórico de trades completos (posições fechadas)
#[tauri::command]
async fn get_trade_history(
    state: State<'_, AppState>,
    symbol: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<TradeDto>, String>;

/// Obtém estatísticas de trades
#[tauri::command]
async fn get_trade_stats(state: State<'_, AppState>) -> Result<TradeStatsDto, String>;

/// Obtém histórico de fills/execuções das exchanges
/// Busca de 10 pares principais: BTC, ETH, BNB, SOL, XRP, DOGE, ADA, AVAX, LINK, DOT
#[tauri::command]
async fn get_fill_history(
    state: State<'_, AppState>,
    symbol: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<FillDto>, String>;
```

## Estado da Aplicação

```rust
/// Estado compartilhado da aplicação
pub struct AppState {
    /// Modo de trading (Paper/Live)
    pub mode: RwLock<TradingMode>,

    /// Status de conexão
    pub connection_status: RwLock<ServiceStatus>,

    /// Posições abertas (cache)
    pub positions: RwLock<Vec<Position>>,

    /// P&L do dia
    pub daily_pnl: RwLock<Decimal>,

    /// Configuração
    pub config: RwLock<AppConfig>,

    /// Pool do banco de dados
    pub db_pool: SqlitePool,

    /// Cliente da exchange
    pub exchange_client: RwLock<Option<BinanceFuturesClient>>,

    /// Provider Fear & Greed
    pub fear_greed_provider: AlternativeMeFearGreedProvider,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig::load().await?;
        let db_pool = init_database().await?;

        Ok(Self {
            mode: RwLock::new(config.trading.mode.clone()),
            connection_status: RwLock::new(ServiceStatus::Disconnected),
            positions: RwLock::new(Vec::new()),
            daily_pnl: RwLock::new(Decimal::ZERO),
            config: RwLock::new(config),
            db_pool,
            exchange_client: RwLock::new(None),
            fear_greed_provider: AlternativeMeFearGreedProvider::new(),
        })
    }
}
```

## System Tray

O system tray fornece acesso rápido às funcionalidades principais.

### Menu

```rust
pub fn create_tray_menu() -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Mostrar"))
        .add_item(CustomMenuItem::new("hide", "Esconder"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("status", "Status: Desconectado"))
        .add_item(CustomMenuItem::new("pnl", "P&L: $0.00"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Sair"));

    SystemTray::new().with_menu(menu)
}
```

### Eventos

```rust
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            // Mostra/esconde janela
            toggle_window(app);
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => show_window(app),
                "hide" => hide_window(app),
                "quit" => app.exit(0),
                _ => {}
            }
        }
        _ => {}
    }
}
```

## Configuração Tauri

### tauri.conf.json

```json
{
  "productName": "RoboTrade",
  "identifier": "br.cypherpunk.robotrade",
  "version": "0.1.0",
  "build": {
    "beforeBuildCommand": "",
    "beforeDevCommand": "",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "RoboTrade",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "plugins": {
    "shell": {
      "open": true
    },
    "notification": {
      "all": true
    }
  }
}
```

### Permissões

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "notification:default"
  ]
}
```

## Uso no Frontend

### TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/core';

// Obter resumo do dashboard
const summary = await invoke('get_dashboard_summary');

// Obter Fear & Greed atual
const fearGreed = await invoke('get_fear_greed_current');

// Criar ordem
const order = await invoke('create_order', {
  request: {
    symbol: 'BTCUSDT',
    side: 'Buy',
    orderType: 'Market',
    quantity: '0.001',
  }
});

// Obter posições
const positions = await invoke('get_positions');

// Alterar modo de trading
await invoke('set_trading_mode', { mode: 'Paper' });
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface DashboardSummary {
  tradingMode: 'Paper' | 'Live';
  connectionStatus: string;
  activePositions: number;
  dailyPnl: string;
  dailyPnlPct: string;
  totalBalance: string;
  fearGreedCurrent: FearGreedData | null;
}

export function useDashboard() {
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchDashboard = async () => {
      try {
        const data = await invoke<DashboardSummary>('get_dashboard_summary');
        setSummary(data);
        setError(null);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };

    fetchDashboard();
    const interval = setInterval(fetchDashboard, 5000);
    return () => clearInterval(interval);
  }, []);

  return { summary, loading, error };
}
```

## Notificações

```rust
use tauri::api::notification::Notification;

// Enviar notificação nativa
Notification::new(&app.config().tauri.bundle.identifier)
    .title("RoboTrade")
    .body("Ordem executada: BTCUSDT Long 0.001 BTC")
    .show()?;
```

## Build e Desenvolvimento

```bash
# Desenvolvimento
cargo tauri dev

# Build de produção
cargo tauri build

# Build para plataforma específica
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target aarch64-apple-darwin
cargo tauri build --target x86_64-pc-windows-msvc
```
