# ADR-003: Design do Sistema de Filas e Workers

## Status

Aceita

## Contexto

O RoboTrade precisa executar diversas tarefas em background:

1. **Coleta de dados**: Fetch periódico de candlesticks e indicadores externos
2. **Análise**: Cálculo de indicadores técnicos quando novos dados chegam
3. **Execução de estratégias**: Avaliação de sinais e geração de ordens
4. **Manutenção**: Limpeza de dados antigos, vacuum do banco, health checks
5. **Backtesting**: Simulações longas que não devem bloquear a UI

Requisitos:

- Priorização de tarefas (ordens > dados > manutenção)
- Resiliência a falhas (retry com backoff)
- Cancelamento de tarefas em andamento
- Visibilidade do estado das tarefas na UI
- Não perder tarefas em caso de crash

## Decisão

Implementamos um **sistema de filas em memória com persistência** usando canais Tokio e SQLite como backing store.

### Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                        Job Scheduler                            │
│  (cron-like triggers, event-based triggers)                    │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Job Queue                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Critical   │  │    High     │  │   Normal    │  │   Low    │
│  │  Priority   │  │  Priority   │  │  Priority   │  │ Priority │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────┬─────┘
│         │                │                │               │      │
│         └────────────────┴────────────────┴───────────────┘      │
│                               │                                  │
└───────────────────────────────┼──────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Worker Pool                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Worker 1 │  │ Worker 2 │  │ Worker 3 │  │ Worker 4 │        │
│  │ (trade)  │  │ (data)   │  │ (general)│  │ (general)│        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

### Modelo de Dados

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    // Market Data
    FetchCandles { symbol: String, timeframe: String },
    FetchFearGreedIndex,

    // Trading
    ExecuteStrategy { strategy_id: Uuid },
    PlaceOrder { order: OrderRequest },
    CancelOrder { order_id: String },

    // Maintenance
    CleanupOldData { days_to_keep: u32 },
    VacuumDatabase,
    HealthCheck,

    // Analytics
    RunBacktest { config: BacktestConfig },
    CalculateMetrics { period: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Critical = 0,  // Ordens, cancelamentos
    High = 1,      // Execução de estratégias
    Normal = 2,    // Coleta de dados
    Low = 3,       // Manutenção, backtests
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
    pub result: Option<serde_json::Value>,
}
```

### Implementação da Fila

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug)]
struct PrioritizedJob {
    job: Job,
    sequence: u64,  // Para FIFO dentro da mesma prioridade
}

impl Ord for PrioritizedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Menor prioridade numérica = maior prioridade real
        other.job.priority.cmp(&self.job.priority)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

pub struct JobQueue {
    heap: RwLock<BinaryHeap<PrioritizedJob>>,
    sequence: AtomicU64,
    notify: tokio::sync::Notify,
    db: SqlitePool,
}

impl JobQueue {
    pub async fn enqueue(&self, job: Job) -> Result<Uuid, Error> {
        // Persistir no banco primeiro (durabilidade)
        self.persist_job(&job).await?;

        // Adicionar à heap em memória
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let mut heap = self.heap.write().await;
        heap.push(PrioritizedJob { job: job.clone(), sequence: seq });

        // Notificar workers
        self.notify.notify_one();

        Ok(job.id)
    }

    pub async fn dequeue(&self) -> Option<Job> {
        loop {
            {
                let mut heap = self.heap.write().await;
                if let Some(pjob) = heap.pop() {
                    self.mark_running(&pjob.job.id).await.ok()?;
                    return Some(pjob.job);
                }
            }

            // Esperar notificação de novo job
            self.notify.notified().await;
        }
    }

    pub async fn recover_on_startup(&self) -> Result<(), Error> {
        // Recuperar jobs pendentes/running do banco após crash
        let pending_jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT * FROM jobs
            WHERE status IN ('pending', 'running', 'retrying')
            ORDER BY priority, created_at
            "#
        )
        .fetch_all(&self.db)
        .await?;

        for job in pending_jobs {
            let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
            let mut heap = self.heap.write().await;
            heap.push(PrioritizedJob { job, sequence: seq });
        }

        Ok(())
    }
}
```

### Worker Implementation

```rust
pub struct TradingWorker {
    id: usize,
    queue: Arc<JobQueue>,
    handlers: JobHandlers,
    shutdown: CancellationToken,
}

impl TradingWorker {
    pub async fn run(self) {
        tracing::info!(worker_id = self.id, "Worker started");

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    tracing::info!(worker_id = self.id, "Worker shutting down");
                    break;
                }

                Some(job) = self.queue.dequeue() => {
                    self.process_job(job).await;
                }
            }
        }
    }

    async fn process_job(&self, mut job: Job) {
        let span = tracing::info_span!(
            "process_job",
            job_id = %job.id,
            job_type = ?job.job_type,
            priority = ?job.priority,
        );
        let _guard = span.enter();

        let result = match &job.job_type {
            JobType::FetchCandles { symbol, timeframe } => {
                self.handlers.fetch_candles(symbol, timeframe).await
            }
            JobType::ExecuteStrategy { strategy_id } => {
                self.handlers.execute_strategy(*strategy_id).await
            }
            JobType::PlaceOrder { order } => {
                self.handlers.place_order(order).await
            }
            // ... outros handlers
        };

        match result {
            Ok(value) => {
                job.status = JobStatus::Completed;
                job.result = Some(value);
                job.completed_at = Some(Utc::now());
                self.queue.update_job(&job).await.ok();
            }
            Err(e) if job.retry_count < job.max_retries => {
                job.retry_count += 1;
                job.status = JobStatus::Retrying;
                job.error_message = Some(e.to_string());

                // Exponential backoff
                let delay = Duration::from_secs(2u64.pow(job.retry_count));
                tokio::time::sleep(delay).await;

                self.queue.enqueue(job).await.ok();
            }
            Err(e) => {
                job.status = JobStatus::Failed;
                job.error_message = Some(e.to_string());
                job.completed_at = Some(Utc::now());
                self.queue.update_job(&job).await.ok();

                tracing::error!(error = %e, "Job failed permanently");
            }
        }
    }
}
```

### Scheduler

```rust
pub struct JobScheduler {
    queue: Arc<JobQueue>,
    schedules: Vec<ScheduledJob>,
}

struct ScheduledJob {
    cron: cron::Schedule,
    job_template: JobType,
    priority: JobPriority,
}

impl JobScheduler {
    pub async fn run(&self, shutdown: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = interval.tick() => {
                    let now = Utc::now();
                    for scheduled in &self.schedules {
                        if scheduled.cron.upcoming(Utc).next() == Some(now) {
                            let job = Job::new(
                                scheduled.job_template.clone(),
                                scheduled.priority,
                            );
                            self.queue.enqueue(job).await.ok();
                        }
                    }
                }
            }
        }
    }
}
```

## Consequências

### Positivas

- **Priorização clara**: Ordens nunca ficam atrás de coleta de dados
- **Resiliência**: Jobs sobrevivem a crashes via persistência em SQLite
- **Retry automático**: Falhas transientes são tratadas automaticamente
- **Visibilidade**: UI pode consultar estado de todos os jobs
- **Cancelamento graceful**: Workers respeitam shutdown signal
- **Extensibilidade**: Novos tipos de job são fáceis de adicionar

### Negativas

- **Complexidade**: Sistema mais complexo que executar tarefas diretamente
- **Overhead**: Persistência adiciona latência (mitigado sendo async)
- **Memória**: Heap em memória duplica dados do banco
- **Debugging**: Fluxo assíncrono mais difícil de rastrear

### Neutras

- Jobs completados são mantidos para histórico (com cleanup periódico)
- Métricas de throughput podem ser derivadas dos dados de jobs

## Alternativas Consideradas

### Alternativa 1: Tokio Tasks Diretas

- **Descrição**: Spawnar tokio::task para cada operação
- **Prós**: Simples, sem overhead de queue
- **Contras**: Sem priorização, sem persistência, sem retry automático
- **Motivo da rejeição**: Não atende requisitos de resiliência

### Alternativa 2: Redis + Sidekiq-like

- **Descrição**: Usar Redis como message broker
- **Prós**: Solução madura, muitas features
- **Contras**: Dependência externa, complexidade de deploy
- **Motivo da rejeição**: Overkill para aplicação desktop single-user

### Alternativa 3: Actor Model (Actix)

- **Descrição**: Cada componente como actor com mailbox
- **Prós**: Isolamento de estado, supervisão hierárquica
- **Contras**: Paradigma diferente, curva de aprendizado
- **Motivo da rejeição**: Adiciona complexidade conceitual sem benefício claro

## Referências

- [Tokio Tutorial - Channels](https://tokio.rs/tokio/tutorial/channels)
- [Designing Data-Intensive Applications - Chapter 11](https://dataintensive.net/)
- [Background Jobs in Rust](https://kerkour.com/rust-background-jobs)
