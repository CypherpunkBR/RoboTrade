//! Actor Handle - referência para enviar mensagens a um actor
//!
//! O `ActorHandle` é um handle clonável que permite enviar mensagens para um actor.

use std::fmt::Debug;
use thiserror::Error;
use tokio::sync::mpsc;

/// Erros que podem ocorrer ao interagir com actors
#[derive(Debug, Error)]
pub enum ActorError {
  /// O canal do actor foi fechado (actor parou)
  #[error("Actor channel closed - actor may have stopped")]
  ChannelClosed,

  /// O mailbox do actor está cheio (try_send falhou)
  #[error("Actor mailbox full - actor may be overloaded")]
  MailboxFull,

  /// Actor não encontrado (usado em registries)
  #[error("Actor not found: {0}")]
  NotFound(String),

  /// Timeout aguardando resposta
  #[error("Timeout waiting for actor response")]
  Timeout,
}

/// Handle para enviar mensagens a um actor
///
/// O handle é clonável, permitindo que múltiplos produtores enviem mensagens
/// para o mesmo actor.
///
/// # Exemplo
///
/// ```rust,ignore
/// let handle: ActorHandle<MyMessage> = ...;
///
/// // Enviar e aguardar espaço no buffer (blocking se cheio)
/// handle.send(MyMessage::Ping).await?;
///
/// // Tentar enviar sem bloquear (retorna erro se cheio)
/// handle.try_send(MyMessage::Ping)?;
///
/// // Verificar se actor ainda está ativo
/// if !handle.is_closed() {
///     handle.send(MyMessage::Ping).await?;
/// }
/// ```
#[derive(Clone)]
pub struct ActorHandle<M: Send + 'static> {
  tx: mpsc::Sender<M>,
  name: &'static str,
}

impl<M: Send + Debug + 'static> ActorHandle<M> {
  /// Cria um novo handle para um actor
  pub(crate) fn new(tx: mpsc::Sender<M>, name: &'static str) -> Self {
    Self { tx, name }
  }

  /// Envia uma mensagem para o actor, aguardando se o buffer estiver cheio
  ///
  /// Esta função só retorna quando:
  /// - A mensagem foi colocada no buffer com sucesso
  /// - O canal foi fechado (actor parou)
  ///
  /// # Erros
  ///
  /// Retorna `ActorError::ChannelClosed` se o actor já parou.
  pub async fn send(&self, msg: M) -> Result<(), ActorError> {
    self
      .tx
      .send(msg)
      .await
      .map_err(|_| ActorError::ChannelClosed)
  }

  /// Tenta enviar uma mensagem sem bloquear
  ///
  /// Útil quando você não quer aguardar espaço no buffer.
  ///
  /// # Erros
  ///
  /// - `ActorError::MailboxFull` - O buffer está cheio
  /// - `ActorError::ChannelClosed` - O actor já parou
  pub fn try_send(&self, msg: M) -> Result<(), ActorError> {
    self.tx.try_send(msg).map_err(|e| match e {
      mpsc::error::TrySendError::Full(_) => ActorError::MailboxFull,
      mpsc::error::TrySendError::Closed(_) => ActorError::ChannelClosed,
    })
  }

  /// Retorna o nome do actor
  pub fn name(&self) -> &'static str {
    self.name
  }

  /// Verifica se o canal foi fechado (actor parou)
  pub fn is_closed(&self) -> bool {
    self.tx.is_closed()
  }

  /// Retorna a capacidade atual do buffer
  ///
  /// Útil para monitoramento de backpressure.
  pub fn capacity(&self) -> usize {
    self.tx.capacity()
  }

  /// Retorna o número máximo de mensagens que podem ser enfileiradas
  pub fn max_capacity(&self) -> usize {
    self.tx.max_capacity()
  }
}

impl<M: Send + Debug + 'static> Debug for ActorHandle<M> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ActorHandle")
      .field("name", &self.name)
      .field("is_closed", &self.is_closed())
      .field("capacity", &self.capacity())
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_handle_send() {
    let (tx, mut rx) = mpsc::channel::<String>(16);
    let handle = ActorHandle::new(tx, "TestActor");

    handle.send("hello".to_string()).await.unwrap();

    let msg = rx.recv().await.unwrap();
    assert_eq!(msg, "hello");
  }

  #[tokio::test]
  async fn test_handle_try_send() {
    let (tx, mut rx) = mpsc::channel::<String>(16);
    let handle = ActorHandle::new(tx, "TestActor");

    handle.try_send("hello".to_string()).unwrap();

    let msg = rx.recv().await.unwrap();
    assert_eq!(msg, "hello");
  }

  #[tokio::test]
  async fn test_handle_try_send_full() {
    let (tx, _rx) = mpsc::channel::<String>(1);
    let handle = ActorHandle::new(tx, "TestActor");

    // Primeiro sucede
    handle.try_send("first".to_string()).unwrap();

    // Segundo falha (buffer cheio)
    let result = handle.try_send("second".to_string());
    assert!(matches!(result, Err(ActorError::MailboxFull)));
  }

  #[tokio::test]
  async fn test_handle_closed() {
    let (tx, rx) = mpsc::channel::<String>(16);
    let handle = ActorHandle::new(tx, "TestActor");

    assert!(!handle.is_closed());

    // Dropar receiver fecha o canal
    drop(rx);

    assert!(handle.is_closed());

    // Enviar para canal fechado falha
    let result = handle.send("hello".to_string()).await;
    assert!(matches!(result, Err(ActorError::ChannelClosed)));
  }

  #[tokio::test]
  async fn test_handle_name() {
    let (tx, _rx) = mpsc::channel::<String>(16);
    let handle = ActorHandle::new(tx, "MyActor");

    assert_eq!(handle.name(), "MyActor");
  }

  #[tokio::test]
  async fn test_handle_clone() {
    let (tx, mut rx) = mpsc::channel::<String>(16);
    let handle1 = ActorHandle::new(tx, "TestActor");
    let handle2 = handle1.clone();

    // Ambos handles podem enviar
    handle1.send("from_1".to_string()).await.unwrap();
    handle2.send("from_2".to_string()).await.unwrap();

    let msg1 = rx.recv().await.unwrap();
    let msg2 = rx.recv().await.unwrap();

    assert_eq!(msg1, "from_1");
    assert_eq!(msg2, "from_2");
  }

  #[tokio::test]
  async fn test_handle_capacity() {
    let (tx, _rx) = mpsc::channel::<String>(32);
    let handle = ActorHandle::new(tx, "TestActor");

    assert_eq!(handle.max_capacity(), 32);
    assert_eq!(handle.capacity(), 32);

    handle.try_send("msg".to_string()).unwrap();
    assert_eq!(handle.capacity(), 31);
  }
}
