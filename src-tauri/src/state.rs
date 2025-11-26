//! Estado global da aplicação

use parking_lot::RwLock;
use robotrade_core::dto::TradingMode;
use robotrade_core::entities::{FearGreedData, Position};
use rust_decimal::Decimal;
use std::sync::Arc;

/// Estado compartilhado da aplicação
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
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
    /// P&L do dia
    daily_pnl: Decimal,
    /// Status das conexões
    binance_connected: bool,
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
                daily_pnl: Decimal::ZERO,
                binance_connected: false,
                fear_greed_api_connected: false,
                database_connected: false,
            })),
        }
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
        }
    }
}
