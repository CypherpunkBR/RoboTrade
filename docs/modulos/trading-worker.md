# Módulo Trading Worker

O crate `robotrade-trading-worker` fornece workers para orquestração de tarefas de trading, incluindo coleta de dados, execução de estratégias e monitoramento de posições.

## Estrutura de Diretórios

```
crates/trading_worker/src/
├── lib.rs              # Exportações públicas
├── scheduler.rs        # Scheduler de eventos
└── collector.rs        # Coletor de dados de mercado
```

## Scheduler

O Scheduler é um sistema event-driven que dispara eventos em intervalos configuráveis.

### Eventos

```rust
/// Eventos emitidos pelo scheduler
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// Coleta dados Fear & Greed
    CollectFearGreed,

    /// Atualiza preços atuais
    UpdatePrices,

    /// Verifica posições abertas
    CheckPositions,

    /// Executa estratégias
    RunStrategies,

    /// Scheduler iniciado
    Started,

    /// Scheduler parado
    Stopped,
}
```

### Configuração

```rust
use robotrade_trading_worker::scheduler::{Scheduler, SchedulerConfig};
use std::time::Duration;

let config = SchedulerConfig {
    fear_greed_interval: Duration::from_secs(3600),  // 1 hora
    prices_interval: Duration::from_secs(5),          // 5 segundos
    positions_interval: Duration::from_secs(10),      // 10 segundos
    strategies_interval: Duration::from_secs(60),     // 1 minuto
};

let scheduler = Scheduler::with_config(config);
```

### Intervalos Padrão

| Evento | Intervalo | Descrição |
|--------|-----------|-----------|
| Fear & Greed | 1 hora | Atualização do índice |
| Preços | 5 segundos | Ticker de preços |
| Posições | 10 segundos | Status de posições abertas |
| Estratégias | 1 minuto | Execução de estratégias |

### Uso

```rust
use robotrade_trading_worker::scheduler::Scheduler;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = Scheduler::new();

    // Subscrever a eventos
    let mut receiver = scheduler.subscribe();

    // Iniciar scheduler
    scheduler.start().await;

    // Processar eventos
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                SchedulerEvent::CollectFearGreed => {
                    println!("Coletando Fear & Greed...");
                    // Chamar provider
                }
                SchedulerEvent::UpdatePrices => {
                    println!("Atualizando preços...");
                    // Chamar exchange
                }
                SchedulerEvent::CheckPositions => {
                    println!("Verificando posições...");
                    // Chamar exchange
                }
                SchedulerEvent::RunStrategies => {
                    println!("Executando estratégias...");
                    // Executar estratégias
                }
                SchedulerEvent::Started => {
                    println!("Scheduler iniciado!");
                }
                SchedulerEvent::Stopped => {
                    println!("Scheduler parado!");
                    break;
                }
            }
        }
    });

    // Aguardar um tempo
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Parar scheduler
    scheduler.stop().await;

    Ok(())
}
```

### Métodos

```rust
impl Scheduler {
    /// Cria scheduler com configuração padrão
    pub fn new() -> Self;

    /// Cria scheduler com configuração customizada
    pub fn with_config(config: SchedulerConfig) -> Self;

    /// Inicia o scheduler
    pub async fn start(&self);

    /// Para o scheduler
    pub async fn stop(&self);

    /// Subscreve a eventos
    pub fn subscribe(&self) -> broadcast::Receiver<SchedulerEvent>;

    /// Verifica se está rodando
    pub fn is_running(&self) -> bool;
}
```

## Data Collector

O DataCollector coleta dados de mercado periodicamente.

### Funcionalidades

- Coleta de candles históricos
- Atualização de Fear & Greed Index
- Cache local de dados
- Persistência em banco de dados

### Exemplo

```rust
use robotrade_trading_worker::collector::DataCollector;
use robotrade_market_data::providers::AlternativeMeFearGreedProvider;
use robotrade_infra::database::SqliteFearGreedRepository;

let fear_greed_provider = AlternativeMeFearGreedProvider::new();
let fear_greed_repo = SqliteFearGreedRepository::new(pool.clone());

let collector = DataCollector::new(
    Box::new(fear_greed_provider),
    Box::new(fear_greed_repo),
);

// Coletar e salvar dados
collector.collect_fear_greed().await?;
```

## Arquitetura Event-Driven

```
┌─────────────────────────────────────────────────────────┐
│                      Scheduler                          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │  Timer  │  │  Timer  │  │  Timer  │  │  Timer  │    │
│  │  F&G    │  │ Prices  │  │  Pos    │  │ Strat   │    │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘    │
│       │            │            │            │          │
│       └────────────┴────────────┴────────────┘          │
│                         │                               │
│                         ▼                               │
│              ┌──────────────────┐                       │
│              │ Broadcast Channel│                       │
│              └────────┬─────────┘                       │
└───────────────────────┼─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ Data Collector│ │Strategy Runner│ │Position Monitor│
│               │ │               │ │               │
│ - Fear&Greed  │ │ - Evaluate    │ │ - Check SL/TP │
│ - Candles     │ │ - Generate    │ │ - Update P&L  │
│ - Persist     │ │   Signals     │ │ - Close if    │
│               │ │               │ │   needed      │
└───────────────┘ └───────────────┘ └───────────────┘
```

## Workers Planejados

### Position Manager

```rust
/// Gerenciador de posições
pub struct PositionManager {
    exchange: Box<dyn ExchangeGateway>,
    config: PositionManagerConfig,
}

impl PositionManager {
    /// Abre posição baseada em sinal
    async fn open_position(&self, signal: &Signal) -> TradingResult<Position>;

    /// Fecha posição
    async fn close_position(&self, position: &Position) -> TradingResult<Trade>;

    /// Atualiza SL/TP
    async fn update_stops(&self, position: &Position, sl: Decimal, tp: Decimal);

    /// Move SL para break-even
    async fn move_to_breakeven(&self, position: &Position);
}
```

### Risk Manager

```rust
/// Gerenciador de risco
pub struct RiskManager {
    config: RiskConfig,
}

impl RiskManager {
    /// Verifica se pode abrir posição
    fn can_open_position(&self, context: &RiskContext) -> bool;

    /// Calcula tamanho da posição
    fn calculate_position_size(&self, context: &RiskContext) -> Decimal;

    /// Verifica limites diários
    fn check_daily_limits(&self, stats: &DailyStats) -> bool;
}
```

### Circuit Breaker

```rust
/// Circuit breaker para proteção
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
}

impl CircuitBreaker {
    /// Verifica se trading está permitido
    fn is_trading_allowed(&self) -> bool;

    /// Registra resultado de trade
    fn record_trade_result(&mut self, won: bool);

    /// Registra drawdown
    fn record_drawdown(&mut self, pct: Decimal);
}

pub struct CircuitBreakerConfig {
    /// Pausar após N perdas consecutivas
    pub max_consecutive_losses: u32,

    /// Pausar se drawdown exceder X%
    pub max_drawdown_pct: Decimal,

    /// Tempo de pausa
    pub pause_duration: Duration,
}
```

## Testes

```rust
#[tokio::test]
async fn test_scheduler_start_stop() {
    let scheduler = Scheduler::new();
    let mut receiver = scheduler.subscribe();

    scheduler.start().await;
    assert!(scheduler.is_running());

    // Deve receber evento Started
    let event = receiver.recv().await.unwrap();
    assert!(matches!(event, SchedulerEvent::Started));

    scheduler.stop().await;
    assert!(!scheduler.is_running());
}

#[tokio::test]
async fn test_scheduler_emits_events() {
    let config = SchedulerConfig {
        fear_greed_interval: Duration::from_millis(50),
        prices_interval: Duration::from_millis(50),
        positions_interval: Duration::from_millis(50),
        strategies_interval: Duration::from_millis(50),
    };

    let scheduler = Scheduler::with_config(config);
    let mut receiver = scheduler.subscribe();

    scheduler.start().await;

    // Aguarda alguns eventos
    tokio::time::sleep(Duration::from_millis(100)).await;

    scheduler.stop().await;

    // Deve ter recebido múltiplos eventos
    let mut event_count = 0;
    while let Ok(_) = receiver.try_recv() {
        event_count += 1;
    }

    assert!(event_count > 2);
}
```
