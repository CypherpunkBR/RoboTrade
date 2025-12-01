//! Paper Trading Client Implementation
//!
//! Simulador de exchange para desenvolvimento e testes.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};

use robotrade_core::{
    Balance, ExchangeError, ExchangeGateway, ExchangeId, ExchangeResult, Order, OrderId,
    OrderRequest, OrderSide, OrderSource, OrderStatus, OrderType, Position, PositionId,
    PositionSide, PositionStatus,
};

/// Cliente para paper trading (simulação local)
///
/// Mantém estado em memória para simular uma exchange real:
/// - Balances
/// - Posições abertas
/// - Ordens pendentes
/// - Configurações de alavancagem
#[derive(Clone)]
pub struct PaperTradingClient {
    balance: Arc<RwLock<Decimal>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    orders: Arc<RwLock<HashMap<String, Order>>>,
    leverage: Arc<RwLock<HashMap<String, u32>>>,
    order_counter: Arc<RwLock<u64>>,
}

impl PaperTradingClient {
    /// Cria um novo cliente paper trading com balance inicial de 10,000 USDT
    pub fn new() -> Self {
        Self::with_initial_balance(dec!(10000))
    }

    /// Cria um novo cliente com balance inicial customizado
    pub fn with_initial_balance(initial_balance: Decimal) -> Self {
        Self {
            balance: Arc::new(RwLock::new(initial_balance)),
            positions: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            leverage: Arc::new(RwLock::new(HashMap::new())),
            order_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Retorna o balance atual
    pub fn balance(&self) -> Decimal {
        *self.balance.read()
    }

    /// Define um novo balance
    pub fn set_balance(&self, balance: Decimal) {
        *self.balance.write() = balance;
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
                id: PositionId::new(),
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

    /// Atualiza preço atual de uma posição
    pub fn update_position_price(&self, symbol: &str, current_price: Decimal) {
        let mut positions = self.positions.write();
        if let Some(pos) = positions.get_mut(symbol) {
            pos.current_price = current_price;
            pos.unrealized_pnl = self.calculate_pnl(pos, current_price);
            if pos.entry_price != Decimal::ZERO {
                pos.unrealized_pnl_pct = (pos.unrealized_pnl / (pos.entry_price * pos.quantity)) * dec!(100);
            }
            pos.updated_at = chrono::Utc::now();
        }
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
            id: OrderId::new(),
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
            source: OrderSource::Manual { nota: None },
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
    use robotrade_core::TimeInForce;

    fn make_order_request(
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> OrderRequest {
        OrderRequest {
            symbol: symbol.into(),
            side,
            order_type,
            quantity,
            price,
            stop_price: None,
            stop_loss: None,
            take_profit: None,
            time_in_force: TimeInForce::GTC,
            leverage: None,
            reduce_only: false,
        }
    }

    #[tokio::test]
    async fn test_paper_trading_market_order() {
        let client = PaperTradingClient::new();

        let request = make_order_request(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Market,
            dec!(0.1),
            Some(dec!(50000)),
        );

        let order = client.submit_order(request).await.unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_quantity, dec!(0.1));

        // Verifica posição criada
        let positions = client.get_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].symbol, "BTCUSDT");
        assert_eq!(positions[0].quantity, dec!(0.1));
    }

    #[tokio::test]
    async fn test_paper_trading_limit_order() {
        let client = PaperTradingClient::new();

        let request = make_order_request(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Limit,
            dec!(0.1),
            Some(dec!(45000)),
        );

        let order = client.submit_order(request).await.unwrap();
        assert_eq!(order.status, OrderStatus::Submitted);

        // Verifica ordem pendente
        let open_orders = client.get_open_orders(None).await.unwrap();
        assert_eq!(open_orders.len(), 1);
    }

    #[tokio::test]
    async fn test_paper_trading_close_position() {
        let client = PaperTradingClient::new();

        // Abre posição long
        let open_request = make_order_request(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Market,
            dec!(0.1),
            Some(dec!(50000)),
        );
        client.submit_order(open_request).await.unwrap();

        // Fecha posição com lucro
        let close_request = make_order_request(
            "BTCUSDT",
            OrderSide::Sell,
            OrderType::Market,
            dec!(0.1),
            Some(dec!(55000)),
        );
        client.submit_order(close_request).await.unwrap();

        // Verifica posição fechada
        let positions = client.get_positions().await.unwrap();
        assert!(positions.is_empty());

        // Verifica lucro no balance
        let balances = client.get_balances().await.unwrap();
        assert!(balances[0].free > dec!(10000));
    }

    #[tokio::test]
    async fn test_paper_trading_leverage() {
        let client = PaperTradingClient::new();

        client.set_leverage("BTCUSDT", 10).await.unwrap();

        let request = make_order_request(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Market,
            dec!(0.1),
            Some(dec!(50000)),
        );
        client.submit_order(request).await.unwrap();

        let positions = client.get_positions().await.unwrap();
        assert_eq!(positions[0].leverage, 10);
        // Margem deve ser 1/10 do valor notional
        assert_eq!(positions[0].margin, dec!(500)); // 0.1 * 50000 / 10
    }
}
