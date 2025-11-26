# Visão Geral da Arquitetura

## Introdução

RoboTrade é um robô de trading automatizado para criptomoedas, construído com uma arquitetura modular em Rust. O sistema foi projetado para ser seguro, extensível e de alta performance.

## Princípios Arquiteturais

### 1. Separação Total de Responsabilidades

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND                                 │
│                      React + TypeScript                          │
│                  (Apenas visualização e UX)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ IPC (Tauri Commands)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          TAURI                                   │
│                    (Bridge + Desktop App)                        │
│              Commands, Events, System Tray, Window               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Rust calls
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND RUST                                │
│                                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │   Core   │ │ Market   │ │Analytics │ │ Trading Worker   │   │
│  │          │ │  Data    │ │          │ │                  │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘   │
│                                                                  │
│  ┌──────────────────────┐ ┌────────────────────────────────┐   │
│  │  Exchange Gateways   │ │           Infra                │   │
│  │                      │ │  Config, DB, Logging           │   │
│  └──────────────────────┘ └────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ SQL
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         SQLite                                   │
│                   (Banco de dados local)                         │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Fluxo de Dependências

As dependências fluem sempre do núcleo para as bordas:

```
                    ┌─────────────┐
                    │    Core     │  ← Sem dependências internas
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
   ┌───────────┐    ┌───────────┐    ┌───────────┐
   │   Infra   │    │  Market   │    │ Analytics │
   │           │    │   Data    │    │           │
   └─────┬─────┘    └─────┬─────┘    └─────┬─────┘
         │                │                │
         │                ▼                │
         │         ┌───────────┐           │
         │         │ Exchange  │           │
         │         │ Gateways  │           │
         │         └─────┬─────┘           │
         │               │                 │
         └───────────────┼─────────────────┘
                         │
                         ▼
                  ┌────────────┐
                  │  Trading   │
                  │   Worker   │
                  └──────┬─────┘
                         │
                         ▼
                  ┌────────────┐
                  │ src-tauri  │
                  │  (App)     │
                  └────────────┘
```

### 3. Camadas do Sistema

#### Camada de Domínio (Core)

- Entidades puras sem side effects
- Traits que definem contratos
- Erros tipados
- DTOs para comunicação

#### Camada de Aplicação (Analytics, Trading Worker)

- Casos de uso e regras de negócio
- Orquestração de operações
- Transformação de dados

#### Camada de Infraestrutura (Infra, Exchange Gateways)

- Implementações concretas
- Acesso a banco de dados
- Comunicação com APIs externas
- Configuração e logging

#### Camada de Apresentação (Tauri + React)

- Comandos Tauri
- Interface gráfica
- System tray
- Notificações

## Diagrama de Componentes

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              RoboTrade                                      │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                         robotrade-core                               │  │
│  │                                                                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │  │
│  │  │ Entities │  │  Traits  │  │  Errors  │  │   DTOs   │            │  │
│  │  │          │  │          │  │          │  │          │            │  │
│  │  │ - Candle │  │-Exchange │  │-Market   │  │-Signal   │            │  │
│  │  │ - Order  │  │ Gateway  │  │ DataErr  │  │ Dto      │            │  │
│  │  │ - Signal │  │-Strategy │  │-Exchange │  │-Candle   │            │  │
│  │  │ - Trade  │  │-Provider │  │ Error    │  │ Dto      │            │  │
│  │  │ - etc... │  │-Reposit. │  │-Worker   │  │-etc...   │            │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│            ┌───────────────────────┼───────────────────────┐              │
│            │                       │                       │              │
│            ▼                       ▼                       ▼              │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐       │
│  │ robotrade-infra │    │robotrade-market │    │robotrade-       │       │
│  │                 │    │     -data       │    │  analytics      │       │
│  │ - Config        │    │                 │    │                 │       │
│  │ - Database      │    │ - Providers     │    │ - Indicators    │       │
│  │ - Logging       │    │   - Binance     │    │ - Strategies    │       │
│  │ - Repositories  │    │   - FearGreed   │    │ - Backtesting   │       │
│  │                 │    │   - etc...      │    │ - Signals       │       │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘       │
│            │                       │                       │              │
│            │                       ▼                       │              │
│            │            ┌─────────────────┐                │              │
│            │            │robotrade-       │                │              │
│            │            │exchange-gateways│                │              │
│            │            │                 │                │              │
│            │            │ - Binance       │                │              │
│            │            │ - Paper Trading │                │              │
│            │            │ - (Kraken)      │                │              │
│            │            └─────────────────┘                │              │
│            │                       │                       │              │
│            └───────────────────────┼───────────────────────┘              │
│                                    │                                       │
│                                    ▼                                       │
│                        ┌─────────────────┐                                │
│                        │robotrade-       │                                │
│                        │trading-worker   │                                │
│                        │                 │                                │
│                        │ - Job Queue     │                                │
│                        │ - Executor      │                                │
│                        │ - Scheduler     │                                │
│                        └─────────────────┘                                │
│                                    │                                       │
│                                    ▼                                       │
│                        ┌─────────────────┐                                │
│                        │  robotrade-app  │                                │
│                        │  (src-tauri)    │                                │
│                        │                 │                                │
│                        │ - Commands      │                                │
│                        │ - Events        │                                │
│                        │ - System Tray   │                                │
│                        └─────────────────┘                                │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## Responsabilidades dos Módulos

### Core (`robotrade-core`)

**Responsabilidade**: Definir o vocabulário do domínio e os contratos do sistema.

- Entidades: `Candle`, `Order`, `Position`, `Signal`, `Trade`, `FearGreedData`
- Traits: `ExchangeGateway`, `MarketDataProvider`, `Strategy`, `Repository`
- Erros: Tipos de erro por domínio com `thiserror`
- DTOs: Objetos de transferência para comunicação entre camadas

**Regra**: Este crate NÃO pode depender de nenhum outro crate interno.

### Infra (`robotrade-infra`)

**Responsabilidade**: Fornecer infraestrutura técnica para todos os módulos.

- Configuração via TOML
- Conexão e migrations do SQLite
- Sistema de logging estruturado
- Implementações de repositórios

### Market Data (`robotrade-market-data`)

**Responsabilidade**: Coletar e normalizar dados de mercado de fontes externas.

- Providers para diferentes fontes (Binance, Fear & Greed, etc.)
- Normalização de dados
- Rate limiting e retry
- Scheduler de coleta

### Analytics (`robotrade-analytics`)

**Responsabilidade**: Analisar dados e gerar insights para trading.

- Indicadores técnicos (RSI, EMA, MACD, etc.)
- Motor de estratégias
- Geração de sinais
- Motor de backtesting

### Exchange Gateways (`robotrade-exchange-gateways`)

**Responsabilidade**: Comunicar com exchanges de forma segura.

- Abstração via trait `ExchangeGateway`
- Implementações específicas (Binance Futures)
- Modo paper trading (simulação)
- Assinatura de requisições (HMAC)

### Trading Worker (`robotrade-trading-worker`)

**Responsabilidade**: Executar operações de trading de forma assíncrona.

- Fila de jobs com prioridade
- Execução de ordens
- Sincronização de estado
- Retry com backoff

### App Tauri (`robotrade-app`)

**Responsabilidade**: Integrar tudo e expor para o frontend.

- Comandos Tauri
- Gerenciamento de estado
- System tray
- Notificações nativas

## Fluxos Principais

### 1. Coleta de Dados de Mercado

```
Scheduler (Tokio)
       │
       ▼
MarketDataProvider.fetch_candles()
       │
       ▼
Normalização de dados
       │
       ▼
CandleRepository.save()
       │
       ▼
SQLite (tabela candles)
       │
       ▼
Event: "market-data-updated"
       │
       ▼
Frontend atualiza UI
```

### 2. Geração e Execução de Sinal

```
Strategy.evaluate()
       │
       ▼
Signal criado
       │
       ▼
SignalRepository.save()
       │
       ▼
TradingWorker.enqueue(PlaceOrderJob)
       │
       ▼
ExchangeGateway.submit_order()
       │
       ▼
Order criada
       │
       ▼
Notificação ao usuário
```

### 3. Backtest

```
BacktestEngine.run()
       │
       ▼
CandleRepository.find_candles_in_period()
       │
       ▼
Replay de candles
       │
       ├──► Strategy.evaluate()
       │          │
       │          ▼
       │    Simulação de ordem
       │          │
       │          ▼
       │    Cálculo de P&L
       │
       ▼
BacktestResult
       │
       ▼
Métricas (Sharpe, Drawdown, Win Rate)
```

## Comunicação Entre Módulos

### Via Traits (Inversão de Dependência)

```rust
// Core define o contrato
#[async_trait]
pub trait ExchangeGateway: Send + Sync {
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;
}

// Exchange Gateways implementa
pub struct BinanceFuturesClient { ... }

impl ExchangeGateway for BinanceFuturesClient {
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        // implementação específica
    }
}

// Trading Worker usa via trait
pub struct TradingWorker<G: ExchangeGateway> {
    gateway: Arc<G>,
}
```

### Via Eventos (Desacoplamento)

```rust
// Emissão de evento
app_handle.emit_all("signal-created", SignalDto::from(&signal))?;

// Frontend escuta
listen("signal-created", (event) => {
    updateSignalsList(event.payload);
});
```

## Considerações de Performance

### Async por Padrão

Todo I/O é assíncrono usando Tokio:
- Requisições HTTP
- Operações de banco de dados
- WebSocket connections
- Jobs do worker

### Pool de Conexões

SQLite usa pool de conexões para evitar contenção:
```rust
SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
```

### Batching

Operações de alto volume são batched:
- Inserção de candles
- Logs de eventos
- Métricas de telemetria

## Considerações de Segurança

Ver [security.md](./security.md) para detalhes completos.

Resumo:
- Modo paper é padrão
- API keys no keyring do sistema
- Validação server-side de todos os dados
- Rate limiting interno
- Limites de risco obrigatórios

## Extensibilidade

### Adicionar Nova Exchange

1. Criar módulo em `exchange_gateways/src/nova_exchange/`
2. Implementar trait `ExchangeGateway`
3. Registrar no factory de gateways
4. Adicionar configuração

### Adicionar Novo Indicador

1. Criar função em `analytics/src/indicators/`
2. Registrar no enum de indicadores
3. Adicionar testes unitários

### Adicionar Nova Estratégia

1. Implementar trait `Strategy`
2. Definir parâmetros configuráveis
3. Registrar no registry de estratégias
4. Criar testes de backtest

---

**Próximo**: [Detalhamento dos Módulos](./modules.md)
