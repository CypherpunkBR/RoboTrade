//! Entidades de Job Queue & Scheduling
//!
//! Modelos para jobs em background e tarefas agendadas.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// ID único de um job
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ID único de uma tarefa agendada
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduledTaskId(pub Uuid);

impl ScheduledTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for ScheduledTaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ScheduledTaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo de job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Sincronização de dados de mercado
    MarketDataSync,
    /// Avaliação de estratégia
    StrategyEvaluation,
    /// Execução de ordem
    OrderExecution,
    /// Atualização de posição
    PositionUpdate,
    /// Cálculo de métricas
    MetricsCalculation,
    /// Sincronização de saldo
    BalanceSync,
    /// Coleta de Fear & Greed
    FearGreedCollection,
    /// Limpeza de dados antigos
    DataCleanup,
    /// Geração de relatório
    ReportGeneration,
    /// Notificação
    Notification,
    /// Backtest
    Backtest,
    /// Job customizado
    Custom(String),
}

impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobType::MarketDataSync => write!(f, "market_data_sync"),
            JobType::StrategyEvaluation => write!(f, "strategy_evaluation"),
            JobType::OrderExecution => write!(f, "order_execution"),
            JobType::PositionUpdate => write!(f, "position_update"),
            JobType::MetricsCalculation => write!(f, "metrics_calculation"),
            JobType::BalanceSync => write!(f, "balance_sync"),
            JobType::FearGreedCollection => write!(f, "fear_greed_collection"),
            JobType::DataCleanup => write!(f, "data_cleanup"),
            JobType::ReportGeneration => write!(f, "report_generation"),
            JobType::Notification => write!(f, "notification"),
            JobType::Backtest => write!(f, "backtest"),
            JobType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl std::str::FromStr for JobType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "market_data_sync" => Ok(JobType::MarketDataSync),
            "strategy_evaluation" => Ok(JobType::StrategyEvaluation),
            "order_execution" => Ok(JobType::OrderExecution),
            "position_update" => Ok(JobType::PositionUpdate),
            "metrics_calculation" => Ok(JobType::MetricsCalculation),
            "balance_sync" => Ok(JobType::BalanceSync),
            "fear_greed_collection" => Ok(JobType::FearGreedCollection),
            "data_cleanup" => Ok(JobType::DataCleanup),
            "report_generation" => Ok(JobType::ReportGeneration),
            "notification" => Ok(JobType::Notification),
            "backtest" => Ok(JobType::Backtest),
            other => {
                if let Some(name) = other.strip_prefix("custom:") {
                    Ok(JobType::Custom(name.to_string()))
                } else {
                    Ok(JobType::Custom(other.to_string()))
                }
            }
        }
    }
}

/// Prioridade do job
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum JobPriority {
    /// Crítica (executa primeiro)
    Critical = 0,
    /// Alta
    High = 1,
    /// Normal
    Normal = 2,
    /// Baixa
    Low = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

impl fmt::Display for JobPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobPriority::Critical => write!(f, "critical"),
            JobPriority::High => write!(f, "high"),
            JobPriority::Normal => write!(f, "normal"),
            JobPriority::Low => write!(f, "low"),
        }
    }
}

impl From<u8> for JobPriority {
    fn from(value: u8) -> Self {
        match value {
            0 => JobPriority::Critical,
            1 => JobPriority::High,
            2 => JobPriority::Normal,
            _ => JobPriority::Low,
        }
    }
}

/// Status do job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Pendente na fila
    Pending,
    /// Agendado para execução futura
    Scheduled,
    /// Em execução
    Running,
    /// Completado com sucesso
    Completed,
    /// Falhou
    Failed,
    /// Cancelado
    Cancelled,
    /// Expirado (deadline passou)
    Expired,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Scheduled => write!(f, "scheduled"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
            JobStatus::Expired => write!(f, "expired"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(JobStatus::Pending),
            "scheduled" => Ok(JobStatus::Scheduled),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            "expired" => Ok(JobStatus::Expired),
            _ => Err(format!("Status de job inválido: {}", s)),
        }
    }
}

impl JobStatus {
    /// Verifica se é um status final
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed
                | JobStatus::Failed
                | JobStatus::Cancelled
                | JobStatus::Expired
        )
    }

    /// Verifica se pode ser executado
    pub fn can_run(&self) -> bool {
        matches!(self, JobStatus::Pending | JobStatus::Scheduled)
    }
}

/// Job em background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// ID único
    pub id: JobId,
    /// Tipo do job
    pub job_type: JobType,
    /// Fila (para processamento separado)
    pub queue: String,
    /// Prioridade
    pub priority: JobPriority,
    /// Payload (dados do job em JSON)
    pub payload: serde_json::Value,
    /// Agendado para
    pub scheduled_for: DateTime<Utc>,
    /// Não executar antes de
    pub not_before: Option<DateTime<Utc>>,
    /// Deadline (expira se não executar antes)
    pub deadline: Option<DateTime<Utc>>,
    /// Status atual
    pub status: JobStatus,
    /// Tentativas realizadas
    pub attempts: u32,
    /// Máximo de tentativas
    pub max_attempts: u32,
    /// ID do worker que está processando
    pub worker_id: Option<String>,
    /// Bloqueado em
    pub locked_at: Option<DateTime<Utc>>,
    /// Lock expira em
    pub lock_expires_at: Option<DateTime<Utc>>,
    /// Resultado (se completado)
    pub result: Option<serde_json::Value>,
    /// Último erro
    pub last_error: Option<String>,
    /// Contador de erros
    pub error_count: u32,
    /// Início da execução
    pub started_at: Option<DateTime<Utc>>,
    /// Fim da execução
    pub completed_at: Option<DateTime<Utc>>,
    /// Tempo de execução em ms
    pub execution_time_ms: Option<i64>,
    /// Jobs dos quais depende
    pub depends_on: Vec<JobId>,
    /// Chave de idempotência
    pub idempotency_key: Option<String>,
    /// Metadados adicionais
    pub metadata: Option<serde_json::Value>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl Job {
    /// Cria um novo job
    pub fn new(job_type: JobType, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: JobId::new(),
            job_type,
            queue: "default".to_string(),
            priority: JobPriority::Normal,
            payload,
            scheduled_for: now,
            not_before: None,
            deadline: None,
            status: JobStatus::Pending,
            attempts: 0,
            max_attempts: 3,
            worker_id: None,
            locked_at: None,
            lock_expires_at: None,
            result: None,
            last_error: None,
            error_count: 0,
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            depends_on: vec![],
            idempotency_key: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Define a fila
    pub fn in_queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = queue.into();
        self
    }

    /// Define a prioridade
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Agenda para um momento específico
    pub fn schedule_for(mut self, time: DateTime<Utc>) -> Self {
        self.scheduled_for = time;
        self.status = JobStatus::Scheduled;
        self
    }

    /// Define deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Define máximo de tentativas
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Define dependências
    pub fn depends_on(mut self, job_ids: Vec<JobId>) -> Self {
        self.depends_on = job_ids;
        self
    }

    /// Define chave de idempotência
    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Verifica se pode ser executado agora
    pub fn can_run_now(&self) -> bool {
        if !self.status.can_run() {
            return false;
        }

        let now = Utc::now();

        if let Some(not_before) = self.not_before {
            if now < not_before {
                return false;
            }
        }

        if let Some(deadline) = self.deadline {
            if now > deadline {
                return false;
            }
        }

        now >= self.scheduled_for
    }

    /// Verifica se pode tentar novamente
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Inicia execução
    pub fn start(&mut self, worker_id: impl Into<String>) {
        let now = Utc::now();
        self.status = JobStatus::Running;
        self.worker_id = Some(worker_id.into());
        self.locked_at = Some(now);
        self.lock_expires_at = Some(now + chrono::Duration::minutes(5));
        self.started_at = Some(now);
        self.attempts += 1;
        self.updated_at = now;
    }

    /// Completa com sucesso
    pub fn complete(&mut self, result: Option<serde_json::Value>) {
        let now = Utc::now();
        self.status = JobStatus::Completed;
        self.result = result;
        self.completed_at = Some(now);

        if let Some(started) = self.started_at {
            self.execution_time_ms = Some((now - started).num_milliseconds());
        }

        self.worker_id = None;
        self.locked_at = None;
        self.lock_expires_at = None;
        self.updated_at = now;
    }

    /// Falha com erro
    pub fn fail(&mut self, error: impl Into<String>) {
        let now = Utc::now();
        self.last_error = Some(error.into());
        self.error_count += 1;

        if self.can_retry() {
            self.status = JobStatus::Pending;
        } else {
            self.status = JobStatus::Failed;
            self.completed_at = Some(now);
        }

        self.worker_id = None;
        self.locked_at = None;
        self.lock_expires_at = None;
        self.updated_at = now;
    }

    /// Cancela o job
    pub fn cancel(&mut self) {
        let now = Utc::now();
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(now);
        self.worker_id = None;
        self.locked_at = None;
        self.lock_expires_at = None;
        self.updated_at = now;
    }
}

/// Tipo de agendamento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleType {
    /// Expressão cron
    Cron,
    /// Intervalo fixo
    Interval,
    /// Hora fixa do dia
    FixedTime,
}

impl fmt::Display for ScheduleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleType::Cron => write!(f, "cron"),
            ScheduleType::Interval => write!(f, "interval"),
            ScheduleType::FixedTime => write!(f, "fixed_time"),
        }
    }
}

impl std::str::FromStr for ScheduleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cron" => Ok(ScheduleType::Cron),
            "interval" => Ok(ScheduleType::Interval),
            "fixed_time" => Ok(ScheduleType::FixedTime),
            _ => Err(format!("Tipo de agendamento inválido: {}", s)),
        }
    }
}

/// Tarefa agendada recorrente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// ID único
    pub id: ScheduledTaskId,
    /// Nome único
    pub name: String,
    /// Descrição
    pub description: Option<String>,
    /// Tipo de job a criar
    pub job_type: JobType,
    /// Template do payload
    pub payload_template: serde_json::Value,
    /// Fila para o job
    pub queue: String,
    /// Prioridade do job
    pub priority: JobPriority,
    /// Tipo de agendamento
    pub schedule_type: ScheduleType,
    /// Expressão cron (se schedule_type = Cron)
    pub cron_expression: Option<String>,
    /// Intervalo em segundos (se schedule_type = Interval)
    pub interval_seconds: Option<u32>,
    /// Timezone
    pub timezone: String,
    /// Habilitado?
    pub is_enabled: bool,
    /// Última execução
    pub last_run_at: Option<DateTime<Utc>>,
    /// Próxima execução
    pub next_run_at: Option<DateTime<Utc>>,
    /// ID do último job criado
    pub last_job_id: Option<JobId>,
    /// Máximo de execuções concorrentes
    pub max_concurrent: u32,
    /// Timeout em segundos
    pub timeout_seconds: u32,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl ScheduledTask {
    /// Cria uma nova tarefa com intervalo
    pub fn with_interval(
        name: impl Into<String>,
        job_type: JobType,
        interval_seconds: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ScheduledTaskId::new(),
            name: name.into(),
            description: None,
            job_type,
            payload_template: serde_json::json!({}),
            queue: "default".to_string(),
            priority: JobPriority::Normal,
            schedule_type: ScheduleType::Interval,
            cron_expression: None,
            interval_seconds: Some(interval_seconds),
            timezone: "UTC".to_string(),
            is_enabled: true,
            last_run_at: None,
            next_run_at: Some(now + chrono::Duration::seconds(interval_seconds as i64)),
            last_job_id: None,
            max_concurrent: 1,
            timeout_seconds: 300,
            created_at: now,
            updated_at: now,
        }
    }

    /// Cria uma nova tarefa com cron
    pub fn with_cron(
        name: impl Into<String>,
        job_type: JobType,
        cron_expression: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ScheduledTaskId::new(),
            name: name.into(),
            description: None,
            job_type,
            payload_template: serde_json::json!({}),
            queue: "default".to_string(),
            priority: JobPriority::Normal,
            schedule_type: ScheduleType::Cron,
            cron_expression: Some(cron_expression.into()),
            interval_seconds: None,
            timezone: "UTC".to_string(),
            is_enabled: true,
            last_run_at: None,
            next_run_at: None, // Deve ser calculado baseado no cron
            last_job_id: None,
            max_concurrent: 1,
            timeout_seconds: 300,
            created_at: now,
            updated_at: now,
        }
    }

    /// Define payload template
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload_template = payload;
        self
    }

    /// Verifica se deve executar agora
    pub fn should_run(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        if let Some(next_run) = self.next_run_at {
            Utc::now() >= next_run
        } else {
            false
        }
    }

    /// Cria um job a partir desta tarefa
    pub fn create_job(&self) -> Job {
        Job::new(self.job_type.clone(), self.payload_template.clone())
            .in_queue(&self.queue)
            .with_priority(self.priority)
    }

    /// Registra execução
    pub fn record_run(&mut self, job_id: JobId) {
        let now = Utc::now();
        self.last_run_at = Some(now);
        self.last_job_id = Some(job_id);

        // Calcula próxima execução
        if let Some(interval) = self.interval_seconds {
            self.next_run_at = Some(now + chrono::Duration::seconds(interval as i64));
        }
        // Para cron, precisa de um parser externo

        self.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_lifecycle() {
        let mut job = Job::new(JobType::MarketDataSync, serde_json::json!({"symbol": "BTCUSDT"}))
            .with_priority(JobPriority::High)
            .with_max_attempts(3);

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_run_now());

        job.start("worker-1");
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.attempts, 1);

        job.complete(Some(serde_json::json!({"success": true})));
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.execution_time_ms.is_some());
    }

    #[test]
    fn test_job_retry() {
        let mut job = Job::new(JobType::OrderExecution, serde_json::json!({}))
            .with_max_attempts(3);

        job.start("worker-1");
        job.fail("Connection timeout");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.error_count, 1);
        assert!(job.can_retry());

        job.start("worker-2");
        job.fail("Connection timeout");
        job.start("worker-3");
        job.fail("Connection timeout");

        assert_eq!(job.status, JobStatus::Failed);
        assert!(!job.can_retry());
    }

    #[test]
    fn test_scheduled_task() {
        let task = ScheduledTask::with_interval(
            "sync_market_data",
            JobType::MarketDataSync,
            60,
        )
        .with_payload(serde_json::json!({"symbols": ["BTCUSDT", "ETHUSDT"]}));

        assert!(task.is_enabled);
        assert_eq!(task.interval_seconds, Some(60));

        let job = task.create_job();
        assert_eq!(job.queue, "default");
    }
}
