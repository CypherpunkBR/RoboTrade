//! Worker de trading integrado com Tauri
//!
//! Gerencia a execução do trading automático, incluindo:
//! - Scheduler para tarefas periódicas
//! - Risk manager para validação de ordens
//! - Circuit breaker para proteção
//! - Position manager para gestão de posições

use parking_lot::RwLock;
use robotrade_trading_worker::{
    CircuitBreaker, CircuitBreakerConfig,
    PositionManager, PositionManagerConfig,
    RiskConfig, RiskManager,
    Scheduler,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tracing::info;

/// Estado do worker de trading
pub struct TradingWorker {
    /// Scheduler de tarefas
    scheduler: Arc<RwLock<Option<Scheduler>>>,
    /// Risk manager
    risk_manager: Arc<RiskManager>,
    /// Circuit breaker
    circuit_breaker: Arc<CircuitBreaker>,
    /// Position manager
    position_manager: Arc<PositionManager>,
    /// Se o worker está rodando
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl TradingWorker {
    /// Cria um novo worker de trading
    pub fn new() -> Self {
        Self {
            scheduler: Arc::new(RwLock::new(None)),
            risk_manager: Arc::new(RiskManager::new(RiskConfig::default(), dec!(10000))),
            circuit_breaker: Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default())),
            position_manager: Arc::new(PositionManager::new(PositionManagerConfig::default())),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Retorna se o worker está rodando
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Inicia o worker
    pub async fn start(&self) -> Result<(), String> {
        if self.is_running() {
            return Err("Worker já está rodando".into());
        }

        info!("Iniciando worker de trading...");

        // Cria e inicia o scheduler
        let mut scheduler = Scheduler::new();
        scheduler.start();

        // Armazena o scheduler
        *self.scheduler.write() = Some(scheduler);
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);

        info!("Worker de trading iniciado com sucesso");
        Ok(())
    }

    /// Para o worker
    pub async fn stop(&self) -> Result<(), String> {
        if !self.is_running() {
            return Err("Worker não está rodando".into());
        }

        info!("Parando worker de trading...");

        // Para o scheduler - take() from RwLock primeiro para evitar hold across await
        let scheduler_opt = self.scheduler.write().take();
        if let Some(mut scheduler) = scheduler_opt {
            scheduler.stop().await;
        }

        self.running.store(false, std::sync::atomic::Ordering::SeqCst);

        info!("Worker de trading parado com sucesso");
        Ok(())
    }

    /// Retorna o risk manager
    pub fn risk_manager(&self) -> &Arc<RiskManager> {
        &self.risk_manager
    }

    /// Retorna o circuit breaker
    pub fn circuit_breaker(&self) -> &Arc<CircuitBreaker> {
        &self.circuit_breaker
    }

    /// Retorna o position manager
    pub fn position_manager(&self) -> &Arc<PositionManager> {
        &self.position_manager
    }

    /// Retorna o estado do circuit breaker
    pub async fn circuit_state(&self) -> String {
        format!("{:?}", self.circuit_breaker.get_state().await)
    }

    /// Verifica se o trading está habilitado
    pub async fn is_trading_enabled(&self) -> bool {
        self.risk_manager.get_state().await.trading_enabled
    }

    /// Habilita trading
    pub async fn enable_trading(&self) {
        self.risk_manager.enable_trading().await;
    }

    /// Desabilita trading
    pub async fn disable_trading(&self, reason: &str) {
        self.risk_manager.disable_trading(reason).await;
    }

    /// Reseta perdas diárias
    pub async fn reset_daily_losses(&self) {
        self.risk_manager.daily_reset().await;
    }

    /// Retorna estatísticas de risco
    pub async fn risk_stats(&self) -> RiskStats {
        let risk_state = self.risk_manager.get_state().await;
        let position_count = self.position_manager.position_count().await;
        let circuit_state = format!("{:?}", self.circuit_breaker.get_state().await);

        RiskStats {
            trading_enabled: risk_state.trading_enabled,
            daily_pnl: risk_state.daily_pnl,
            total_exposure: risk_state.total_exposure,
            position_count: position_count as u32,
            circuit_state,
        }
    }
}

impl Default for TradingWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TradingWorker {
    fn clone(&self) -> Self {
        Self {
            scheduler: Arc::clone(&self.scheduler),
            risk_manager: Arc::clone(&self.risk_manager),
            circuit_breaker: Arc::clone(&self.circuit_breaker),
            position_manager: Arc::clone(&self.position_manager),
            running: Arc::clone(&self.running),
        }
    }
}

/// Estatísticas de risco
#[derive(Debug, Clone, serde::Serialize)]
pub struct RiskStats {
    pub trading_enabled: bool,
    pub daily_pnl: Decimal,
    pub total_exposure: Decimal,
    pub position_count: u32,
    pub circuit_state: String,
}
