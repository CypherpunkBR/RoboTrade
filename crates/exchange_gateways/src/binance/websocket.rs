//! Binance Futures WebSocket Client
//!
//! User Data Stream para receber atualizações de conta em tempo real.
//! Documentação: https://binance-docs.github.io/apidocs/futures/en/#user-data-streams

use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use super::client::BinanceFuturesClient;

/// Estado da conexão WebSocket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
  Disconnected,
  Connecting,
  Connected,
  Reconnecting,
  Failed,
}

/// Configuração do WebSocket
#[derive(Debug, Clone)]
pub struct BinanceWsConfig {
  /// Se deve usar testnet
  pub testnet: bool,
  /// Intervalo de keepalive (padrão: 30 min)
  pub keepalive_interval: Duration,
  /// Timeout de conexão
  pub connect_timeout: Duration,
  /// Máximo de tentativas de reconexão
  pub max_reconnect_attempts: u32,
  /// Backoff inicial para reconexão
  pub initial_backoff: Duration,
  /// Backoff máximo
  pub max_backoff: Duration,
}

impl Default for BinanceWsConfig {
  fn default() -> Self {
    Self {
      testnet: true,
      keepalive_interval: Duration::from_secs(30 * 60), // 30 min
      connect_timeout: Duration::from_secs(30),
      max_reconnect_attempts: 10,
      initial_backoff: Duration::from_secs(1),
      max_backoff: Duration::from_secs(60),
    }
  }
}

/// Cliente WebSocket para Binance Futures User Data Stream
pub struct BinanceWsClient {
  /// Cliente REST para operações de listen key
  rest_client: BinanceFuturesClient,
  /// Listen key atual
  listen_key: Option<String>,
  /// Stream WebSocket
  ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
  /// Estado da conexão
  state: ConnectionState,
  /// Configuração
  config: BinanceWsConfig,
  /// Última vez que enviamos keepalive
  last_keepalive: Instant,
  /// Contador de reconexões
  reconnect_count: u32,
}

impl BinanceWsClient {
  /// Cria novo cliente WebSocket
  pub fn new(rest_client: BinanceFuturesClient, config: BinanceWsConfig) -> Self {
    Self {
      rest_client,
      listen_key: None,
      ws_stream: None,
      state: ConnectionState::Disconnected,
      config,
      last_keepalive: Instant::now(),
      reconnect_count: 0,
    }
  }

  /// Retorna estado atual da conexão
  pub fn state(&self) -> ConnectionState {
    self.state
  }

  /// Verifica se está conectado
  pub fn is_connected(&self) -> bool {
    self.state == ConnectionState::Connected
  }

  /// Conecta ao User Data Stream
  pub async fn connect(&mut self) -> Result<(), String> {
    if self.state == ConnectionState::Connected {
      return Ok(());
    }

    self.state = ConnectionState::Connecting;
    info!("Conectando ao Binance User Data Stream...");

    // 1. Obter listen key via REST API
    let listen_key = self.rest_client.create_listen_key().await?;
    self.listen_key = Some(listen_key.clone());

    // 2. Construir URL do WebSocket
    let ws_url = if self.config.testnet {
      format!("wss://stream.binancefuture.com/ws/{}", listen_key)
    } else {
      format!("wss://fstream.binance.com/ws/{}", listen_key)
    };

    // 3. Conectar ao WebSocket
    let (ws_stream, _) = connect_async(&ws_url)
      .await
      .map_err(|e| format!("Erro ao conectar WebSocket: {}", e))?;

    self.ws_stream = Some(ws_stream);
    self.state = ConnectionState::Connected;
    self.last_keepalive = Instant::now();
    self.reconnect_count = 0;

    info!("Conectado ao Binance User Data Stream");
    Ok(())
  }

  /// Desconecta do WebSocket
  pub async fn disconnect(&mut self) {
    if let Some(mut ws) = self.ws_stream.take() {
      let _ = ws.close(None).await;
    }

    // Opcionalmente deletar listen key
    if let Some(key) = self.listen_key.take() {
      let _ = self.rest_client.delete_listen_key(&key).await;
    }

    self.state = ConnectionState::Disconnected;
    info!("Desconectado do Binance User Data Stream");
  }

  /// Envia keepalive para manter a conexão ativa
  /// Deve ser chamado a cada 30 minutos
  pub async fn keepalive(&mut self) -> Result<(), String> {
    if let Some(ref key) = self.listen_key {
      self.rest_client.keepalive_listen_key(key).await?;
      self.last_keepalive = Instant::now();
      debug!("Keepalive enviado para listen key");
    }
    Ok(())
  }

  /// Verifica se precisa enviar keepalive
  pub fn needs_keepalive(&self) -> bool {
    self.last_keepalive.elapsed() >= self.config.keepalive_interval
  }

  /// Recebe próximo evento do WebSocket
  pub async fn recv(&mut self) -> Option<BinanceWsEvent> {
    let ws = self.ws_stream.as_mut()?;

    match ws.next().await {
      Some(Ok(Message::Text(text))) => match serde_json::from_str::<BinanceWsMessage>(&text) {
        Ok(msg) => Some(self.parse_message(msg)),
        Err(e) => {
          warn!("Erro ao parsear mensagem: {} - {}", e, text);
          None
        }
      },
      Some(Ok(Message::Ping(data))) => {
        // Responde ao ping
        if let Some(ws) = self.ws_stream.as_mut() {
          let _ = ws.send(Message::Pong(data)).await;
        }
        None
      }
      Some(Ok(Message::Close(_))) => {
        self.state = ConnectionState::Disconnected;
        Some(BinanceWsEvent::Disconnected {
          reason: "Server closed connection".to_string(),
        })
      }
      Some(Err(e)) => {
        error!("Erro no WebSocket: {}", e);
        self.state = ConnectionState::Disconnected;
        Some(BinanceWsEvent::Error {
          message: e.to_string(),
        })
      }
      None => {
        self.state = ConnectionState::Disconnected;
        Some(BinanceWsEvent::Disconnected {
          reason: "Stream ended".to_string(),
        })
      }
      _ => None,
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

    // Calcula backoff exponencial
    let backoff = self
      .config
      .initial_backoff
      .mul_f32(2.0_f32.powi(self.reconnect_count as i32 - 1));
    let backoff = backoff.min(self.config.max_backoff);

    // Adiciona jitter (10%)
    let jitter = backoff.mul_f32(0.1 * rand_jitter());
    let wait_time = backoff + jitter;

    warn!(
      "Reconectando em {:?} (tentativa {}/{})",
      wait_time, self.reconnect_count, self.config.max_reconnect_attempts
    );

    tokio::time::sleep(wait_time).await;

    self.connect().await
  }

  /// Parseia mensagem do WebSocket
  fn parse_message(&self, msg: BinanceWsMessage) -> BinanceWsEvent {
    match msg.event_type.as_str() {
      "ACCOUNT_UPDATE" => {
        if let Some(data) = msg.account_update {
          BinanceWsEvent::AccountUpdate(AccountUpdateEvent {
            event_time: msg.event_time,
            transaction_time: msg.transaction_time.unwrap_or(msg.event_time),
            balances: data
              .balances
              .into_iter()
              .map(|b| BalanceUpdate {
                asset: b.asset,
                wallet_balance: parse_decimal(&b.wallet_balance),
                cross_wallet_balance: parse_decimal(&b.cross_wallet_balance),
                balance_change: parse_decimal(&b.balance_change),
              })
              .collect(),
            positions: data
              .positions
              .into_iter()
              .map(|p| PositionUpdate {
                symbol: p.symbol,
                position_amount: parse_decimal(&p.position_amount),
                entry_price: parse_decimal(&p.entry_price),
                unrealized_pnl: parse_decimal(&p.unrealized_pnl),
                margin_type: p.margin_type,
                position_side: p.position_side,
              })
              .collect(),
          })
        } else {
          BinanceWsEvent::Unknown {
            event_type: msg.event_type,
          }
        }
      }
      "ORDER_TRADE_UPDATE" => {
        if let Some(data) = msg.order_update {
          BinanceWsEvent::OrderUpdate(OrderUpdateEvent {
            event_time: msg.event_time,
            transaction_time: msg.transaction_time.unwrap_or(msg.event_time),
            symbol: data.symbol,
            client_order_id: data.client_order_id,
            side: data.side,
            order_type: data.order_type,
            time_in_force: data.time_in_force,
            original_quantity: parse_decimal(&data.original_quantity),
            original_price: parse_decimal(&data.original_price),
            average_price: parse_decimal(&data.average_price),
            stop_price: parse_decimal(&data.stop_price),
            execution_type: data.execution_type,
            order_status: data.order_status,
            order_id: data.order_id,
            last_filled_quantity: parse_decimal(&data.last_filled_quantity),
            cumulative_filled_quantity: parse_decimal(&data.cumulative_filled_quantity),
            last_filled_price: parse_decimal(&data.last_filled_price),
            commission_asset: data.commission_asset,
            commission: parse_decimal(&data.commission),
            order_trade_time: data.order_trade_time,
            trade_id: data.trade_id,
            realized_profit: parse_decimal(&data.realized_profit),
          })
        } else {
          BinanceWsEvent::Unknown {
            event_type: msg.event_type,
          }
        }
      }
      "ACCOUNT_CONFIG_UPDATE" => BinanceWsEvent::ConfigUpdate {
        event_time: msg.event_time,
      },
      "MARGIN_CALL" => BinanceWsEvent::MarginCall {
        event_time: msg.event_time,
      },
      "listenKeyExpired" => {
        warn!("Listen key expirado!");
        BinanceWsEvent::ListenKeyExpired
      }
      _ => BinanceWsEvent::Unknown {
        event_type: msg.event_type,
      },
    }
  }
}

/// Eventos recebidos do WebSocket
#[derive(Debug, Clone)]
pub enum BinanceWsEvent {
  /// Atualização de conta (balances e posições)
  AccountUpdate(AccountUpdateEvent),
  /// Atualização de ordem
  OrderUpdate(OrderUpdateEvent),
  /// Atualização de configuração
  ConfigUpdate { event_time: i64 },
  /// Chamada de margem
  MarginCall { event_time: i64 },
  /// Listen key expirado
  ListenKeyExpired,
  /// Desconectado
  Disconnected { reason: String },
  /// Erro
  Error { message: String },
  /// Evento desconhecido
  Unknown { event_type: String },
}

/// Evento de atualização de conta
#[derive(Debug, Clone)]
pub struct AccountUpdateEvent {
  pub event_time: i64,
  pub transaction_time: i64,
  pub balances: Vec<BalanceUpdate>,
  pub positions: Vec<PositionUpdate>,
}

/// Atualização de balance
#[derive(Debug, Clone)]
pub struct BalanceUpdate {
  pub asset: String,
  pub wallet_balance: Decimal,
  pub cross_wallet_balance: Decimal,
  pub balance_change: Decimal,
}

/// Atualização de posição
#[derive(Debug, Clone)]
pub struct PositionUpdate {
  pub symbol: String,
  pub position_amount: Decimal,
  pub entry_price: Decimal,
  pub unrealized_pnl: Decimal,
  pub margin_type: String,
  pub position_side: String,
}

/// Evento de atualização de ordem
#[derive(Debug, Clone)]
pub struct OrderUpdateEvent {
  pub event_time: i64,
  pub transaction_time: i64,
  pub symbol: String,
  pub client_order_id: String,
  pub side: String,
  pub order_type: String,
  pub time_in_force: String,
  pub original_quantity: Decimal,
  pub original_price: Decimal,
  pub average_price: Decimal,
  pub stop_price: Decimal,
  pub execution_type: String,
  pub order_status: String,
  pub order_id: i64,
  pub last_filled_quantity: Decimal,
  pub cumulative_filled_quantity: Decimal,
  pub last_filled_price: Decimal,
  pub commission_asset: Option<String>,
  pub commission: Decimal,
  pub order_trade_time: i64,
  pub trade_id: i64,
  pub realized_profit: Decimal,
}

// ============================================================================
// Estruturas de deserialização (formato Binance)
// ============================================================================

#[derive(Debug, Deserialize)]
struct BinanceWsMessage {
  #[serde(rename = "e")]
  event_type: String,
  #[serde(rename = "E")]
  event_time: i64,
  #[serde(rename = "T")]
  transaction_time: Option<i64>,
  #[serde(rename = "a")]
  account_update: Option<AccountUpdateData>,
  #[serde(rename = "o")]
  order_update: Option<OrderUpdateData>,
}

#[derive(Debug, Deserialize)]
struct AccountUpdateData {
  #[serde(rename = "B", default)]
  balances: Vec<BalanceData>,
  #[serde(rename = "P", default)]
  positions: Vec<PositionData>,
}

#[derive(Debug, Deserialize)]
struct BalanceData {
  #[serde(rename = "a")]
  asset: String,
  #[serde(rename = "wb")]
  wallet_balance: String,
  #[serde(rename = "cw")]
  cross_wallet_balance: String,
  #[serde(rename = "bc")]
  balance_change: String,
}

#[derive(Debug, Deserialize)]
struct PositionData {
  #[serde(rename = "s")]
  symbol: String,
  #[serde(rename = "pa")]
  position_amount: String,
  #[serde(rename = "ep")]
  entry_price: String,
  #[serde(rename = "up")]
  unrealized_pnl: String,
  #[serde(rename = "mt")]
  margin_type: String,
  #[serde(rename = "ps")]
  position_side: String,
}

#[derive(Debug, Deserialize)]
struct OrderUpdateData {
  #[serde(rename = "s")]
  symbol: String,
  #[serde(rename = "c")]
  client_order_id: String,
  #[serde(rename = "S")]
  side: String,
  #[serde(rename = "o")]
  order_type: String,
  #[serde(rename = "f")]
  time_in_force: String,
  #[serde(rename = "q")]
  original_quantity: String,
  #[serde(rename = "p")]
  original_price: String,
  #[serde(rename = "ap")]
  average_price: String,
  #[serde(rename = "sp")]
  stop_price: String,
  #[serde(rename = "x")]
  execution_type: String,
  #[serde(rename = "X")]
  order_status: String,
  #[serde(rename = "i")]
  order_id: i64,
  #[serde(rename = "l")]
  last_filled_quantity: String,
  #[serde(rename = "z")]
  cumulative_filled_quantity: String,
  #[serde(rename = "L")]
  last_filled_price: String,
  #[serde(rename = "N")]
  commission_asset: Option<String>,
  #[serde(rename = "n")]
  commission: String,
  #[serde(rename = "T")]
  order_trade_time: i64,
  #[serde(rename = "t")]
  trade_id: i64,
  #[serde(rename = "rp")]
  realized_profit: String,
}

/// Helper para parsear decimais
fn parse_decimal(s: &str) -> Decimal {
  Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

/// Helper para gerar jitter aleatório simples
fn rand_jitter() -> f32 {
  // Usa timestamp como fonte de entropia simples
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
  fn test_parse_account_update() {
    let json = r#"{
            "e": "ACCOUNT_UPDATE",
            "E": 1564745798939,
            "T": 1564745798938,
            "a": {
                "B": [
                    {"a": "USDT", "wb": "100.0", "cw": "100.0", "bc": "0"}
                ],
                "P": [
                    {"s": "BTCUSDT", "pa": "0.1", "ep": "50000", "up": "100", "mt": "cross", "ps": "LONG"}
                ]
            }
        }"#;

    let msg: BinanceWsMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.event_type, "ACCOUNT_UPDATE");
    assert!(msg.account_update.is_some());

    let data = msg.account_update.unwrap();
    assert_eq!(data.balances.len(), 1);
    assert_eq!(data.positions.len(), 1);
  }

  #[test]
  fn test_parse_order_update() {
    let json = r#"{
            "e": "ORDER_TRADE_UPDATE",
            "E": 1564745798939,
            "T": 1564745798938,
            "o": {
                "s": "BTCUSDT",
                "c": "abc123",
                "S": "BUY",
                "o": "LIMIT",
                "f": "GTC",
                "q": "0.1",
                "p": "50000",
                "ap": "50000",
                "sp": "0",
                "x": "TRADE",
                "X": "FILLED",
                "i": 12345,
                "l": "0.1",
                "z": "0.1",
                "L": "50000",
                "N": "USDT",
                "n": "0.5",
                "T": 1564745798938,
                "t": 67890,
                "rp": "0"
            }
        }"#;

    let msg: BinanceWsMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.event_type, "ORDER_TRADE_UPDATE");
    assert!(msg.order_update.is_some());

    let data = msg.order_update.unwrap();
    assert_eq!(data.symbol, "BTCUSDT");
    assert_eq!(data.order_status, "FILLED");
  }

  #[test]
  fn test_connection_state() {
    assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
    assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
  }
}
