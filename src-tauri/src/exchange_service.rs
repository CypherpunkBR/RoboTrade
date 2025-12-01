//! Serviço de integração com exchanges
//!
//! Gerencia a conexão com exchanges e execução de operações.
//! Suporta Binance Futures, Kraken Futures e Paper Trading.

use async_trait::async_trait;
use parking_lot::RwLock;
use robotrade_core::entities::{
  Balance, ExchangeId, Order, OrderRequest, OrderSide, OrderStatus, OrderType, Position,
  PositionSide, PositionStatus,
};
use robotrade_core::error::{ExchangeError, ExchangeResult};
use robotrade_core::traits::ExchangeGateway;
use robotrade_exchange_gateways::binance::BinanceFuturesClient;
use robotrade_exchange_gateways::kraken::KrakenFuturesClient;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{debug, info, warn};

/// Tipo de exchange ativo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveExchange {
  BinanceFutures,
  KrakenFutures,
  Paper,
}

/// Credenciais de exchange
#[derive(Debug, Clone)]
pub struct ExchangeCredentials {
  pub api_key: String,
  pub api_secret: String,
  pub is_testnet: bool,
}

/// Serviço de exchange com suporte a múltiplas exchanges
pub struct ExchangeService {
  paper_client: PaperTradingClient,
  binance_client: Arc<TokioRwLock<Option<BinanceFuturesClient>>>,
  kraken_client: Arc<TokioRwLock<Option<KrakenFuturesClient>>>,
  active_exchange: Arc<RwLock<ActiveExchange>>,
}

impl ExchangeService {
  /// Cria um novo serviço
  pub fn new() -> Self {
    Self {
      paper_client: PaperTradingClient::new(),
      binance_client: Arc::new(TokioRwLock::new(None)),
      kraken_client: Arc::new(TokioRwLock::new(None)),
      active_exchange: Arc::new(RwLock::new(ActiveExchange::Paper)),
    }
  }

  /// Inicializa a exchange Binance com credenciais (async)
  pub async fn initialize_binance_async(&self, credentials: ExchangeCredentials) {
    let client = if credentials.is_testnet {
      BinanceFuturesClient::testnet(credentials.api_key, credentials.api_secret)
    } else {
      BinanceFuturesClient::mainnet(credentials.api_key, credentials.api_secret)
    };

    *self.binance_client.write().await = Some(client);
    info!(
      testnet = credentials.is_testnet,
      "Cliente Binance Futures inicializado"
    );
  }

  /// Inicializa a exchange Kraken com credenciais (async)
  pub async fn initialize_kraken_async(&self, credentials: ExchangeCredentials) {
    let client = if credentials.is_testnet {
      KrakenFuturesClient::demo(credentials.api_key, credentials.api_secret)
    } else {
      KrakenFuturesClient::mainnet(credentials.api_key, credentials.api_secret)
    };

    *self.kraken_client.write().await = Some(client);
    info!(
      demo = credentials.is_testnet,
      "Cliente Kraken Futures inicializado"
    );
  }

  /// Define a exchange ativa
  pub fn set_active_exchange(&self, exchange: ActiveExchange) {
    *self.active_exchange.write() = exchange;
    info!(exchange = ?exchange, "Exchange ativa definida");
  }

  /// Retorna exchange ativo
  pub fn active_exchange(&self) -> ActiveExchange {
    *self.active_exchange.read()
  }

  /// Verifica se está conectado
  pub async fn is_connected(&self) -> bool {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.ping().await.is_ok(),
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          client.ping().await.is_ok()
        } else {
          false
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          client.ping().await.is_ok()
        } else {
          false
        }
      }
    }
  }

  /// Verifica se Binance está inicializada
  pub async fn is_binance_initialized(&self) -> bool {
    self.binance_client.read().await.is_some()
  }

  /// Verifica se Kraken está inicializada
  pub async fn is_kraken_initialized(&self) -> bool {
    self.kraken_client.read().await.is_some()
  }

  /// Envia uma ordem
  pub async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.submit_order(request).await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          // Use trait method which returns core::Order
          ExchangeGateway::submit_order(client, request).await
        } else {
          Err(ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: "Cliente Binance não inicializado".into(),
          })
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::submit_order(client, request).await
        } else {
          Err(ExchangeError::ApiError {
            exchange: "kraken".into(),
            code: 0,
            message: "Cliente Kraken não inicializado".into(),
          })
        }
      }
    }
  }

  /// Cancela uma ordem
  pub async fn cancel_order(&self, _symbol: &str, order_id: &str) -> ExchangeResult<bool> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.cancel_order(order_id).await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::cancel_order(client, order_id).await
        } else {
          Err(ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: "Cliente Binance não inicializado".into(),
          })
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::cancel_order(client, order_id).await
        } else {
          Err(ExchangeError::ApiError {
            exchange: "kraken".into(),
            code: 0,
            message: "Cliente Kraken não inicializado".into(),
          })
        }
      }
    }
  }

  /// Busca ordens abertas
  pub async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.get_open_orders(symbol).await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_open_orders(client, symbol).await
        } else {
          Ok(vec![])
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_open_orders(client, symbol).await
        } else {
          Ok(vec![])
        }
      }
    }
  }

  /// Busca posições
  pub async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.get_positions().await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_positions(client).await
        } else {
          Ok(vec![])
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_positions(client).await
        } else {
          Ok(vec![])
        }
      }
    }
  }

  /// Busca posições de todas as exchanges inicializadas
  pub async fn get_all_positions(&self) -> Vec<Position> {
    let mut all_positions = Vec::new();

    // Paper positions
    if let Ok(positions) = self.paper_client.get_positions().await {
      all_positions.extend(positions);
    }

    // Binance positions
    {
      let guard = self.binance_client.read().await;
      if let Some(ref client) = *guard {
        match ExchangeGateway::get_positions(client).await {
          Ok(positions) => all_positions.extend(positions),
          Err(e) => warn!("Erro ao buscar posições Binance: {}", e),
        }
      }
    }

    // Kraken positions
    {
      let guard = self.kraken_client.read().await;
      if let Some(ref client) = *guard {
        match ExchangeGateway::get_positions(client).await {
          Ok(positions) => all_positions.extend(positions),
          Err(e) => warn!("Erro ao buscar posições Kraken: {}", e),
        }
      }
    }

    all_positions
  }

  /// Busca ordens de todas as exchanges inicializadas
  pub async fn get_all_open_orders(&self) -> Vec<Order> {
    let mut all_orders = Vec::new();

    // Paper orders
    if let Ok(orders) = self.paper_client.get_open_orders(None).await {
      all_orders.extend(orders);
    }

    // Binance orders
    {
      let guard = self.binance_client.read().await;
      if let Some(ref client) = *guard {
        match ExchangeGateway::get_open_orders(client, None).await {
          Ok(orders) => all_orders.extend(orders),
          Err(e) => warn!("Erro ao buscar ordens Binance: {}", e),
        }
      }
    }

    // Kraken orders
    {
      let guard = self.kraken_client.read().await;
      if let Some(ref client) = *guard {
        match ExchangeGateway::get_open_orders(client, None).await {
          Ok(orders) => all_orders.extend(orders),
          Err(e) => warn!("Erro ao buscar ordens Kraken: {}", e),
        }
      }
    }

    all_orders
  }

  /// Busca saldos
  pub async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.get_balances().await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_balances(client).await
        } else {
          Ok(vec![])
        }
      }
      ActiveExchange::KrakenFutures => {
        let guard = self.kraken_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::get_balances(client).await
        } else {
          Ok(vec![])
        }
      }
    }
  }

  /// Busca saldos de todas as exchanges
  pub async fn get_all_balances(&self) -> HashMap<String, Vec<Balance>> {
    let mut balances = HashMap::new();

    // Paper balances
    if let Ok(b) = self.paper_client.get_balances().await {
      debug!(count = b.len(), "Saldos paper trading carregados");
      balances.insert("paper".to_string(), b);
    }

    // Binance balances
    {
      let guard = self.binance_client.read().await;
      if let Some(ref client) = *guard {
        debug!("Buscando saldos da Binance...");
        match ExchangeGateway::get_balances(client).await {
          Ok(b) => {
            info!(
                count = b.len(),
                balances = ?b.iter().map(|bal| format!("{}: {} free, {} locked", bal.asset, bal.free, bal.locked)).collect::<Vec<_>>(),
                "Saldos Binance carregados"
            );
            balances.insert("binance".to_string(), b);
          }
          Err(e) => {
            warn!("Erro ao buscar saldos Binance: {}", e);
          }
        }
      } else {
        debug!("Cliente Binance não inicializado");
      }
    }

    // Kraken balances
    {
      let guard = self.kraken_client.read().await;
      if let Some(ref client) = *guard {
        debug!("Buscando saldos da Kraken...");
        match ExchangeGateway::get_balances(client).await {
          Ok(b) => {
            info!(
                count = b.len(),
                balances = ?b.iter().map(|bal| format!("{}: {} free, {} locked", bal.asset, bal.free, bal.locked)).collect::<Vec<_>>(),
                "Saldos Kraken carregados"
            );
            balances.insert("kraken".to_string(), b);
          }
          Err(e) => {
            warn!("Erro ao buscar saldos Kraken: {}", e);
          }
        }
      } else {
        debug!("Cliente Kraken não inicializado");
      }
    }

    balances
  }

  /// Define alavancagem
  pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()> {
    match self.active_exchange() {
      ActiveExchange::Paper => self.paper_client.set_leverage(symbol, leverage).await,
      ActiveExchange::BinanceFutures => {
        let guard = self.binance_client.read().await;
        if let Some(ref client) = *guard {
          ExchangeGateway::set_leverage(client, symbol, leverage).await
        } else {
          Err(ExchangeError::ApiError {
            exchange: "binance".into(),
            code: 0,
            message: "Cliente Binance não inicializado".into(),
          })
        }
      }
      ActiveExchange::KrakenFutures => {
        // Kraken não suporta mudança de alavancagem via API
        warn!("Kraken não suporta mudança de alavancagem via API");
        Ok(())
      }
    }
  }

  /// Retorna referência ao cliente Kraken (para acesso direto)
  pub async fn kraken_client(
    &self,
  ) -> tokio::sync::RwLockReadGuard<'_, Option<KrakenFuturesClient>> {
    self.kraken_client.read().await
  }

  /// Retorna referência ao cliente Binance (para acesso direto)
  pub async fn binance_client(
    &self,
  ) -> tokio::sync::RwLockReadGuard<'_, Option<BinanceFuturesClient>> {
    self.binance_client.read().await
  }
}

impl Default for ExchangeService {
  fn default() -> Self {
    Self::new()
  }
}

impl Clone for ExchangeService {
  fn clone(&self) -> Self {
    Self {
      paper_client: self.paper_client.clone(),
      binance_client: Arc::clone(&self.binance_client),
      kraken_client: Arc::clone(&self.kraken_client),
      active_exchange: Arc::clone(&self.active_exchange),
    }
  }
}

// =============================================================================
// Paper Trading Client
// =============================================================================

/// Cliente para paper trading (simulação local)
#[derive(Clone)]
pub struct PaperTradingClient {
  balance: Arc<RwLock<Decimal>>,
  positions: Arc<RwLock<HashMap<String, Position>>>,
  orders: Arc<RwLock<HashMap<String, Order>>>,
  leverage: Arc<RwLock<HashMap<String, u32>>>,
  order_counter: Arc<RwLock<u64>>,
}

impl PaperTradingClient {
  /// Cria um novo cliente paper trading
  pub fn new() -> Self {
    Self {
      balance: Arc::new(RwLock::new(dec!(10000))),
      positions: Arc::new(RwLock::new(HashMap::new())),
      orders: Arc::new(RwLock::new(HashMap::new())),
      leverage: Arc::new(RwLock::new(HashMap::new())),
      order_counter: Arc::new(RwLock::new(0)),
    }
  }

  /// Gera próximo ID de ordem
  fn next_order_id(&self) -> String {
    let mut counter = self.order_counter.write();
    *counter += 1;
    format!("PAPER_{}", *counter)
  }

  /// Simula execução de ordem de mercado
  fn execute_market_order(&self, order: &mut Order, price: Decimal) {
    order.status = OrderStatus::Filled;
    order.filled_quantity = order.quantity;
    order.average_fill_price = Some(price);
    order.filled_at = Some(chrono::Utc::now());

    // Atualiza posições
    let mut positions = self.positions.write();
    let symbol = &order.symbol;

    if let Some(pos) = positions.get_mut(symbol) {
      // Atualiza posição existente
      match (order.side, pos.side) {
        (OrderSide::Buy, PositionSide::Long) | (OrderSide::Sell, PositionSide::Short) => {
          // Aumenta posição
          let total_value = pos.entry_price * pos.quantity + price * order.filled_quantity;
          pos.quantity += order.filled_quantity;
          pos.entry_price = total_value / pos.quantity;
        }
        (OrderSide::Buy, PositionSide::Short) | (OrderSide::Sell, PositionSide::Long) => {
          // Reduz ou fecha posição
          if order.filled_quantity >= pos.quantity {
            // Fecha posição
            let pnl = self.calculate_pnl(pos, price);
            *self.balance.write() += pnl;
            positions.remove(symbol);
          } else {
            // Reduz posição
            pos.quantity -= order.filled_quantity;
          }
        }
      }
    } else {
      // Cria nova posição
      let leverage = self.leverage.read().get(symbol).copied().unwrap_or(1);
      let side = match order.side {
        OrderSide::Buy => PositionSide::Long,
        OrderSide::Sell => PositionSide::Short,
      };

      let new_position = Position {
        id: robotrade_core::entities::PositionId::new(),
        exchange: ExchangeId::Paper,
        symbol: symbol.clone(),
        side,
        quantity: order.filled_quantity,
        entry_price: price,
        current_price: price,
        leverage,
        margin: order.filled_quantity * price / Decimal::from(leverage),
        unrealized_pnl: Decimal::ZERO,
        unrealized_pnl_pct: Decimal::ZERO,
        realized_pnl: Decimal::ZERO,
        stop_loss_order_id: None,
        stop_loss_price: order.stop_loss,
        take_profit_order_id: None,
        take_profit_price: order.take_profit,
        entry_order_id: order.id.clone(),
        status: PositionStatus::Open,
        liquidation_price: None,
        opened_at: chrono::Utc::now(),
        closed_at: None,
        updated_at: chrono::Utc::now(),
      };

      positions.insert(symbol.clone(), new_position);
    }
  }

  /// Calcula P&L de uma posição
  fn calculate_pnl(&self, pos: &Position, exit_price: Decimal) -> Decimal {
    let diff = exit_price - pos.entry_price;
    let direction = match pos.side {
      PositionSide::Long => Decimal::ONE,
      PositionSide::Short => -Decimal::ONE,
    };
    diff * pos.quantity * direction * Decimal::from(pos.leverage)
  }
}

impl Default for PaperTradingClient {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl ExchangeGateway for PaperTradingClient {
  fn exchange_id(&self) -> ExchangeId {
    ExchangeId::Paper
  }

  fn is_paper_trading(&self) -> bool {
    true
  }

  async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
    let order_id = self.next_order_id();

    let mut order = Order {
      id: robotrade_core::entities::OrderId::new(),
      client_order_id: order_id.clone(),
      exchange_order_id: Some(order_id),
      exchange: ExchangeId::Paper,
      symbol: request.symbol.clone(),
      side: request.side,
      order_type: request.order_type,
      quantity: request.quantity,
      price: request.price,
      stop_price: request.stop_price,
      stop_loss: request.stop_loss,
      take_profit: request.take_profit,
      time_in_force: request.time_in_force,
      status: OrderStatus::Submitted,
      filled_quantity: Decimal::ZERO,
      average_fill_price: None,
      source: robotrade_core::entities::OrderSource::Manual { nota: None },
      error_message: None,
      created_at: chrono::Utc::now(),
      submitted_at: Some(chrono::Utc::now()),
      filled_at: None,
      updated_at: chrono::Utc::now(),
    };

    // Simula execução imediata para ordens de mercado
    if request.order_type == OrderType::Market {
      // Usa preço de entrada sugerido ou um preço simulado
      let price = request.price.unwrap_or(dec!(50000)); // Preço simulado para BTC
      self.execute_market_order(&mut order, price);
    } else {
      // Ordens limite ficam pendentes
      self
        .orders
        .write()
        .insert(order.id.to_string(), order.clone());
    }

    info!(
        symbol = %request.symbol,
        side = ?request.side,
        quantity = %request.quantity,
        status = ?order.status,
        "Ordem paper trading processada"
    );

    Ok(order)
  }

  async fn cancel_order(&self, order_id: &str) -> ExchangeResult<bool> {
    let mut orders = self.orders.write();
    if let Some(mut order) = orders.remove(order_id) {
      order.status = OrderStatus::Cancelled;
      debug!(order_id = %order_id, "Ordem paper trading cancelada");
      Ok(true)
    } else {
      Ok(false)
    }
  }

  async fn get_order_status(&self, order_id: &str) -> ExchangeResult<Order> {
    let orders = self.orders.read();
    orders
      .get(order_id)
      .cloned()
      .ok_or(ExchangeError::ApiError {
        exchange: "paper".into(),
        code: 0,
        message: format!("Ordem {} não encontrada", order_id),
      })
  }

  async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
    let orders = self.orders.read();
    let open_orders: Vec<Order> = orders
      .values()
      .filter(|o| o.status == OrderStatus::Submitted || o.status == OrderStatus::PartiallyFilled)
      .filter(|o| symbol.map(|s| o.symbol == s).unwrap_or(true))
      .cloned()
      .collect();
    Ok(open_orders)
  }

  async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
    let positions = self.positions.read();
    Ok(positions.values().cloned().collect())
  }

  async fn get_position(&self, symbol: &str) -> ExchangeResult<Option<Position>> {
    let positions = self.positions.read();
    Ok(positions.get(symbol).cloned())
  }

  async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
    let balance = *self.balance.read();

    // Calcula margem usada
    let margin_used: Decimal = self.positions.read().values().map(|p| p.margin).sum();

    Ok(vec![Balance {
      asset: "USDT".into(),
      free: balance - margin_used,
      locked: margin_used,
    }])
  }

  async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()> {
    self.leverage.write().insert(symbol.to_string(), leverage);
    debug!(symbol = %symbol, leverage = leverage, "Alavancagem paper trading definida");
    Ok(())
  }

  async fn ping(&self) -> ExchangeResult<()> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use robotrade_core::entities::TimeInForce;

  #[tokio::test]
  async fn test_paper_trading_order() {
    let client = PaperTradingClient::new();

    let request = OrderRequest {
      symbol: "BTCUSDT".into(),
      side: OrderSide::Buy,
      order_type: OrderType::Market,
      quantity: dec!(0.1),
      price: Some(dec!(50000)),
      stop_price: None,
      stop_loss: None,
      take_profit: None,
      time_in_force: TimeInForce::GTC,
      leverage: Some(10),
      reduce_only: false,
    };

    let order = client.submit_order(request).await.unwrap();
    assert_eq!(order.status, OrderStatus::Filled);

    let positions = client.get_positions().await.unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].symbol, "BTCUSDT");
  }

  #[tokio::test]
  async fn test_paper_trading_balances() {
    let client = PaperTradingClient::new();

    let balances = client.get_balances().await.unwrap();
    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].asset, "USDT");
    assert_eq!(balances[0].free, dec!(10000));
  }

  #[tokio::test]
  async fn test_exchange_service_initialization() {
    let service = ExchangeService::new();

    // Inicialmente não tem exchanges reais
    assert!(!service.is_binance_initialized().await);
    assert!(!service.is_kraken_initialized().await);
    assert_eq!(service.active_exchange(), ActiveExchange::Paper);
  }
}
