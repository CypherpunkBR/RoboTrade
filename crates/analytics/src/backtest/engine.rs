//! Motor de backtest
//!
//! Responsável por simular estratégias em dados históricos

use chrono::{DateTime, Utc};
use robotrade_core::dto::{BacktestResultDto, BacktestTradeDto, EquityPoint};
use robotrade_core::entities::{Candle, Signal, PositionSide, TimeFrame};
use robotrade_core::error::RoboTradeError;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use tracing::{debug, info};

use crate::strategies::Strategy;

/// Configuração do backtest
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Capital inicial em USDT
    pub initial_capital: Decimal,
    /// Tamanho da posição (% do capital)
    pub position_size_pct: Decimal,
    /// Taxa de trading (maker/taker)
    pub trading_fee_pct: Decimal,
    /// Slippage esperado
    pub slippage_pct: Decimal,
    /// Stop loss padrão (%)
    pub default_stop_loss_pct: Option<Decimal>,
    /// Take profit padrão (%)
    pub default_take_profit_pct: Option<Decimal>,
    /// Alavancagem
    pub leverage: u32,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: Decimal::from(10000),
            position_size_pct: Decimal::from(10), // 10% do capital por trade
            trading_fee_pct: Decimal::new(4, 4),  // 0.04%
            slippage_pct: Decimal::new(1, 3),     // 0.1%
            default_stop_loss_pct: Some(Decimal::from(2)), // 2%
            default_take_profit_pct: Some(Decimal::from(4)), // 4%
            leverage: 1,
        }
    }
}

/// Motivo de fechamento da posição simulada
#[derive(Debug, Clone, Copy)]
enum CloseReason {
    StopLoss,
    TakeProfit,
    Signal,
    EndOfBacktest,
}

/// Posição simulada durante backtest
#[derive(Debug, Clone)]
struct SimulatedPosition {
    side: PositionSide,
    entry_price: Decimal,
    quantity: Decimal,
    stop_loss: Option<Decimal>,
    take_profit: Option<Decimal>,
    opened_at: DateTime<Utc>,
    entry_fee: Decimal,
}

/// Trade simulado
#[derive(Debug, Clone)]
struct SimulatedTrade {
    side: PositionSide,
    entry_price: Decimal,
    exit_price: Decimal,
    pnl_pct: Decimal,
    net_pnl: Decimal,
    opened_at: DateTime<Utc>,
    closed_at: DateTime<Utc>,
}

/// Estado do backtest
struct BacktestState {
    capital: Decimal,
    position: Option<SimulatedPosition>,
    trades: Vec<SimulatedTrade>,
    equity_curve: Vec<(DateTime<Utc>, Decimal)>,
    max_equity: Decimal,
    max_drawdown: Decimal,
}

impl BacktestState {
    fn new(initial_capital: Decimal) -> Self {
        Self {
            capital: initial_capital,
            position: None,
            trades: Vec::new(),
            equity_curve: Vec::new(),
            max_equity: initial_capital,
            max_drawdown: Decimal::ZERO,
        }
    }

    fn update_drawdown(&mut self, current_equity: Decimal) {
        if current_equity > self.max_equity {
            self.max_equity = current_equity;
        }
        let drawdown = if self.max_equity > Decimal::ZERO {
            (self.max_equity - current_equity) / self.max_equity * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }
    }
}

/// Motor de backtest
pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    /// Cria um novo motor de backtest
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Executa backtest de uma estratégia
    pub async fn run<S: Strategy>(
        &self,
        strategy: &S,
        candles: &[Candle],
        symbol: &str,
    ) -> Result<BacktestResultDto, RoboTradeError> {
        info!(
            strategy = %strategy.name(),
            symbol = %symbol,
            candles = %candles.len(),
            "Iniciando backtest..."
        );

        if candles.is_empty() {
            return Err(RoboTradeError::Analytics(
                robotrade_core::error::AnalyticsError::StrategyError {
                    strategy: strategy.name().to_string(),
                    message: "Sem candles para backtest".to_string(),
                },
            ));
        }

        let mut state = BacktestState::new(self.config.initial_capital);

        // Itera pelos candles
        for (i, candle) in candles.iter().enumerate() {
            // Verifica stop loss / take profit primeiro
            if let Some(ref pos) = state.position {
                if let Some(close_reason) = self.check_exit_conditions(pos, candle) {
                    self.close_position(&mut state, candle, close_reason);
                }
            }

            // Gera sinal da estratégia
            let historical = &candles[..=i];
            if let Some(signal) = strategy.generate_signal(historical).await? {
                self.process_signal(&mut state, &signal, candle);
            }

            // Atualiza equity curve
            let equity = self.calculate_equity(&state, candle);
            state.equity_curve.push((candle.close_time, equity));
            state.update_drawdown(equity);
        }

        // Fecha posição aberta ao final
        if state.position.is_some() {
            let last_candle = candles.last().unwrap();
            self.close_position(&mut state, last_candle, CloseReason::EndOfBacktest);
        }

        // Calcula métricas
        let result = self.calculate_metrics(&state, candles, strategy);

        info!(
            total_trades = %result.total_trades,
            win_rate = %result.win_rate_pct,
            total_return = %result.total_return_pct,
            max_drawdown = %result.max_drawdown_pct,
            "Backtest concluído"
        );

        Ok(result)
    }

    /// Verifica condições de saída (SL/TP)
    fn check_exit_conditions(&self, pos: &SimulatedPosition, candle: &Candle) -> Option<CloseReason> {
        let is_long = matches!(pos.side, PositionSide::Long);

        // Verifica stop loss
        if let Some(sl) = pos.stop_loss {
            if is_long && candle.low <= sl {
                return Some(CloseReason::StopLoss);
            } else if !is_long && candle.high >= sl {
                return Some(CloseReason::StopLoss);
            }
        }

        // Verifica take profit
        if let Some(tp) = pos.take_profit {
            if is_long && candle.high >= tp {
                return Some(CloseReason::TakeProfit);
            } else if !is_long && candle.low <= tp {
                return Some(CloseReason::TakeProfit);
            }
        }

        None
    }

    /// Processa sinal da estratégia
    fn process_signal(
        &self,
        state: &mut BacktestState,
        signal: &Signal,
        candle: &Candle,
    ) {
        // Determina o lado baseado na direção do sinal
        let signal_side = match signal.direction {
            robotrade_core::entities::TradeDirection::Long => PositionSide::Long,
            robotrade_core::entities::TradeDirection::Short => PositionSide::Short,
        };

        // Se já tem posição, verifica se deve fechar
        if let Some(ref pos) = state.position {
            let should_close = match (signal_side, pos.side) {
                (PositionSide::Long, PositionSide::Short) => true,
                (PositionSide::Short, PositionSide::Long) => true,
                _ => false,
            };

            if should_close {
                self.close_position(state, candle, CloseReason::Signal);
            } else {
                return;
            }
        }

        // Abre nova posição
        self.open_position(state, signal_side, candle);
    }

    /// Abre uma posição
    fn open_position(
        &self,
        state: &mut BacktestState,
        side: PositionSide,
        candle: &Candle,
    ) {
        let entry_price = candle.close;

        // Calcula slippage
        let slippage = entry_price * self.config.slippage_pct / Decimal::from(100);
        let adjusted_price = match side {
            PositionSide::Long => entry_price + slippage,
            PositionSide::Short => entry_price - slippage,
        };

        // Calcula tamanho da posição
        let position_value = state.capital * self.config.position_size_pct / Decimal::from(100);
        let leverage_value = position_value * Decimal::from(self.config.leverage);
        let quantity = leverage_value / adjusted_price;

        // Calcula taxa
        let fee = leverage_value * self.config.trading_fee_pct / Decimal::from(100);

        // Calcula SL/TP
        let (stop_loss, take_profit) = match side {
            PositionSide::Long => {
                let sl = self.config.default_stop_loss_pct.map(|pct| {
                    adjusted_price * (Decimal::ONE - pct / Decimal::from(100))
                });
                let tp = self.config.default_take_profit_pct.map(|pct| {
                    adjusted_price * (Decimal::ONE + pct / Decimal::from(100))
                });
                (sl, tp)
            }
            PositionSide::Short => {
                let sl = self.config.default_stop_loss_pct.map(|pct| {
                    adjusted_price * (Decimal::ONE + pct / Decimal::from(100))
                });
                let tp = self.config.default_take_profit_pct.map(|pct| {
                    adjusted_price * (Decimal::ONE - pct / Decimal::from(100))
                });
                (sl, tp)
            }
        };

        debug!(
            side = ?side,
            price = %adjusted_price,
            quantity = %quantity,
            "Abrindo posição simulada"
        );

        state.position = Some(SimulatedPosition {
            side,
            entry_price: adjusted_price,
            quantity,
            stop_loss,
            take_profit,
            opened_at: candle.close_time,
            entry_fee: fee,
        });

        state.capital -= position_value;
    }

    /// Fecha uma posição
    fn close_position(
        &self,
        state: &mut BacktestState,
        candle: &Candle,
        reason: CloseReason,
    ) {
        let pos = match state.position.take() {
            Some(p) => p,
            None => return,
        };

        let exit_price = match reason {
            CloseReason::StopLoss => pos.stop_loss.unwrap_or(candle.close),
            CloseReason::TakeProfit => pos.take_profit.unwrap_or(candle.close),
            _ => candle.close,
        };

        let slippage = exit_price * self.config.slippage_pct / Decimal::from(100);
        let adjusted_exit = match pos.side {
            PositionSide::Long => exit_price - slippage,
            PositionSide::Short => exit_price + slippage,
        };

        let position_value = pos.quantity * pos.entry_price;
        let exit_value = pos.quantity * adjusted_exit;
        let exit_fee = exit_value * self.config.trading_fee_pct / Decimal::from(100);

        let gross_pnl = match pos.side {
            PositionSide::Long => exit_value - position_value,
            PositionSide::Short => position_value - exit_value,
        };

        let net_pnl = gross_pnl - pos.entry_fee - exit_fee;
        let pnl_pct = if position_value > Decimal::ZERO {
            net_pnl / (position_value / Decimal::from(self.config.leverage)) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        debug!(
            side = ?pos.side,
            entry = %pos.entry_price,
            exit = %adjusted_exit,
            pnl = %net_pnl,
            reason = ?reason,
            "Fechando posição simulada"
        );

        let margin = position_value / Decimal::from(self.config.leverage);
        state.capital += margin + net_pnl;

        state.trades.push(SimulatedTrade {
            side: pos.side,
            entry_price: pos.entry_price,
            exit_price: adjusted_exit,
            pnl_pct,
            net_pnl,
            opened_at: pos.opened_at,
            closed_at: candle.close_time,
        });
    }

    /// Calcula equity atual
    fn calculate_equity(&self, state: &BacktestState, candle: &Candle) -> Decimal {
        let mut equity = state.capital;

        if let Some(ref pos) = state.position {
            let position_value = pos.quantity * pos.entry_price;
            let current_value = pos.quantity * candle.close;

            let unrealized_pnl = match pos.side {
                PositionSide::Long => current_value - position_value,
                PositionSide::Short => position_value - current_value,
            };

            let margin = position_value / Decimal::from(self.config.leverage);
            equity += margin + unrealized_pnl;
        }

        equity
    }

    /// Calcula métricas finais
    fn calculate_metrics<S: Strategy>(
        &self,
        state: &BacktestState,
        candles: &[Candle],
        strategy: &S,
    ) -> BacktestResultDto {
        let total_trades = state.trades.len() as u32;
        let winning_trades = state.trades.iter().filter(|t| t.net_pnl > Decimal::ZERO).count() as u32;
        let _losing_trades = state.trades.iter().filter(|t| t.net_pnl < Decimal::ZERO).count() as u32;

        let gross_profit: Decimal = state.trades
            .iter()
            .filter(|t| t.net_pnl > Decimal::ZERO)
            .map(|t| t.net_pnl)
            .sum();

        let gross_loss: Decimal = state.trades
            .iter()
            .filter(|t| t.net_pnl < Decimal::ZERO)
            .map(|t| t.net_pnl.abs())
            .sum();

        let win_rate = if total_trades > 0 {
            Decimal::from(winning_trades) / Decimal::from(total_trades) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            Decimal::from(1000)
        } else {
            Decimal::ZERO
        };

        let total_return_pct = if self.config.initial_capital > Decimal::ZERO {
            (state.capital - self.config.initial_capital)
                / self.config.initial_capital
                * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Sharpe simplificado
        let returns: Vec<Decimal> = state.trades.iter().map(|t| t.pnl_pct).collect();
        let sharpe_ratio = if returns.len() > 1 {
            let mean = returns.iter().copied().sum::<Decimal>() / Decimal::from(returns.len());
            let variance = returns
                .iter()
                .map(|r| (*r - mean) * (*r - mean))
                .sum::<Decimal>()
                / Decimal::from(returns.len() - 1);

            let std_dev = sqrt_decimal(variance);

            if std_dev > Decimal::ZERO {
                mean / std_dev * Decimal::new(1732, 2)
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        let start_date = candles.first().map(|c| c.close_time);
        let end_date = candles.last().map(|c| c.close_time);

        let period = match (start_date, end_date) {
            (Some(s), Some(e)) => format!("{} - {}", s.format("%Y-%m-%d"), e.format("%Y-%m-%d")),
            _ => String::new(),
        };

        let timeframe = candles.first().map(|c| c.timeframe).unwrap_or(TimeFrame::H1);

        // Converte trades para DTO
        let trades_dto: Vec<BacktestTradeDto> = state.trades.iter().map(|t| BacktestTradeDto {
            entry_date: t.opened_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            exit_date: t.closed_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            side: t.side,
            entry_price: t.entry_price,
            exit_price: t.exit_price,
            pnl_pct: t.pnl_pct,
        }).collect();

        // Converte equity curve
        let equity_curve: Vec<EquityPoint> = state.equity_curve.iter().enumerate().map(|(i, (dt, eq))| {
            let drawdown = if i > 0 {
                let max_so_far = state.equity_curve[..=i].iter().map(|(_, e)| *e).max().unwrap_or(*eq);
                if max_so_far > Decimal::ZERO {
                    (max_so_far - *eq) / max_so_far * Decimal::from(100)
                } else {
                    Decimal::ZERO
                }
            } else {
                Decimal::ZERO
            };

            EquityPoint {
                date: dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                equity: *eq,
                drawdown,
            }
        }).collect();

        BacktestResultDto {
            id: uuid::Uuid::new_v4().to_string(),
            strategy_id: strategy.name().to_string(),
            symbol: candles.first().map(|c| format!("{:?}", timeframe)).unwrap_or_default(),
            period,
            initial_capital: self.config.initial_capital,
            final_capital: state.capital,
            total_return_pct,
            total_trades,
            winning_trades,
            win_rate_pct: win_rate,
            profit_factor,
            sharpe_ratio,
            max_drawdown_pct: state.max_drawdown,
            trades: trades_dto,
            equity_curve,
        }
    }
}

/// Aproximação de raiz quadrada para Decimal usando Newton-Raphson
fn sqrt_decimal(n: Decimal) -> Decimal {
    if n <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    let mut guess = n / Decimal::from(2);
    let epsilon = Decimal::new(1, 10);

    for _ in 0..100 {
        let new_guess = (guess + n / guess) / Decimal::from(2);
        if (new_guess - guess).abs() < epsilon {
            return new_guess;
        }
        guess = new_guess;
    }

    guess
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::MockStrategy;
    use rust_decimal_macros::dec;

    fn create_test_candles(count: usize, start_price: Decimal) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let mut price = start_price;
        let base_time = Utc::now() - chrono::Duration::hours(count as i64);

        for i in 0..count {
            let change = if i % 3 == 0 {
                dec!(1.02)
            } else if i % 3 == 1 {
                dec!(0.99)
            } else {
                dec!(1.005)
            };

            let new_price = price * change;
            let high = price.max(new_price) * dec!(1.005);
            let low = price.min(new_price) * dec!(0.995);

            candles.push(Candle {
                symbol: "BTCUSDT".to_string(),
                timeframe: TimeFrame::H1,
                open_time: base_time + chrono::Duration::hours(i as i64),
                close_time: base_time + chrono::Duration::hours((i + 1) as i64),
                open: price,
                high,
                low,
                close: new_price,
                volume: dec!(1000),
                quote_volume: dec!(1000) * new_price,
                trades_count: 100,
            });

            price = new_price;
        }

        candles
    }

    #[tokio::test]
    async fn test_backtest_with_no_trades() {
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);
        let strategy = MockStrategy::new("NoSignal", vec![]);

        let candles = create_test_candles(100, dec!(50000));
        let result = engine.run(&strategy, &candles, "BTCUSDT").await.unwrap();

        assert_eq!(result.total_trades, 0);
        assert_eq!(result.final_capital, result.initial_capital);
    }

    #[tokio::test]
    async fn test_backtest_empty_candles() {
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);
        let strategy = MockStrategy::new("Test", vec![]);

        let result = engine.run(&strategy, &[], "BTCUSDT").await;
        assert!(result.is_err());
    }
}
