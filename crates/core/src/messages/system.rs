//! Mensagens de sistema para controle de actors

use serde::{Deserialize, Serialize};

/// Mensagens de sistema comuns a todos os actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemMessage {
    /// Comando de shutdown graceful
    Shutdown,

    /// Verificação de health
    HealthCheck,

    /// Resposta de health check
    HealthStatus(HealthStatus),

    /// Ping para manter conexão ativa
    Ping,

    /// Pong em resposta ao ping
    Pong,

    /// Recarregar configuração
    ReloadConfig,

    /// Pausar processamento
    Pause,

    /// Retomar processamento
    Resume,
}

/// Status de saúde de um actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Nome do actor
    pub actor_name: String,
    /// Se está saudável
    pub is_healthy: bool,
    /// Mensagem de status
    pub message: Option<String>,
    /// Métricas opcionais
    pub metrics: Option<ActorMetrics>,
}

/// Métricas de um actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMetrics {
    /// Mensagens processadas
    pub messages_processed: u64,
    /// Mensagens pendentes no mailbox
    pub pending_messages: usize,
    /// Erros desde o início
    pub errors_count: u64,
    /// Uptime em segundos
    pub uptime_seconds: u64,
    /// Uso de memória em bytes (se disponível)
    pub memory_bytes: Option<u64>,
}

impl HealthStatus {
    /// Cria um status saudável
    pub fn healthy(actor_name: impl Into<String>) -> Self {
        Self {
            actor_name: actor_name.into(),
            is_healthy: true,
            message: None,
            metrics: None,
        }
    }

    /// Cria um status saudável com mensagem
    pub fn healthy_with_message(actor_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            actor_name: actor_name.into(),
            is_healthy: true,
            message: Some(message.into()),
            metrics: None,
        }
    }

    /// Cria um status não saudável
    pub fn unhealthy(actor_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            actor_name: actor_name.into(),
            is_healthy: false,
            message: Some(reason.into()),
            metrics: None,
        }
    }

    /// Adiciona métricas ao status
    pub fn with_metrics(mut self, metrics: ActorMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

/// Mensagem wrapper que inclui sistema ou mensagem específica do actor
#[derive(Debug, Clone)]
pub enum ActorMessage<M> {
    /// Mensagem de sistema
    System(SystemMessage),
    /// Mensagem específica do actor
    App(M),
}

impl<M> ActorMessage<M> {
    /// Cria uma mensagem de sistema
    pub fn system(msg: SystemMessage) -> Self {
        ActorMessage::System(msg)
    }

    /// Cria uma mensagem de aplicação
    pub fn app(msg: M) -> Self {
        ActorMessage::App(msg)
    }

    /// Verifica se é uma mensagem de shutdown
    pub fn is_shutdown(&self) -> bool {
        matches!(self, ActorMessage::System(SystemMessage::Shutdown))
    }
}
