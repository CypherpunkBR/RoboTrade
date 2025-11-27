//! Implementações de repositórios SQLite
//!
//! Repositórios para persistência de dados usando SQLite.
//!
//! Este módulo contém as implementações de repositório para persistir
//! entidades de domínio no banco de dados SQLite.

mod candle;
mod cost_basis;
mod exchange;
mod fear_greed;
mod ledger;
mod reconciliation;
mod symbol;
mod sync_state;

pub use candle::*;
pub use cost_basis::*;
pub use exchange::*;
pub use fear_greed::*;
pub use ledger::*;
pub use reconciliation::*;
pub use symbol::*;
pub use sync_state::*;

// Os seguintes módulos de repositório serão implementados em uma próxima fase:
// - account: Gerenciamento de contas de trading
// - order: Persistência de ordens
// - position: Persistência de posições
// - trade: Histórico de trades
// - signal: Sinais de estratégia
// - strategy: Definições e instâncias de estratégia
// - risk: Limites e eventos de risco
// - job: Fila de jobs e tarefas agendadas
// - notification: Alertas e notificações
// - tax_report: Relatórios fiscais (a implementar)
