//! Entidades de Account & Balance
//!
//! Modelos para contas, saldos e snapshots.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use super::ExchangeId;

/// ID único de uma conta
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub Uuid);

impl AccountId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for AccountId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo de conta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    /// Conta principal
    Main,
    /// Conta de futuros
    Futures,
    /// Conta de margem
    Margin,
    /// Sub-conta
    Sub(String),
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::Main => write!(f, "main"),
            AccountType::Futures => write!(f, "futures"),
            AccountType::Margin => write!(f, "margin"),
            AccountType::Sub(name) => write!(f, "sub:{}", name),
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "main" => Ok(AccountType::Main),
            "futures" => Ok(AccountType::Futures),
            "margin" => Ok(AccountType::Margin),
            other => {
                if let Some(name) = other.strip_prefix("sub:") {
                    Ok(AccountType::Sub(name.to_string()))
                } else {
                    Ok(AccountType::Sub(other.to_string()))
                }
            }
        }
    }
}

/// Status de uma conta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// Conta ativa
    Active,
    /// Conta suspensa
    Suspended,
    /// Conta desativada
    Disabled,
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "active"),
            AccountStatus::Suspended => write!(f, "suspended"),
            AccountStatus::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for AccountStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(AccountStatus::Active),
            "suspended" => Ok(AccountStatus::Suspended),
            "disabled" => Ok(AccountStatus::Disabled),
            _ => Err(format!("Status de conta inválido: {}", s)),
        }
    }
}

/// Tipo de margem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarginType {
    /// Margem cruzada
    Cross,
    /// Margem isolada
    Isolated,
}

impl fmt::Display for MarginType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarginType::Cross => write!(f, "cross"),
            MarginType::Isolated => write!(f, "isolated"),
        }
    }
}

impl std::str::FromStr for MarginType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cross" => Ok(MarginType::Cross),
            "isolated" => Ok(MarginType::Isolated),
            _ => Err(format!("Tipo de margem inválido: {}", s)),
        }
    }
}

/// Modo de posição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionMode {
    /// Modo unidirecional (uma posição por símbolo)
    OneWay,
    /// Modo hedge (long e short simultâneos)
    Hedge,
}

impl fmt::Display for PositionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionMode::OneWay => write!(f, "one_way"),
            PositionMode::Hedge => write!(f, "hedge"),
        }
    }
}

impl std::str::FromStr for PositionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "one_way" | "oneway" => Ok(PositionMode::OneWay),
            "hedge" => Ok(PositionMode::Hedge),
            _ => Err(format!("Modo de posição inválido: {}", s)),
        }
    }
}

/// Conta de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// ID único
    pub id: AccountId,
    /// ID da exchange
    pub exchange_id: ExchangeId,
    /// Tipo de conta
    pub account_type: AccountType,
    /// Alias/nome amigável
    pub alias: Option<String>,
    /// Referência da API key no keyring
    pub api_key_ref: Option<String>,
    /// Tem permissão de trade?
    pub has_trade_permission: bool,
    /// Tem permissão de saque?
    pub has_withdraw_permission: bool,
    /// Lista de IPs permitidos
    pub ip_whitelist: Option<Vec<String>>,
    /// Status da conta
    pub status: AccountStatus,
    /// Última sincronização
    pub last_sync_at: Option<DateTime<Utc>>,
    /// Erro na última sincronização
    pub sync_error: Option<String>,
    /// Alavancagem padrão
    pub default_leverage: u32,
    /// Tipo de margem
    pub margin_type: MarginType,
    /// Modo de posição
    pub position_mode: PositionMode,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Cria uma nova conta
    pub fn new(exchange_id: ExchangeId, account_type: AccountType) -> Self {
        let now = Utc::now();
        Self {
            id: AccountId::new(),
            exchange_id,
            account_type,
            alias: None,
            api_key_ref: None,
            has_trade_permission: false,
            has_withdraw_permission: false,
            ip_whitelist: None,
            status: AccountStatus::Active,
            last_sync_at: None,
            sync_error: None,
            default_leverage: 1,
            margin_type: MarginType::Cross,
            position_mode: PositionMode::OneWay,
            created_at: now,
            updated_at: now,
        }
    }

    /// Define alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Define permissões
    pub fn with_permissions(mut self, trade: bool, withdraw: bool) -> Self {
        self.has_trade_permission = trade;
        self.has_withdraw_permission = withdraw;
        self
    }

    /// Verifica se está ativa
    pub fn is_active(&self) -> bool {
        self.status == AccountStatus::Active
    }

    /// Verifica se pode fazer trades
    pub fn can_trade(&self) -> bool {
        self.is_active() && self.has_trade_permission
    }

    /// Registra sincronização bem-sucedida
    pub fn record_sync_success(&mut self) {
        self.last_sync_at = Some(Utc::now());
        self.sync_error = None;
        self.updated_at = Utc::now();
    }

    /// Registra erro de sincronização
    pub fn record_sync_error(&mut self, error: impl Into<String>) {
        self.last_sync_at = Some(Utc::now());
        self.sync_error = Some(error.into());
        self.updated_at = Utc::now();
    }
}

/// Saldo de um ativo específico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    /// Nome do ativo (ex: BTC, USDT)
    pub asset: String,
    /// Saldo disponível
    pub free: Decimal,
    /// Saldo bloqueado (em ordens)
    pub locked: Decimal,
}

impl AssetBalance {
    /// Cria um novo saldo
    pub fn new(asset: impl Into<String>, free: Decimal, locked: Decimal) -> Self {
        Self {
            asset: asset.into(),
            free,
            locked,
        }
    }

    /// Saldo total
    pub fn total(&self) -> Decimal {
        self.free + self.locked
    }
}

/// Tipo de snapshot de saldo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotType {
    /// Snapshot periódico
    Periodic,
    /// Snapshot pré-trade
    PreTrade,
    /// Snapshot pós-trade
    PostTrade,
    /// Snapshot diário
    Daily,
    /// Snapshot manual
    Manual,
}

impl fmt::Display for SnapshotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotType::Periodic => write!(f, "periodic"),
            SnapshotType::PreTrade => write!(f, "pre_trade"),
            SnapshotType::PostTrade => write!(f, "post_trade"),
            SnapshotType::Daily => write!(f, "daily"),
            SnapshotType::Manual => write!(f, "manual"),
        }
    }
}

impl std::str::FromStr for SnapshotType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "periodic" => Ok(SnapshotType::Periodic),
            "pre_trade" => Ok(SnapshotType::PreTrade),
            "post_trade" => Ok(SnapshotType::PostTrade),
            "daily" => Ok(SnapshotType::Daily),
            "manual" => Ok(SnapshotType::Manual),
            _ => Err(format!("Tipo de snapshot inválido: {}", s)),
        }
    }
}

/// Snapshot de saldo de uma conta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    /// ID único
    pub id: i64,
    /// ID da conta
    pub account_id: AccountId,
    /// Tipo do snapshot
    pub snapshot_type: SnapshotType,
    /// Saldo total em USDT
    pub total_balance_usdt: Decimal,
    /// Saldo disponível em USDT
    pub available_balance_usdt: Decimal,
    /// Saldo bloqueado em USDT
    pub locked_balance_usdt: Decimal,
    /// Margem total
    pub total_margin: Option<Decimal>,
    /// Margem utilizada
    pub used_margin: Option<Decimal>,
    /// Margem disponível
    pub available_margin: Option<Decimal>,
    /// Nível de margem (%)
    pub margin_level: Option<Decimal>,
    /// P&L não realizado
    pub unrealized_pnl: Decimal,
    /// Saldos por ativo
    pub assets: Vec<AssetBalance>,
    /// Timestamp do snapshot
    pub timestamp: DateTime<Utc>,
}

impl BalanceSnapshot {
    /// Cria um novo snapshot
    pub fn new(
        account_id: AccountId,
        snapshot_type: SnapshotType,
        total_usdt: Decimal,
        available_usdt: Decimal,
        locked_usdt: Decimal,
        assets: Vec<AssetBalance>,
    ) -> Self {
        Self {
            id: 0, // será definido pelo banco
            account_id,
            snapshot_type,
            total_balance_usdt: total_usdt,
            available_balance_usdt: available_usdt,
            locked_balance_usdt: locked_usdt,
            total_margin: None,
            used_margin: None,
            available_margin: None,
            margin_level: None,
            unrealized_pnl: Decimal::ZERO,
            assets,
            timestamp: Utc::now(),
        }
    }

    /// Define informações de margem
    pub fn with_margin(
        mut self,
        total: Decimal,
        used: Decimal,
        available: Decimal,
        level: Option<Decimal>,
    ) -> Self {
        self.total_margin = Some(total);
        self.used_margin = Some(used);
        self.available_margin = Some(available);
        self.margin_level = level;
        self
    }

    /// Define P&L não realizado
    pub fn with_unrealized_pnl(mut self, pnl: Decimal) -> Self {
        self.unrealized_pnl = pnl;
        self
    }

    /// Calcula variação em relação a outro snapshot
    pub fn calculate_change(&self, previous: &BalanceSnapshot) -> BalanceChange {
        let absolute_change = self.total_balance_usdt - previous.total_balance_usdt;
        let percent_change = if previous.total_balance_usdt != Decimal::ZERO {
            (absolute_change / previous.total_balance_usdt) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        BalanceChange {
            absolute: absolute_change,
            percent: percent_change,
            from_timestamp: previous.timestamp,
            to_timestamp: self.timestamp,
        }
    }
}

/// Variação de saldo entre dois snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    /// Variação absoluta
    pub absolute: Decimal,
    /// Variação percentual
    pub percent: Decimal,
    /// Timestamp inicial
    pub from_timestamp: DateTime<Utc>,
    /// Timestamp final
    pub to_timestamp: DateTime<Utc>,
}

impl BalanceChange {
    /// Verifica se é positiva
    pub fn is_positive(&self) -> bool {
        self.absolute > Decimal::ZERO
    }

    /// Verifica se é negativa
    pub fn is_negative(&self) -> bool {
        self.absolute < Decimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_account_creation() {
        let account = Account::new(ExchangeId::BinanceFutures, AccountType::Futures)
            .with_alias("Main Futures Account")
            .with_permissions(true, false);

        assert!(account.can_trade());
        assert!(!account.has_withdraw_permission);
        assert_eq!(account.alias, Some("Main Futures Account".to_string()));
    }

    #[test]
    fn test_asset_balance() {
        let balance = AssetBalance::new("USDT", dec!(1000), dec!(200));
        assert_eq!(balance.total(), dec!(1200));
    }

    #[test]
    fn test_balance_snapshot_change() {
        let snapshot1 = BalanceSnapshot::new(
            AccountId::new(),
            SnapshotType::Daily,
            dec!(10000),
            dec!(8000),
            dec!(2000),
            vec![],
        );

        let mut snapshot2 = BalanceSnapshot::new(
            snapshot1.account_id.clone(),
            SnapshotType::Daily,
            dec!(10500),
            dec!(8500),
            dec!(2000),
            vec![],
        );
        snapshot2.timestamp = snapshot1.timestamp + chrono::Duration::hours(24);

        let change = snapshot2.calculate_change(&snapshot1);
        assert_eq!(change.absolute, dec!(500));
        assert_eq!(change.percent, dec!(5)); // 5%
        assert!(change.is_positive());
    }
}
