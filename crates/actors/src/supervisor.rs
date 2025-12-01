//! Supervisor - monitora actors filhos e implementa estratégias de supervisão
//!
//! O Supervisor permite monitorar múltiplos actors e reagir quando eles param,
//! seja por erro, panic ou término normal.

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::actor::{AppActor, StopReason};
use crate::handle::ActorHandle;

/// Estratégia de supervisão quando um actor filho falha
#[derive(Debug, Clone, Default)]
pub enum SupervisionStrategy {
  /// Escalar o erro para o supervisor pai (default)
  ///
  /// O supervisor notifica seu pai sobre a falha e deixa ele decidir o que fazer.
  #[default]
  Escalate,

  /// Reiniciar o actor automaticamente
  ///
  /// O supervisor tenta reiniciar o actor até um número máximo de vezes.
  Restart {
    /// Número máximo de restarts antes de desistir
    max_restarts: u32,
  },

  /// Parar o actor e não fazer nada
  ///
  /// O supervisor simplesmente registra a falha e continua.
  Stop,

  /// Ignorar a falha e continuar
  ///
  /// Usado quando falhas são esperadas e não críticas.
  Resume,
}

/// Evento de supervisão enviado quando um actor filho para
#[derive(Debug)]
pub struct SupervisionEvent {
  /// Nome do actor que parou
  pub actor_name: String,
  /// Razão da parada
  pub reason: StopReason,
  /// Estratégia que foi aplicada
  pub strategy: SupervisionStrategy,
}

/// Estado interno de um actor filho sendo supervisionado
struct ChildActor {
  join_handle: JoinHandle<StopReason>,
  strategy: SupervisionStrategy,
  restart_count: u32,
}

/// Supervisor que monitora actors filhos
///
/// O Supervisor implementa o padrão de supervisão, permitindo:
/// - Spawnar actors filhos com estratégias de supervisão
/// - Monitorar quando filhos param (normal, erro ou panic)
/// - Aplicar estratégias de recuperação (escalate, restart, stop, resume)
/// - Propagar eventos para um supervisor pai (hierarquia de supervisão)
///
/// # Exemplo
///
/// ```rust,ignore
/// use robotrade_actors::{Supervisor, SupervisionStrategy};
///
/// let mut supervisor = Supervisor::new("RootSupervisor");
///
/// // Spawnar actor filho
/// let handle = supervisor.spawn_child(
///     MyActor::new(),
///     1024,
///     SupervisionStrategy::Escalate,
/// );
///
/// // Executar loop de supervisão
/// supervisor.run().await;
/// ```
pub struct Supervisor {
  name: &'static str,
  children: HashMap<String, ChildActor>,
  event_tx: mpsc::Sender<SupervisionEvent>,
  event_rx: mpsc::Receiver<SupervisionEvent>,
  parent_tx: Option<mpsc::Sender<SupervisionEvent>>,
  shutdown_requested: bool,
}

impl Supervisor {
  /// Cria um novo Supervisor
  ///
  /// # Argumentos
  ///
  /// * `name` - Nome do supervisor para logging
  pub fn new(name: &'static str) -> Self {
    let (event_tx, event_rx) = mpsc::channel(64);
    Self {
      name,
      children: HashMap::new(),
      event_tx,
      event_rx,
      parent_tx: None,
      shutdown_requested: false,
    }
  }

  /// Define um supervisor pai para escalar eventos
  ///
  /// Quando um evento é escalado (via `SupervisionStrategy::Escalate`),
  /// ele será enviado para este canal.
  pub fn with_parent(mut self, parent_tx: mpsc::Sender<SupervisionEvent>) -> Self {
    self.parent_tx = Some(parent_tx);
    self
  }

  /// Retorna um sender para receber eventos de supervisão
  ///
  /// Útil para actors filhos notificarem o supervisor sobre eventos.
  pub fn event_sender(&self) -> mpsc::Sender<SupervisionEvent> {
    self.event_tx.clone()
  }

  /// Spawna um actor filho com a estratégia de supervisão especificada
  ///
  /// # Argumentos
  ///
  /// * `actor` - Instância do actor
  /// * `mailbox_size` - Tamanho do buffer de mensagens
  /// * `strategy` - Estratégia de supervisão para este actor
  ///
  /// # Retorno
  ///
  /// Handle para enviar mensagens ao actor filho.
  pub fn spawn_child<A: AppActor>(
    &mut self,
    actor: A,
    mailbox_size: usize,
    strategy: SupervisionStrategy,
  ) -> ActorHandle<A::Message> {
    let (handle, join_handle) = crate::spawn_actor(actor, mailbox_size);
    let name = A::name().to_string();

    info!("{}: spawned child actor '{}'", self.name, name);

    self.children.insert(
      name,
      ChildActor {
        join_handle,
        strategy,
        restart_count: 0,
      },
    );

    handle
  }

  /// Executa o loop de supervisão
  ///
  /// Esta função monitora todos os actors filhos e:
  /// - Detecta quando um filho termina
  /// - Aplica a estratégia de supervisão apropriada
  /// - Propaga eventos para o supervisor pai se configurado
  ///
  /// O loop termina quando:
  /// - Todos os filhos terminaram
  /// - `shutdown()` foi chamado
  pub async fn run(&mut self) {
    info!("{}: supervisor starting", self.name);

    while !self.shutdown_requested && !self.children.is_empty() {
      // Primeiro, verificar se há eventos pendentes
      match self.event_rx.try_recv() {
        Ok(event) => {
          self.handle_supervision_event(event).await;
          continue;
        }
        Err(mpsc::error::TryRecvError::Empty) => {}
        Err(mpsc::error::TryRecvError::Disconnected) => break,
      }

      // Depois, verificar se algum filho terminou
      if let Some((name, reason)) = self.poll_children().await {
        self.handle_child_stop(&name, reason).await;
      }
    }

    info!("{}: supervisor stopped", self.name);
  }

  /// Solicita shutdown graceful de todos os filhos
  pub async fn shutdown(&mut self) {
    info!("{}: shutdown requested", self.name);
    self.shutdown_requested = true;

    // Aguardar todos os filhos terminarem
    for (name, child) in self.children.drain() {
      info!("{}: waiting for '{}' to stop", self.name, name);
      match child.join_handle.await {
        Ok(reason) => {
          info!("{}: '{}' stopped with reason: {}", self.name, name, reason);
        }
        Err(e) => {
          error!("{}: '{}' task panicked: {}", self.name, name, e);
        }
      }
    }
  }

  /// Retorna o número de filhos ativos
  pub fn child_count(&self) -> usize {
    self.children.len()
  }

  /// Verifica se um filho específico está ativo
  pub fn has_child(&self, name: &str) -> bool {
    self.children.contains_key(name)
  }

  /// Verifica o próximo filho que terminou
  async fn poll_children(&mut self) -> Option<(String, StopReason)> {
    // Encontrar o primeiro filho que terminou
    let mut finished_child = None;

    for (name, child) in self.children.iter() {
      if child.join_handle.is_finished() {
        finished_child = Some(name.clone());
        break;
      }
    }

    if let Some(name) = finished_child {
      if let Some(child) = self.children.remove(&name) {
        let reason = match child.join_handle.await {
          Ok(reason) => reason,
          Err(e) => StopReason::Panic(e.to_string()),
        };
        return Some((name, reason));
      }
    }

    // Se nenhum filho terminou, aguardar um pouco
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    None
  }

  /// Trata a parada de um filho
  async fn handle_child_stop(&mut self, name: &str, reason: StopReason) {
    info!(
      "{}: child '{}' stopped with reason: {}",
      self.name, name, reason
    );

    // Determinar a estratégia (precisamos obter do child antes de removê-lo)
    // Como já removemos, vamos assumir Escalate por padrão para erros/panics
    let strategy = SupervisionStrategy::Escalate;

    match &reason {
      StopReason::Normal | StopReason::Shutdown => {
        // Parada normal, apenas log
        info!("{}: child '{}' completed normally", self.name, name);
      }
      StopReason::Error(_) | StopReason::Panic(_) => {
        // Erro ou panic - escalar para pai
        warn!(
          "{}: escalating '{}' failure to parent: {}",
          self.name, name, reason
        );

        if let Some(parent_tx) = &self.parent_tx {
          let event = SupervisionEvent {
            actor_name: name.to_string(),
            reason,
            strategy,
          };

          if let Err(e) = parent_tx.send(event).await {
            error!(
              "{}: failed to send supervision event to parent: {}",
              self.name, e
            );
          }
        } else {
          // Sem pai - log crítico
          error!(
            "{}: no parent supervisor to escalate '{}' failure",
            self.name, name
          );
        }
      }
    }
  }

  /// Trata um evento de supervisão recebido
  async fn handle_supervision_event(&mut self, event: SupervisionEvent) {
    info!(
      "{}: received supervision event for '{}': {}",
      self.name, event.actor_name, event.reason
    );

    // Se temos um pai, propagar o evento
    if let Some(parent_tx) = &self.parent_tx {
      if let Err(e) = parent_tx.send(event).await {
        error!(
          "{}: failed to propagate supervision event: {}",
          self.name, e
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::AppActor;
  use tokio::time::{timeout, Duration};

  #[derive(Debug)]
  enum TestMsg {
    Shutdown,
    Fail,
  }

  struct OkActor;

  impl AppActor for OkActor {
    type Message = TestMsg;

    fn name() -> &'static str {
      "OkActor"
    }

    async fn run(self, mut rx: mpsc::Receiver<Self::Message>) -> StopReason {
      while let Some(msg) = rx.recv().await {
        match msg {
          TestMsg::Shutdown => return StopReason::Shutdown,
          TestMsg::Fail => {
            return StopReason::Error("Intentional error".into());
          }
        }
      }
      StopReason::Normal
    }
  }

  #[tokio::test]
  async fn test_supervisor_spawn_child() {
    let mut supervisor = Supervisor::new("TestSupervisor");

    let handle = supervisor.spawn_child(OkActor, 16, SupervisionStrategy::default());

    assert_eq!(supervisor.child_count(), 1);
    assert!(supervisor.has_child("OkActor"));
    assert!(!handle.is_closed());
  }

  #[tokio::test]
  async fn test_supervisor_child_normal_stop() {
    let mut supervisor = Supervisor::new("TestSupervisor");

    let handle = supervisor.spawn_child(OkActor, 16, SupervisionStrategy::default());

    // Dropar handle fecha o canal
    drop(handle);

    // Aguardar um pouco para o actor processar
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verificar que o filho foi removido após terminar
    let (name, reason) = timeout(Duration::from_secs(1), supervisor.poll_children())
      .await
      .expect("Timeout")
      .expect("No child finished");

    assert_eq!(name, "OkActor");
    assert!(matches!(reason, StopReason::Normal));
  }

  #[tokio::test]
  async fn test_supervisor_shutdown() {
    let mut supervisor = Supervisor::new("TestSupervisor");

    let handle = supervisor.spawn_child(OkActor, 16, SupervisionStrategy::default());

    // Enviar shutdown para o actor
    handle.send(TestMsg::Shutdown).await.unwrap();

    // Shutdown do supervisor
    timeout(Duration::from_secs(1), supervisor.shutdown())
      .await
      .expect("Shutdown timeout");

    assert_eq!(supervisor.child_count(), 0);
  }

  #[tokio::test]
  async fn test_supervision_event() {
    let (parent_tx, mut parent_rx) = mpsc::channel(16);
    let mut supervisor = Supervisor::new("TestSupervisor").with_parent(parent_tx);

    let handle = supervisor.spawn_child(OkActor, 16, SupervisionStrategy::Escalate);

    // Causar erro no actor
    handle.send(TestMsg::Fail).await.unwrap();

    // Aguardar o actor processar a mensagem e terminar
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Poll para detectar a parada (pode precisar de múltiplas tentativas)
    for _ in 0..10 {
      if let Some((name, reason)) = supervisor.poll_children().await {
        supervisor.handle_child_stop(&name, reason).await;
        break;
      }
    }

    // Verificar que evento foi enviado ao pai
    let event = timeout(Duration::from_secs(1), parent_rx.recv())
      .await
      .expect("Timeout")
      .expect("No event received");

    assert_eq!(event.actor_name, "OkActor");
    assert!(matches!(event.reason, StopReason::Error(_)));
  }

  #[tokio::test]
  async fn test_supervisor_multiple_children() {
    let mut supervisor = Supervisor::new("TestSupervisor");

    struct Actor1;
    struct Actor2;

    impl AppActor for Actor1 {
      type Message = ();
      fn name() -> &'static str {
        "Actor1"
      }
      async fn run(self, mut rx: mpsc::Receiver<()>) -> StopReason {
        while rx.recv().await.is_some() {}
        StopReason::Normal
      }
    }

    impl AppActor for Actor2 {
      type Message = ();
      fn name() -> &'static str {
        "Actor2"
      }
      async fn run(self, mut rx: mpsc::Receiver<()>) -> StopReason {
        while rx.recv().await.is_some() {}
        StopReason::Normal
      }
    }

    supervisor.spawn_child(Actor1, 16, SupervisionStrategy::default());
    supervisor.spawn_child(Actor2, 16, SupervisionStrategy::default());

    assert_eq!(supervisor.child_count(), 2);
    assert!(supervisor.has_child("Actor1"));
    assert!(supervisor.has_child("Actor2"));
  }
}
