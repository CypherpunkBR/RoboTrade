//! Mock Exchange Client - Deterministic for Testing (Work in Progress)

use async_trait::async_trait;
use parking_lot::RwLock;
use robotrade_core::traits::ExchangeGateway;
use robotrade_core::{
  Balance, ExchangeError, ExchangeId, ExchangeResult, Order, OrderRequest, Position,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock exchange client - 100% deterministic
#[allow(dead_code)]
pub struct MockExchangeClient {
  fixed_prices: Arc<RwLock<HashMap<String, Decimal>>>,
  balances: Arc<RwLock<HashMap<String, Decimal>>>,
}

#[allow(dead_code)]
impl MockExchangeClient {
  pub fn new() -> Self {
    Self {
      fixed_prices: Arc::new(RwLock::new(HashMap::new())),
      balances: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn deterministic() -> Self {
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), dec!(50000.0));

    Self {
      fixed_prices: Arc::new(RwLock::new(prices)),
      balances: Arc::new(RwLock::new(HashMap::new())),
    }
  }
}

impl Default for MockExchangeClient {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl ExchangeGateway for MockExchangeClient {
  fn exchange_id(&self) -> ExchangeId {
    ExchangeId::from("mock")
  }

  fn is_paper_trading(&self) -> bool {
    true
  }

  async fn submit_order(&self, _request: OrderRequest) -> ExchangeResult<Order> {
    Err(ExchangeError::InternalError {
      message: "Mock not fully implemented yet".to_string(),
    })
  }

  async fn cancel_order(&self, _order_id: &str) -> ExchangeResult<bool> {
    Ok(false)
  }

  async fn get_order_status(&self, _order_id: &str) -> ExchangeResult<Order> {
    Err(ExchangeError::InternalError {
      message: "Mock not fully implemented yet".to_string(),
    })
  }

  async fn get_open_orders(&self, _symbol: Option<&str>) -> ExchangeResult<Vec<Order>> {
    Ok(vec![])
  }

  async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
    Ok(vec![])
  }

  async fn get_position(&self, _symbol: &str) -> ExchangeResult<Option<Position>> {
    Ok(None)
  }

  async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
    Ok(vec![])
  }

  async fn set_leverage(&self, _symbol: &str, _leverage: u32) -> ExchangeResult<()> {
    Ok(())
  }

  async fn ping(&self) -> ExchangeResult<()> {
    Ok(())
  }
}
