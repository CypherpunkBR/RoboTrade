//! Cliente HTTP para Binance Futures API

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use robotrade_core::entities::{
    Balance, Candle, ExchangeId, Order, OrderId, OrderRequest, OrderSide, OrderSource, OrderStatus,
    OrderType, Position, PositionId, PositionSide, PositionStatus, TimeInForce,
};
use robotrade_core::error::{ExchangeError, ExchangeResult};
use robotrade_core::traits::ExchangeGateway;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use tracing::debug;

use super::models::*;
use super::signer::sign;

/// URLs da API
const MAINNET_URL: &str = "https://fapi.binance.com";
const TESTNET_URL: &str = "https://testnet.binancefuture.com";

/// Timeout padrão
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Recv window padrão
const DEFAULT_RECV_WINDOW: u64 = 5000;

/// Cliente para Binance Futures
pub struct BinanceFuturesClient {
    client: Client,
    base_url: String,
    api_key: String,
    api_secret: String,
    recv_window: u64,
    is_testnet: bool,
}

impl BinanceFuturesClient {
    /// Cria um novo cliente para testnet
    pub fn testnet(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::new(api_key, api_secret, true)
    }

    /// Cria um novo cliente para mainnet (produção)
    pub fn mainnet(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::new(api_key, api_secret, false)
    }

    /// Cria um novo cliente
    fn new(api_key: impl Into<String>, api_secret: impl Into<String>, is_testnet: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Erro ao criar cliente HTTP");

        let base_url = if is_testnet { TESTNET_URL } else { MAINNET_URL };

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            recv_window: DEFAULT_RECV_WINDOW,
            is_testnet,
        }
    }

    /// Retorna timestamp atual em milissegundos
    fn timestamp() -> u64 {
        chrono::Utc::now().timestamp_millis() as u64
    }

    /// Faz requisição GET pública (sem autenticação)
    async fn get_public<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> ExchangeResult<T> {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "binance".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Faz requisição GET autenticada
    async fn get_signed<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &mut Vec<(&str, String)>,
    ) -> ExchangeResult<T> {
        let timestamp = Self::timestamp().to_string();
        let recv_window = self.recv_window.to_string();

        params.push(("timestamp", timestamp));
        params.push(("recvWindow", recv_window));

        // Constrói query string
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Assina
        let signature = sign(&self.api_secret, &query_string);

        let url = format!(
            "{}{}?{}&signature={}",
            self.base_url, endpoint, query_string, signature
        );

        debug!(url = %url, "Requisição assinada");

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "binance".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Faz requisição POST autenticada
    async fn post_signed<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &mut Vec<(&str, String)>,
    ) -> ExchangeResult<T> {
        let timestamp = Self::timestamp().to_string();
        let recv_window = self.recv_window.to_string();

        params.push(("timestamp", timestamp));
        params.push(("recvWindow", recv_window));

        // Constrói query string
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Assina
        let signature = sign(&self.api_secret, &query_string);

        let url = format!("{}{}", self.base_url, endpoint);
        let body = format!("{}&signature={}", query_string, signature);

        debug!(endpoint = %endpoint, "Requisição POST assinada");

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "binance".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Faz requisição DELETE autenticada
    async fn delete_signed<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &mut Vec<(&str, String)>,
    ) -> ExchangeResult<T> {
        let timestamp = Self::timestamp().to_string();
        let recv_window = self.recv_window.to_string();

        params.push(("timestamp", timestamp));
        params.push(("recvWindow", recv_window));

        // Constrói query string
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Assina
        let signature = sign(&self.api_secret, &query_string);

        let url = format!(
            "{}{}?{}&signature={}",
            self.base_url, endpoint, query_string, signature
        );

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "binance".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Processa resposta da API
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> ExchangeResult<T> {
        let status = response.status();
        let body = response.text().await.map_err(|e| ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: format!("Erro ao ler resposta: {}", e),
        })?;

        if !status.is_success() {
            // Tenta parsear erro da Binance
            if let Ok(error) = serde_json::from_str::<BinanceError>(&body) {
                return Err(self.map_binance_error(error));
            }

            return Err(ExchangeError::ApiError {
                exchange: "binance".into(),
                code: status.as_u16() as i32,
                message: body,
            });
        }

        serde_json::from_str(&body).map_err(|e| ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: format!("Erro ao parsear resposta: {} - Body: {}", e, body),
        })
    }

    /// Mapeia erro da Binance para ExchangeError
    fn map_binance_error(&self, error: BinanceError) -> ExchangeError {
        match error.code {
            -1021 => ExchangeError::ApiError {
                exchange: "binance".into(),
                code: error.code,
                message: "Timestamp fora da janela de recvWindow".into(),
            },
            -2010 => ExchangeError::InsufficientBalance {
                asset: "".into(),
                required: "".into(),
                available: "".into(),
            },
            -2019 => ExchangeError::InsufficientBalance {
                asset: "margin".into(),
                required: "".into(),
                available: "".into(),
            },
            -1102 | -1013 => ExchangeError::InvalidQuantity {
                symbol: "".into(),
                reason: error.msg.clone(),
            },
            -1111 => ExchangeError::InvalidQuantity {
                symbol: "".into(),
                reason: "Precisão inválida".into(),
            },
            -4003 => ExchangeError::InvalidQuantity {
                symbol: "".into(),
                reason: "Quantidade deve ser maior que zero".into(),
            },
            _ => ExchangeError::ApiError {
                exchange: "binance".into(),
                code: error.code,
                message: error.msg,
            },
        }
    }

    // =========================================================================
    // API Pública
    // =========================================================================

    /// Busca candles/klines
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Candle>> {
        let limit_str = limit.unwrap_or(500).to_string();
        let params = vec![
            ("symbol", symbol),
            ("interval", interval),
            ("limit", &limit_str),
        ];

        let klines: Vec<Vec<serde_json::Value>> =
            self.get_public("/fapi/v1/klines", &params).await?;

        let candles = klines
            .into_iter()
            .filter_map(|k| self.parse_kline(&k).ok())
            .collect();

        Ok(candles)
    }

    /// Busca candles/klines com range de tempo
    pub async fn get_klines_range(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Candle>> {
        let limit_str = limit.unwrap_or(1000).to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("symbol", symbol),
            ("interval", interval),
            ("limit", &limit_str),
        ];

        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());

        if let Some(ref s) = start_str {
            params.push(("startTime", s));
        }
        if let Some(ref e) = end_str {
            params.push(("endTime", e));
        }

        let klines: Vec<Vec<serde_json::Value>> =
            self.get_public("/fapi/v1/klines", &params).await?;

        let candles = klines
            .into_iter()
            .filter_map(|k| self.parse_kline(&k).ok())
            .collect();

        Ok(candles)
    }

    /// Busca todo o historico de klines paginando automaticamente
    /// Binance retorna no maximo 1500 candles por request
    pub async fn get_all_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Candle>> {
        let mut all_candles = Vec::new();
        let mut current_start = start_time;
        let final_end = end_time.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let limit = 1500u32;

        loop {
            let candles = self
                .get_klines_range(symbol, interval, Some(current_start), Some(final_end), Some(limit))
                .await?;

            if candles.is_empty() {
                break;
            }

            let last_time = candles.last().map(|c| c.close_time.timestamp_millis());
            all_candles.extend(candles);

            // Se o ultimo candle esta proximo do end_time, terminamos
            if let Some(lt) = last_time {
                if lt >= final_end - 60000 {
                    break;
                }
                current_start = lt + 1;
            } else {
                break;
            }

            // Rate limiting: pequena pausa entre requisicoes
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Remove duplicatas baseado no open_time
        all_candles.sort_by_key(|c| c.open_time);
        all_candles.dedup_by_key(|c| c.open_time);

        Ok(all_candles)
    }

    /// Parseia kline da API
    fn parse_kline(&self, k: &[serde_json::Value]) -> ExchangeResult<Candle> {
        if k.len() < 12 {
            return Err(ExchangeError::ApiError {
                exchange: "binance".into(),
                code: 0,
                message: "Kline com dados insuficientes".into(),
            });
        }

        let open_time = k[0].as_i64().unwrap_or(0);
        let close_time = k[6].as_i64().unwrap_or(0);

        let parse_decimal = |v: &serde_json::Value| -> Decimal {
            v.as_str()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO)
        };

        Ok(Candle {
            open: parse_decimal(&k[1]),
            high: parse_decimal(&k[2]),
            low: parse_decimal(&k[3]),
            close: parse_decimal(&k[4]),
            volume: parse_decimal(&k[5]),
            quote_volume: Some(parse_decimal(&k[7])),
            trade_count: k[8].as_i64().map(|n| n as u32),
            open_time: DateTime::from_timestamp_millis(open_time).unwrap_or_else(Utc::now),
            close_time: DateTime::from_timestamp_millis(close_time).unwrap_or_else(Utc::now),
        })
    }

    /// Busca ticker 24h
    pub async fn get_ticker_24h(&self, symbol: &str) -> ExchangeResult<Ticker24h> {
        let params = vec![("symbol", symbol)];
        self.get_public("/fapi/v1/ticker/24hr", &params).await
    }

    /// Busca preço atual
    pub async fn get_price(&self, symbol: &str) -> ExchangeResult<TickerPrice> {
        let params = vec![("symbol", symbol)];
        self.get_public("/fapi/v1/ticker/price", &params).await
    }

    /// Busca book ticker (melhor bid/ask)
    pub async fn get_book_ticker(&self, symbol: &str) -> ExchangeResult<BookTicker> {
        let params = vec![("symbol", symbol)];
        self.get_public("/fapi/v1/ticker/bookTicker", &params).await
    }

    // =========================================================================
    // API Privada - Conta
    // =========================================================================

    /// Busca informações da conta
    pub async fn get_account(&self) -> ExchangeResult<AccountInfo> {
        let mut params = vec![];
        self.get_signed("/fapi/v2/account", &mut params).await
    }

    /// Define alavancagem para um símbolo
    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("leverage", leverage.to_string()),
        ];

        let _: serde_json::Value = self.post_signed("/fapi/v1/leverage", &mut params).await?;
        Ok(())
    }

    // =========================================================================
    // API Privada - Ordens
    // =========================================================================

    /// Cria uma nova ordem
    pub async fn create_order(&self, request: &NewOrderRequest) -> ExchangeResult<BinanceOrder> {
        let mut params = vec![
            ("symbol", request.symbol.clone()),
            ("side", request.side.clone()),
            ("type", request.order_type.clone()),
        ];

        if let Some(ref ps) = request.position_side {
            params.push(("positionSide", ps.clone()));
        }
        if let Some(ref tif) = request.time_in_force {
            params.push(("timeInForce", tif.clone()));
        }
        if let Some(ref qty) = request.quantity {
            params.push(("quantity", qty.clone()));
        }
        if let Some(ref price) = request.price {
            params.push(("price", price.clone()));
        }
        if let Some(ref stop) = request.stop_price {
            params.push(("stopPrice", stop.clone()));
        }
        if let Some(reduce_only) = request.reduce_only {
            params.push(("reduceOnly", reduce_only.to_string()));
        }
        if let Some(ref client_id) = request.new_client_order_id {
            params.push(("newClientOrderId", client_id.clone()));
        }

        self.post_signed("/fapi/v1/order", &mut params).await
    }

    /// Cancela uma ordem
    pub async fn cancel_order(&self, symbol: &str, order_id: i64) -> ExchangeResult<BinanceOrder> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("orderId", order_id.to_string()),
        ];

        self.delete_signed("/fapi/v1/order", &mut params).await
    }

    /// Busca uma ordem
    pub async fn get_order(&self, symbol: &str, order_id: i64) -> ExchangeResult<BinanceOrder> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("orderId", order_id.to_string()),
        ];

        self.get_signed("/fapi/v1/order", &mut params).await
    }

    /// Lista ordens abertas
    pub async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<BinanceOrder>> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(("symbol", s.to_string()));
        }

        self.get_signed("/fapi/v1/openOrders", &mut params).await
    }

    // =========================================================================
    // API Privada - Histórico
    // =========================================================================

    /// Busca histórico de trades do usuário
    pub async fn get_user_trades(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = vec![("symbol", symbol.to_string())];

        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        self.get_signed("/fapi/v1/userTrades", &mut params).await
    }

    /// Busca histórico de income (P&L, funding fees, etc.)
    /// Útil para calcular P&L total e perdas
    pub async fn get_income_history(
        &self,
        symbol: Option<&str>,
        income_type: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<IncomeRecord>> {
        let mut params = vec![];

        if let Some(s) = symbol {
            params.push(("symbol", s.to_string()));
        }
        if let Some(t) = income_type {
            params.push(("incomeType", t.to_string()));
        }
        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        self.get_signed("/fapi/v1/income", &mut params).await
    }

    /// Busca histórico completo de P&L realizado
    /// Itera por todos os registros paginando automaticamente
    pub async fn get_all_realized_pnl(
        &self,
        symbol: Option<&str>,
        start_time: Option<i64>,
    ) -> ExchangeResult<Vec<IncomeRecord>> {
        let mut all_records = Vec::new();
        let mut current_start = start_time;
        let limit = 1000u32;

        loop {
            let records = self
                .get_income_history(
                    symbol,
                    Some("REALIZED_PNL"),
                    current_start,
                    None,
                    Some(limit),
                )
                .await?;

            if records.is_empty() {
                break;
            }

            let last_time = records.last().map(|r| r.time);
            all_records.extend(records);

            // Se recebeu menos que o limite, chegou ao fim
            if all_records.len() % (limit as usize) != 0 {
                break;
            }

            // Próxima página
            if let Some(lt) = last_time {
                current_start = Some(lt + 1);
            } else {
                break;
            }

            // Rate limiting: pequena pausa entre requisições
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(all_records)
    }

    /// Calcula estatísticas do histórico de trades
    pub async fn calculate_trade_stats(
        &self,
        symbol: Option<&str>,
        start_time: Option<i64>,
    ) -> ExchangeResult<TradeHistoryStats> {
        let records = self.get_all_realized_pnl(symbol, start_time).await?;

        let mut stats = TradeHistoryStats::default();

        for record in &records {
            let pnl = Decimal::from_str(&record.income).unwrap_or(Decimal::ZERO);

            stats.total_trades += 1;
            stats.total_pnl += pnl;

            if pnl > Decimal::ZERO {
                stats.winning_trades += 1;
                if pnl > stats.largest_win {
                    stats.largest_win = pnl;
                }
            } else if pnl < Decimal::ZERO {
                stats.losing_trades += 1;
                if pnl < stats.largest_loss {
                    stats.largest_loss = pnl;
                }
            }

            // P&L por símbolo
            *stats
                .pnl_by_symbol
                .entry(record.symbol.clone())
                .or_insert(Decimal::ZERO) += pnl;
        }

        // Busca fees separadamente
        let fees = self
            .get_income_history(symbol, Some("COMMISSION"), start_time, None, Some(1000))
            .await
            .unwrap_or_default();

        for fee in &fees {
            let commission = Decimal::from_str(&fee.income).unwrap_or(Decimal::ZERO);
            stats.total_fees += commission.abs();
        }

        Ok(stats)
    }
}

#[async_trait]
impl ExchangeGateway for BinanceFuturesClient {
    fn exchange_id(&self) -> ExchangeId {
        if self.is_testnet {
            ExchangeId::Paper
        } else {
            ExchangeId::BinanceFutures
        }
    }

    fn is_paper_trading(&self) -> bool {
        self.is_testnet
    }

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        let binance_request = NewOrderRequest {
            symbol: request.symbol.clone(),
            side: match request.side {
                OrderSide::Buy => "BUY".into(),
                OrderSide::Sell => "SELL".into(),
            },
            order_type: match request.order_type {
                OrderType::Market => "MARKET".into(),
                OrderType::Limit => "LIMIT".into(),
                OrderType::StopLoss => "STOP_MARKET".into(),
                OrderType::StopLossLimit => "STOP".into(),
                OrderType::TakeProfit => "TAKE_PROFIT_MARKET".into(),
                OrderType::TakeProfitLimit => "TAKE_PROFIT".into(),
                OrderType::TrailingStop => "TRAILING_STOP_MARKET".into(),
            },
            position_side: None,
            time_in_force: Some(match request.time_in_force {
                TimeInForce::GTC => "GTC".into(),
                TimeInForce::IOC => "IOC".into(),
                TimeInForce::FOK => "FOK".into(),
                TimeInForce::GTD => "GTC".into(),
            }),
            quantity: Some(request.quantity.to_string()),
            reduce_only: Some(request.reduce_only),
            price: request.price.map(|p| p.to_string()),
            stop_price: request.stop_price.map(|p| p.to_string()),
            close_position: None,
            new_client_order_id: None,
        };

        let _binance_order = self.create_order(&binance_request).await?;

        // Converte para Order do core
        Ok(Order::from_request(
            request,
            self.exchange_id(),
            OrderSource::Manual { nota: None },
        ))
    }

    async fn cancel_order(&self, _order_id: &str) -> ExchangeResult<bool> {
        // Precisamos do símbolo para cancelar na Binance
        // Por enquanto, retornamos erro
        Err(ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: "Use cancel_order_with_symbol para cancelar ordens na Binance".into(),
        })
    }

    async fn get_order_status(&self, _order_id: &str) -> ExchangeResult<Order> {
        Err(ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: "Use get_order com símbolo para buscar ordem na Binance".into(),
        })
    }

    async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
        let binance_orders = self.get_open_orders(symbol).await?;

        let orders = binance_orders
            .into_iter()
            .map(|bo| self.convert_binance_order(&bo))
            .collect();

        Ok(orders)
    }

    async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
        let account = self.get_account().await?;

        let positions: Vec<Position> = account
            .positions
            .into_iter()
            .filter(|p| {
                let amt = Decimal::from_str(&p.position_amt).unwrap_or(Decimal::ZERO);
                amt != Decimal::ZERO
            })
            .map(|p| self.convert_binance_position(&p))
            .collect();

        Ok(positions)
    }

    async fn get_position(&self, symbol: &str) -> ExchangeResult<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        let account = self.get_account().await?;

        let balances: Vec<Balance> = account
            .assets
            .into_iter()
            .filter(|a| {
                let balance = Decimal::from_str(&a.wallet_balance).unwrap_or(Decimal::ZERO);
                balance > Decimal::ZERO
            })
            .map(|a| Balance {
                asset: a.asset,
                free: Decimal::from_str(&a.available_balance).unwrap_or(Decimal::ZERO),
                locked: Decimal::from_str(&a.initial_margin).unwrap_or(Decimal::ZERO),
            })
            .collect();

        Ok(balances)
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()> {
        self.set_leverage(symbol, leverage).await
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _: serde_json::Value = self.get_public("/fapi/v1/ping", &[]).await?;
        Ok(())
    }
}

impl BinanceFuturesClient {
    /// Converte ordem da Binance para Order do core
    fn convert_binance_order(&self, bo: &BinanceOrder) -> Order {
        let status = match bo.status.as_str() {
            "NEW" => OrderStatus::Submitted,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Cancelled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::Pending,
        };

        let side = match bo.side.as_str() {
            "BUY" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };

        let order_type = match bo.order_type.as_str() {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit,
            "STOP" | "STOP_MARKET" => OrderType::StopLoss,
            "TAKE_PROFIT" | "TAKE_PROFIT_MARKET" => OrderType::TakeProfit,
            "TRAILING_STOP_MARKET" => OrderType::TrailingStop,
            _ => OrderType::Market,
        };

        Order {
            id: OrderId::new(),
            client_order_id: bo.client_order_id.clone(),
            exchange_order_id: Some(bo.order_id.to_string()),
            exchange: self.exchange_id(),
            symbol: bo.symbol.clone(),
            side,
            order_type,
            quantity: Decimal::from_str(&bo.orig_qty).unwrap_or(Decimal::ZERO),
            price: Decimal::from_str(&bo.price)
                .ok()
                .filter(|p| *p != Decimal::ZERO),
            stop_price: Decimal::from_str(&bo.stop_price)
                .ok()
                .filter(|p| *p != Decimal::ZERO),
            stop_loss: None,
            take_profit: None,
            time_in_force: TimeInForce::GTC,
            status,
            filled_quantity: Decimal::from_str(&bo.executed_qty).unwrap_or(Decimal::ZERO),
            average_fill_price: Decimal::from_str(&bo.avg_price)
                .ok()
                .filter(|p| *p != Decimal::ZERO),
            source: OrderSource::Manual { nota: None },
            error_message: None,
            created_at: Utc::now(),
            submitted_at: Some(Utc::now()),
            filled_at: None,
            updated_at: Utc::now(),
        }
    }

    /// Converte posição da Binance para Position do core
    fn convert_binance_position(&self, bp: &AccountPosition) -> Position {
        let amt = Decimal::from_str(&bp.position_amt).unwrap_or(Decimal::ZERO);
        let side = if amt > Decimal::ZERO {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Position {
            id: PositionId::new(),
            exchange: self.exchange_id(),
            symbol: bp.symbol.clone(),
            side,
            quantity: amt.abs(),
            entry_price: Decimal::from_str(&bp.entry_price).unwrap_or(Decimal::ZERO),
            current_price: Decimal::from_str(&bp.entry_price).unwrap_or(Decimal::ZERO),
            leverage: bp.leverage.parse().unwrap_or(1),
            margin: Decimal::from_str(&bp.initial_margin).unwrap_or(Decimal::ZERO),
            unrealized_pnl: Decimal::from_str(&bp.unrealized_profit).unwrap_or(Decimal::ZERO),
            unrealized_pnl_pct: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            stop_loss_order_id: None,
            stop_loss_price: None,
            take_profit_order_id: None,
            take_profit_price: None,
            entry_order_id: OrderId::new(),
            status: PositionStatus::Open,
            liquidation_price: Some(
                Decimal::from_str(&bp.break_even_price).unwrap_or(Decimal::ZERO),
            ),
            opened_at: Utc::now(),
            closed_at: None,
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BinanceFuturesClient::testnet("api_key", "api_secret");
        assert!(client.is_paper_trading());
        assert_eq!(client.base_url, TESTNET_URL);
    }

    #[test]
    fn test_mainnet_client() {
        let client = BinanceFuturesClient::mainnet("api_key", "api_secret");
        assert!(!client.is_paper_trading());
        assert_eq!(client.base_url, MAINNET_URL);
    }
}
