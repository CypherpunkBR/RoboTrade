//! Entidade Order (Ordem) e tipos relacionados

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ExchangeId;

/// ID único de uma ordem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

impl OrderId {
    /// Cria um novo ID de ordem
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Cria a partir de uma string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lado da ordem (compra ou venda)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    /// Compra
    Buy,
    /// Venda
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

/// Tipo de ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Ordem a mercado (executa imediatamente no melhor preço)
    Market,
    /// Ordem limitada (executa apenas no preço especificado ou melhor)
    Limit,
    /// Stop loss (vende quando o preço cai abaixo do especificado)
    StopLoss,
    /// Stop loss com limite
    StopLossLimit,
    /// Take profit (vende quando o preço atinge o objetivo)
    TakeProfit,
    /// Take profit com limite
    TakeProfitLimit,
    /// Trailing stop
    TrailingStop,
}

/// Tempo de validade da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    /// Good Till Cancelled - válida até cancelar
    GTC,
    /// Immediate Or Cancel - executa imediatamente ou cancela
    IOC,
    /// Fill Or Kill - executa tudo ou nada
    FOK,
    /// Good Till Date - válida até uma data
    GTD,
}

/// Status da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Pendente (na fila interna)
    Pending,
    /// Enviada para a exchange
    Submitted,
    /// Parcialmente preenchida
    PartiallyFilled,
    /// Totalmente preenchida
    Filled,
    /// Cancelada
    Cancelled,
    /// Rejeitada pela exchange
    Rejected,
    /// Expirada
    Expired,
    /// Falha interna
    Failed,
}

impl OrderStatus {
    /// Verifica se a ordem está em um estado final
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Expired
                | OrderStatus::Failed
        )
    }

    /// Verifica se a ordem está ativa
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending | OrderStatus::Submitted | OrderStatus::PartiallyFilled
        )
    }
}

/// Origem da ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderSource {
    /// Gerada por um sinal de estratégia
    Signal {
        signal_id: Uuid,
        strategy_id: String,
    },
    /// Criada manualmente pelo usuário
    Manual { nota: Option<String> },
    /// Stop loss de uma posição
    StopLoss { position_id: Uuid },
    /// Take profit de uma posição
    TakeProfit { position_id: Uuid },
}

/// Requisição para criar uma ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// Símbolo do par (ex: "BTCUSDT")
    pub symbol: String,
    /// Lado (compra/venda)
    pub side: OrderSide,
    /// Tipo de ordem
    pub order_type: OrderType,
    /// Quantidade
    pub quantity: Decimal,
    /// Preço (obrigatório para ordens limit)
    pub price: Option<Decimal>,
    /// Preço de stop (para ordens stop)
    pub stop_price: Option<Decimal>,
    /// Take profit
    pub take_profit: Option<Decimal>,
    /// Stop loss
    pub stop_loss: Option<Decimal>,
    /// Tempo de validade
    pub time_in_force: TimeInForce,
    /// Alavancagem (para futuros)
    pub leverage: Option<u32>,
    /// Reduce only (apenas fecha posição)
    pub reduce_only: bool,
}

impl OrderRequest {
    /// Cria uma ordem de mercado simples
    pub fn market(symbol: impl Into<String>, side: OrderSide, quantity: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            take_profit: None,
            stop_loss: None,
            time_in_force: TimeInForce::GTC,
            leverage: None,
            reduce_only: false,
        }
    }

    /// Cria uma ordem limitada
    pub fn limit(
        symbol: impl Into<String>,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            take_profit: None,
            stop_loss: None,
            time_in_force: TimeInForce::GTC,
            leverage: None,
            reduce_only: false,
        }
    }

    /// Define o stop loss
    pub fn with_stop_loss(mut self, price: Decimal) -> Self {
        self.stop_loss = Some(price);
        self
    }

    /// Define o take profit
    pub fn with_take_profit(mut self, price: Decimal) -> Self {
        self.take_profit = Some(price);
        self
    }

    /// Define a alavancagem
    pub fn with_leverage(mut self, leverage: u32) -> Self {
        self.leverage = Some(leverage);
        self
    }
}

/// Ordem completa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// ID interno único
    pub id: OrderId,
    /// ID da ordem no cliente (para rastreamento)
    pub client_order_id: String,
    /// ID da ordem na exchange (após submissão)
    pub exchange_order_id: Option<String>,
    /// Exchange onde a ordem foi enviada
    pub exchange: ExchangeId,
    /// Símbolo do par
    pub symbol: String,
    /// Lado (compra/venda)
    pub side: OrderSide,
    /// Tipo de ordem
    pub order_type: OrderType,
    /// Quantidade solicitada
    pub quantity: Decimal,
    /// Preço solicitado (para ordens limit)
    pub price: Option<Decimal>,
    /// Preço de stop
    pub stop_price: Option<Decimal>,
    /// Stop loss
    pub stop_loss: Option<Decimal>,
    /// Take profit
    pub take_profit: Option<Decimal>,
    /// Tempo de validade
    pub time_in_force: TimeInForce,
    /// Status atual
    pub status: OrderStatus,
    /// Quantidade já preenchida
    pub filled_quantity: Decimal,
    /// Preço médio de preenchimento
    pub average_fill_price: Option<Decimal>,
    /// Origem da ordem
    pub source: OrderSource,
    /// Mensagem de erro (se houver)
    pub error_message: Option<String>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data de submissão à exchange
    pub submitted_at: Option<DateTime<Utc>>,
    /// Data de preenchimento total
    pub filled_at: Option<DateTime<Utc>>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Cria uma nova ordem a partir de uma requisição
    pub fn from_request(request: OrderRequest, exchange: ExchangeId, source: OrderSource) -> Self {
        let now = Utc::now();
        let id = OrderId::new();

        Self {
            id: id.clone(),
            client_order_id: format!("RT_{}", id.0.simple()),
            exchange_order_id: None,
            exchange,
            symbol: request.symbol,
            side: request.side,
            order_type: request.order_type,
            quantity: request.quantity,
            price: request.price,
            stop_price: request.stop_price,
            stop_loss: request.stop_loss,
            take_profit: request.take_profit,
            time_in_force: request.time_in_force,
            status: OrderStatus::Pending,
            filled_quantity: Decimal::ZERO,
            average_fill_price: None,
            source,
            error_message: None,
            created_at: now,
            submitted_at: None,
            filled_at: None,
            updated_at: now,
        }
    }

    /// Verifica se a ordem está completamente preenchida
    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    /// Calcula a quantidade restante
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Calcula o valor total da ordem
    pub fn total_value(&self) -> Option<Decimal> {
        self.price.map(|p| p * self.quantity)
    }
}

/// Evento de preenchimento de ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFill {
    /// ID do preenchimento
    pub fill_id: String,
    /// ID da ordem
    pub order_id: OrderId,
    /// Quantidade preenchida neste fill
    pub quantity: Decimal,
    /// Preço de execução
    pub price: Decimal,
    /// Taxa cobrada
    pub fee: Decimal,
    /// Ativo da taxa
    pub fee_asset: String,
    /// Se foi maker (true) ou taker (false)
    pub is_maker: bool,
    /// Timestamp do fill
    pub filled_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_request_market() {
        let request = OrderRequest::market("BTCUSDT", OrderSide::Buy, dec!(0.01));

        assert_eq!(request.symbol, "BTCUSDT");
        assert_eq!(request.side, OrderSide::Buy);
        assert_eq!(request.order_type, OrderType::Market);
        assert_eq!(request.quantity, dec!(0.01));
        assert!(request.price.is_none());
    }

    #[test]
    fn test_order_request_with_stop_loss() {
        let request = OrderRequest::market("BTCUSDT", OrderSide::Buy, dec!(0.01))
            .with_stop_loss(dec!(40000))
            .with_take_profit(dec!(50000));

        assert_eq!(request.stop_loss, Some(dec!(40000)));
        assert_eq!(request.take_profit, Some(dec!(50000)));
    }

    #[test]
    fn test_order_status_is_final() {
        assert!(OrderStatus::Filled.is_final());
        assert!(OrderStatus::Cancelled.is_final());
        assert!(!OrderStatus::Pending.is_final());
        assert!(!OrderStatus::PartiallyFilled.is_final());
    }
}
