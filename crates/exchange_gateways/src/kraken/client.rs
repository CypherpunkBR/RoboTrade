//! Cliente HTTP para Kraken Futures API
//!
//! Implementa integração com a API REST da Kraken Futures para trading
//! de contratos perpétuos e futuros.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use robotrade_core::entities::{
    Balance, Candle, ExchangeId, Order, OrderId, OrderRequest, OrderSide, OrderSource,
    OrderStatus, OrderType, Position, PositionId, PositionSide, PositionStatus, TimeInForce,
};
use robotrade_core::error::{ExchangeError, ExchangeResult};
use robotrade_core::traits::ExchangeGateway;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use tracing::debug;

use super::models::*;
use super::signer::sign;

/// URLs da API Kraken Futures
const MAINNET_URL: &str = "https://futures.kraken.com";
const DEMO_URL: &str = "https://demo-futures.kraken.com";

/// Caminho base da API
const API_PATH: &str = "/derivatives/api/v3";

/// Timeout padrão
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Cliente para Kraken Futures
pub struct KrakenFuturesClient {
    client: Client,
    base_url: String,
    api_key: String,
    api_secret: String,
    is_demo: bool,
}

impl KrakenFuturesClient {
    /// Cria um novo cliente para ambiente demo (testnet)
    pub fn demo(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::new(api_key, api_secret, true)
    }

    /// Cria um novo cliente para mainnet (produção)
    pub fn mainnet(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::new(api_key, api_secret, false)
    }

    /// Cria um novo cliente
    fn new(api_key: impl Into<String>, api_secret: impl Into<String>, is_demo: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Erro ao criar cliente HTTP");

        let base_url = if is_demo { DEMO_URL } else { MAINNET_URL };

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            is_demo,
        }
    }

    /// Retorna nonce atual (timestamp em milissegundos)
    fn nonce() -> String {
        chrono::Utc::now().timestamp_millis().to_string()
    }

    /// Constrói URL completa do endpoint
    fn endpoint_url(&self, endpoint: &str) -> String {
        format!("{}{}{}", self.base_url, API_PATH, endpoint)
    }

    /// Constrói path do endpoint (para assinatura)
    fn endpoint_path(endpoint: &str) -> String {
        format!("{}{}", API_PATH, endpoint)
    }

    /// Faz requisição GET pública (sem autenticação)
    async fn get_public<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> ExchangeResult<T> {
        let url = self.endpoint_url(endpoint);

        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Faz requisição GET autenticada
    async fn get_signed<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> ExchangeResult<T> {
        let nonce = Self::nonce();
        let endpoint_path = Self::endpoint_path(endpoint);
        let post_data = "";

        let authent = sign(&self.api_secret, post_data, &nonce, &endpoint_path);

        let url = self.endpoint_url(endpoint);

        debug!(url = %url, "Requisição GET assinada");

        let response = self
            .client
            .get(&url)
            .header("APIKey", &self.api_key)
            .header("Nonce", &nonce)
            .header("Authent", authent)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: e.to_string(),
            })?;

        self.handle_response(response).await
    }

    /// Faz requisição POST autenticada
    async fn post_signed<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> ExchangeResult<T> {
        let nonce = Self::nonce();
        let endpoint_path = Self::endpoint_path(endpoint);

        // Constrói post_data URL-encoded
        let post_data: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let authent = sign(&self.api_secret, &post_data, &nonce, &endpoint_path);

        let url = self.endpoint_url(endpoint);

        debug!(endpoint = %endpoint, "Requisição POST assinada");

        let response = self
            .client
            .post(&url)
            .header("APIKey", &self.api_key)
            .header("Nonce", &nonce)
            .header("Authent", authent)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data)
            .send()
            .await
            .map_err(|e| ExchangeError::ApiError {
                exchange: "kraken".into(),
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
            exchange: "kraken".into(),
            code: 0,
            message: format!("Erro ao ler resposta: {}", e),
        })?;

        if !status.is_success() {
            // Tenta parsear erro da Kraken
            if let Ok(error) = serde_json::from_str::<KrakenError>(&body) {
                return Err(self.map_kraken_error(&error));
            }

            return Err(ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: status.as_u16() as i32,
                message: body,
            });
        }

        // Verifica se a resposta indica erro
        if let Ok(error) = serde_json::from_str::<KrakenError>(&body) {
            if error.result != "success" {
                return Err(self.map_kraken_error(&error));
            }
        }

        serde_json::from_str(&body).map_err(|e| ExchangeError::ApiError {
            exchange: "kraken".into(),
            code: 0,
            message: format!("Erro ao parsear resposta: {} - Body: {}", e, body),
        })
    }

    /// Mapeia erro da Kraken para ExchangeError
    fn map_kraken_error(&self, error: &KrakenError) -> ExchangeError {
        let error_msg = error.error.as_deref().unwrap_or("Unknown error");

        // Mapeia erros conhecidos
        if error_msg.contains("Insufficient") {
            return ExchangeError::InsufficientBalance {
                asset: "".into(),
                required: "".into(),
                available: "".into(),
            };
        }

        if error_msg.contains("Invalid") && error_msg.contains("quantity") {
            return ExchangeError::InvalidQuantity {
                symbol: "".into(),
                reason: error_msg.to_string(),
            };
        }

        if error_msg.contains("nonceDuplicate") || error_msg.contains("nonceBelowThreshold") {
            return ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: -1,
                message: "Erro de nonce - tente novamente".into(),
            };
        }

        ExchangeError::ApiError {
            exchange: "kraken".into(),
            code: 0,
            message: error_msg.to_string(),
        }
    }

    // =========================================================================
    // API Pública
    // =========================================================================

    /// Busca lista de instrumentos disponíveis
    pub async fn get_instruments(&self) -> ExchangeResult<Vec<Instrument>> {
        let response: KrakenResponse<Instruments> =
            self.get_public("/instruments", &[]).await?;

        Ok(response.data.map(|d| d.instruments).unwrap_or_default())
    }

    /// Busca tickers de um ou mais símbolos
    pub async fn get_tickers(&self, symbols: Option<&[&str]>) -> ExchangeResult<Vec<Ticker>> {
        let response: KrakenResponse<TickerInfo> = if let Some(syms) = symbols {
            let symbols_param = syms.join(",");
            self.get_public("/tickers", &[("symbols", &symbols_param)])
                .await?
        } else {
            self.get_public("/tickers", &[]).await?
        };

        Ok(response.data.map(|d| d.tickers).unwrap_or_default())
    }

    /// Busca ticker de um símbolo específico
    pub async fn get_ticker(&self, symbol: &str) -> ExchangeResult<Option<Ticker>> {
        let tickers = self.get_tickers(Some(&[symbol])).await?;
        Ok(tickers.into_iter().find(|t| t.symbol == symbol))
    }

    /// Busca orderbook
    pub async fn get_orderbook(&self, symbol: &str) -> ExchangeResult<OrderbookData> {
        let response: KrakenResponse<Orderbook> = self
            .get_public("/orderbook", &[("symbol", symbol)])
            .await?;

        response
            .data
            .map(|d| d.orderbook)
            .ok_or_else(|| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: "Orderbook não encontrado".into(),
            })
    }

    /// Busca candles/OHLC
    pub async fn get_candles(
        &self,
        symbol: &str,
        interval: CandleInterval,
        from: Option<i64>,
        to: Option<i64>,
    ) -> ExchangeResult<Vec<Candle>> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("interval", interval.as_api_str().to_string()),
        ];

        if let Some(f) = from {
            params.push(("from", f.to_string()));
        }
        if let Some(t) = to {
            params.push(("to", t.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: KrakenResponse<CandlesResponse> =
            self.get_public("/candles", &params_ref).await?;

        let candles = response
            .data
            .map(|d| d.candles)
            .unwrap_or_default()
            .into_iter()
            .map(|kc| self.convert_candle(&kc))
            .collect();

        Ok(candles)
    }

    /// Converte candle Kraken para Candle do core
    fn convert_candle(&self, kc: &KrakenCandle) -> Candle {
        Candle {
            open: Decimal::from_str(&kc.open).unwrap_or(Decimal::ZERO),
            high: Decimal::from_str(&kc.high).unwrap_or(Decimal::ZERO),
            low: Decimal::from_str(&kc.low).unwrap_or(Decimal::ZERO),
            close: Decimal::from_str(&kc.close).unwrap_or(Decimal::ZERO),
            volume: Decimal::from_str(&kc.volume).unwrap_or(Decimal::ZERO),
            quote_volume: None,
            trade_count: None,
            open_time: DateTime::from_timestamp_millis(kc.time).unwrap_or_else(Utc::now),
            close_time: DateTime::from_timestamp_millis(kc.time).unwrap_or_else(Utc::now),
        }
    }

    // =========================================================================
    // API Privada - Conta
    // =========================================================================

    /// Busca informações da conta
    pub async fn get_account(&self) -> ExchangeResult<AccountData> {
        let response: KrakenResponse<AccountInfo> = self.get_signed("/accounts").await?;

        response
            .data
            .and_then(|d| d.accounts)
            .ok_or_else(|| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: "Dados da conta não encontrados".into(),
            })
    }

    // =========================================================================
    // API Privada - Posições
    // =========================================================================

    /// Busca posições abertas
    pub async fn get_open_positions(&self) -> ExchangeResult<Vec<KrakenPosition>> {
        let response: KrakenResponse<OpenPositions> = self.get_signed("/openpositions").await?;

        Ok(response.data.map(|d| d.open_positions).unwrap_or_default())
    }

    // =========================================================================
    // API Privada - Ordens
    // =========================================================================

    /// Envia uma nova ordem
    pub async fn send_order(&self, request: &NewOrderRequest) -> ExchangeResult<SendStatus> {
        let mut params = vec![
            ("orderType", request.order_type.clone()),
            ("symbol", request.symbol.clone()),
            ("side", request.side.clone()),
            ("size", request.size.clone()),
        ];

        if let Some(ref price) = request.limit_price {
            params.push(("limitPrice", price.clone()));
        }
        if let Some(ref stop) = request.stop_price {
            params.push(("stopPrice", stop.clone()));
        }
        if let Some(reduce_only) = request.reduce_only {
            params.push(("reduceOnly", reduce_only.to_string()));
        }
        if let Some(ref cli_id) = request.cli_ord_id {
            params.push(("cliOrdId", cli_id.clone()));
        }
        if let Some(ref trigger) = request.trigger_signal {
            params.push(("triggerSignal", trigger.clone()));
        }

        let response: KrakenResponse<SendOrderResult> =
            self.post_signed("/sendorder", &params).await?;

        response
            .data
            .map(|d| d.send_status)
            .ok_or_else(|| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: "Resposta de ordem inválida".into(),
            })
    }

    /// Cancela uma ordem por ID
    pub async fn cancel_order(&self, order_id: &str) -> ExchangeResult<CancelStatus> {
        let params = vec![("order_id", order_id.to_string())];

        let response: KrakenResponse<CancelOrderResult> =
            self.post_signed("/cancelorder", &params).await?;

        response
            .data
            .map(|d| d.cancel_status)
            .ok_or_else(|| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: "Resposta de cancelamento inválida".into(),
            })
    }

    /// Cancela uma ordem por client order ID
    pub async fn cancel_order_by_cli_id(&self, cli_ord_id: &str) -> ExchangeResult<CancelStatus> {
        let params = vec![("cliOrdId", cli_ord_id.to_string())];

        let response: KrakenResponse<CancelOrderResult> =
            self.post_signed("/cancelorder", &params).await?;

        response
            .data
            .map(|d| d.cancel_status)
            .ok_or_else(|| ExchangeError::ApiError {
                exchange: "kraken".into(),
                code: 0,
                message: "Resposta de cancelamento inválida".into(),
            })
    }

    /// Busca ordens abertas
    pub async fn get_open_orders(&self) -> ExchangeResult<Vec<KrakenOrder>> {
        let response: KrakenResponse<OpenOrders> = self.get_signed("/openorders").await?;

        Ok(response.data.map(|d| d.open_orders).unwrap_or_default())
    }

    // =========================================================================
    // API Privada - Histórico
    // =========================================================================

    /// Busca histórico de fills
    pub async fn get_fills(&self, last_fill_time: Option<&str>) -> ExchangeResult<Vec<Fill>> {
        let endpoint = if let Some(time) = last_fill_time {
            format!("/fills?lastFillTime={}", time)
        } else {
            "/fills".to_string()
        };

        let response: KrakenResponse<Fills> = self.get_signed(&endpoint).await?;

        Ok(response.data.map(|d| d.fills).unwrap_or_default())
    }
}

#[async_trait]
impl ExchangeGateway for KrakenFuturesClient {
    fn exchange_id(&self) -> ExchangeId {
        if self.is_demo {
            ExchangeId::Paper
        } else {
            ExchangeId::KrakenFutures
        }
    }

    fn is_paper_trading(&self) -> bool {
        self.is_demo
    }

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        let kraken_request = NewOrderRequest {
            order_type: match request.order_type {
                OrderType::Market => "mkt".into(),
                OrderType::Limit => "lmt".into(),
                OrderType::StopLoss | OrderType::StopLossLimit => "stp".into(),
                OrderType::TakeProfit | OrderType::TakeProfitLimit => "take_profit".into(),
                OrderType::TrailingStop => "stp".into(),
            },
            symbol: request.symbol.clone(),
            side: match request.side {
                OrderSide::Buy => "buy".into(),
                OrderSide::Sell => "sell".into(),
            },
            size: request.quantity.to_string(),
            limit_price: request.price.map(|p| p.to_string()),
            stop_price: request.stop_price.map(|p| p.to_string()),
            reduce_only: Some(request.reduce_only),
            cli_ord_id: None,
            trigger_signal: None,
        };

        let _status = self.send_order(&kraken_request).await?;

        // Converte para Order do core
        Ok(Order::from_request(
            request,
            self.exchange_id(),
            OrderSource::Manual { nota: None },
        ))
    }

    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<bool> {
        let status = self.cancel_order(order_id).await?;
        Ok(status.status == "cancelled")
    }

    async fn get_order_status(&self, _order_id: &str) -> ExchangeResult<Order> {
        // Kraken não tem endpoint para buscar ordem individual
        // Precisaria buscar todas as ordens e filtrar
        Err(ExchangeError::ApiError {
            exchange: "kraken".into(),
            code: 0,
            message: "Use get_open_orders para buscar ordens na Kraken".into(),
        })
    }

    async fn get_open_orders(&self, _symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
        let kraken_orders = self.get_open_orders().await?;

        let orders = kraken_orders
            .into_iter()
            .map(|ko| self.convert_kraken_order(&ko))
            .collect();

        Ok(orders)
    }

    async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
        let kraken_positions = self.get_open_positions().await?;

        let positions = kraken_positions
            .into_iter()
            .map(|kp| self.convert_kraken_position(&kp))
            .collect();

        Ok(positions)
    }

    async fn get_position(&self, symbol: &str) -> ExchangeResult<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        let account = self.get_account().await?;

        let mut balances = Vec::new();

        // Processa conta flex (cross margin)
        if let Some(flex) = account.flex {
            for (currency, data) in flex.currencies {
                let quantity = data
                    .quantity
                    .and_then(|q| Decimal::from_str(&q).ok())
                    .unwrap_or(Decimal::ZERO);
                let available = data
                    .available
                    .and_then(|a| Decimal::from_str(&a).ok())
                    .unwrap_or(Decimal::ZERO);

                if quantity > Decimal::ZERO {
                    balances.push(Balance {
                        asset: currency,
                        free: available,
                        locked: quantity - available,
                    });
                }
            }
        }

        // Processa conta cash (isolated margin)
        if let Some(cash) = account.cash {
            for (asset, balance_str) in cash.balances {
                let balance = Decimal::from_str(&balance_str).unwrap_or(Decimal::ZERO);
                if balance > Decimal::ZERO {
                    // Verifica se já existe na lista
                    if !balances.iter().any(|b| b.asset == asset) {
                        balances.push(Balance {
                            asset,
                            free: balance,
                            locked: Decimal::ZERO,
                        });
                    }
                }
            }
        }

        Ok(balances)
    }

    async fn set_leverage(&self, _symbol: &str, _leverage: u32) -> ExchangeResult<()> {
        // Kraken Futures usa níveis de margem baseados no tamanho da posição
        // Não há endpoint para definir alavancagem diretamente
        Ok(())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Usa endpoint público de instrumentos como health check
        let _: Vec<Instrument> = self.get_instruments().await?;
        Ok(())
    }
}

impl KrakenFuturesClient {
    /// Converte ordem da Kraken para Order do core
    fn convert_kraken_order(&self, ko: &KrakenOrder) -> Order {
        let status = match ko.status.as_str() {
            "untouched" => OrderStatus::Submitted,
            "partiallyFilled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Pending,
        };

        let side = match ko.side.as_str() {
            "buy" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };

        let order_type = match ko.order_type.as_str() {
            "mkt" => OrderType::Market,
            "lmt" => OrderType::Limit,
            "stp" => OrderType::StopLoss,
            "take_profit" => OrderType::TakeProfit,
            "ioc" => OrderType::Market,
            _ => OrderType::Market,
        };

        Order {
            id: OrderId::new(),
            client_order_id: ko.cli_ord_id.clone().unwrap_or_default(),
            exchange_order_id: Some(ko.order_id.clone()),
            exchange: self.exchange_id(),
            symbol: ko.symbol.clone(),
            side,
            order_type,
            quantity: Decimal::from_str(&ko.quantity).unwrap_or(Decimal::ZERO),
            price: ko
                .limit_price
                .as_ref()
                .and_then(|p| Decimal::from_str(p).ok()),
            stop_price: ko
                .stop_price
                .as_ref()
                .and_then(|p| Decimal::from_str(p).ok()),
            stop_loss: None,
            take_profit: None,
            time_in_force: TimeInForce::GTC,
            status,
            filled_quantity: Decimal::from_str(&ko.filled).unwrap_or(Decimal::ZERO),
            average_fill_price: None,
            source: OrderSource::Manual { nota: None },
            error_message: None,
            created_at: Utc::now(),
            submitted_at: Some(Utc::now()),
            filled_at: None,
            updated_at: Utc::now(),
        }
    }

    /// Converte posição da Kraken para Position do core
    fn convert_kraken_position(&self, kp: &KrakenPosition) -> Position {
        let side = match kp.side.as_str() {
            "long" => PositionSide::Long,
            _ => PositionSide::Short,
        };

        let size = Decimal::from_str(&kp.size).unwrap_or(Decimal::ZERO);
        let entry_price = Decimal::from_str(&kp.price).unwrap_or(Decimal::ZERO);
        let pnl = kp
            .pnl
            .as_ref()
            .and_then(|p| Decimal::from_str(p).ok())
            .unwrap_or(Decimal::ZERO);

        Position {
            id: PositionId::new(),
            exchange: self.exchange_id(),
            symbol: kp.symbol.clone(),
            side,
            quantity: size.abs(),
            entry_price,
            current_price: entry_price, // Será atualizado via ticker
            leverage: 1,                // Kraken usa margem dinâmica
            margin: Decimal::ZERO,
            unrealized_pnl: pnl,
            unrealized_pnl_pct: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            stop_loss_order_id: None,
            stop_loss_price: None,
            take_profit_order_id: None,
            take_profit_price: None,
            entry_order_id: OrderId::new(),
            status: PositionStatus::Open,
            liquidation_price: None,
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
    fn test_client_creation_demo() {
        let client = KrakenFuturesClient::demo("api_key", "api_secret");
        assert!(client.is_paper_trading());
        assert_eq!(client.base_url, DEMO_URL);
    }

    #[test]
    fn test_client_creation_mainnet() {
        let client = KrakenFuturesClient::mainnet("api_key", "api_secret");
        assert!(!client.is_paper_trading());
        assert_eq!(client.base_url, MAINNET_URL);
    }

    #[test]
    fn test_endpoint_url() {
        let client = KrakenFuturesClient::demo("key", "secret");
        let url = client.endpoint_url("/accounts");
        assert_eq!(url, "https://demo-futures.kraken.com/derivatives/api/v3/accounts");
    }

    #[test]
    fn test_new_order_request_market() {
        let request = NewOrderRequest::market("PI_XBTUSD", "buy", Decimal::from(1));
        assert_eq!(request.order_type, "mkt");
        assert_eq!(request.symbol, "PI_XBTUSD");
        assert_eq!(request.side, "buy");
        assert!(request.limit_price.is_none());
    }

    #[test]
    fn test_new_order_request_limit() {
        let request = NewOrderRequest::limit(
            "PI_XBTUSD",
            "sell",
            Decimal::from(1),
            Decimal::from(50000),
        );
        assert_eq!(request.order_type, "lmt");
        assert_eq!(request.limit_price, Some("50000".to_string()));
    }
}
