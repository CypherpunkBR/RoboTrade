//! Modelos de dados da API Kraken Futures

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// =============================================================================
// Respostas da API
// =============================================================================

/// Resposta genérica da API Kraken
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KrakenResponse<T> {
    pub result: String,
    #[serde(flatten)]
    pub data: Option<T>,
    pub error: Option<String>,
    pub server_time: Option<String>,
}

impl<T> KrakenResponse<T> {
    /// Verifica se a resposta foi bem sucedida
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Erro da API Kraken
#[derive(Debug, Clone, Deserialize)]
pub struct KrakenError {
    pub result: String,
    pub error: Option<String>,
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

impl std::fmt::Display for KrakenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Kraken error: {}",
            self.error.as_deref().unwrap_or("Unknown error")
        )
    }
}

// =============================================================================
// Account
// =============================================================================

/// Informações da conta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub accounts: Option<AccountData>,
}

/// Dados da conta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub cash: Option<CashAccount>,
    pub flex: Option<FlexAccount>,
}

/// Conta cash (margem isolada)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashAccount {
    #[serde(rename = "type")]
    pub account_type: String,
    pub balances: std::collections::HashMap<String, String>,
}

/// Conta flex (margem cross/portfolio)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexAccount {
    #[serde(rename = "type")]
    pub account_type: String,
    pub currencies: std::collections::HashMap<String, FlexCurrency>,
    pub initial_margin: Option<String>,
    pub initial_margin_with_orders: Option<String>,
    pub maintenance_margin: Option<String>,
    pub balance_value: Option<String>,
    pub portfolio_value: Option<String>,
    pub collateral_value: Option<String>,
    pub pnl: Option<String>,
    pub unrealized_funding: Option<String>,
    pub total_unrealized: Option<String>,
    pub total_unrealized_as_margin: Option<String>,
    pub available_margin: Option<String>,
    pub margin_equity: Option<String>,
}

/// Moeda na conta flex
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexCurrency {
    pub quantity: Option<String>,
    pub value: Option<String>,
    pub collateral: Option<String>,
    pub available: Option<String>,
}

// =============================================================================
// Positions
// =============================================================================

/// Lista de posições abertas
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPositions {
    pub open_positions: Vec<KrakenPosition>,
}

/// Posição da Kraken
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KrakenPosition {
    pub side: String,
    pub symbol: String,
    pub price: String,
    pub fill_time: String,
    pub size: String,
    pub unrealized_funding: Option<String>,
    pub pnl: Option<String>,
}

// =============================================================================
// Orders
// =============================================================================

/// Lista de ordens abertas
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrders {
    pub open_orders: Vec<KrakenOrder>,
}

/// Ordem da Kraken
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KrakenOrder {
    pub order_id: String,
    pub cli_ord_id: Option<String>,
    #[serde(rename = "type")]
    pub order_type: String,
    pub symbol: String,
    pub side: String,
    pub quantity: String,
    pub filled: String,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub reduce_only: bool,
    pub last_update_time: Option<String>,
    pub status: String,
}

/// Resposta de envio de ordem
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOrderResult {
    pub send_status: SendStatus,
}

/// Status do envio de ordem
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendStatus {
    pub order_id: Option<String>,
    pub cli_ord_id: Option<String>,
    pub status: String,
    pub received_time: Option<String>,
    pub order_events: Option<Vec<OrderEvent>>,
}

/// Evento de ordem
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderEvent {
    pub execution_id: Option<String>,
    pub price: Option<String>,
    pub amount: Option<String>,
    pub order_prior_edit: Option<serde_json::Value>,
    pub order_prior_execution: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Resposta de cancelamento de ordem
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResult {
    pub cancel_status: CancelStatus,
}

/// Status do cancelamento
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelStatus {
    pub order_id: Option<String>,
    pub cli_ord_id: Option<String>,
    pub status: String,
    pub received_time: Option<String>,
}

// =============================================================================
// Market Data
// =============================================================================

/// Ticker de instrumento
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerInfo {
    pub tickers: Vec<Ticker>,
}

/// Ticker individual
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    pub symbol: String,
    pub tag: Option<String>,
    pub pair: Option<String>,
    pub mark_price: Option<f64>,
    pub bid: Option<f64>,
    pub bid_size: Option<f64>,
    pub ask: Option<f64>,
    pub ask_size: Option<f64>,
    pub vol24h: Option<f64>,
    pub open_interest: Option<f64>,
    pub last: Option<f64>,
    pub last_time: Option<String>,
    pub last_size: Option<f64>,
    pub suspended: Option<bool>,
    pub funding_rate: Option<f64>,
    pub funding_rate_prediction: Option<f64>,
    pub open24h: Option<f64>,
    pub high24h: Option<f64>,
    pub low24h: Option<f64>,
    pub change24h: Option<f64>,
    pub index_price: Option<f64>,
    pub volume_quote: Option<f64>,
    pub post_only: Option<bool>,
}

/// Orderbook
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Orderbook {
    pub orderbook: OrderbookData,
}

/// Dados do orderbook
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookData {
    pub bids: Vec<Vec<String>>,
    pub asks: Vec<Vec<String>>,
}

/// Candle/OHLC (resposta da API /derivatives/api/v3/candles - não funciona mais)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesResponse {
    pub candles: Vec<KrakenCandle>,
}

/// Candle individual (formato antigo)
#[derive(Debug, Clone, Deserialize)]
pub struct KrakenCandle {
    pub time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

/// Resposta da Charts API (/api/charts/v1/trade/{symbol}/{resolution})
/// Formato igual ao CandlesResponse - usa mesmo KrakenCandle
#[derive(Debug, Clone, Deserialize)]
pub struct ChartsResponse {
    pub candles: Vec<KrakenCandle>,
}

// =============================================================================
// Instruments
// =============================================================================

/// Lista de instrumentos (wrapper para KrakenResponse)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruments {
    pub instruments: Vec<Instrument>,
}

/// Resposta direta da API de instrumentos (sem KrakenResponse genérico)
/// Necessário porque #[serde(flatten)] + Option<T> pode engolir erros de parsing
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentsResponse {
    pub result: String,
    #[serde(default)]
    pub instruments: Vec<Instrument>,
    pub error: Option<String>,
    pub server_time: Option<String>,
}

/// Instrumento (contrato futuro)
/// Usa default para campos opcionais e ignora campos desconhecidos da API
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Instrument {
    #[serde(default)]
    pub symbol: String,
    #[serde(rename = "type", default)]
    pub instrument_type: String,
    #[serde(default)]
    pub underlying: Option<String>,
    #[serde(default)]
    pub tick_size: Option<f64>,
    #[serde(default)]
    pub contract_size: Option<f64>,
    #[serde(default)]
    pub tradeable: bool,
    #[serde(default)]
    pub margin_levels: Option<Vec<MarginLevel>>,
    #[serde(default)]
    pub max_position_size: Option<f64>,
    #[serde(default)]
    pub opening_date: Option<String>,
    #[serde(default)]
    pub funding_rate_coefficient: Option<f64>,
    #[serde(default)]
    pub max_relative_funding_rate: Option<f64>,
    #[serde(default)]
    pub impact_mid_size: Option<f64>,
    #[serde(default)]
    pub isin: Option<String>,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub quote: Option<String>,
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub post_only: Option<bool>,
    #[serde(default)]
    pub mtf: Option<bool>,
}

/// Nível de margem
/// Alguns instrumentos usam `contracts`, outros usam `numNonContractUnits`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MarginLevel {
    /// Número de contratos (usado por futures_inverse)
    #[serde(default)]
    pub contracts: Option<i64>,
    /// Número de unidades não-contrato (usado por flexible_futures)
    #[serde(default)]
    pub num_non_contract_units: Option<f64>,
    #[serde(default)]
    pub initial_margin: f64,
    #[serde(default)]
    pub maintenance_margin: f64,
}

// =============================================================================
// Fills/Trades
// =============================================================================

/// Histórico de fills
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fills {
    pub fills: Vec<Fill>,
}

/// Fill individual
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub fill_id: String,
    pub symbol: String,
    pub side: String,
    pub order_id: String,
    pub size: String,
    pub price: String,
    pub fill_time: String,
    pub fill_type: String,
    pub fee_paid: Option<String>,
    pub fee_currency: Option<String>,
}

// =============================================================================
// Request Models
// =============================================================================

/// Request para criar ordem
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderRequest {
    /// Tipo de ordem: lmt, mkt, stp, take_profit, ioc
    pub order_type: String,
    /// Símbolo do contrato (ex: PI_XBTUSD)
    pub symbol: String,
    /// Lado: buy ou sell
    pub side: String,
    /// Quantidade
    pub size: String,
    /// Preço limite (obrigatório para lmt, stp, take_profit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    /// Preço de stop (obrigatório para stp, take_profit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    /// Se é reduce only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Client order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_ord_id: Option<String>,
    /// Trigger signal: mark ou last
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_signal: Option<String>,
}

impl NewOrderRequest {
    /// Cria uma ordem de mercado
    pub fn market(symbol: impl Into<String>, side: impl Into<String>, size: Decimal) -> Self {
        Self {
            order_type: "mkt".to_string(),
            symbol: symbol.into(),
            side: side.into(),
            size: size.to_string(),
            limit_price: None,
            stop_price: None,
            reduce_only: None,
            cli_ord_id: None,
            trigger_signal: None,
        }
    }

    /// Cria uma ordem limite
    pub fn limit(
        symbol: impl Into<String>,
        side: impl Into<String>,
        size: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            order_type: "lmt".to_string(),
            symbol: symbol.into(),
            side: side.into(),
            size: size.to_string(),
            limit_price: Some(price.to_string()),
            stop_price: None,
            reduce_only: None,
            cli_ord_id: None,
            trigger_signal: None,
        }
    }

    /// Define reduce only
    pub fn with_reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }

    /// Define client order ID
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.cli_ord_id = Some(client_id.into());
        self
    }
}

/// Tipos de ordem Kraken
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KrakenOrderType {
    /// Ordem limite
    Limit,
    /// Ordem de mercado
    Market,
    /// Stop order
    Stop,
    /// Take profit order
    TakeProfit,
    /// Immediate or cancel
    Ioc,
}

impl KrakenOrderType {
    /// Retorna o código da API
    pub fn as_api_str(&self) -> &'static str {
        match self {
            KrakenOrderType::Limit => "lmt",
            KrakenOrderType::Market => "mkt",
            KrakenOrderType::Stop => "stp",
            KrakenOrderType::TakeProfit => "take_profit",
            KrakenOrderType::Ioc => "ioc",
        }
    }
}

/// Lado da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KrakenSide {
    Buy,
    Sell,
}

impl KrakenSide {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            KrakenSide::Buy => "buy",
            KrakenSide::Sell => "sell",
        }
    }
}

/// Intervalos de candles suportados
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleInterval {
    /// 1 minuto
    M1,
    /// 5 minutos
    M5,
    /// 15 minutos
    M15,
    /// 30 minutos
    M30,
    /// 1 hora
    H1,
    /// 4 horas
    H4,
    /// 12 horas
    H12,
    /// 1 dia
    D1,
    /// 1 semana
    W1,
}

impl CandleInterval {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            CandleInterval::M1 => "1m",
            CandleInterval::M5 => "5m",
            CandleInterval::M15 => "15m",
            CandleInterval::M30 => "30m",
            CandleInterval::H1 => "1h",
            CandleInterval::H4 => "4h",
            CandleInterval::H12 => "12h",
            CandleInterval::D1 => "1d",
            CandleInterval::W1 => "1w",
        }
    }
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Estatísticas de histórico de trades
#[derive(Debug, Clone, Default)]
pub struct KrakenTradeStats {
    pub total_trades: u64,
    pub total_pnl: Decimal,
    pub total_fees: Decimal,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub pnl_by_symbol: std::collections::HashMap<String, Decimal>,
}
