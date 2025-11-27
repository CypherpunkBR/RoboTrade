//! # Circuit Breaker
//!
//! Sistema de proteção que para o trading automaticamente quando:
//! - Muitos erros consecutivos ocorrem
//! - Perdas rápidas acontecem
//! - Problemas de conectividade são detectados
//!
//! Baseado no padrão Circuit Breaker com estados:
//! - Closed: Operando normalmente
//! - Open: Trading pausado
//! - HalfOpen: Testando se pode retomar

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Estado do Circuit Breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Operando normalmente
    Closed,
    /// Trading pausado - não permite operações
    Open,
    /// Testando recuperação - permite operações limitadas
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Fechado (Operacional)"),
            CircuitState::Open => write!(f, "Aberto (Pausado)"),
            CircuitState::HalfOpen => write!(f, "Semi-aberto (Testando)"),
        }
    }
}

/// Tipo de falha que pode acionar o circuit breaker
#[derive(Debug, Clone)]
pub enum FailureType {
    /// Erro de API/conexão
    ConnectionError,
    /// Erro de execução de ordem
    OrderExecutionError,
    /// Perda rápida
    RapidLoss { amount: Decimal },
    /// Timeout
    Timeout,
    /// Erro de autenticação
    AuthenticationError,
    /// Rate limit
    RateLimited,
    /// Outro erro
    Other(String),
}

/// Evento do Circuit Breaker
#[derive(Debug, Clone)]
pub enum CircuitBreakerEvent {
    /// Estado mudou
    StateChanged { from: CircuitState, to: CircuitState },
    /// Falha registrada
    FailureRecorded { failure_type: FailureType, count: u32 },
    /// Sucesso registrado (em half-open)
    SuccessRecorded { count: u32 },
    /// Circuit breaker foi resetado manualmente
    ManualReset,
    /// Recuperação automática
    AutoRecovered,
}

/// Configuração do Circuit Breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Número de falhas para abrir o circuito
    pub failure_threshold: u32,
    /// Tempo para tentar recuperação (em segundos)
    pub recovery_timeout_secs: u64,
    /// Número de sucessos necessários para fechar em half-open
    pub success_threshold: u32,
    /// Janela de tempo para contar falhas (em segundos)
    pub failure_window_secs: u64,
    /// Perda rápida que aciona o breaker (em valor)
    pub rapid_loss_threshold: Decimal,
    /// Janela de tempo para perda rápida (em segundos)
    pub rapid_loss_window_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_secs: 300, // 5 minutos
            success_threshold: 3,
            failure_window_secs: 60, // 1 minuto
            rapid_loss_threshold: dec!(500),
            rapid_loss_window_secs: 60,
        }
    }
}

/// Registro de falha
#[derive(Debug, Clone)]
struct FailureRecord {
    timestamp: DateTime<Utc>,
    failure_type: FailureType,
}

/// Registro de perda
#[derive(Debug, Clone)]
struct LossRecord {
    timestamp: DateTime<Utc>,
    amount: Decimal,
}

/// Circuit Breaker
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<RwLock<VecDeque<FailureRecord>>>,
    losses: Arc<RwLock<VecDeque<LossRecord>>>,
    successes_in_half_open: AtomicU32,
    opened_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    event_tx: tokio::sync::broadcast::Sender<CircuitBreakerEvent>,
}

impl CircuitBreaker {
    /// Cria um novo Circuit Breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(RwLock::new(VecDeque::new())),
            losses: Arc::new(RwLock::new(VecDeque::new())),
            successes_in_half_open: AtomicU32::new(0),
            opened_at: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }

    /// Retorna um receiver para eventos
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CircuitBreakerEvent> {
        self.event_tx.subscribe()
    }

    /// Verifica se operações são permitidas
    pub async fn is_allowed(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true, // Permite operações limitadas
            CircuitState::Open => {
                // Verifica se é hora de tentar recuperação
                drop(state); // Libera lock para poder mudar estado
                self.check_recovery().await
            }
        }
    }

    /// Retorna o estado atual
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Registra uma falha
    pub async fn record_failure(&self, failure_type: FailureType) {
        let now = Utc::now();

        // Adiciona falha ao histórico
        {
            let mut failures = self.failures.write().await;
            failures.push_back(FailureRecord {
                timestamp: now,
                failure_type: failure_type.clone(),
            });

            // Remove falhas antigas
            let cutoff = now - Duration::seconds(self.config.failure_window_secs as i64);
            while let Some(front) = failures.front() {
                if front.timestamp < cutoff {
                    failures.pop_front();
                } else {
                    break;
                }
            }
        }

        // Se for perda rápida, registra separadamente
        if let FailureType::RapidLoss { amount } = &failure_type {
            let mut losses = self.losses.write().await;
            losses.push_back(LossRecord {
                timestamp: now,
                amount: *amount,
            });

            // Remove perdas antigas
            let cutoff = now - Duration::seconds(self.config.rapid_loss_window_secs as i64);
            while let Some(front) = losses.front() {
                if front.timestamp < cutoff {
                    losses.pop_front();
                } else {
                    break;
                }
            }
        }

        let failure_count = self.failures.read().await.len() as u32;

        // Emite evento
        let _ = self.event_tx.send(CircuitBreakerEvent::FailureRecorded {
            failure_type: failure_type.clone(),
            count: failure_count,
        });

        // Verifica se deve abrir o circuito
        let should_open = self.should_open().await;

        if should_open {
            self.open_circuit().await;
        }

        // Em half-open, uma falha volta para open
        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen {
            self.open_circuit().await;
        }
    }

    /// Registra um sucesso
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        if state == CircuitState::HalfOpen {
            let count = self.successes_in_half_open.fetch_add(1, Ordering::SeqCst) + 1;

            let _ = self.event_tx.send(CircuitBreakerEvent::SuccessRecorded { count });

            if count >= self.config.success_threshold {
                self.close_circuit().await;
            }
        }
    }

    /// Registra uma perda para verificação de perda rápida
    pub async fn record_loss(&self, amount: Decimal) {
        if amount > Decimal::ZERO {
            self.record_failure(FailureType::RapidLoss { amount }).await;
        }
    }

    /// Reset manual do circuit breaker
    pub async fn reset(&self) {
        self.close_circuit().await;

        // Limpa histórico
        self.failures.write().await.clear();
        self.losses.write().await.clear();

        let _ = self.event_tx.send(CircuitBreakerEvent::ManualReset);
        info!("Circuit breaker resetado manualmente");
    }

    /// Verifica se deve abrir o circuito
    async fn should_open(&self) -> bool {
        let failures = self.failures.read().await;
        let losses = self.losses.read().await;

        // Verifica threshold de falhas
        if failures.len() as u32 >= self.config.failure_threshold {
            return true;
        }

        // Verifica perda rápida
        let total_loss: Decimal = losses.iter().map(|l| l.amount).sum();
        if total_loss >= self.config.rapid_loss_threshold {
            return true;
        }

        false
    }

    /// Abre o circuito
    async fn open_circuit(&self) {
        let mut state = self.state.write().await;
        let old_state = *state;

        if old_state != CircuitState::Open {
            *state = CircuitState::Open;
            *self.opened_at.write().await = Some(Utc::now());

            warn!("Circuit breaker ABERTO - trading pausado");

            let _ = self.event_tx.send(CircuitBreakerEvent::StateChanged {
                from: old_state,
                to: CircuitState::Open,
            });
        }
    }

    /// Fecha o circuito
    async fn close_circuit(&self) {
        let mut state = self.state.write().await;
        let old_state = *state;

        if old_state != CircuitState::Closed {
            *state = CircuitState::Closed;
            *self.opened_at.write().await = None;
            self.successes_in_half_open.store(0, Ordering::SeqCst);

            info!("Circuit breaker FECHADO - trading retomado");

            let _ = self.event_tx.send(CircuitBreakerEvent::StateChanged {
                from: old_state,
                to: CircuitState::Closed,
            });
        }
    }

    /// Verifica se é hora de tentar recuperação
    async fn check_recovery(&self) -> bool {
        let opened_at = self.opened_at.read().await;

        if let Some(opened) = *opened_at {
            let elapsed = Utc::now() - opened;

            if elapsed >= Duration::seconds(self.config.recovery_timeout_secs as i64) {
                // Tenta transição para half-open
                drop(opened_at); // Libera lock
                self.transition_to_half_open().await;
                return true;
            }
        }

        false
    }

    /// Transiciona para half-open
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;

        if *state == CircuitState::Open {
            let old_state = *state;
            *state = CircuitState::HalfOpen;
            self.successes_in_half_open.store(0, Ordering::SeqCst);

            info!("Circuit breaker SEMI-ABERTO - testando recuperação");

            let _ = self.event_tx.send(CircuitBreakerEvent::StateChanged {
                from: old_state,
                to: CircuitState::HalfOpen,
            });
        }
    }

    /// Tempo restante até tentar recuperação (em segundos)
    pub async fn time_until_recovery(&self) -> Option<i64> {
        let state = *self.state.read().await;

        if state != CircuitState::Open {
            return None;
        }

        let opened_at = self.opened_at.read().await;
        if let Some(opened) = *opened_at {
            let elapsed = (Utc::now() - opened).num_seconds();
            let remaining = self.config.recovery_timeout_secs as i64 - elapsed;
            return Some(remaining.max(0));
        }

        None
    }

    /// Contagem de falhas na janela atual
    pub async fn failure_count(&self) -> usize {
        self.failures.read().await.len()
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_state() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.is_allowed().await);
    }

    #[tokio::test]
    async fn test_opens_after_failures() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_failure(FailureType::ConnectionError).await;
        cb.record_failure(FailureType::ConnectionError).await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_failure(FailureType::ConnectionError).await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_rapid_loss_opens() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            rapid_loss_threshold: dec!(100),
            failure_threshold: 100, // Alto para não interferir
            ..Default::default()
        });

        cb.record_loss(dec!(50)).await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_loss(dec!(60)).await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_manual_reset() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        });

        cb.record_failure(FailureType::ConnectionError).await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        cb.reset().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert_eq!(cb.failure_count().await, 0);
    }

    #[tokio::test]
    async fn test_half_open_success_closes() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            recovery_timeout_secs: 0, // Imediato para teste
            ..Default::default()
        });

        // Abre o circuito
        cb.record_failure(FailureType::ConnectionError).await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Tenta is_allowed para transicionar para half-open
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = cb.is_allowed().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Sucessos fecham o circuito
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_failure_reopens() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout_secs: 0,
            ..Default::default()
        });

        // Abre o circuito
        cb.record_failure(FailureType::ConnectionError).await;

        // Transiciona para half-open
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = cb.is_allowed().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Falha reabre o circuito
        cb.record_failure(FailureType::ConnectionError).await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }
}
