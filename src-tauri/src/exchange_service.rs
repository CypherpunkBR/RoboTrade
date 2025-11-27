//! Serviço de integração com exchanges
//!
//! Gerencia a conexão com exchanges e execução de operações.
//! Por ora, foca em paper trading com simulação local.

use async_trait::async_trait;
use parking_lot::RwLock;
use robotrade_core::entities::{
    Balance, ExchangeId, Order, OrderRequest, OrderSide, OrderStatus, OrderType, Position,
    PositionSide, PositionStatus,
};
use robotrade_core::error::{ExchangeError, ExchangeResult};
use robotrade_core::traits::ExchangeGateway;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Tipo de exchange ativo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveExchange {
    BinanceFutures,
    KrakenFutures,
    Paper,
}

/// Serviço de exchange simplificado (apenas paper trading por ora)
pub struct ExchangeService {
    paper_client: PaperTradingClient,
    active_exchange: Arc<RwLock<ActiveExchange>>,
}

impl ExchangeService {
    /// Cria um novo serviço
    pub fn new() -> Self {
        Self {
            paper_client: PaperTradingClient::new(),
            active_exchange: Arc::new(RwLock::new(ActiveExchange::Paper)),
        }
    }

    /// Retorna exchange ativo
    pub fn active_exchange(&self) -> ActiveExchange {
        *self.active_exchange.read()
    }

    /// Verifica se está conectado
    pub async fn is_connected(&self) -> bool {
        self.paper_client.ping().await.is_ok()
    }

    /// Envia uma ordem
    pub async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        self.paper_client.submit_order(request).await
    }

    /// Cancela uma ordem
    pub async fn cancel_order(&self, _symbol: &str, order_id: &str) -> ExchangeResult<bool> {
        self.paper_client.cancel_order(order_id).await
    }

    /// Busca ordens abertas
    pub async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
        self.paper_client.get_open_orders(symbol).await
    }

    /// Busca posições
    pub async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
        self.paper_client.get_positions().await
    }

    /// Busca saldos
    pub async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        self.paper_client.get_balances().await
    }

    /// Define alavancagem
    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()> {
        self.paper_client.set_leverage(symbol, leverage).await
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
                    let total_value =
                        pos.entry_price * pos.quantity + price * order.filled_quantity;
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
            self.orders.write().insert(order.id.to_string(), order.clone());
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
        orders.get(order_id).cloned().ok_or(ExchangeError::ApiError {
            exchange: "paper".into(),
            code: 0,
            message: format!("Ordem {} não encontrada", order_id),
        })
    }

    async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
        let orders = self.orders.read();
        let open_orders: Vec<Order> = orders
            .values()
            .filter(|o| {
                o.status == OrderStatus::Submitted || o.status == OrderStatus::PartiallyFilled
            })
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
        let margin_used: Decimal = self
            .positions
            .read()
            .values()
            .map(|p| p.margin)
            .sum();

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
}
