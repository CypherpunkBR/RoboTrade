//! Ledger Service
//!
//! Business logic for ledger operations:
//! - Processing trades into ledger entries
//! - Processing deposits/withdrawals
//! - Processing funding payments
//! - Balance calculations
//! - Ledger queries and reports

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use super::sync_engine::{SyncedDeposit, SyncedFunding, SyncedTrade, SyncedWithdrawal};
use super::ws_manager::{Exchange, TradeEvent};

/// Ledger entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LedgerEntryType {
    /// Deposit
    Deposit,
    /// Withdrawal
    Withdrawal,
    /// Trade realized P&L
    TradePnL,
    /// Funding payment
    Funding,
    /// Commission/Fee
    Commission,
    /// Transfer between accounts
    Transfer,
    /// Adjustment
    Adjustment,
}

impl std::fmt::Display for LedgerEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerEntryType::Deposit => write!(f, "DEPOSIT"),
            LedgerEntryType::Withdrawal => write!(f, "WITHDRAWAL"),
            LedgerEntryType::TradePnL => write!(f, "TRADE_PNL"),
            LedgerEntryType::Funding => write!(f, "FUNDING"),
            LedgerEntryType::Commission => write!(f, "COMMISSION"),
            LedgerEntryType::Transfer => write!(f, "TRANSFER"),
            LedgerEntryType::Adjustment => write!(f, "ADJUSTMENT"),
        }
    }
}

/// Ledger entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub exchange: Exchange,
    pub entry_type: LedgerEntryType,
    pub asset: String,
    pub amount: Decimal,
    pub balance_after: Decimal,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Balance summary for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub exchange: Exchange,
    pub asset: String,
    pub balance: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Ledger filters
#[derive(Debug, Clone, Default)]
pub struct LedgerFilters {
    pub exchange: Option<Exchange>,
    pub entry_type: Option<LedgerEntryType>,
    pub asset: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Ledger summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    pub exchange: Exchange,
    pub asset: String,
    pub total_deposits: Decimal,
    pub total_withdrawals: Decimal,
    pub total_realized_pnl: Decimal,
    pub total_funding: Decimal,
    pub total_commissions: Decimal,
    pub net_change: Decimal,
    pub current_balance: Decimal,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

/// Trait for ledger data persistence
#[async_trait::async_trait]
pub trait LedgerStore: Send + Sync {
    /// Insert a ledger entry
    async fn insert_entry(&self, entry: &LedgerEntry) -> Result<(), String>;

    /// Insert multiple ledger entries
    async fn insert_entries(&self, entries: &[LedgerEntry]) -> Result<(), String>;

    /// Get entry by ID
    async fn get_entry(&self, id: &str) -> Option<LedgerEntry>;

    /// Get entry by reference
    async fn get_entry_by_reference(
        &self,
        exchange: Exchange,
        reference_type: &str,
        reference_id: &str,
    ) -> Option<LedgerEntry>;

    /// Get entries with filters
    async fn get_entries(&self, filters: &LedgerFilters) -> Vec<LedgerEntry>;

    /// Get current balance for an asset
    async fn get_balance(&self, exchange: Exchange, asset: &str) -> Decimal;

    /// Get all balances for an exchange
    async fn get_all_balances(&self, exchange: Exchange) -> Vec<AssetBalance>;

    /// Get last entry for an asset (for balance calculation)
    async fn get_last_entry(&self, exchange: Exchange, asset: &str) -> Option<LedgerEntry>;

    /// Count entries with filters
    async fn count_entries(&self, filters: &LedgerFilters) -> u64;
}

/// Ledger Service
pub struct LedgerService<S: LedgerStore> {
    store: S,
}

impl<S: LedgerStore> LedgerService<S> {
    /// Create a new ledger service
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Process a trade and create ledger entries
    ///
    /// Creates entries for:
    /// - Commission payment (debit)
    /// - Realized P&L (if any)
    pub async fn process_trade(&self, trade: &SyncedTrade) -> Result<Vec<LedgerEntry>, String> {
        let mut entries = Vec::new();
        let timestamp = trade.timestamp;

        // Check if already processed
        if self
            .store
            .get_entry_by_reference(trade.exchange, "trade", &trade.trade_id)
            .await
            .is_some()
        {
            debug!("Trade {} already processed", trade.trade_id);
            return Ok(entries);
        }

        // 1. Commission entry (always negative)
        if trade.commission > Decimal::ZERO {
            let current_balance = self
                .store
                .get_balance(trade.exchange, &trade.commission_asset)
                .await;

            let commission_entry = LedgerEntry {
                id: Uuid::new_v4().to_string(),
                exchange: trade.exchange,
                entry_type: LedgerEntryType::Commission,
                asset: trade.commission_asset.clone(),
                amount: -trade.commission, // Negative for expense
                balance_after: current_balance - trade.commission,
                reference_id: Some(trade.trade_id.clone()),
                reference_type: Some("trade".to_string()),
                description: Some(format!(
                    "Commission for {} {} {}",
                    trade.side, trade.quantity, trade.symbol
                )),
                timestamp,
                created_at: Utc::now(),
            };

            self.store.insert_entry(&commission_entry).await?;
            entries.push(commission_entry);
        }

        // 2. Realized P&L entry (if any)
        if let Some(pnl) = trade.realized_pnl {
            if pnl != Decimal::ZERO {
                // Assuming P&L is in USD/USDT
                let pnl_asset = "USDT".to_string();
                let current_balance = self.store.get_balance(trade.exchange, &pnl_asset).await;

                let pnl_entry = LedgerEntry {
                    id: Uuid::new_v4().to_string(),
                    exchange: trade.exchange,
                    entry_type: LedgerEntryType::TradePnL,
                    asset: pnl_asset,
                    amount: pnl,
                    balance_after: current_balance + pnl,
                    reference_id: Some(trade.trade_id.clone()),
                    reference_type: Some("trade".to_string()),
                    description: Some(format!(
                        "Realized P&L for {} {} {} @ {}",
                        trade.side, trade.quantity, trade.symbol, trade.price
                    )),
                    timestamp,
                    created_at: Utc::now(),
                };

                self.store.insert_entry(&pnl_entry).await?;
                entries.push(pnl_entry);
            }
        }

        Ok(entries)
    }

    /// Process a real-time trade event
    pub async fn process_trade_event(&self, event: &TradeEvent) -> Result<Vec<LedgerEntry>, String> {
        let synced_trade = SyncedTrade {
            exchange: event.exchange,
            trade_id: event.trade_id.clone(),
            order_id: event.order_id.clone(),
            symbol: event.symbol.clone(),
            side: format!("{:?}", event.side),
            price: event.price,
            quantity: event.quantity,
            commission: event.commission,
            commission_asset: event.commission_asset.clone(),
            realized_pnl: event.realized_pnl,
            timestamp: event.timestamp,
        };

        self.process_trade(&synced_trade).await
    }

    /// Process a deposit
    pub async fn process_deposit(&self, deposit: &SyncedDeposit) -> Result<LedgerEntry, String> {
        // Check if already processed
        if self
            .store
            .get_entry_by_reference(deposit.exchange, "deposit", &deposit.deposit_id)
            .await
            .is_some()
        {
            return Err(format!("Deposit {} already processed", deposit.deposit_id));
        }

        let current_balance = self
            .store
            .get_balance(deposit.exchange, &deposit.asset)
            .await;

        let entry = LedgerEntry {
            id: Uuid::new_v4().to_string(),
            exchange: deposit.exchange,
            entry_type: LedgerEntryType::Deposit,
            asset: deposit.asset.clone(),
            amount: deposit.amount,
            balance_after: current_balance + deposit.amount,
            reference_id: Some(deposit.deposit_id.clone()),
            reference_type: Some("deposit".to_string()),
            description: deposit.tx_id.clone().map(|tx| format!("Deposit TX: {}", tx)),
            timestamp: deposit.timestamp,
            created_at: Utc::now(),
        };

        self.store.insert_entry(&entry).await?;
        Ok(entry)
    }

    /// Process a withdrawal
    pub async fn process_withdrawal(
        &self,
        withdrawal: &SyncedWithdrawal,
    ) -> Result<LedgerEntry, String> {
        // Check if already processed
        if self
            .store
            .get_entry_by_reference(withdrawal.exchange, "withdrawal", &withdrawal.withdrawal_id)
            .await
            .is_some()
        {
            return Err(format!(
                "Withdrawal {} already processed",
                withdrawal.withdrawal_id
            ));
        }

        let current_balance = self
            .store
            .get_balance(withdrawal.exchange, &withdrawal.asset)
            .await;

        // Total debit = amount + fee
        let total_amount = -(withdrawal.amount + withdrawal.fee);

        let entry = LedgerEntry {
            id: Uuid::new_v4().to_string(),
            exchange: withdrawal.exchange,
            entry_type: LedgerEntryType::Withdrawal,
            asset: withdrawal.asset.clone(),
            amount: total_amount,
            balance_after: current_balance + total_amount,
            reference_id: Some(withdrawal.withdrawal_id.clone()),
            reference_type: Some("withdrawal".to_string()),
            description: Some(format!(
                "Withdrawal to {} (fee: {})",
                withdrawal.address.as_deref().unwrap_or("unknown"),
                withdrawal.fee
            )),
            timestamp: withdrawal.timestamp,
            created_at: Utc::now(),
        };

        self.store.insert_entry(&entry).await?;
        Ok(entry)
    }

    /// Process a funding payment
    pub async fn process_funding(&self, funding: &SyncedFunding) -> Result<LedgerEntry, String> {
        // Check if already processed
        if self
            .store
            .get_entry_by_reference(funding.exchange, "funding", &funding.funding_id)
            .await
            .is_some()
        {
            return Err(format!("Funding {} already processed", funding.funding_id));
        }

        // Funding is typically in USD/USDT
        let asset = "USDT".to_string();
        let current_balance = self.store.get_balance(funding.exchange, &asset).await;

        let entry = LedgerEntry {
            id: Uuid::new_v4().to_string(),
            exchange: funding.exchange,
            entry_type: LedgerEntryType::Funding,
            asset,
            amount: funding.amount,
            balance_after: current_balance + funding.amount,
            reference_id: Some(funding.funding_id.clone()),
            reference_type: Some("funding".to_string()),
            description: Some(format!(
                "Funding {} @ {} (position: {})",
                funding.symbol, funding.funding_rate, funding.position_size
            )),
            timestamp: funding.timestamp,
            created_at: Utc::now(),
        };

        self.store.insert_entry(&entry).await?;
        Ok(entry)
    }

    /// Get ledger entries with filters
    pub async fn get_entries(&self, filters: &LedgerFilters) -> Vec<LedgerEntry> {
        self.store.get_entries(filters).await
    }

    /// Get current balance for an asset
    pub async fn get_balance(&self, exchange: Exchange, asset: &str) -> Decimal {
        self.store.get_balance(exchange, asset).await
    }

    /// Get all balances for an exchange
    pub async fn get_all_balances(&self, exchange: Exchange) -> Vec<AssetBalance> {
        self.store.get_all_balances(exchange).await
    }

    /// Get ledger summary for an asset
    pub async fn get_summary(
        &self,
        exchange: Exchange,
        asset: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> LedgerSummary {
        let filters = LedgerFilters {
            exchange: Some(exchange),
            asset: Some(asset.to_string()),
            start_time,
            end_time,
            ..Default::default()
        };

        let entries = self.store.get_entries(&filters).await;

        let mut summary = LedgerSummary {
            exchange,
            asset: asset.to_string(),
            total_deposits: Decimal::ZERO,
            total_withdrawals: Decimal::ZERO,
            total_realized_pnl: Decimal::ZERO,
            total_funding: Decimal::ZERO,
            total_commissions: Decimal::ZERO,
            net_change: Decimal::ZERO,
            current_balance: Decimal::ZERO,
            period_start: start_time,
            period_end: end_time,
        };

        for entry in &entries {
            match entry.entry_type {
                LedgerEntryType::Deposit => summary.total_deposits += entry.amount,
                LedgerEntryType::Withdrawal => summary.total_withdrawals += entry.amount.abs(),
                LedgerEntryType::TradePnL => summary.total_realized_pnl += entry.amount,
                LedgerEntryType::Funding => summary.total_funding += entry.amount,
                LedgerEntryType::Commission => summary.total_commissions += entry.amount.abs(),
                _ => {}
            }
            summary.net_change += entry.amount;
        }

        summary.current_balance = self.store.get_balance(exchange, asset).await;

        summary
    }

    /// Get summaries for all assets on an exchange
    pub async fn get_all_summaries(
        &self,
        exchange: Exchange,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<LedgerSummary> {
        let balances = self.store.get_all_balances(exchange).await;

        let mut summaries = Vec::new();
        for balance in balances {
            let summary = self
                .get_summary(exchange, &balance.asset, start_time, end_time)
                .await;
            summaries.push(summary);
        }

        summaries
    }

    /// Count entries matching filters
    pub async fn count_entries(&self, filters: &LedgerFilters) -> u64 {
        self.store.count_entries(filters).await
    }

    /// Recalculate balances from ledger entries
    ///
    /// This is useful for consistency checks or after data corrections
    pub async fn recalculate_balances(
        &self,
        exchange: Exchange,
    ) -> Result<HashMap<String, Decimal>, String> {
        let filters = LedgerFilters {
            exchange: Some(exchange),
            ..Default::default()
        };

        let entries = self.store.get_entries(&filters).await;
        let mut balances: HashMap<String, Decimal> = HashMap::new();

        for entry in entries {
            *balances.entry(entry.asset.clone()).or_insert(Decimal::ZERO) += entry.amount;
        }

        info!(
            "Recalculated balances for {}: {:?}",
            exchange,
            balances.keys().collect::<Vec<_>>()
        );

        Ok(balances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_entry_type_display() {
        assert_eq!(format!("{}", LedgerEntryType::Deposit), "DEPOSIT");
        assert_eq!(format!("{}", LedgerEntryType::TradePnL), "TRADE_PNL");
    }

    #[test]
    fn test_ledger_filters_default() {
        let filters = LedgerFilters::default();
        assert!(filters.exchange.is_none());
        assert!(filters.limit.is_none());
    }
}
