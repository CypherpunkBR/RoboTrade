# Arquitetura do RoboTrade

## Visão Geral

O RoboTrade é um robô de trading automatizado construído em Rust, seguindo uma arquitetura modular baseada em workspace Cargo. O sistema é composto por 7 crates interconectados que seguem os princípios de separação de responsabilidades e inversão de dependências.

## Diagrama de Camadas

```
┌─────────────────────────────────────────────────────────────┐
│                    Aplicação Tauri                          │
│                   (src-tauri/robotrade-app)                 │
├─────────────────────────────────────────────────────────────┤
│                    Camada de Serviços                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Analytics  │  │Trading Worker│  │ Exchange Gateways│   │
│  │  (backtest)  │  │ (scheduler)  │  │   (binance)      │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Camada de Dados                          │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │   Market Data    │  │         Infra                │     │
│  │ (fear & greed)   │  │ (config, db, logging)        │     │
│  └──────────────────┘  └──────────────────────────────┘     │
├─────────────────────────────────────────────────────────────┤
│                    Camada Core                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    Core                              │    │
│  │  (entities, traits, errors, dtos)                   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Fluxo de Dependências

```
robotrade-app (Tauri)
    ├── robotrade-analytics
    │       └── robotrade-core
    ├── robotrade-trading-worker
    │       └── robotrade-core
    ├── robotrade-exchange-gateways
    │       └── robotrade-core
    ├── robotrade-market-data
    │       └── robotrade-core
    └── robotrade-infra
            └── robotrade-core
```

## Princípios de Design

### 1. Inversão de Dependências
O crate `core` define traits (interfaces) que são implementados pelos crates de camadas superiores:

```rust
// core define o trait
pub trait ExchangeGateway: Send + Sync {
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;
}

// exchange_gateways implementa
impl ExchangeGateway for BinanceFuturesClient { ... }
```

### 2. Separação de Responsabilidades
Cada crate tem uma responsabilidade única e bem definida:
- **core**: Tipos e contratos
- **infra**: Persistência e configuração
- **market_data**: Coleta de dados externos
- **exchange_gateways**: Comunicação com exchanges
- **analytics**: Análise e backtest
- **trading_worker**: Orquestração de tarefas

### 3. Imutabilidade e Segurança
- Entidades são imutáveis por padrão
- IDs tipados (newtype pattern) para segurança em tempo de compilação
- Result types específicos para cada domínio de erro

## Comunicação Entre Camadas

### Padrão de Eventos
O `Scheduler` no `trading_worker` emite eventos que são consumidos por outros componentes:

```rust
pub enum SchedulerEvent {
    CollectFearGreed,
    UpdatePrices,
    CheckPositions,
    RunStrategies,
    Started,
    Stopped,
}
```

### Padrão Repository
Acesso a dados é abstraído via traits de repositório:

```rust
pub trait CandleRepository: Send + Sync {
    async fn save(&self, candle: &Candle) -> CoreResult<()>;
    async fn get_range(&self, ...) -> CoreResult<Vec<Candle>>;
}
```

## Tecnologias Utilizadas

| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| Runtime | Tokio | Async/await de alta performance |
| Database | SQLite/SQLx | Banco local sem servidor |
| HTTP | Reqwest | Cliente HTTP async com TLS |
| Decimal | rust_decimal | Precisão financeira sem float |
| Desktop | Tauri | App nativo leve |
| Logging | Tracing | Logging estruturado assíncrono |

## Próximos Passos Arquiteturais

1. **Event Sourcing**: Persistir todos os eventos para auditoria
2. **CQRS**: Separar comandos de queries para otimização
3. **Plugin System**: Permitir estratégias como plugins dinâmicos
4. **WebSocket Layer**: Camada unificada para dados em tempo real
