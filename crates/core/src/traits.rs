//! Traits fundamentais do RoboTrade
//!
//! Define as interfaces/contratos que as implementações concretas devem seguir.
//! Permite injeção de dependência e mocking para testes.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::entities::{
    Balance, Candle, ExchangeId, FearGreedData, Order, OrderId, OrderRequest, Position, PositionId,
    Signal, SignalId, Ticker, TimeFrame, Trade, TradeId,
};
use crate::error::{ExchangeResult, InfraResult, MarketDataResult};

// ============================================================================
// Traits de Identificação
// ============================================================================

/// Trait para entidades com ID
pub trait Identifiable {
    /// Tipo do ID
    type Id;

    /// Retorna o ID da entidade
    fn id(&self) -> &Self::Id;
}

// Implementações de Identifiable
impl Identifiable for Order {
    type Id = OrderId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Identifiable for Position {
    type Id = PositionId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Identifiable for Signal {
    type Id = SignalId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Identifiable for Trade {
    type Id = TradeId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

// ============================================================================
// Traits de Repositório
// ============================================================================

/// Trait genérico para repositórios
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    /// Busca entidade por ID
    async fn find_by_id(&self, id: &Id) -> InfraResult<Option<T>>;

    /// Salva ou atualiza entidade
    async fn save(&self, entity: &T) -> InfraResult<()>;

    /// Remove entidade por ID
    async fn delete(&self, id: &Id) -> InfraResult<bool>;

    /// Lista todas as entidades
    async fn find_all(&self) -> InfraResult<Vec<T>>;
}

/// Repositório de ordens
#[async_trait]
pub trait OrderRepository: Repository<Order, OrderId> {
    /// Busca ordens por símbolo
    async fn find_by_symbol(&self, symbol: &str) -> InfraResult<Vec<Order>>;

    /// Busca ordens ativas
    async fn find_active(&self) -> InfraResult<Vec<Order>>;

    /// Busca ordens por exchange
    async fn find_by_exchange(&self, exchange: ExchangeId) -> InfraResult<Vec<Order>>;
}

/// Repositório de posições
#[async_trait]
pub trait PositionRepository: Repository<Position, PositionId> {
    /// Busca posições abertas
    async fn find_open(&self) -> InfraResult<Vec<Position>>;

    /// Busca posição aberta por símbolo
    async fn find_open_by_symbol(&self, symbol: &str) -> InfraResult<Option<Position>>;

    /// Busca posições por exchange
    async fn find_by_exchange(&self, exchange: ExchangeId) -> InfraResult<Vec<Position>>;
}

/// Repositório de sinais
#[async_trait]
pub trait SignalRepository: Repository<Signal, SignalId> {
    /// Busca sinais ativos
    async fn find_active(&self) -> InfraResult<Vec<Signal>>;

    /// Busca sinais aguardando confirmação
    async fn find_awaiting_confirmation(&self) -> InfraResult<Vec<Signal>>;

    /// Busca sinais por estratégia
    async fn find_by_strategy(&self, strategy_id: &str) -> InfraResult<Vec<Signal>>;

    /// Busca sinais expirados
    async fn find_expired(&self) -> InfraResult<Vec<Signal>>;
}

/// Repositório de trades
#[async_trait]
pub trait TradeRepository: Repository<Trade, TradeId> {
    /// Busca trades por símbolo
    async fn find_by_symbol(&self, symbol: &str) -> InfraResult<Vec<Trade>>;

    /// Busca trades em um período
    async fn find_in_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> InfraResult<Vec<Trade>>;

    /// Busca trades por exchange
    async fn find_by_exchange(&self, exchange: ExchangeId) -> InfraResult<Vec<Trade>>;

    /// Conta total de trades
    async fn count(&self) -> InfraResult<u64>;
}

/// Repositório de candles
#[async_trait]
pub trait CandleRepository: Send + Sync {
    /// Busca candles por símbolo e timeframe
    async fn find_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        limit: usize,
    ) -> InfraResult<Vec<Candle>>;

    /// Busca candles em um período
    async fn find_candles_in_period(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> InfraResult<Vec<Candle>>;

    /// Salva candles
    async fn save_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        candles: &[Candle],
    ) -> InfraResult<()>;

    /// Retorna o último candle salvo
    async fn get_last_candle(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
    ) -> InfraResult<Option<Candle>>;
}

/// Repositório de Fear & Greed Index
#[async_trait]
pub trait FearGreedRepository: Send + Sync {
    /// Busca dados históricos do índice
    async fn find_history(&self, days: u32) -> InfraResult<Vec<FearGreedData>>;

    /// Busca dado por data
    async fn find_by_date(&self, date: NaiveDate) -> InfraResult<Option<FearGreedData>>;

    /// Salva dado do índice
    async fn save(&self, data: &FearGreedData) -> InfraResult<()>;

    /// Retorna o dado mais recente
    async fn get_latest(&self) -> InfraResult<Option<FearGreedData>>;
}

// ============================================================================
// Traits de Provedores de Dados
// ============================================================================

/// Provedor de dados de mercado (candles, ticker)
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Identificador do provedor
    fn provider_id(&self) -> &str;

    /// Exchange associada
    fn exchange(&self) -> ExchangeId;

    /// Busca candles históricos
    async fn fetch_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        limit: usize,
    ) -> MarketDataResult<Vec<Candle>>;

    /// Busca ticker atual
    async fn fetch_ticker(&self, symbol: &str) -> MarketDataResult<Ticker>;

    /// Busca múltiplos tickers
    async fn fetch_tickers(&self, symbols: &[String]) -> MarketDataResult<Vec<Ticker>>;

    /// Verifica se o provedor está saudável
    async fn health_check(&self) -> MarketDataResult<bool>;
}

/// Provedor do Fear & Greed Index
#[async_trait]
pub trait FearGreedProvider: Send + Sync {
    /// Identificador do provedor
    fn provider_id(&self) -> &str;

    /// Busca o índice atual
    async fn fetch_current(&self) -> MarketDataResult<FearGreedData>;

    /// Busca histórico do índice
    async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>>;

    /// Verifica se o provedor está saudável
    async fn health_check(&self) -> MarketDataResult<bool>;
}

// ============================================================================
// Traits de Exchange
// ============================================================================

/// Gateway para operações na exchange
#[async_trait]
pub trait ExchangeGateway: Send + Sync {
    /// Identificador da exchange
    fn exchange_id(&self) -> ExchangeId;

    /// Verifica se está em modo paper trading
    fn is_paper_trading(&self) -> bool;

    /// Envia uma ordem
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;

    /// Cancela uma ordem
    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<bool>;

    /// Busca status de uma ordem
    async fn get_order_status(&self, order_id: &str) -> ExchangeResult<Order>;

    /// Lista ordens abertas
    async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>>;

    /// Busca posições abertas
    async fn get_positions(&self) -> ExchangeResult<Vec<Position>>;

    /// Busca posição por símbolo
    async fn get_position(&self, symbol: &str) -> ExchangeResult<Option<Position>>;

    /// Busca saldos da conta
    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>>;

    /// Define alavancagem para um símbolo
    async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()>;

    /// Verifica conectividade
    async fn ping(&self) -> ExchangeResult<()>;
}

// ============================================================================
// Traits de Estratégia
// ============================================================================

/// Parâmetros de uma estratégia
pub trait StrategyParams: Send + Sync + Clone {
    /// Valida os parâmetros
    fn validate(&self) -> Result<(), String>;
}

/// Contexto fornecido para a estratégia tomar decisões
pub struct StrategyContext {
    /// Candles recentes
    pub candles: Vec<Candle>,
    /// Ticker atual
    pub ticker: Option<Ticker>,
    /// Posição atual (se houver)
    pub current_position: Option<Position>,
    /// Fear & Greed atual
    pub fear_greed: Option<FearGreedData>,
    /// Timestamp atual
    pub timestamp: DateTime<Utc>,
}

/// Estratégia de trading
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Identificador único da estratégia
    fn id(&self) -> &str;

    /// Nome amigável
    fn name(&self) -> &str;

    /// Descrição da estratégia
    fn description(&self) -> &str;

    /// Símbolos que a estratégia opera
    fn symbols(&self) -> &[String];

    /// Timeframes necessários
    fn required_timeframes(&self) -> &[TimeFrame];

    /// Mínimo de candles necessários para operar
    fn min_candles_required(&self) -> usize;

    /// Avalia o mercado e gera sinais
    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal>;

    /// Verifica se deve fechar uma posição existente
    async fn should_close_position(&self, context: &StrategyContext) -> Option<Signal>;
}

// ============================================================================
// Traits de Notificação
// ============================================================================

/// Nível de urgência da notificação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    /// Informativo
    Info,
    /// Sucesso
    Success,
    /// Aviso
    Warning,
    /// Erro/Urgente
    Error,
}

/// Payload de notificação a ser enviada (para serviços de notificação)
#[derive(Debug, Clone)]
pub struct NotificationPayload {
    /// Título
    pub title: String,
    /// Mensagem
    pub body: String,
    /// Nível de urgência
    pub level: NotificationLevel,
    /// Dados extras (JSON)
    pub data: Option<serde_json::Value>,
}

/// Serviço de notificações
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Envia uma notificação
    async fn send(&self, notification: NotificationPayload) -> InfraResult<()>;

    /// Notifica sobre novo sinal
    async fn notify_signal(&self, signal: &Signal) -> InfraResult<()>;

    /// Notifica sobre ordem preenchida
    async fn notify_order_filled(&self, order: &Order) -> InfraResult<()>;

    /// Notifica sobre posição fechada
    async fn notify_position_closed(&self, trade: &Trade) -> InfraResult<()>;

    /// Notifica sobre erro crítico
    async fn notify_error(&self, error: &str) -> InfraResult<()>;
}

// ============================================================================
// Traits de Configuração
// ============================================================================

/// Serviço de configuração
#[async_trait]
pub trait ConfigService: Send + Sync {
    /// Busca valor de configuração
    async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> InfraResult<Option<T>>;

    /// Define valor de configuração
    async fn set<T: serde::Serialize + Send + Sync>(&self, key: &str, value: &T)
        -> InfraResult<()>;

    /// Remove configuração
    async fn remove(&self, key: &str) -> InfraResult<bool>;

    /// Verifica se configuração existe
    async fn exists(&self, key: &str) -> InfraResult<bool>;
}

// ============================================================================
// Traits de Backtest
// ============================================================================

/// Resultado de um backtest
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// ID do backtest
    pub id: String,
    /// Estratégia testada
    pub strategy_id: String,
    /// Símbolo testado
    pub symbol: String,
    /// Período inicial
    pub start_date: DateTime<Utc>,
    /// Período final
    pub end_date: DateTime<Utc>,
    /// Capital inicial
    pub initial_capital: Decimal,
    /// Capital final
    pub final_capital: Decimal,
    /// Total de trades
    pub total_trades: u32,
    /// Trades vencedores
    pub winning_trades: u32,
    /// Win rate
    pub win_rate: Decimal,
    /// Profit factor
    pub profit_factor: Decimal,
    /// Sharpe ratio
    pub sharpe_ratio: Decimal,
    /// Max drawdown
    pub max_drawdown: Decimal,
    /// Trades executados
    pub trades: Vec<Trade>,
}

/// Motor de backtest
#[async_trait]
pub trait BacktestEngine: Send + Sync {
    /// Executa backtest de uma estratégia
    async fn run(
        &self,
        strategy: &dyn Strategy,
        symbol: &str,
        timeframe: TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        initial_capital: Decimal,
    ) -> crate::error::AnalyticsResult<BacktestResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_levels() {
        let notification = NotificationPayload {
            title: "Teste".into(),
            body: "Mensagem".into(),
            level: NotificationLevel::Info,
            data: None,
        };

        assert_eq!(notification.level, NotificationLevel::Info);
    }
}
