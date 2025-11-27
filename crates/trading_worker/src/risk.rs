//! # Risk Manager
//!
//! Gerenciador de risco responsável por:
//! - Validar novas ordens contra limites
//! - Monitorar exposição total
//! - Verificar limites de perda diária
//! - Controlar alavancagem máxima

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use robotrade_core::entities::{
    OrderRequest, OrderSide, Position, PositionSide, Trade,
};

/// Resultado da validação de risco
#[derive(Debug, Clone)]
pub enum RiskCheckResult {
    /// Ordem aprovada
    Approved,
    /// Ordem rejeitada com motivo
    Rejected(RiskRejectionReason),
    /// Ordem aprovada com ajustes
    ApprovedWithAdjustment {
        original_quantity: Decimal,
        adjusted_quantity: Decimal,
        reason: String,
    },
}

/// Motivo de rejeição
#[derive(Debug, Clone)]
pub enum RiskRejectionReason {
    /// Limite de perda diária atingido
    DailyLossLimitReached { current_loss: Decimal, limit: Decimal },
    /// Exposição máxima excedida
    MaxExposureExceeded { current: Decimal, max: Decimal },
    /// Número máximo de posições atingido
    MaxPositionsReached { current: usize, max: usize },
    /// Alavancagem muito alta
    LeverageTooHigh { requested: u32, max: u32 },
    /// Tamanho de posição muito grande
    PositionSizeTooLarge { size: Decimal, max: Decimal },
    /// Concentração em um único ativo
    ConcentrationTooHigh { symbol: String, percentage: Decimal },
    /// Trading desabilitado
    TradingDisabled { reason: String },
    /// Circuit breaker ativo
    CircuitBreakerActive,
}

/// Evento de risco
#[derive(Debug, Clone)]
pub enum RiskEvent {
    /// Limite de perda diária atingido
    DailyLossLimitReached { loss: Decimal },
    /// Alerta de perda (80% do limite)
    DailyLossWarning { loss: Decimal, limit: Decimal },
    /// Exposição alta
    HighExposure { current: Decimal, max: Decimal },
    /// Risco de liquidação
    LiquidationRisk { position_id: String, distance_pct: Decimal },
    /// Trade registrado
    TradeRecorded { trade_id: String, pnl: Decimal },
}

/// Configuração de limites de risco
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Perda máxima diária em valor absoluto
    pub max_daily_loss: Decimal,
    /// Perda máxima diária em percentual do capital
    pub max_daily_loss_pct: Decimal,
    /// Exposição máxima total
    pub max_total_exposure: Decimal,
    /// Número máximo de posições simultâneas
    pub max_positions: usize,
    /// Alavancagem máxima permitida
    pub max_leverage: u32,
    /// Tamanho máximo de uma única posição (% do capital)
    pub max_position_size_pct: Decimal,
    /// Concentração máxima em um único ativo (% da exposição)
    pub max_concentration_pct: Decimal,
    /// Percentual de alerta (antes de atingir limite)
    pub warning_threshold_pct: Decimal,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_daily_loss: dec!(1000),
            max_daily_loss_pct: dec!(5),
            max_total_exposure: dec!(100000),
            max_positions: 10,
            max_leverage: 20,
            max_position_size_pct: dec!(20),
            max_concentration_pct: dec!(50),
            warning_threshold_pct: dec!(80),
        }
    }
}

/// Estado de risco atual
#[derive(Debug, Clone, Default)]
pub struct RiskState {
    /// P&L do dia
    pub daily_pnl: Decimal,
    /// Exposição total atual
    pub total_exposure: Decimal,
    /// Número de posições abertas
    pub open_positions: usize,
    /// Trading está habilitado
    pub trading_enabled: bool,
    /// Motivo de desabilitação (se aplicável)
    pub disabled_reason: Option<String>,
    /// Última atualização
    pub last_update: DateTime<Utc>,
}

/// Histórico de trades para cálculo de P&L diário
#[derive(Debug, Clone)]
struct TradeRecord {
    timestamp: DateTime<Utc>,
    pnl: Decimal,
}

/// Risk Manager
pub struct RiskManager {
    config: RiskConfig,
    state: Arc<RwLock<RiskState>>,
    trade_history: Arc<RwLock<VecDeque<TradeRecord>>>,
    capital: Arc<RwLock<Decimal>>,
    event_tx: tokio::sync::broadcast::Sender<RiskEvent>,
}

impl RiskManager {
    /// Cria um novo Risk Manager
    pub fn new(config: RiskConfig, initial_capital: Decimal) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(100);
        let mut state = RiskState::default();
        state.trading_enabled = true;
        state.last_update = Utc::now();

        Self {
            config,
            state: Arc::new(RwLock::new(state)),
            trade_history: Arc::new(RwLock::new(VecDeque::new())),
            capital: Arc::new(RwLock::new(initial_capital)),
            event_tx,
        }
    }

    /// Retorna um receiver para eventos
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<RiskEvent> {
        self.event_tx.subscribe()
    }

    /// Valida uma ordem contra os limites de risco
    pub async fn validate_order(
        &self,
        order: &OrderRequest,
        current_positions: &[Position],
    ) -> RiskCheckResult {
        let state = self.state.read().await;
        let capital = *self.capital.read().await;

        // 1. Verifica se trading está habilitado
        if !state.trading_enabled {
            return RiskCheckResult::Rejected(RiskRejectionReason::TradingDisabled {
                reason: state.disabled_reason.clone().unwrap_or_default(),
            });
        }

        // 2. Verifica limite de perda diária
        let daily_loss_limit = self.config.max_daily_loss.min(
            capital * self.config.max_daily_loss_pct / dec!(100)
        );

        if state.daily_pnl < Decimal::ZERO && state.daily_pnl.abs() >= daily_loss_limit {
            return RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached {
                current_loss: state.daily_pnl.abs(),
                limit: daily_loss_limit,
            });
        }

        // 3. Verifica número máximo de posições
        let current_count = current_positions.len();
        if current_count >= self.config.max_positions {
            // Verifica se não é uma ordem para fechar posição existente
            let is_closing = current_positions.iter().any(|p| {
                p.symbol == order.symbol &&
                ((p.side == PositionSide::Long && order.side == OrderSide::Sell) ||
                 (p.side == PositionSide::Short && order.side == OrderSide::Buy))
            });

            if !is_closing {
                return RiskCheckResult::Rejected(RiskRejectionReason::MaxPositionsReached {
                    current: current_count,
                    max: self.config.max_positions,
                });
            }
        }

        // 4. Verifica alavancagem
        let leverage = order.leverage.unwrap_or(1);
        if leverage > self.config.max_leverage {
            return RiskCheckResult::Rejected(RiskRejectionReason::LeverageTooHigh {
                requested: leverage,
                max: self.config.max_leverage,
            });
        }

        // 5. Verifica tamanho da posição
        let order_value = order.quantity * order.price.unwrap_or(Decimal::ZERO);
        let max_position_value = capital * self.config.max_position_size_pct / dec!(100);

        if order_value > max_position_value {
            // Calcula quantidade ajustada
            let adjusted_quantity = if order.price.unwrap_or(Decimal::ZERO) > Decimal::ZERO {
                max_position_value / order.price.unwrap()
            } else {
                order.quantity
            };

            return RiskCheckResult::ApprovedWithAdjustment {
                original_quantity: order.quantity,
                adjusted_quantity,
                reason: format!(
                    "Tamanho reduzido de {} para {} para respeitar limite de {}% do capital",
                    order.quantity, adjusted_quantity, self.config.max_position_size_pct
                ),
            };
        }

        // 6. Verifica exposição total
        let new_exposure = state.total_exposure + order_value * Decimal::from(leverage);
        if new_exposure > self.config.max_total_exposure {
            return RiskCheckResult::Rejected(RiskRejectionReason::MaxExposureExceeded {
                current: state.total_exposure,
                max: self.config.max_total_exposure,
            });
        }

        // 7. Verifica concentração (apenas se já houver outras posições)
        if !current_positions.is_empty() {
            let symbol_exposure: Decimal = current_positions.iter()
                .filter(|p| p.symbol == order.symbol)
                .map(|p| p.quantity * p.current_price * Decimal::from(p.leverage))
                .sum();

            let new_symbol_exposure = symbol_exposure + order_value * Decimal::from(leverage);
            let total_with_new = state.total_exposure + order_value * Decimal::from(leverage);

            if total_with_new > Decimal::ZERO {
                let concentration = (new_symbol_exposure / total_with_new) * dec!(100);
                if concentration > self.config.max_concentration_pct {
                    return RiskCheckResult::Rejected(RiskRejectionReason::ConcentrationTooHigh {
                        symbol: order.symbol.clone(),
                        percentage: concentration,
                    });
                }
            }
        }

        RiskCheckResult::Approved
    }

    /// Registra um trade completado
    pub async fn record_trade(&self, trade: &Trade) {
        let mut history = self.trade_history.write().await;
        history.push_back(TradeRecord {
            timestamp: trade.exited_at,
            pnl: trade.net_pnl,
        });

        // Remove trades antigos (mais de 24h)
        let cutoff = Utc::now() - Duration::hours(24);
        while let Some(front) = history.front() {
            if front.timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }

        // Atualiza P&L diário
        let daily_pnl: Decimal = history.iter().map(|t| t.pnl).sum();

        let mut state = self.state.write().await;
        state.daily_pnl = daily_pnl;
        state.last_update = Utc::now();

        // Emite evento
        let _ = self.event_tx.send(RiskEvent::TradeRecorded {
            trade_id: trade.id.to_string(),
            pnl: trade.net_pnl,
        });

        // Verifica limites
        let capital = *self.capital.read().await;
        let daily_loss_limit = self.config.max_daily_loss.min(
            capital * self.config.max_daily_loss_pct / dec!(100)
        );

        if daily_pnl < Decimal::ZERO {
            let loss = daily_pnl.abs();

            // Verifica alerta (80% do limite)
            let warning_level = daily_loss_limit * self.config.warning_threshold_pct / dec!(100);
            if loss >= warning_level && loss < daily_loss_limit {
                let _ = self.event_tx.send(RiskEvent::DailyLossWarning {
                    loss,
                    limit: daily_loss_limit,
                });
            }

            // Verifica limite atingido
            if loss >= daily_loss_limit {
                state.trading_enabled = false;
                state.disabled_reason = Some("Limite de perda diária atingido".to_string());

                let _ = self.event_tx.send(RiskEvent::DailyLossLimitReached { loss });
                warn!("Trading desabilitado: limite de perda diária atingido ({} >= {})", loss, daily_loss_limit);
            }
        }
    }

    /// Atualiza o estado de exposição
    pub async fn update_exposure(&self, positions: &[Position]) {
        let mut state = self.state.write().await;

        state.total_exposure = positions.iter()
            .map(|p| p.quantity * p.current_price * Decimal::from(p.leverage))
            .sum();

        state.open_positions = positions.len();
        state.last_update = Utc::now();

        // Verifica alerta de exposição alta
        if state.total_exposure > self.config.max_total_exposure * self.config.warning_threshold_pct / dec!(100) {
            let _ = self.event_tx.send(RiskEvent::HighExposure {
                current: state.total_exposure,
                max: self.config.max_total_exposure,
            });
        }
    }

    /// Atualiza o capital
    pub async fn update_capital(&self, new_capital: Decimal) {
        let mut capital = self.capital.write().await;
        *capital = new_capital;
    }

    /// Habilita trading
    pub async fn enable_trading(&self) {
        let mut state = self.state.write().await;
        state.trading_enabled = true;
        state.disabled_reason = None;
        info!("Trading habilitado manualmente");
    }

    /// Desabilita trading
    pub async fn disable_trading(&self, reason: &str) {
        let mut state = self.state.write().await;
        state.trading_enabled = false;
        state.disabled_reason = Some(reason.to_string());
        warn!("Trading desabilitado: {}", reason);
    }

    /// Retorna o estado atual de risco
    pub async fn get_state(&self) -> RiskState {
        self.state.read().await.clone()
    }

    /// Retorna a configuração
    pub fn config(&self) -> &RiskConfig {
        &self.config
    }

    /// Reset diário (chamado à meia-noite)
    pub async fn daily_reset(&self) {
        let mut history = self.trade_history.write().await;
        history.clear();

        let mut state = self.state.write().await;
        state.daily_pnl = Decimal::ZERO;
        state.trading_enabled = true;
        state.disabled_reason = None;
        state.last_update = Utc::now();

        info!("Risk manager resetado para novo dia");
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::new(RiskConfig::default(), dec!(10000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robotrade_core::entities::{ExchangeId, OrderId, OrderType, PositionId, TimeInForce, TradeCloseReason};

    fn create_test_order() -> OrderRequest {
        OrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: dec!(0.01), // 0.01 * 50000 = 500 (5% of 10000 capital)
            price: Some(dec!(50000)),
            stop_price: None,
            stop_loss: None,
            take_profit: None,
            time_in_force: TimeInForce::GTC,
            leverage: Some(10),
            reduce_only: false,
        }
    }

    fn create_test_position() -> Position {
        Position::from_entry_order(
            ExchangeId::BinanceFutures,
            "BTCUSDT".to_string(),
            PositionSide::Long,
            dec!(0.1),
            dec!(50000),
            10, // leverage
            OrderId::new(),
        )
    }

    #[tokio::test]
    async fn test_validate_order_approved() {
        // Use higher capital for test to avoid position size limits
        let manager = RiskManager::new(RiskConfig::default(), dec!(100000));
        let order = create_test_order();

        let result = manager.validate_order(&order, &[]).await;
        assert!(matches!(result, RiskCheckResult::Approved), "Expected Approved, got {:?}", result);
    }

    #[tokio::test]
    async fn test_validate_order_leverage_too_high() {
        let manager = RiskManager::new(
            RiskConfig { max_leverage: 5, ..Default::default() },
            dec!(10000)
        );

        let mut order = create_test_order();
        order.leverage = Some(20);

        let result = manager.validate_order(&order, &[]).await;
        assert!(matches!(result, RiskCheckResult::Rejected(RiskRejectionReason::LeverageTooHigh { .. })));
    }

    #[tokio::test]
    async fn test_validate_order_max_positions() {
        let manager = RiskManager::new(
            RiskConfig { max_positions: 1, ..Default::default() },
            dec!(10000)
        );

        let order = create_test_order();
        let positions = vec![create_test_position()];

        let result = manager.validate_order(&order, &positions).await;
        assert!(matches!(result, RiskCheckResult::Rejected(RiskRejectionReason::MaxPositionsReached { .. })));
    }

    #[tokio::test]
    async fn test_daily_loss_limit() {
        let manager = RiskManager::new(
            RiskConfig { max_daily_loss: dec!(100), ..Default::default() },
            dec!(10000)
        );

        // Registra trade com perda usando o método do core
        let trade = Trade::from_closed_position(
            ExchangeId::BinanceFutures,
            "BTCUSDT".to_string(),
            PositionSide::Long,
            PositionId::new(),
            None, // signal_id
            OrderId::new(),
            OrderId::new(),
            dec!(0.1), // quantity
            dec!(50000), // entry_price
            dec!(49000), // exit_price
            dec!(5), // fees
            10, // leverage
            TradeCloseReason::StopLoss,
            Utc::now(),
        );

        manager.record_trade(&trade).await;

        let state = manager.get_state().await;
        assert!(!state.trading_enabled);
    }

    #[tokio::test]
    async fn test_enable_disable_trading() {
        let manager = RiskManager::default();

        manager.disable_trading("Test reason").await;
        let state = manager.get_state().await;
        assert!(!state.trading_enabled);
        assert_eq!(state.disabled_reason, Some("Test reason".to_string()));

        manager.enable_trading().await;
        let state = manager.get_state().await;
        assert!(state.trading_enabled);
        assert!(state.disabled_reason.is_none());
    }
}
