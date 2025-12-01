//! Modelos de dados da API Binance Futures

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Informações da conta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
  pub total_initial_margin: String,
  pub total_maint_margin: String,
  pub total_wallet_balance: String,
  pub total_unrealized_profit: String,
  pub total_margin_balance: String,
  pub total_position_initial_margin: String,
  pub total_open_order_initial_margin: String,
  pub total_cross_wallet_balance: String,
  pub total_cross_un_pnl: String,
  pub available_balance: String,
  pub max_withdraw_amount: String,
  pub assets: Vec<AccountAsset>,
  pub positions: Vec<AccountPosition>,
  pub can_trade: bool,
  pub can_deposit: bool,
  pub can_withdraw: bool,
  pub update_time: i64,
}

/// Ativo da conta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
  pub asset: String,
  pub wallet_balance: String,
  pub unrealized_profit: String,
  pub margin_balance: String,
  pub maint_margin: String,
  pub initial_margin: String,
  pub position_initial_margin: String,
  pub open_order_initial_margin: String,
  pub cross_wallet_balance: String,
  pub cross_un_pnl: String,
  pub available_balance: String,
  pub max_withdraw_amount: String,
  pub margin_available: bool,
  pub update_time: i64,
}

/// Posição da conta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPosition {
  pub symbol: String,
  pub initial_margin: String,
  pub maint_margin: String,
  pub unrealized_profit: String,
  pub position_initial_margin: String,
  pub open_order_initial_margin: String,
  pub leverage: String,
  pub isolated: bool,
  pub entry_price: String,
  pub break_even_price: String,
  pub max_notional: String,
  pub position_side: String,
  pub position_amt: String,
  pub notional: String,
  pub isolated_wallet: String,
  pub update_time: i64,
  pub bid_notional: String,
  pub ask_notional: String,
}

/// Kline/Candle
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceKline {
  pub open_time: i64,
  pub open: String,
  pub high: String,
  pub low: String,
  pub close: String,
  pub volume: String,
  pub close_time: i64,
  pub quote_asset_volume: String,
  pub number_of_trades: i64,
  pub taker_buy_base_volume: String,
  pub taker_buy_quote_volume: String,
  pub ignore: String,
}

/// Ticker 24h
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
  pub symbol: String,
  pub price_change: String,
  pub price_change_percent: String,
  pub weighted_avg_price: String,
  pub last_price: String,
  pub last_qty: String,
  pub open_price: String,
  pub high_price: String,
  pub low_price: String,
  pub volume: String,
  pub quote_volume: String,
  pub open_time: i64,
  pub close_time: i64,
  pub first_id: i64,
  pub last_id: i64,
  pub count: i64,
}

/// Preço atual
#[derive(Debug, Clone, Deserialize)]
pub struct TickerPrice {
  pub symbol: String,
  pub price: String,
  pub time: i64,
}

/// Book ticker (melhor bid/ask)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
  pub symbol: String,
  pub bid_price: String,
  pub bid_qty: String,
  pub ask_price: String,
  pub ask_qty: String,
  pub time: i64,
}

/// Ordem
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceOrder {
  pub order_id: i64,
  pub symbol: String,
  pub status: String,
  pub client_order_id: String,
  pub price: String,
  pub avg_price: String,
  pub orig_qty: String,
  pub executed_qty: String,
  #[serde(default)]
  pub cum_qty: Option<String>,
  pub cum_quote: String,
  #[serde(rename = "type")]
  pub order_type: String,
  pub side: String,
  pub position_side: String,
  pub stop_price: String,
  pub time_in_force: String,
  pub reduce_only: bool,
  pub close_position: bool,
  pub orig_type: String,
  pub working_type: String,
  pub price_protect: bool,
  pub update_time: i64,
  #[serde(default)]
  pub time: Option<i64>,
  #[serde(default)]
  pub price_match: Option<String>,
  #[serde(default)]
  pub self_trade_prevention_mode: Option<String>,
  #[serde(default)]
  pub good_till_date: Option<i64>,
}

/// Trade histórico do usuário
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrade {
  pub symbol: String,
  pub id: i64,
  pub order_id: i64,
  pub side: String,
  pub price: String,
  pub qty: String,
  pub realized_pnl: String,
  pub margin_asset: String,
  pub quote_qty: String,
  pub commission: String,
  pub commission_asset: String,
  pub time: i64,
  pub position_side: String,
  pub buyer: bool,
  pub maker: bool,
}

/// Income/Transação (para histórico de P&L)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeRecord {
  pub symbol: String,
  pub income_type: String,
  pub income: String,
  pub asset: String,
  pub info: String,
  pub time: i64,
  pub tran_id: i64,
  pub trade_id: String,
}

/// Tipos de income
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomeType {
  Transfer,
  WelcomeBonus,
  RealizedPnl,
  FundingFee,
  Commission,
  InsuranceClear,
  ReferralKickback,
  CommissionRebate,
  ApiRebate,
  ContestReward,
  CrossCollateralTransfer,
  OptionsPremiumFee,
  OptionsSettleProfit,
  InternalTransfer,
  AutoExchange,
  DeliveredSettelment,
  CoinSwapDeposit,
  CoinSwapWithdraw,
  PositionLimitIncreaseFee,
  Other(String),
}

impl From<&str> for IncomeType {
  fn from(s: &str) -> Self {
    match s {
      "TRANSFER" => IncomeType::Transfer,
      "WELCOME_BONUS" => IncomeType::WelcomeBonus,
      "REALIZED_PNL" => IncomeType::RealizedPnl,
      "FUNDING_FEE" => IncomeType::FundingFee,
      "COMMISSION" => IncomeType::Commission,
      "INSURANCE_CLEAR" => IncomeType::InsuranceClear,
      "REFERRAL_KICKBACK" => IncomeType::ReferralKickback,
      "COMMISSION_REBATE" => IncomeType::CommissionRebate,
      "API_REBATE" => IncomeType::ApiRebate,
      "CONTEST_REWARD" => IncomeType::ContestReward,
      "CROSS_COLLATERAL_TRANSFER" => IncomeType::CrossCollateralTransfer,
      "OPTIONS_PREMIUM_FEE" => IncomeType::OptionsPremiumFee,
      "OPTIONS_SETTLE_PROFIT" => IncomeType::OptionsSettleProfit,
      "INTERNAL_TRANSFER" => IncomeType::InternalTransfer,
      "AUTO_EXCHANGE" => IncomeType::AutoExchange,
      "DELIVERED_SETTELMENT" => IncomeType::DeliveredSettelment,
      "COIN_SWAP_DEPOSIT" => IncomeType::CoinSwapDeposit,
      "COIN_SWAP_WITHDRAW" => IncomeType::CoinSwapWithdraw,
      "POSITION_LIMIT_INCREASE_FEE" => IncomeType::PositionLimitIncreaseFee,
      other => IncomeType::Other(other.to_string()),
    }
  }
}

/// Request para criar ordem
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderRequest {
  pub symbol: String,
  pub side: String,
  #[serde(rename = "type")]
  pub order_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub position_side: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub time_in_force: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quantity: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reduce_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub price: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_price: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub close_position: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_client_order_id: Option<String>,
}

/// Resposta de erro da Binance
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceError {
  pub code: i32,
  pub msg: String,
}

impl std::fmt::Display for BinanceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Binance error {}: {}", self.code, self.msg)
  }
}

/// Estatísticas de histórico de trades
#[derive(Debug, Clone, Default)]
pub struct TradeHistoryStats {
  pub total_trades: u64,
  pub total_pnl: Decimal,
  pub total_fees: Decimal,
  pub winning_trades: u64,
  pub losing_trades: u64,
  pub largest_win: Decimal,
  pub largest_loss: Decimal,
  pub pnl_by_symbol: std::collections::HashMap<String, Decimal>,
}
