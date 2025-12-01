//! Kraken Futures WebSocket Client
//!
//! WebSocket API for real-time account updates.
//! Documentation: https://docs.kraken.com/api/docs/guides/global-intro

use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

use super::signer::KrakenSigner;

/// Estado da conexão WebSocket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Reconnecting,
    Failed,
}

/// Configuração do WebSocket Kraken
#[derive(Debug, Clone)]
pub struct KrakenWsConfig {
    /// Se deve usar demo/testnet
    pub demo: bool,
    /// Timeout de conexão
    pub connect_timeout: Duration,
    /// Máximo de tentativas de reconexão
    pub max_reconnect_attempts: u32,
    /// Backoff inicial para reconexão
    pub initial_backoff: Duration,
    /// Backoff máximo
    pub max_backoff: Duration,
    /// Intervalo de heartbeat
    pub heartbeat_interval: Duration,
}

impl Default for KrakenWsConfig {
    fn default() -> Self {
        Self {
            demo: true,
            connect_timeout: Duration::from_secs(30),
            max_reconnect_attempts: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

/// Cliente WebSocket para Kraken Futures
pub struct KrakenWsClient {
    /// API Key
    api_key: String,
    /// API Secret
    api_secret: String,
    /// Signer para autenticação
    signer: KrakenSigner,
    /// Stream WebSocket
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    /// Estado da conexão
    state: ConnectionState,
    /// Configuração
    config: KrakenWsConfig,
    /// Último heartbeat
    last_heartbeat: Instant,
    /// Contador de reconexões
    reconnect_count: u32,
    /// Challenge recebido do servidor
    challenge: Option<String>,
}

impl KrakenWsClient {
    /// Cria novo cliente WebSocket
    pub fn new(api_key: String, api_secret: String, config: KrakenWsConfig) -> Self {
        Self {
            signer: KrakenSigner::new(&api_secret),
            api_key,
            api_secret,
            ws_stream: None,
            state: ConnectionState::Disconnected,
            config,
            last_heartbeat: Instant::now(),
            reconnect_count: 0,
            challenge: None,
        }
    }

    /// Retorna estado atual da conexão
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Verifica se está conectado e autenticado
    pub fn is_authenticated(&self) -> bool {
        self.state == ConnectionState::Authenticated
    }

    /// Conecta ao WebSocket
    pub async fn connect(&mut self) -> Result<(), String> {
        if self.state == ConnectionState::Connected || self.state == ConnectionState::Authenticated {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;
        info!("Conectando ao Kraken Futures WebSocket...");

        // URL do WebSocket
        let ws_url = if self.config.demo {
            "wss://demo-futures.kraken.com/ws/v1"
        } else {
            "wss://futures.kraken.com/ws/v1"
        };

        // Conectar
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| format!("Erro ao conectar WebSocket: {}", e))?;

        self.ws_stream = Some(ws_stream);
        self.state = ConnectionState::Connected;
        self.last_heartbeat = Instant::now();
        self.reconnect_count = 0;

        info!("Conectado ao Kraken Futures WebSocket");
        Ok(())
    }

    /// Autentica no WebSocket
    pub async fn authenticate(&mut self) -> Result<(), String> {
        if self.state != ConnectionState::Connected {
            return Err("Não conectado".to_string());
        }

        // Primeiro, solicita um challenge
        let challenge_request = serde_json::json!({
            "event": "challenge",
            "api_key": self.api_key
        });

        self.send_json(&challenge_request).await?;

        // Aguarda resposta com challenge
        // O challenge será processado no recv()
        info!("Challenge solicitado, aguardando resposta...");

        Ok(())
    }

    /// Completa autenticação com o challenge
    async fn complete_authentication(&mut self, challenge: &str) -> Result<(), String> {
        // Assina o challenge
        let signed_challenge = self.signer.sign_challenge(challenge)?;

        let auth_request = serde_json::json!({
            "event": "subscribe",
            "feed": "fills",
            "api_key": self.api_key,
            "original_challenge": challenge,
            "signed_challenge": signed_challenge
        });

        self.send_json(&auth_request).await?;

        // Também se inscreve em outras feeds
        let feeds = ["open_orders", "open_positions", "balances", "account_log"];
        for feed in feeds {
            let sub_request = serde_json::json!({
                "event": "subscribe",
                "feed": feed,
                "api_key": self.api_key,
                "original_challenge": challenge,
                "signed_challenge": &signed_challenge
            });
            self.send_json(&sub_request).await?;
        }

        self.state = ConnectionState::Authenticated;
        info!("Autenticado no Kraken Futures WebSocket");
        Ok(())
    }

    /// Envia mensagem JSON
    async fn send_json(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let text = serde_json::to_string(value)
            .map_err(|e| format!("Erro ao serializar JSON: {}", e))?;

        if let Some(ws) = self.ws_stream.as_mut() {
            ws.send(Message::Text(text.into()))
                .await
                .map_err(|e| format!("Erro ao enviar mensagem: {}", e))?;
        }
        Ok(())
    }

    /// Desconecta do WebSocket
    pub async fn disconnect(&mut self) {
        if let Some(mut ws) = self.ws_stream.take() {
            let _ = ws.close(None).await;
        }

        self.state = ConnectionState::Disconnected;
        self.challenge = None;
        info!("Desconectado do Kraken Futures WebSocket");
    }

    /// Envia heartbeat
    pub async fn heartbeat(&mut self) -> Result<(), String> {
        let ping = serde_json::json!({
            "event": "ping"
        });
        self.send_json(&ping).await?;
        self.last_heartbeat = Instant::now();
        debug!("Heartbeat enviado");
        Ok(())
    }

    /// Verifica se precisa enviar heartbeat
    pub fn needs_heartbeat(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.config.heartbeat_interval
    }

    /// Recebe próximo evento do WebSocket
    pub async fn recv(&mut self) -> Option<KrakenWsEvent> {
        let ws = self.ws_stream.as_mut()?;

        match ws.next().await {
            Some(Ok(Message::Text(text))) => {
                debug!("Mensagem recebida: {}", text);
                self.parse_message(&text).await
            }
            Some(Ok(Message::Ping(data))) => {
                if let Some(ws) = self.ws_stream.as_mut() {
                    let _ = ws.send(Message::Pong(data)).await;
                }
                None
            }
            Some(Ok(Message::Close(_))) => {
                self.state = ConnectionState::Disconnected;
                Some(KrakenWsEvent::Disconnected {
                    reason: "Server closed connection".to_string(),
                })
            }
            Some(Err(e)) => {
                error!("Erro no WebSocket: {}", e);
                self.state = ConnectionState::Disconnected;
                Some(KrakenWsEvent::Error {
                    message: e.to_string(),
                })
            }
            None => {
                self.state = ConnectionState::Disconnected;
                Some(KrakenWsEvent::Disconnected {
                    reason: "Stream ended".to_string(),
                })
            }
            _ => None,
        }
    }

    /// Parseia mensagem do WebSocket
    async fn parse_message(&mut self, text: &str) -> Option<KrakenWsEvent> {
        // Tenta parsear como JSON
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                warn!("Erro ao parsear JSON: {} - {}", e, text);
                return None;
            }
        };

        // Verifica se é um evento
        if let Some(event) = value.get("event").and_then(|e| e.as_str()) {
            match event {
                "challenge" => {
                    if let Some(challenge) = value.get("message").and_then(|m| m.as_str()) {
                        self.challenge = Some(challenge.to_string());
                        // Automaticamente completa autenticação
                        if let Err(e) = self.complete_authentication(challenge).await {
                            error!("Erro ao completar autenticação: {}", e);
                            return Some(KrakenWsEvent::Error { message: e });
                        }
                        return Some(KrakenWsEvent::ChallengeReceived);
                    }
                }
                "subscribed" => {
                    let feed = value.get("feed").and_then(|f| f.as_str()).unwrap_or("unknown");
                    info!("Inscrito em feed: {}", feed);
                    return Some(KrakenWsEvent::Subscribed {
                        feed: feed.to_string(),
                    });
                }
                "error" => {
                    let message = value.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                    return Some(KrakenWsEvent::Error {
                        message: message.to_string(),
                    });
                }
                "pong" => {
                    return None; // Ignorar pongs
                }
                _ => {}
            }
        }

        // Verifica se é uma mensagem de feed
        if let Some(feed) = value.get("feed").and_then(|f| f.as_str()) {
            match feed {
                "fills" => {
                    return self.parse_fills(&value);
                }
                "open_orders" | "open_orders_snapshot" => {
                    return self.parse_orders(&value);
                }
                "open_positions" => {
                    return self.parse_positions(&value);
                }
                "balances" => {
                    return self.parse_balances(&value);
                }
                "account_log" | "account_log_snapshot" => {
                    return self.parse_account_log(&value);
                }
                _ => {
                    debug!("Feed desconhecido: {}", feed);
                }
            }
        }

        None
    }

    /// Parseia fills (trades)
    fn parse_fills(&self, value: &serde_json::Value) -> Option<KrakenWsEvent> {
        let fills = value.get("fills")?.as_array()?;

        let trades: Vec<KrakenFillEvent> = fills
            .iter()
            .filter_map(|f| {
                Some(KrakenFillEvent {
                    fill_id: f.get("fill_id")?.as_str()?.to_string(),
                    symbol: f.get("instrument")?.as_str()?.to_string(),
                    side: f.get("side")?.as_str()?.to_string(),
                    price: parse_decimal(f.get("price")?.as_str()?),
                    size: parse_decimal(f.get("size")?.as_str()?),
                    order_id: f.get("order_id")?.as_str()?.to_string(),
                    fill_type: f.get("fillType")?.as_str()?.to_string(),
                    fill_time: f.get("time")?.as_i64()?,
                })
            })
            .collect();

        if trades.is_empty() {
            None
        } else {
            Some(KrakenWsEvent::Fills(trades))
        }
    }

    /// Parseia orders
    fn parse_orders(&self, value: &serde_json::Value) -> Option<KrakenWsEvent> {
        let orders = value.get("orders")?.as_array()?;

        let order_updates: Vec<KrakenOrderEvent> = orders
            .iter()
            .filter_map(|o| {
                Some(KrakenOrderEvent {
                    order_id: o.get("order_id")?.as_str()?.to_string(),
                    symbol: o.get("instrument")?.as_str()?.to_string(),
                    side: o.get("side")?.as_str()?.to_string(),
                    order_type: o.get("orderType")?.as_str()?.to_string(),
                    quantity: parse_decimal(o.get("qty")?.as_str().unwrap_or("0")),
                    filled_quantity: parse_decimal(o.get("filled")?.as_str().unwrap_or("0")),
                    limit_price: o.get("limitPrice").and_then(|p| p.as_str()).map(parse_decimal),
                    stop_price: o.get("stopPrice").and_then(|p| p.as_str()).map(parse_decimal),
                    status: o.get("status")?.as_str()?.to_string(),
                    timestamp: o.get("lastUpdateTimestamp")?.as_i64()?,
                })
            })
            .collect();

        if order_updates.is_empty() {
            None
        } else {
            Some(KrakenWsEvent::Orders(order_updates))
        }
    }

    /// Parseia positions
    fn parse_positions(&self, value: &serde_json::Value) -> Option<KrakenWsEvent> {
        let positions = value.get("positions")?.as_array()?;

        let position_updates: Vec<KrakenPositionEvent> = positions
            .iter()
            .filter_map(|p| {
                Some(KrakenPositionEvent {
                    symbol: p.get("instrument")?.as_str()?.to_string(),
                    side: p.get("side")?.as_str()?.to_string(),
                    size: parse_decimal(p.get("size")?.as_str()?),
                    entry_price: parse_decimal(p.get("price")?.as_str()?),
                    unrealized_pnl: parse_decimal(p.get("unrealizedPnl").and_then(|v| v.as_str()).unwrap_or("0")),
                    realized_pnl: parse_decimal(p.get("realizedPnl").and_then(|v| v.as_str()).unwrap_or("0")),
                    liquidation_threshold: p.get("liquidationThreshold").and_then(|v| v.as_str()).map(parse_decimal),
                })
            })
            .collect();

        if position_updates.is_empty() {
            None
        } else {
            Some(KrakenWsEvent::Positions(position_updates))
        }
    }

    /// Parseia balances
    fn parse_balances(&self, value: &serde_json::Value) -> Option<KrakenWsEvent> {
        // Kraken retorna balances como objeto com currencies
        let balances_obj = value.get("flex_futures")?;

        let mut balance_updates = Vec::new();

        if let Some(currencies) = balances_obj.get("currencies").and_then(|c| c.as_object()) {
            for (currency, data) in currencies {
                if let Some(balance_data) = data.as_object() {
                    let quantity = balance_data
                        .get("quantity")
                        .and_then(|q| q.as_str())
                        .map(parse_decimal)
                        .unwrap_or(Decimal::ZERO);

                    let available = balance_data
                        .get("available")
                        .and_then(|a| a.as_str())
                        .map(parse_decimal)
                        .unwrap_or(Decimal::ZERO);

                    balance_updates.push(KrakenBalanceEvent {
                        currency: currency.clone(),
                        quantity,
                        available,
                    });
                }
            }
        }

        if balance_updates.is_empty() {
            None
        } else {
            Some(KrakenWsEvent::Balances(balance_updates))
        }
    }

    /// Parseia account log
    fn parse_account_log(&self, value: &serde_json::Value) -> Option<KrakenWsEvent> {
        let logs = value.get("logs")?.as_array()?;

        let log_entries: Vec<KrakenAccountLogEntry> = logs
            .iter()
            .filter_map(|l| {
                Some(KrakenAccountLogEntry {
                    id: l.get("id")?.as_str()?.to_string(),
                    log_type: l.get("info")?.as_str()?.to_string(),
                    asset: l.get("asset")?.as_str()?.to_string(),
                    amount: parse_decimal(l.get("amount")?.as_str()?),
                    balance: parse_decimal(l.get("new_balance").and_then(|v| v.as_str()).unwrap_or("0")),
                    timestamp: l.get("booking_uid").and_then(|v| v.as_i64()).unwrap_or(0),
                })
            })
            .collect();

        if log_entries.is_empty() {
            None
        } else {
            Some(KrakenWsEvent::AccountLog(log_entries))
        }
    }

    /// Tenta reconectar com backoff exponencial
    pub async fn reconnect(&mut self) -> Result<(), String> {
        if self.reconnect_count >= self.config.max_reconnect_attempts {
            self.state = ConnectionState::Failed;
            return Err("Máximo de tentativas de reconexão atingido".to_string());
        }

        self.state = ConnectionState::Reconnecting;
        self.reconnect_count += 1;

        let backoff = self
            .config
            .initial_backoff
            .mul_f32(2.0_f32.powi(self.reconnect_count as i32 - 1));
        let backoff = backoff.min(self.config.max_backoff);

        // Adiciona jitter
        let jitter = backoff.mul_f32(0.1 * rand_jitter());
        let wait_time = backoff + jitter;

        warn!(
            "Reconectando em {:?} (tentativa {}/{})",
            wait_time, self.reconnect_count, self.config.max_reconnect_attempts
        );

        tokio::time::sleep(wait_time).await;

        self.connect().await?;
        self.authenticate().await
    }
}

/// Eventos recebidos do WebSocket Kraken
#[derive(Debug, Clone)]
pub enum KrakenWsEvent {
    /// Challenge recebido (para autenticação)
    ChallengeReceived,
    /// Inscrito em feed
    Subscribed { feed: String },
    /// Fills (trades executados)
    Fills(Vec<KrakenFillEvent>),
    /// Atualizações de ordens
    Orders(Vec<KrakenOrderEvent>),
    /// Atualizações de posições
    Positions(Vec<KrakenPositionEvent>),
    /// Atualizações de saldo
    Balances(Vec<KrakenBalanceEvent>),
    /// Log de conta (transferências, funding, etc)
    AccountLog(Vec<KrakenAccountLogEntry>),
    /// Desconectado
    Disconnected { reason: String },
    /// Erro
    Error { message: String },
}

/// Evento de fill (trade executado)
#[derive(Debug, Clone)]
pub struct KrakenFillEvent {
    pub fill_id: String,
    pub symbol: String,
    pub side: String,
    pub price: Decimal,
    pub size: Decimal,
    pub order_id: String,
    pub fill_type: String,
    pub fill_time: i64,
}

/// Evento de ordem
#[derive(Debug, Clone)]
pub struct KrakenOrderEvent {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub status: String,
    pub timestamp: i64,
}

/// Evento de posição
#[derive(Debug, Clone)]
pub struct KrakenPositionEvent {
    pub symbol: String,
    pub side: String,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub liquidation_threshold: Option<Decimal>,
}

/// Evento de saldo
#[derive(Debug, Clone)]
pub struct KrakenBalanceEvent {
    pub currency: String,
    pub quantity: Decimal,
    pub available: Decimal,
}

/// Entrada do log de conta
#[derive(Debug, Clone)]
pub struct KrakenAccountLogEntry {
    pub id: String,
    pub log_type: String,
    pub asset: String,
    pub amount: Decimal,
    pub balance: Decimal,
    pub timestamp: i64,
}

/// Helper para parsear decimais
fn parse_decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

/// Helper para gerar jitter aleatório simples
fn rand_jitter() -> f32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (now % 1000) as f32 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Authenticated);
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_decimal("100.5"), Decimal::from_str("100.5").unwrap());
        assert_eq!(parse_decimal("invalid"), Decimal::ZERO);
    }
}
