//! Actor trait e funções de spawn
//!
//! Define o trait `AppActor` que é a base para todos os actors do sistema.

use std::fmt::Debug;
use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::handle::ActorHandle;

/// Razão pela qual um actor parou de executar
#[derive(Debug)]
pub enum StopReason {
    /// Actor terminou normalmente (canal fechado, sem mais mensagens)
    Normal,
    /// Actor recebeu comando de shutdown
    Shutdown,
    /// Actor parou devido a um erro
    Error(String),
    /// Actor teve um panic
    Panic(String),
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopReason::Normal => write!(f, "Normal"),
            StopReason::Shutdown => write!(f, "Shutdown"),
            StopReason::Error(e) => write!(f, "Error: {}", e),
            StopReason::Panic(msg) => write!(f, "Panic: {}", msg),
        }
    }
}

/// Trait principal para actors da aplicação
///
/// # Exemplo
///
/// ```rust,ignore
/// struct PingActor;
///
/// impl AppActor for PingActor {
///     type Message = String;
///
///     fn name() -> &'static str { "PingActor" }
///
///     async fn run(self, mut rx: mpsc::Receiver<String>) -> StopReason {
///         while let Some(msg) = rx.recv().await {
///             println!("Received: {}", msg);
///         }
///         StopReason::Normal
///     }
/// }
/// ```
pub trait AppActor: Send + 'static {
    /// Tipo de mensagem que este actor processa
    type Message: Send + Debug + 'static;

    /// Nome do actor para logging e identificação
    fn name() -> &'static str;

    /// Loop principal do actor - processa mensagens até terminar
    ///
    /// O actor deve retornar um `StopReason` indicando por que parou.
    /// O canal será fechado quando não houver mais senders.
    fn run(
        self,
        rx: mpsc::Receiver<Self::Message>,
    ) -> impl Future<Output = StopReason> + Send;

    /// Hook chamado antes do actor começar a processar mensagens
    ///
    /// Use para inicialização que precisa ser feita dentro da task do actor.
    fn on_start(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }

    /// Hook chamado após o actor parar de processar mensagens
    ///
    /// Use para cleanup e logging do motivo da parada.
    fn on_stop(&self, reason: StopReason) -> impl Future<Output = ()> + Send
    where
        Self: Sync,
    {
        async move {
            match &reason {
                StopReason::Normal => {
                    info!("{} stopped normally", Self::name());
                }
                StopReason::Shutdown => {
                    info!("{} shutdown requested", Self::name());
                }
                StopReason::Error(e) => {
                    error!("{} stopped with error: {}", Self::name(), e);
                }
                StopReason::Panic(msg) => {
                    error!("{} panicked: {}", Self::name(), msg);
                }
            }
        }
    }
}

/// Spawna um actor e retorna um handle para enviar mensagens
///
/// # Argumentos
///
/// * `actor` - Instância do actor a ser executado
/// * `mailbox_size` - Tamanho do buffer do canal (número máximo de mensagens pendentes)
///
/// # Retorno
///
/// Tupla contendo:
/// * `ActorHandle` - Handle para enviar mensagens ao actor
/// * `JoinHandle` - Handle para aguardar o término do actor
///
/// # Exemplo
///
/// ```rust,ignore
/// let (handle, join) = spawn_actor(MyActor::new(), 1024);
/// handle.send(MyMessage::Ping).await?;
/// let reason = join.await?;
/// ```
pub fn spawn_actor<A: AppActor>(
    mut actor: A,
    mailbox_size: usize,
) -> (ActorHandle<A::Message>, tokio::task::JoinHandle<StopReason>) {
    let (tx, rx) = mpsc::channel(mailbox_size);
    let name = A::name();

    let handle = ActorHandle::new(tx, name);

    let join_handle = tokio::spawn(async move {
        info!("{} starting", name);

        // Chamar on_start antes de começar o loop
        actor.on_start().await;

        // Executar o loop principal com proteção contra panic
        let reason = AssertUnwindSafe(actor.run(rx))
            .catch_unwind()
            .await
            .unwrap_or_else(|panic| {
                let msg = panic
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "Unknown panic".to_string());
                StopReason::Panic(msg)
            });

        // Log do resultado
        match &reason {
            StopReason::Normal => info!("{} stopped normally", name),
            StopReason::Shutdown => info!("{} shutdown completed", name),
            StopReason::Error(e) => error!("{} stopped with error: {}", name, e),
            StopReason::Panic(msg) => error!("{} panicked: {}", name, msg),
        }

        reason
    });

    (handle, join_handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[derive(Debug)]
    enum TestMessage {
        Ping,
        Increment,
        GetCount(tokio::sync::oneshot::Sender<u32>),
        Shutdown,
    }

    struct TestActor {
        count: u32,
    }

    impl TestActor {
        fn new() -> Self {
            Self { count: 0 }
        }
    }

    impl AppActor for TestActor {
        type Message = TestMessage;

        fn name() -> &'static str {
            "TestActor"
        }

        async fn run(mut self, mut rx: mpsc::Receiver<Self::Message>) -> StopReason {
            while let Some(msg) = rx.recv().await {
                match msg {
                    TestMessage::Ping => {
                        // Apenas ignora
                    }
                    TestMessage::Increment => {
                        self.count += 1;
                    }
                    TestMessage::GetCount(reply) => {
                        let _ = reply.send(self.count);
                    }
                    TestMessage::Shutdown => {
                        return StopReason::Shutdown;
                    }
                }
            }
            StopReason::Normal
        }
    }

    #[tokio::test]
    async fn test_spawn_actor() {
        let (handle, _join) = spawn_actor(TestActor::new(), 16);

        // Enviar mensagem
        handle.send(TestMessage::Ping).await.unwrap();

        // Verificar contagem
        let (tx, rx) = tokio::sync::oneshot::channel();
        handle.send(TestMessage::GetCount(tx)).await.unwrap();
        let count = rx.await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_actor_increment() {
        let (handle, _join) = spawn_actor(TestActor::new(), 16);

        // Incrementar 3 vezes
        handle.send(TestMessage::Increment).await.unwrap();
        handle.send(TestMessage::Increment).await.unwrap();
        handle.send(TestMessage::Increment).await.unwrap();

        // Verificar contagem
        let (tx, rx) = tokio::sync::oneshot::channel();
        handle.send(TestMessage::GetCount(tx)).await.unwrap();
        let count = rx.await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_actor_shutdown() {
        let (handle, join) = spawn_actor(TestActor::new(), 16);

        // Enviar shutdown
        handle.send(TestMessage::Shutdown).await.unwrap();

        // Aguardar término
        let reason = timeout(Duration::from_secs(1), join)
            .await
            .expect("Timeout aguardando actor")
            .expect("Actor panicked");

        assert!(matches!(reason, StopReason::Shutdown));
    }

    #[tokio::test]
    async fn test_actor_normal_stop() {
        let (handle, join) = spawn_actor(TestActor::new(), 16);

        // Dropar o handle fecha o canal
        drop(handle);

        // Aguardar término
        let reason = timeout(Duration::from_secs(1), join)
            .await
            .expect("Timeout aguardando actor")
            .expect("Actor panicked");

        assert!(matches!(reason, StopReason::Normal));
    }

    #[derive(Debug)]
    struct PanicMessage;

    struct PanicActor;

    impl AppActor for PanicActor {
        type Message = PanicMessage;

        fn name() -> &'static str {
            "PanicActor"
        }

        async fn run(self, mut rx: mpsc::Receiver<Self::Message>) -> StopReason {
            if rx.recv().await.is_some() {
                panic!("Intentional panic for testing");
            }
            StopReason::Normal
        }
    }

    #[tokio::test]
    async fn test_actor_panic_recovery() {
        let (handle, join) = spawn_actor(PanicActor, 16);

        // Enviar mensagem que causa panic
        handle.send(PanicMessage).await.unwrap();

        // Aguardar término
        let reason = timeout(Duration::from_secs(1), join)
            .await
            .expect("Timeout aguardando actor")
            .expect("Actor task panicked");

        assert!(matches!(reason, StopReason::Panic(_)));
    }
}
