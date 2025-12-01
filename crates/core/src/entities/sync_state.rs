//! Entidades relacionadas ao estado de sincronização
//!
//! Rastreamento de progresso de sincronização de dados com exchanges.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// ID único de um estado de sync
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SyncStateId(pub String);

impl SyncStateId {
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }

  pub fn from_string(s: String) -> Self {
    Self(s)
  }

  /// Cria ID baseado em exchange e data_type (para uniqueness)
  pub fn for_exchange_data(exchange_id: &str, data_type: SyncDataType) -> Self {
    Self(format!("{}:{}", exchange_id, data_type))
  }
}

impl Default for SyncStateId {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for SyncStateId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for SyncStateId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Tipo de dados sendo sincronizados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDataType {
  /// Trades executados
  Trades,
  /// Ordens (abertas e históricas)
  Orders,
  /// Posições abertas
  Positions,
  /// Pagamentos de funding
  Funding,
  /// Depósitos
  Deposits,
  /// Saques
  Withdrawals,
  /// Saldos
  Balances,
  /// Receitas (income history)
  Income,
}

impl fmt::Display for SyncDataType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SyncDataType::Trades => write!(f, "trades"),
      SyncDataType::Orders => write!(f, "orders"),
      SyncDataType::Positions => write!(f, "positions"),
      SyncDataType::Funding => write!(f, "funding"),
      SyncDataType::Deposits => write!(f, "deposits"),
      SyncDataType::Withdrawals => write!(f, "withdrawals"),
      SyncDataType::Balances => write!(f, "balances"),
      SyncDataType::Income => write!(f, "income"),
    }
  }
}

impl std::str::FromStr for SyncDataType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "trades" => Ok(SyncDataType::Trades),
      "orders" => Ok(SyncDataType::Orders),
      "positions" => Ok(SyncDataType::Positions),
      "funding" => Ok(SyncDataType::Funding),
      "deposits" => Ok(SyncDataType::Deposits),
      "withdrawals" => Ok(SyncDataType::Withdrawals),
      "balances" => Ok(SyncDataType::Balances),
      "income" => Ok(SyncDataType::Income),
      _ => Err(format!("Unknown sync data type: {}", s)),
    }
  }
}

/// Status da sincronização
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStatus {
  /// Pronto para sincronizar
  Idle,
  /// Sincronização em progresso
  Syncing,
  /// Pausado
  Paused,
  /// Erro na sincronização
  Error,
  /// Sincronização completa
  Completed,
}

impl fmt::Display for SyncStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SyncStatus::Idle => write!(f, "IDLE"),
      SyncStatus::Syncing => write!(f, "SYNCING"),
      SyncStatus::Paused => write!(f, "PAUSED"),
      SyncStatus::Error => write!(f, "ERROR"),
      SyncStatus::Completed => write!(f, "COMPLETED"),
    }
  }
}

impl std::str::FromStr for SyncStatus {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "IDLE" => Ok(SyncStatus::Idle),
      "SYNCING" => Ok(SyncStatus::Syncing),
      "PAUSED" => Ok(SyncStatus::Paused),
      "ERROR" => Ok(SyncStatus::Error),
      "COMPLETED" => Ok(SyncStatus::Completed),
      _ => Err(format!("Unknown sync status: {}", s)),
    }
  }
}

/// Estado de sincronização para um tipo de dados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
  /// ID único
  pub id: SyncStateId,
  /// ID da exchange
  pub exchange_id: String,
  /// Tipo de dados sendo sincronizados
  pub data_type: SyncDataType,
  /// Último ID sincronizado
  pub last_sync_id: Option<String>,
  /// Último timestamp sincronizado
  pub last_sync_timestamp: Option<DateTime<Utc>>,
  /// Cursor de paginação (se necessário)
  pub cursor: Option<String>,
  /// Início do range sincronizado
  pub sync_start_time: Option<DateTime<Utc>>,
  /// Fim do range sincronizado
  pub sync_end_time: Option<DateTime<Utc>>,
  /// Status atual
  pub status: SyncStatus,
  /// Mensagem de erro (se houver)
  pub error_message: Option<String>,
  /// Contagem de retries
  pub retry_count: u32,
  /// Total de registros a sincronizar
  pub total_records: Option<u64>,
  /// Registros já processados
  pub processed_records: Option<u64>,
  /// Última sincronização bem sucedida
  pub last_success_at: Option<DateTime<Utc>>,
  /// Última atualização
  pub updated_at: DateTime<Utc>,
}

impl SyncState {
  /// Cria um novo estado de sincronização
  pub fn new(exchange_id: String, data_type: SyncDataType) -> Self {
    Self {
      id: SyncStateId::for_exchange_data(&exchange_id, data_type),
      exchange_id,
      data_type,
      last_sync_id: None,
      last_sync_timestamp: None,
      cursor: None,
      sync_start_time: None,
      sync_end_time: None,
      status: SyncStatus::Idle,
      error_message: None,
      retry_count: 0,
      total_records: None,
      processed_records: None,
      last_success_at: None,
      updated_at: Utc::now(),
    }
  }

  /// Inicia sincronização
  pub fn start_sync(&mut self) {
    self.status = SyncStatus::Syncing;
    self.error_message = None;
    self.processed_records = Some(0);
    self.updated_at = Utc::now();
  }

  /// Atualiza progresso
  pub fn update_progress(&mut self, processed: u64, total: Option<u64>) {
    self.processed_records = Some(processed);
    if let Some(t) = total {
      self.total_records = Some(t);
    }
    self.updated_at = Utc::now();
  }

  /// Atualiza cursor de paginação
  pub fn update_cursor(
    &mut self,
    cursor: Option<String>,
    last_id: Option<String>,
    last_ts: Option<DateTime<Utc>>,
  ) {
    self.cursor = cursor;
    if let Some(id) = last_id {
      self.last_sync_id = Some(id);
    }
    if let Some(ts) = last_ts {
      self.last_sync_timestamp = Some(ts);
    }
    self.updated_at = Utc::now();
  }

  /// Marca como concluído com sucesso
  pub fn complete(&mut self) {
    self.status = SyncStatus::Completed;
    self.error_message = None;
    self.retry_count = 0;
    self.cursor = None;
    self.last_success_at = Some(Utc::now());
    self.updated_at = Utc::now();
  }

  /// Marca como erro
  pub fn mark_error(&mut self, message: String) {
    self.status = SyncStatus::Error;
    self.error_message = Some(message);
    self.retry_count += 1;
    self.updated_at = Utc::now();
  }

  /// Pausa sincronização
  pub fn pause(&mut self) {
    self.status = SyncStatus::Paused;
    self.updated_at = Utc::now();
  }

  /// Reseta para idle
  pub fn reset(&mut self) {
    self.status = SyncStatus::Idle;
    self.error_message = None;
    self.retry_count = 0;
    self.cursor = None;
    self.processed_records = None;
    self.total_records = None;
    self.updated_at = Utc::now();
  }

  /// Calcula progresso percentual
  pub fn progress_percentage(&self) -> Option<f64> {
    match (self.processed_records, self.total_records) {
      (Some(processed), Some(total)) if total > 0 => {
        Some((processed as f64 / total as f64) * 100.0)
      }
      _ => None,
    }
  }

  /// Verifica se pode fazer retry
  pub fn can_retry(&self, max_retries: u32) -> bool {
    self.status == SyncStatus::Error && self.retry_count < max_retries
  }

  /// Verifica se está sincronizando
  pub fn is_syncing(&self) -> bool {
    self.status == SyncStatus::Syncing
  }

  /// Verifica se está em erro
  pub fn is_error(&self) -> bool {
    self.status == SyncStatus::Error
  }
}

/// Progresso geral de sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
  /// ID da exchange
  pub exchange_id: String,
  /// Estados por tipo de dados
  pub states: Vec<SyncState>,
  /// Progresso geral (0-100)
  pub overall_progress: f64,
  /// Se há erros
  pub has_errors: bool,
  /// Se está sincronizando algo
  pub is_syncing: bool,
}

impl SyncProgress {
  /// Cria novo progresso a partir de estados
  pub fn from_states(exchange_id: String, states: Vec<SyncState>) -> Self {
    let has_errors = states.iter().any(|s| s.is_error());
    let is_syncing = states.iter().any(|s| s.is_syncing());

    // Calcula progresso geral
    let progresses: Vec<f64> = states
      .iter()
      .filter_map(|s| s.progress_percentage())
      .collect();

    let overall_progress = if progresses.is_empty() {
      0.0
    } else {
      progresses.iter().sum::<f64>() / progresses.len() as f64
    };

    Self {
      exchange_id,
      states,
      overall_progress,
      has_errors,
      is_syncing,
    }
  }
}

/// Relatório de sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
  /// ID da exchange
  pub exchange_id: String,
  /// Trades sincronizados
  pub trades_synced: u64,
  /// Orders sincronizadas
  pub orders_synced: u64,
  /// Positions sincronizadas
  pub positions_synced: u64,
  /// Funding payments sincronizados
  pub funding_synced: u64,
  /// Depósitos sincronizados
  pub deposits_synced: u64,
  /// Saques sincronizados
  pub withdrawals_synced: u64,
  /// Erros encontrados
  pub errors: Vec<String>,
  /// Duração total
  pub duration_secs: u64,
  /// Timestamp de início
  pub started_at: DateTime<Utc>,
  /// Timestamp de fim
  pub completed_at: DateTime<Utc>,
}

impl SyncReport {
  /// Cria um novo relatório vazio
  pub fn new(exchange_id: String) -> Self {
    let now = Utc::now();
    Self {
      exchange_id,
      trades_synced: 0,
      orders_synced: 0,
      positions_synced: 0,
      funding_synced: 0,
      deposits_synced: 0,
      withdrawals_synced: 0,
      errors: Vec::new(),
      duration_secs: 0,
      started_at: now,
      completed_at: now,
    }
  }

  /// Adiciona erro ao relatório
  pub fn add_error(&mut self, error: String) {
    self.errors.push(error);
  }

  /// Finaliza relatório
  pub fn finalize(&mut self) {
    self.completed_at = Utc::now();
    self.duration_secs = (self.completed_at - self.started_at).num_seconds() as u64;
  }

  /// Total de registros sincronizados
  pub fn total_synced(&self) -> u64 {
    self.trades_synced
      + self.orders_synced
      + self.positions_synced
      + self.funding_synced
      + self.deposits_synced
      + self.withdrawals_synced
  }

  /// Verifica se teve sucesso (sem erros)
  pub fn is_success(&self) -> bool {
    self.errors.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sync_state_creation() {
    let state = SyncState::new("binance".to_string(), SyncDataType::Trades);

    assert_eq!(state.exchange_id, "binance");
    assert_eq!(state.data_type, SyncDataType::Trades);
    assert_eq!(state.status, SyncStatus::Idle);
  }

  #[test]
  fn test_sync_state_lifecycle() {
    let mut state = SyncState::new("binance".to_string(), SyncDataType::Trades);

    // Start
    state.start_sync();
    assert!(state.is_syncing());

    // Update progress
    state.update_progress(50, Some(100));
    assert_eq!(state.progress_percentage(), Some(50.0));

    // Complete
    state.complete();
    assert_eq!(state.status, SyncStatus::Completed);
    assert!(state.last_success_at.is_some());
  }

  #[test]
  fn test_sync_state_error_retry() {
    let mut state = SyncState::new("binance".to_string(), SyncDataType::Trades);

    state.start_sync();
    state.mark_error("Connection failed".to_string());

    assert!(state.is_error());
    assert_eq!(state.retry_count, 1);
    assert!(state.can_retry(3));

    state.mark_error("Connection failed again".to_string());
    state.mark_error("Still failing".to_string());

    assert!(!state.can_retry(3)); // Max retries reached
  }

  #[test]
  fn test_sync_report() {
    let mut report = SyncReport::new("binance".to_string());

    report.trades_synced = 100;
    report.orders_synced = 50;
    report.finalize();

    assert_eq!(report.total_synced(), 150);
    assert!(report.is_success());
    assert!(report.duration_secs >= 0);
  }

  #[test]
  fn test_sync_data_type_parsing() {
    assert_eq!(
      "trades".parse::<SyncDataType>().unwrap(),
      SyncDataType::Trades
    );
    assert_eq!(
      "FUNDING".parse::<SyncDataType>().unwrap(),
      SyncDataType::Funding
    );
  }
}
