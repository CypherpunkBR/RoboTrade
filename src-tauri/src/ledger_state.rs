//! Ledger State and Service Integration
//!
//! This module provides the integration layer between the trading_worker's
//! LedgerService and the infrastructure repositories.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use robotrade_infra::database::DbPool;
use robotrade_infra::repositories::{LedgerRepository, SqliteLedgerRepository};
use robotrade_trading_worker::{
  ledger_service::{AssetBalance, LedgerEntry, LedgerFilters, LedgerService, LedgerStore},
  ws_manager::Exchange,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Adapter that implements LedgerStore using SqliteLedgerRepository
pub struct SqliteLedgerAdapter {
  repo: SqliteLedgerRepository,
}

impl SqliteLedgerAdapter {
  pub fn new(pool: DbPool) -> Self {
    Self {
      repo: SqliteLedgerRepository::new(pool),
    }
  }

  fn exchange_to_id(exchange: Exchange) -> &'static str {
    match exchange {
      Exchange::Binance => "binance_futures",
      Exchange::Kraken => "kraken_futures",
    }
  }

  fn id_to_exchange(id: &str) -> Exchange {
    match id {
      "binance_futures" | "binance" => Exchange::Binance,
      "kraken_futures" | "kraken" => Exchange::Kraken,
      // Default to Binance for unknown exchanges
      _ => Exchange::Binance,
    }
  }
}

#[async_trait]
impl LedgerStore for SqliteLedgerAdapter {
  async fn insert_entry(&self, entry: &LedgerEntry) -> Result<(), String> {
    use robotrade_core::entities::{
      LedgerEntry as CoreLedgerEntry, LedgerEntryId, LedgerEntryType as CoreLedgerEntryType,
      ReferenceType,
    };

    let exchange_id = Self::exchange_to_id(entry.exchange);

    let core_entry_type = match entry.entry_type {
      robotrade_trading_worker::ledger_service::LedgerEntryType::Deposit => {
        CoreLedgerEntryType::Deposit
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::Withdrawal => {
        CoreLedgerEntryType::Withdrawal
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::TradePnL => {
        CoreLedgerEntryType::TradePnl
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::Funding => {
        CoreLedgerEntryType::FundingPayment
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::Commission => {
        CoreLedgerEntryType::Fee
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::Transfer => {
        // Default to TransferIn; actual direction should be determined by amount sign
        CoreLedgerEntryType::TransferIn
      }
      robotrade_trading_worker::ledger_service::LedgerEntryType::Adjustment => {
        CoreLedgerEntryType::Adjustment
      }
    };

    let ref_type = entry.reference_type.as_ref().map(|rt| match rt.as_str() {
      "trade" => ReferenceType::Trade,
      "order" => ReferenceType::Order,
      "deposit" => ReferenceType::Deposit,
      "withdrawal" => ReferenceType::Withdrawal,
      "funding" => ReferenceType::Funding,
      _ => ReferenceType::Trade,
    });

    let core_entry = CoreLedgerEntry {
      id: LedgerEntryId::from_string(entry.id.clone()),
      exchange_id: exchange_id.to_string(),
      account_id: None,
      entry_type: core_entry_type,
      asset: entry.asset.clone(),
      amount: entry.amount,
      balance_after: entry.balance_after,
      reference_type: ref_type,
      reference_id: entry.reference_id.clone(),
      external_id: entry.reference_id.clone(), // Use reference_id as external_id for dedup
      description: entry.description.clone(),
      metadata: None,
      timestamp: entry.timestamp,
      created_at: entry.created_at,
    };

    self
      .repo
      .insert(&core_entry)
      .await
      .map_err(|e| format!("Failed to insert ledger entry: {}", e))
  }

  async fn insert_entries(&self, entries: &[LedgerEntry]) -> Result<(), String> {
    for entry in entries {
      self.insert_entry(entry).await?;
    }
    Ok(())
  }

  async fn get_entry(&self, id: &str) -> Option<LedgerEntry> {
    use robotrade_core::entities::LedgerEntryId;

    match self
      .repo
      .find_by_id(&LedgerEntryId::from_string(id.to_string()))
      .await
    {
      Ok(Some(e)) => Some(convert_core_to_service_entry(e)),
      _ => None,
    }
  }

  async fn get_entry_by_reference(
    &self,
    exchange: Exchange,
    _reference_type: &str,
    reference_id: &str,
  ) -> Option<LedgerEntry> {
    let exchange_id = Self::exchange_to_id(exchange);

    match self
      .repo
      .find_by_external_id(exchange_id, reference_id)
      .await
    {
      Ok(Some(e)) => Some(convert_core_to_service_entry(e)),
      _ => None,
    }
  }

  async fn get_entries(&self, filters: &LedgerFilters) -> Vec<LedgerEntry> {
    use robotrade_core::entities::LedgerFilters as CoreFilters;

    let exchange_id = filters
      .exchange
      .map(|e| Self::exchange_to_id(e).to_string())
      .unwrap_or_else(|| "binance_futures".to_string());

    let core_filters = CoreFilters {
      asset: filters.asset.clone(),
      entry_types: None, // TODO: Convert entry types
      reference_type: None,
      start_time: filters.start_time,
      end_time: filters.end_time,
      limit: filters.limit.map(|l| l as u32),
      offset: filters.offset.map(|o| o as u32),
    };

    match self
      .repo
      .find_with_filters(&exchange_id, &core_filters)
      .await
    {
      Ok(entries) => entries
        .into_iter()
        .map(convert_core_to_service_entry)
        .collect(),
      Err(e) => {
        error!("Failed to get ledger entries: {}", e);
        Vec::new()
      }
    }
  }

  async fn get_balance(&self, exchange: Exchange, asset: &str) -> Decimal {
    let exchange_id = Self::exchange_to_id(exchange);

    match self.repo.get_balance(exchange_id, asset).await {
      Ok(balance) => balance,
      Err(e) => {
        error!("Failed to get balance: {}", e);
        Decimal::ZERO
      }
    }
  }

  async fn get_all_balances(&self, exchange: Exchange) -> Vec<AssetBalance> {
    let exchange_id = Self::exchange_to_id(exchange);

    match self.repo.get_balance_summaries(exchange_id).await {
      Ok(summaries) => summaries
        .into_iter()
        .map(|s| AssetBalance {
          exchange,
          asset: s.asset,
          balance: s.balance,
          last_updated: s.last_update,
        })
        .collect(),
      Err(e) => {
        error!("Failed to get all balances: {}", e);
        Vec::new()
      }
    }
  }

  async fn get_last_entry(&self, exchange: Exchange, asset: &str) -> Option<LedgerEntry> {
    let exchange_id = Self::exchange_to_id(exchange);

    match self.repo.get_last_entry(exchange_id, asset).await {
      Ok(Some(e)) => Some(convert_core_to_service_entry(e)),
      _ => None,
    }
  }

  async fn count_entries(&self, filters: &LedgerFilters) -> u64 {
    let exchange_id = filters
      .exchange
      .map(|e| Self::exchange_to_id(e).to_string())
      .unwrap_or_else(|| "binance_futures".to_string());

    match self.repo.count(&exchange_id).await {
      Ok(count) => count,
      Err(e) => {
        error!("Failed to count entries: {}", e);
        0
      }
    }
  }
}

fn convert_core_to_service_entry(entry: robotrade_core::entities::LedgerEntry) -> LedgerEntry {
  use robotrade_core::entities::LedgerEntryType as CoreType;
  use robotrade_trading_worker::ledger_service::LedgerEntryType as ServiceType;

  let entry_type = match entry.entry_type {
    CoreType::Deposit => ServiceType::Deposit,
    CoreType::Withdrawal => ServiceType::Withdrawal,
    CoreType::TradePnl => ServiceType::TradePnL,
    CoreType::FundingPayment => ServiceType::Funding,
    CoreType::Fee => ServiceType::Commission,
    CoreType::TransferIn | CoreType::TransferOut => ServiceType::Transfer,
    CoreType::Adjustment => ServiceType::Adjustment,
    _ => ServiceType::Adjustment,
  };

  let exchange = match entry.exchange_id.as_str() {
    "binance_futures" | "binance" => Exchange::Binance,
    "kraken_futures" | "kraken" => Exchange::Kraken,
    // Default to Binance for unknown exchanges
    _ => Exchange::Binance,
  };

  LedgerEntry {
    id: entry.id.0,
    exchange,
    entry_type,
    asset: entry.asset,
    amount: entry.amount,
    balance_after: entry.balance_after,
    reference_id: entry.reference_id,
    reference_type: entry
      .reference_type
      .map(|r| format!("{:?}", r).to_lowercase()),
    description: entry.description,
    timestamp: entry.timestamp,
    created_at: entry.created_at,
  }
}

/// State container for ledger-related services
pub struct LedgerState {
  inner: Arc<RwLock<LedgerStateInner>>,
}

struct LedgerStateInner {
  service: Option<LedgerService<SqliteLedgerAdapter>>,
  pool: Option<DbPool>,
}

impl LedgerState {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(RwLock::new(LedgerStateInner {
        service: None,
        pool: None,
      })),
    }
  }

  /// Initialize with a database pool
  pub fn initialize(&self, pool: DbPool) {
    debug!("Initializing LedgerState with database pool");
    let adapter = SqliteLedgerAdapter::new(pool.clone());
    let service = LedgerService::new(adapter);

    // Use blocking_write since this is called from sync context during setup
    let mut inner = futures::executor::block_on(self.inner.write());
    inner.pool = Some(pool);
    inner.service = Some(service);
  }

  /// Check if initialized
  pub fn is_initialized(&self) -> bool {
    // Use try_read for non-blocking check
    if let Ok(inner) = self.inner.try_read() {
      inner.service.is_some()
    } else {
      false
    }
  }

  /// Get entries with filters
  pub async fn get_entries(&self, filters: &LedgerFilters) -> Vec<LedgerEntry> {
    let inner = self.inner.read().await;
    if let Some(ref service) = inner.service {
      service.get_entries(filters).await
    } else {
      Vec::new()
    }
  }

  /// Get balance for an asset
  pub async fn get_balance(&self, exchange: Exchange, asset: &str) -> Decimal {
    let inner = self.inner.read().await;
    if let Some(ref service) = inner.service {
      service.get_balance(exchange, asset).await
    } else {
      Decimal::ZERO
    }
  }

  /// Get all balances for an exchange
  pub async fn get_all_balances(&self, exchange: Exchange) -> Vec<AssetBalance> {
    let inner = self.inner.read().await;
    if let Some(ref service) = inner.service {
      service.get_all_balances(exchange).await
    } else {
      Vec::new()
    }
  }

  /// Get summary for an asset
  pub async fn get_summary(
    &self,
    exchange: Exchange,
    asset: &str,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
  ) -> Option<robotrade_trading_worker::ledger_service::LedgerSummary> {
    let inner = self.inner.read().await;
    if let Some(ref service) = inner.service {
      Some(
        service
          .get_summary(exchange, asset, start_time, end_time)
          .await,
      )
    } else {
      None
    }
  }

  /// Count entries
  pub async fn count_entries(&self, filters: &LedgerFilters) -> u64 {
    let inner = self.inner.read().await;
    if let Some(ref service) = inner.service {
      service.count_entries(filters).await
    } else {
      0
    }
  }
}

impl Default for LedgerState {
  fn default() -> Self {
    Self::new()
  }
}

impl Clone for LedgerState {
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}
