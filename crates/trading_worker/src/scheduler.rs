//! Scheduler de coleta de dados e execução de tarefas periódicas
//!
//! Responsável por:
//! - Coletar Fear & Greed Index periodicamente
//! - Atualizar preços e posições
//! - Executar estratégias em intervalos definidos

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Evento emitido pelo scheduler
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// Hora de coletar Fear & Greed Index
    CollectFearGreed,
    /// Hora de atualizar preços
    UpdatePrices,
    /// Hora de verificar posições
    CheckPositions,
    /// Hora de executar estratégias
    RunStrategies,
    /// Scheduler foi iniciado
    Started,
    /// Scheduler foi parado
    Stopped,
}

/// Configuração do scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Intervalo de coleta do Fear & Greed (em segundos)
    pub fear_greed_interval_secs: u64,
    /// Intervalo de atualização de preços (em segundos)
    pub price_update_interval_secs: u64,
    /// Intervalo de verificação de posições (em segundos)
    pub position_check_interval_secs: u64,
    /// Intervalo de execução de estratégias (em segundos)
    pub strategy_interval_secs: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            // Fear & Greed atualiza a cada 12 horas, coletamos a cada hora
            fear_greed_interval_secs: 3600,
            // Preços atualizados a cada 5 segundos
            price_update_interval_secs: 5,
            // Posições verificadas a cada 10 segundos
            position_check_interval_secs: 10,
            // Estratégias executadas a cada minuto
            strategy_interval_secs: 60,
        }
    }
}

/// Scheduler de tarefas periódicas
pub struct Scheduler {
    config: SchedulerConfig,
    event_tx: broadcast::Sender<SchedulerEvent>,
    running: Arc<std::sync::atomic::AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl Scheduler {
    /// Cria um novo scheduler com configuração padrão
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }

    /// Cria um novo scheduler com configuração customizada
    pub fn with_config(config: SchedulerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            config,
            event_tx,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            handles: Vec::new(),
        }
    }

    /// Retorna um receiver para eventos do scheduler
    pub fn subscribe(&self) -> broadcast::Receiver<SchedulerEvent> {
        self.event_tx.subscribe()
    }

    /// Inicia o scheduler
    pub fn start(&mut self) {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            warn!("Scheduler já está rodando");
            return;
        }

        info!("Iniciando scheduler...");
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);

        // Emite evento de início
        let _ = self.event_tx.send(SchedulerEvent::Started);

        // Inicia task de Fear & Greed
        let handle = self.spawn_periodic_task(
            "FearGreed",
            Duration::from_secs(self.config.fear_greed_interval_secs),
            SchedulerEvent::CollectFearGreed,
        );
        self.handles.push(handle);

        // Inicia task de preços
        let handle = self.spawn_periodic_task(
            "Preços",
            Duration::from_secs(self.config.price_update_interval_secs),
            SchedulerEvent::UpdatePrices,
        );
        self.handles.push(handle);

        // Inicia task de posições
        let handle = self.spawn_periodic_task(
            "Posições",
            Duration::from_secs(self.config.position_check_interval_secs),
            SchedulerEvent::CheckPositions,
        );
        self.handles.push(handle);

        // Inicia task de estratégias
        let handle = self.spawn_periodic_task(
            "Estratégias",
            Duration::from_secs(self.config.strategy_interval_secs),
            SchedulerEvent::RunStrategies,
        );
        self.handles.push(handle);

        info!("Scheduler iniciado com {} tasks", self.handles.len());
    }

    /// Para o scheduler
    pub async fn stop(&mut self) {
        if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
            warn!("Scheduler não está rodando");
            return;
        }

        info!("Parando scheduler...");
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);

        // Aguarda todas as tasks terminarem
        for handle in self.handles.drain(..) {
            handle.abort();
        }

        // Emite evento de parada
        let _ = self.event_tx.send(SchedulerEvent::Stopped);

        info!("Scheduler parado");
    }

    /// Verifica se o scheduler está rodando
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Spawna uma task periódica
    fn spawn_periodic_task(
        &self,
        name: &'static str,
        interval: Duration,
        event: SchedulerEvent,
    ) -> JoinHandle<()> {
        let running = Arc::clone(&self.running);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            debug!("Task {} iniciada (intervalo: {:?})", name, interval);

            // Primeira execução imediata
            if running.load(std::sync::atomic::Ordering::SeqCst) {
                if let Err(e) = event_tx.send(event.clone()) {
                    error!("Erro ao enviar evento {}: {}", name, e);
                }
            }

            // Loop de execução periódica
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.tick().await; // Descarta o primeiro tick (já executamos acima)

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;

                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                debug!("Task {} executando...", name);
                if let Err(e) = event_tx.send(event.clone()) {
                    error!("Erro ao enviar evento {}: {}", name, e);
                }
            }

            debug!("Task {} encerrada", name);
        })
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Garante que as tasks são canceladas
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let mut scheduler = Scheduler::with_config(SchedulerConfig {
            fear_greed_interval_secs: 1,
            price_update_interval_secs: 1,
            position_check_interval_secs: 1,
            strategy_interval_secs: 1,
        });

        let mut rx = scheduler.subscribe();

        assert!(!scheduler.is_running());

        scheduler.start();
        assert!(scheduler.is_running());

        // Deve receber evento Started
        let event = timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(event.is_ok());
        if let Ok(Ok(SchedulerEvent::Started)) = event {
            // OK
        } else {
            panic!("Esperava evento Started");
        }

        scheduler.stop().await;
        assert!(!scheduler.is_running());
    }

    #[tokio::test]
    async fn test_scheduler_emits_events() {
        let mut scheduler = Scheduler::with_config(SchedulerConfig {
            fear_greed_interval_secs: 100,
            price_update_interval_secs: 100,
            position_check_interval_secs: 100,
            strategy_interval_secs: 100,
        });

        let mut rx = scheduler.subscribe();
        scheduler.start();

        // Coleta eventos por um curto período
        let mut events_received = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(500);

        while tokio::time::Instant::now() < deadline {
            match timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Ok(_)) => events_received += 1,
                _ => break,
            }
        }

        scheduler.stop().await;

        // Deve ter recebido pelo menos o evento Started + eventos iniciais das tasks
        assert!(events_received >= 1, "Recebeu {} eventos", events_received);
    }
}
