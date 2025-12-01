//! Mensagens relacionadas a trading e execução de ordens

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::entities::{
  Balance, Exchange, Order, OrderId, OrderSide, OrderType, Position, PositionSide, TimeInForce,
};

/// Mensagens de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMessage {
  /// Criar nova ordem
  PlaceOrder(PlaceOrderRequest),

  /// Cancelar ordem existente
  CancelOrder { order_id: OrderId },

  /// Cancelar todas as ordens de um símbolo
  CancelAllOrders { symbol: Option<String> },

  /// Ordem criada com sucesso
  OrderCreated { order: Order },

  /// Ordem cancelada
  OrderCancelled { order_id: OrderId },

  /// Ordem preenchida (total ou parcialmente)
  OrderFilled {
    order_id: OrderId,
    filled_qty: Decimal,
    avg_price: Decimal,
    is_fully_filled: bool,
  },

  /// Ordem rejeitada
  OrderRejected { order_id: OrderId, reason: String },

  /// Atualização de posição
  PositionUpdate { position: Position },

  /// Posição fechada
  PositionClosed {
    symbol: String,
    side: PositionSide,
    realized_pnl: Decimal,
  },

  /// Atualização de balance
  BalanceUpdate { balance: Balance },

  /// Erro de trading
  TradingError { message: String },
}

/// Request para criar ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
  /// Exchange alvo
  pub exchange: Exchange,
  /// Símbolo do par
  pub symbol: String,
  /// Lado da ordem (compra/venda)
  pub side: OrderSide,
  /// Tipo da ordem
  pub order_type: OrderType,
  /// Quantidade
  pub quantity: Decimal,
  /// Preço (para ordens limit)
  pub price: Option<Decimal>,
  /// Stop price (para stop orders)
  pub stop_price: Option<Decimal>,
  /// Time in force
  pub time_in_force: Option<TimeInForce>,
  /// Reduce only (apenas reduz posição)
  pub reduce_only: Option<bool>,
  /// Client order ID (opcional)
  pub client_order_id: Option<String>,
}

/// Request para ajustar posição
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustPositionRequest {
  /// Exchange alvo
  pub exchange: Exchange,
  /// Símbolo do par
  pub symbol: String,
  /// Nova quantidade alvo (positivo = long, negativo = short)
  pub target_quantity: Decimal,
  /// Preço limite (opcional, usa market se None)
  pub limit_price: Option<Decimal>,
}

/// Request para fechar posição
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosePositionRequest {
  /// Exchange alvo
  pub exchange: Exchange,
  /// Símbolo do par
  pub symbol: String,
  /// Lado da posição a fechar
  pub side: PositionSide,
  /// Quantidade a fechar (None = fechar tudo)
  pub quantity: Option<Decimal>,
}

/// Comando de request-response para trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingRequest {
  /// Obter ordem por ID
  GetOrder { order_id: OrderId },

  /// Listar ordens abertas
  GetOpenOrders {
    exchange: Option<Exchange>,
    symbol: Option<String>,
  },

  /// Obter posição
  GetPosition { exchange: Exchange, symbol: String },

  /// Listar todas as posições
  GetAllPositions { exchange: Option<Exchange> },

  /// Obter balances
  GetBalances { exchange: Exchange },

  /// Definir alavancagem
  SetLeverage {
    exchange: Exchange,
    symbol: String,
    leverage: u32,
  },
}

/// Resposta para requests de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingResponse {
  /// Ordem retornada
  Order(Order),

  /// Lista de ordens
  Orders(Vec<Order>),

  /// Posição retornada
  Position(Option<Position>),

  /// Lista de posições
  Positions(Vec<Position>),

  /// Balances retornados
  Balances(Vec<Balance>),

  /// Alavancagem definida
  LeverageSet {
    exchange: Exchange,
    symbol: String,
    leverage: u32,
  },

  /// Erro na requisição
  Error { message: String },
}
