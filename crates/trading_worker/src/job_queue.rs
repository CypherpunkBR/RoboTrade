//! # Job Queue
//!
//! Fila de jobs para processamento de sinais e execução de ordens.
//!
//! Responsável por:
//! - Enfileirar sinais para processamento
//! - Converter sinais em ordens
//! - Gerenciar prioridades e retries
//! - Garantir execução ordenada

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use robotrade_core::entities::{
    ExchangeId, OrderRequest, OrderSide, OrderType, Signal, TimeInForce, TradeDirection,
};

/// Tipo de job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    /// Processar sinal de entrada
    ProcessSignal,
    /// Executar ordem
    ExecuteOrder,
    /// Atualizar posição
    UpdatePosition,
    /// Verificar stop loss/take profit
    CheckStopLoss,
    /// Fechar posição
    ClosePosition,
    /// Sincronizar com exchange
    SyncExchange,
}

/// Prioridade do job
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    /// Baixa prioridade
    Low = 0,
    /// Prioridade normal
    Normal = 1,
    /// Alta prioridade
    High = 2,
    /// Crítica (stop loss, liquidação)
    Critical = 3,
}

/// Status do job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Aguardando processamento
    Pending,
    /// Em processamento
    Processing,
    /// Completado com sucesso
    Completed,
    /// Falhou
    Failed { error: String, retries: u32 },
    /// Cancelado
    Cancelled,
}

/// Job na fila
#[derive(Debug, Clone)]
pub struct Job {
    /// ID único do job
    pub id: Uuid,
    /// Tipo do job
    pub job_type: JobType,
    /// Prioridade
    pub priority: JobPriority,
    /// Dados do job (JSON serializado)
    pub payload: JobPayload,
    /// Status atual
    pub status: JobStatus,
    /// Quando foi criado
    pub created_at: DateTime<Utc>,
    /// Quando começou a processar
    pub started_at: Option<DateTime<Utc>>,
    /// Quando completou
    pub completed_at: Option<DateTime<Utc>>,
    /// Número de tentativas
    pub attempts: u32,
    /// Máximo de tentativas
    pub max_attempts: u32,
}

/// Payload do job
#[derive(Debug, Clone)]
pub enum JobPayload {
    /// Sinal para processar
    Signal(Signal),
    /// Ordem para executar
    Order(OrderRequest),
    /// ID de posição para atualizar/fechar
    PositionId(String),
    /// ID de exchange para sincronizar
    ExchangeSync { exchange: ExchangeId },
    /// Vazio
    Empty,
}

impl Job {
    /// Cria um novo job
    pub fn new(job_type: JobType, priority: JobPriority, payload: JobPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            priority,
            payload,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            attempts: 0,
            max_attempts: 3,
        }
    }

    /// Job de alta prioridade (stop loss, etc)
    pub fn critical(job_type: JobType, payload: JobPayload) -> Self {
        Self::new(job_type, JobPriority::Critical, payload)
    }

    /// Marca como em processamento
    pub fn mark_processing(&mut self) {
        self.status = JobStatus::Processing;
        self.started_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// Marca como completado
    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Marca como falhou
    pub fn mark_failed(&mut self, error: String) {
        self.status = JobStatus::Failed {
            error,
            retries: self.attempts,
        };
        self.completed_at = Some(Utc::now());
    }

    /// Verifica se pode tentar novamente
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }
}

// Implementação para BinaryHeap (maior prioridade primeiro)
impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        // Maior prioridade primeiro, depois mais antigo primeiro
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.created_at.cmp(&self.created_at),
            other => other,
        }
    }
}

/// Evento da Job Queue
#[derive(Debug, Clone)]
pub enum JobQueueEvent {
    /// Job adicionado à fila
    JobAdded { job_id: Uuid, job_type: JobType },
    /// Job iniciou processamento
    JobStarted { job_id: Uuid },
    /// Job completado
    JobCompleted { job_id: Uuid },
    /// Job falhou
    JobFailed { job_id: Uuid, error: String },
    /// Job cancelado
    JobCancelled { job_id: Uuid },
    /// Fila está vazia
    QueueEmpty,
}

/// Configuração da Job Queue
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    /// Tamanho máximo da fila
    pub max_queue_size: usize,
    /// Timeout de processamento (segundos)
    pub processing_timeout_secs: u64,
    /// Máximo de jobs em processamento simultâneo
    pub max_concurrent_jobs: usize,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            processing_timeout_secs: 30,
            max_concurrent_jobs: 5,
        }
    }
}

/// Job Queue
pub struct JobQueue {
    config: JobQueueConfig,
    pending: Arc<RwLock<BinaryHeap<Job>>>,
    processing: Arc<RwLock<HashMap<Uuid, Job>>>,
    completed: Arc<RwLock<Vec<Job>>>,
    event_tx: tokio::sync::broadcast::Sender<JobQueueEvent>,
}

impl JobQueue {
    /// Cria uma nova Job Queue
    pub fn new(config: JobQueueConfig) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            config,
            pending: Arc::new(RwLock::new(BinaryHeap::new())),
            processing: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(Vec::new())),
            event_tx,
        }
    }

    /// Retorna um receiver para eventos
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<JobQueueEvent> {
        self.event_tx.subscribe()
    }

    /// Adiciona um job à fila
    pub async fn enqueue(&self, job: Job) -> Result<Uuid, String> {
        let mut pending = self.pending.write().await;

        if pending.len() >= self.config.max_queue_size {
            return Err("Fila cheia".to_string());
        }

        let job_id = job.id;
        let job_type = job.job_type.clone();

        pending.push(job);

        debug!("Job {:?} adicionado à fila: {}", job_type, job_id);

        let _ = self.event_tx.send(JobQueueEvent::JobAdded { job_id, job_type });

        Ok(job_id)
    }

    /// Adiciona um sinal para processamento
    pub async fn enqueue_signal(&self, signal: Signal) -> Result<Uuid, String> {
        let priority = match signal.strength {
            robotrade_core::entities::SignalStrength::Strong => JobPriority::High,
            robotrade_core::entities::SignalStrength::Moderate => JobPriority::Normal,
            robotrade_core::entities::SignalStrength::Weak => JobPriority::Low,
        };

        let job = Job::new(
            JobType::ProcessSignal,
            priority,
            JobPayload::Signal(signal),
        );

        self.enqueue(job).await
    }

    /// Adiciona uma ordem para execução
    pub async fn enqueue_order(&self, order: OrderRequest, priority: JobPriority) -> Result<Uuid, String> {
        let job = Job::new(
            JobType::ExecuteOrder,
            priority,
            JobPayload::Order(order),
        );

        self.enqueue(job).await
    }

    /// Pega o próximo job para processar
    pub async fn dequeue(&self) -> Option<Job> {
        // Verifica se pode processar mais jobs
        let processing_count = self.processing.read().await.len();
        if processing_count >= self.config.max_concurrent_jobs {
            return None;
        }

        let mut pending = self.pending.write().await;
        let mut job = pending.pop()?;

        job.mark_processing();

        let job_id = job.id;

        // Move para processamento
        drop(pending);
        self.processing.write().await.insert(job_id, job.clone());

        let _ = self.event_tx.send(JobQueueEvent::JobStarted { job_id });

        Some(job)
    }

    /// Marca job como completado
    pub async fn complete(&self, job_id: Uuid) {
        let mut processing = self.processing.write().await;

        if let Some(mut job) = processing.remove(&job_id) {
            job.mark_completed();

            // Move para histórico
            drop(processing);

            let mut completed = self.completed.write().await;
            completed.push(job);

            // Limita histórico
            if completed.len() > 1000 {
                completed.remove(0);
            }

            let _ = self.event_tx.send(JobQueueEvent::JobCompleted { job_id });
            debug!("Job completado: {}", job_id);
        }
    }

    /// Marca job como falhou
    pub async fn fail(&self, job_id: Uuid, error: String) {
        let mut processing = self.processing.write().await;

        if let Some(mut job) = processing.remove(&job_id) {
            if job.can_retry() {
                // Re-enfileira para retry
                job.status = JobStatus::Pending;
                drop(processing);

                let mut pending = self.pending.write().await;
                pending.push(job);

                warn!("Job {} falhou, tentando novamente: {}", job_id, error);
            } else {
                job.mark_failed(error.clone());

                drop(processing);

                let mut completed = self.completed.write().await;
                completed.push(job);

                error!("Job {} falhou definitivamente: {}", job_id, error);
                let _ = self.event_tx.send(JobQueueEvent::JobFailed { job_id, error });
            }
        }
    }

    /// Cancela um job pendente
    pub async fn cancel(&self, job_id: Uuid) -> bool {
        let mut pending = self.pending.write().await;

        // Reconstrói a heap sem o job cancelado
        let jobs: Vec<Job> = pending.drain().collect();
        let mut found = false;

        for job in jobs {
            if job.id == job_id {
                found = true;
                let _ = self.event_tx.send(JobQueueEvent::JobCancelled { job_id });
            } else {
                pending.push(job);
            }
        }

        found
    }

    /// Tamanho da fila pendente
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }

    /// Número de jobs em processamento
    pub async fn processing_count(&self) -> usize {
        self.processing.read().await.len()
    }

    /// Verifica se a fila está vazia
    pub async fn is_empty(&self) -> bool {
        let pending = self.pending.read().await;
        let processing = self.processing.read().await;
        pending.is_empty() && processing.is_empty()
    }

    /// Limpa a fila
    pub async fn clear(&self) {
        self.pending.write().await.clear();
        info!("Fila de jobs limpa");
    }

    /// Retorna estatísticas da fila
    pub async fn stats(&self) -> QueueStats {
        let pending = self.pending.read().await;
        let processing = self.processing.read().await;
        let completed = self.completed.read().await;

        let completed_count = completed.iter().filter(|j| matches!(j.status, JobStatus::Completed)).count();
        let failed_count = completed.iter().filter(|j| matches!(j.status, JobStatus::Failed { .. })).count();

        QueueStats {
            pending: pending.len(),
            processing: processing.len(),
            completed: completed_count,
            failed: failed_count,
        }
    }
}

/// Estatísticas da fila
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new(JobQueueConfig::default())
    }
}

/// Converte um Signal em OrderRequest
pub fn signal_to_order(signal: &Signal, quantity: Decimal) -> Option<OrderRequest> {
    let side = match signal.direction {
        TradeDirection::Long => OrderSide::Buy,
        TradeDirection::Short => OrderSide::Sell,
    };

    Some(OrderRequest {
        symbol: signal.symbol.clone(),
        side,
        order_type: OrderType::Limit,
        quantity,
        price: Some(signal.suggested_entry.unwrap_or(signal.trigger_price)),
        stop_price: None,
        stop_loss: signal.suggested_stop_loss,
        take_profit: signal.suggested_take_profit,
        time_in_force: TimeInForce::GTC,
        leverage: Some(10), // Padrão, pode ser configurável
        reduce_only: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use robotrade_core::entities::{SignalStrength, SignalType};

    fn create_test_signal() -> Signal {
        Signal::new(
            "test_strategy",
            "Test Strategy",
            "BTCUSDT",
            SignalType::Custom { name: "test".to_string(), description: "Test signal".to_string() },
            TradeDirection::Long,
            SignalStrength::Strong,
            dec!(50000),
            "Test signal",
        )
        .with_targets(dec!(50000), dec!(49000), dec!(52000))
    }

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = JobQueue::default();

        let job = Job::new(JobType::ProcessSignal, JobPriority::Normal, JobPayload::Empty);
        let job_id = queue.enqueue(job).await.unwrap();

        assert_eq!(queue.pending_count().await, 1);

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, job_id);
        assert_eq!(queue.pending_count().await, 0);
        assert_eq!(queue.processing_count().await, 1);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = JobQueue::default();

        // Adiciona jobs com diferentes prioridades
        let low = Job::new(JobType::UpdatePosition, JobPriority::Low, JobPayload::Empty);
        let normal = Job::new(JobType::ProcessSignal, JobPriority::Normal, JobPayload::Empty);
        let high = Job::new(JobType::ExecuteOrder, JobPriority::High, JobPayload::Empty);
        let critical = Job::new(JobType::CheckStopLoss, JobPriority::Critical, JobPayload::Empty);

        queue.enqueue(low).await.unwrap();
        queue.enqueue(normal).await.unwrap();
        queue.enqueue(high).await.unwrap();
        queue.enqueue(critical).await.unwrap();

        // Deve sair na ordem: critical, high, normal, low
        let first = queue.dequeue().await.unwrap();
        assert_eq!(first.priority, JobPriority::Critical);

        let second = queue.dequeue().await.unwrap();
        assert_eq!(second.priority, JobPriority::High);

        let third = queue.dequeue().await.unwrap();
        assert_eq!(third.priority, JobPriority::Normal);

        let fourth = queue.dequeue().await.unwrap();
        assert_eq!(fourth.priority, JobPriority::Low);
    }

    #[tokio::test]
    async fn test_complete_and_fail() {
        let queue = JobQueue::default();

        let job = Job::new(JobType::ProcessSignal, JobPriority::Normal, JobPayload::Empty);
        let job_id = queue.enqueue(job).await.unwrap();

        let _dequeued = queue.dequeue().await.unwrap();
        queue.complete(job_id).await;

        let stats = queue.stats().await;
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.processing, 0);
    }

    #[tokio::test]
    async fn test_signal_enqueue() {
        let queue = JobQueue::default();
        let signal = create_test_signal();

        let _job_id = queue.enqueue_signal(signal).await.unwrap();
        assert!(queue.pending_count().await > 0);
    }

    #[tokio::test]
    async fn test_signal_to_order_conversion() {
        let signal = create_test_signal();
        let order = signal_to_order(&signal, dec!(0.1)).unwrap();

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.stop_loss, Some(dec!(49000)));
    }
}
