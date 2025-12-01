//! # RoboTrade Actors
//!
//! Actor Model runtime leve para o RoboTrade, usando `tokio::sync::mpsc` para message passing.
//!
//! ## Conceitos
//!
//! - **AppActor**: Trait principal que define um actor com nome, loop de execução e hooks de lifecycle
//! - **ActorHandle**: Handle para enviar mensagens a um actor
//! - **Supervisor**: Monitora actors filhos e implementa estratégias de supervisão
//! - **StopReason**: Enum indicando por que um actor parou (Normal, Shutdown, Error, Panic)
//!
//! ## Exemplo de Uso
//!
//! ```rust,ignore
//! use robotrade_actors::{AppActor, ActorHandle, spawn_actor, StopReason};
//! use tokio::sync::mpsc;
//!
//! #[derive(Debug)]
//! enum MyMessage {
//!     Ping,
//!     Shutdown,
//! }
//!
//! struct MyActor {
//!     count: u32,
//! }
//!
//! impl AppActor for MyActor {
//!     type Message = MyMessage;
//!
//!     fn name() -> &'static str { "MyActor" }
//!
//!     async fn run(mut self, mut rx: mpsc::Receiver<Self::Message>) -> StopReason {
//!         while let Some(msg) = rx.recv().await {
//!             match msg {
//!                 MyMessage::Ping => self.count += 1,
//!                 MyMessage::Shutdown => return StopReason::Shutdown,
//!             }
//!         }
//!         StopReason::Normal
//!     }
//! }
//! ```

mod actor;
mod handle;
mod supervisor;

pub use actor::{AppActor, StopReason, spawn_actor};
pub use handle::{ActorHandle, ActorError};
pub use supervisor::{Supervisor, SupervisionStrategy, SupervisionEvent};
