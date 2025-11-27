//! Estado global da aplicação

use parking_lot::RwLock;
use robotrade_core::dto::TradingMode;
use robotrade_core::entities::{FearGreedData, Position, Trade};
use robotrade_core::traits::FearGreedRepository;
use robotrade_infra::database::DbPool;
use robotrade_infra::repositories::SqliteFearGreedRepository;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::exchange_service::ExchangeService;
use crate::market_data_service::MarketDataService;
use crate::trading_worker::TradingWorker;

/// Estado compartilhado da aplicação
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
    /// Serviço de exchange
    pub exchange: ExchangeService,
    /// Pool do banco de dados
    db_pool: Arc<RwLock<Option<DbPool>>>,
    /// Worker de trading
    pub trading_worker: TradingWorker,
    /// Serviço de dados de mercado
    pub market_data: MarketDataService,
}

/// Dados internos do estado
struct AppStateInner {
    /// Modo de trading atual
    trading_mode: TradingMode,
    /// Se o trading automático está habilitado
    auto_trading_enabled: bool,
    /// Fear & Greed Index atual
    fear_greed: Option<FearGreedData>,
    /// Posições abertas em cache
    positions: Vec<Position>,
    /// Trades do histórico em cache
    trades: Vec<Trade>,
    /// P&L do dia
    daily_pnl: Decimal,
    /// Status das conexões
    binance_connected: bool,
    kraken_connected: bool,
    fear_greed_api_connected: bool,
    database_connected: bool,
}

impl AppState {
    /// Cria um novo estado
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                trading_mode: TradingMode::Paper,
                auto_trading_enabled: false,
                fear_greed: None,
                positions: Vec::new(),
                trades: Vec::new(),
                daily_pnl: Decimal::ZERO,
                binance_connected: false,
                kraken_connected: false,
                fear_greed_api_connected: false,
                database_connected: false,
            })),
            exchange: ExchangeService::new(),
            db_pool: Arc::new(RwLock::new(None)),
            trading_worker: TradingWorker::new(),
            market_data: MarketDataService::new(),
        }
    }

    /// Define o pool do banco de dados
    pub fn set_db_pool(&self, pool: DbPool) {
        *self.db_pool.write() = Some(pool);
    }

    /// Retorna o repositório Fear & Greed (se disponível)
    pub fn fear_greed_repository(&self) -> Option<SqliteFearGreedRepository> {
        self.db_pool
            .read()
            .clone()
            .map(SqliteFearGreedRepository::new)
    }

    /// Busca histórico do Fear & Greed do banco de dados
    pub async fn get_fear_greed_history(
        &self,
        days: u32,
    ) -> Result<Vec<FearGreedData>, String> {
        let repo = self
            .fear_greed_repository()
            .ok_or("Banco de dados não conectado")?;

        repo.find_history(days)
            .await
            .map_err(|e| format!("Erro ao buscar histórico: {}", e))
    }

    /// Retorna os trades em cache
    pub fn trades(&self) -> Vec<Trade> {
        self.inner.read().trades.clone()
    }

    /// Atualiza os trades
    pub fn set_trades(&self, trades: Vec<Trade>) {
        self.inner.write().trades = trades;
    }

    /// Adiciona um trade ao histórico
    pub fn add_trade(&self, trade: Trade) {
        self.inner.write().trades.push(trade);
    }

    /// Retorna o modo de trading atual
    pub fn trading_mode(&self) -> TradingMode {
        self.inner.read().trading_mode
    }

    /// Define o modo de trading
    pub fn set_trading_mode(&self, mode: TradingMode) {
        self.inner.write().trading_mode = mode;
    }

    /// Retorna se o trading automático está habilitado
    pub fn is_auto_trading_enabled(&self) -> bool {
        self.inner.read().auto_trading_enabled
    }

    /// Habilita/desabilita trading automático
    pub fn set_auto_trading(&self, enabled: bool) {
        self.inner.write().auto_trading_enabled = enabled;
    }

    /// Retorna o Fear & Greed Index atual
    pub fn fear_greed(&self) -> Option<FearGreedData> {
        self.inner.read().fear_greed.clone()
    }

    /// Atualiza o Fear & Greed Index
    pub fn set_fear_greed(&self, data: FearGreedData) {
        self.inner.write().fear_greed = Some(data);
    }

    /// Retorna as posições em cache
    pub fn positions(&self) -> Vec<Position> {
        self.inner.read().positions.clone()
    }

    /// Atualiza as posições
    pub fn set_positions(&self, positions: Vec<Position>) {
        self.inner.write().positions = positions;
    }

    /// Retorna o P&L do dia
    pub fn daily_pnl(&self) -> Decimal {
        self.inner.read().daily_pnl
    }

    /// Atualiza o P&L do dia
    pub fn set_daily_pnl(&self, pnl: Decimal) {
        self.inner.write().daily_pnl = pnl;
    }

    /// Retorna status das conexões
    pub fn connection_status(&self) -> (bool, bool, bool) {
        let inner = self.inner.read();
        (
            inner.binance_connected,
            inner.fear_greed_api_connected,
            inner.database_connected,
        )
    }

    /// Atualiza status da conexão Binance
    pub fn set_binance_connected(&self, connected: bool) {
        self.inner.write().binance_connected = connected;
    }

    /// Atualiza status da conexão Kraken
    pub fn set_kraken_connected(&self, connected: bool) {
        self.inner.write().kraken_connected = connected;
    }

    /// Atualiza status da conexão Fear & Greed API
    pub fn set_fear_greed_api_connected(&self, connected: bool) {
        self.inner.write().fear_greed_api_connected = connected;
    }

    /// Atualiza status da conexão do banco de dados
    pub fn set_database_connected(&self, connected: bool) {
        self.inner.write().database_connected = connected;
    }

    /// Número de posições abertas
    pub fn open_positions_count(&self) -> usize {
        self.inner.read().positions.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            exchange: self.exchange.clone(),
            db_pool: Arc::clone(&self.db_pool),
            trading_worker: self.trading_worker.clone(),
            market_data: self.market_data.clone(),
        }
    }
}
