# Especificação de Backtest

Este documento descreve o motor de backtesting do RoboTrade.

## Visão Geral

O motor de backtest permite simular estratégias de trading usando dados históricos para avaliar performance antes de operar com capital real.

## Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                     BACKTEST ENGINE                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      INPUT                                │  │
│  │                                                           │  │
│  │  Strategy + Config + Historical Data + Initial Capital    │  │
│  │                                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    REPLAY ENGINE                          │  │
│  │                                                           │  │
│  │  For each candle:                                         │  │
│  │    1. Update market state                                 │  │
│  │    2. Check stop loss / take profit                       │  │
│  │    3. Evaluate strategy                                   │  │
│  │    4. Execute simulated orders                            │  │
│  │    5. Update portfolio                                    │  │
│  │    6. Record equity                                       │  │
│  │                                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   METRICS CALCULATOR                      │  │
│  │                                                           │  │
│  │  - Total Return                                           │  │
│  │  - Win Rate                                               │  │
│  │  - Profit Factor                                          │  │
│  │  - Sharpe Ratio                                           │  │
│  │  - Sortino Ratio                                          │  │
│  │  - Max Drawdown                                           │  │
│  │  - Calmar Ratio                                           │  │
│  │                                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      OUTPUT                               │  │
│  │                                                           │  │
│  │  BacktestResult {                                         │  │
│  │    metrics, equity_curve, drawdown_curve, trades          │  │
│  │  }                                                        │  │
│  │                                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Configuração do Backtest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// ID da estratégia
    pub strategy_id: String,

    /// Configuração da estratégia
    pub strategy_config: serde_json::Value,

    /// Símbolo a testar
    pub symbol: String,

    /// Timeframe
    pub timeframe: TimeFrame,

    /// Data de início
    pub start_date: DateTime<Utc>,

    /// Data de fim
    pub end_date: DateTime<Utc>,

    /// Capital inicial
    pub initial_capital: Decimal,

    /// Taxa de comissão por trade (0.0004 = 0.04%)
    pub commission_rate: Decimal,

    /// Slippage simulado (0.001 = 0.1%)
    pub slippage_rate: Decimal,

    /// Usar alavancagem
    pub use_leverage: bool,

    /// Alavancagem máxima
    pub max_leverage: u32,

    /// Margem isolada ou cross
    pub margin_type: MarginType,

    /// Permitir shorts
    pub allow_shorts: bool,

    /// Modo de execução de ordens
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Executa no close do candle
    ClosePrice,

    /// Executa no open do próximo candle
    NextOpen,

    /// Simula execução intra-candle (mais realista)
    IntraCandle,
}
```

## Motor de Backtest

```rust
pub struct BacktestEngine {
    candle_repo: Arc<dyn CandleRepository>,
    fear_greed_repo: Arc<dyn FearGreedRepository>,
    strategy_registry: Arc<StrategyRegistry>,
    indicator_calculator: Arc<IndicatorCalculator>,
}

impl BacktestEngine {
    pub async fn run(&self, config: BacktestConfig) -> AnalyticsResult<BacktestResult> {
        let start = Instant::now();

        // 1. Carregar estratégia
        let strategy = self.strategy_registry.get(&config.strategy_id)
            .ok_or(AnalyticsError::StrategyNotFound {
                strategy_id: config.strategy_id.clone(),
            })?;

        // 2. Carregar dados históricos
        let candles = self.candle_repo
            .find_candles_in_period(
                &config.symbol,
                config.timeframe,
                config.start_date,
                config.end_date,
            )
            .await?;

        if candles.len() < strategy.min_candles_required() {
            return Err(AnalyticsError::InsufficientData {
                required: strategy.min_candles_required(),
                available: candles.len(),
            });
        }

        tracing::info!(
            strategy = config.strategy_id,
            symbol = config.symbol,
            candles = candles.len(),
            start_date = %config.start_date,
            end_date = %config.end_date,
            "Starting backtest"
        );

        // 3. Inicializar estado
        let mut state = BacktestState::new(config.initial_capital);
        let mut trades: Vec<BacktestTrade> = Vec::new();
        let mut equity_curve: Vec<EquityPoint> = Vec::new();
        let mut drawdown_curve: Vec<DrawdownPoint> = Vec::new();

        // 4. Replay dos candles
        let min_candles = strategy.min_candles_required();

        for i in min_candles..candles.len() {
            let current_candle = &candles[i];
            let history = &candles[..=i];

            // Atualizar preço de mercado
            state.update_market_price(current_candle.close);

            // Verificar stop loss / take profit
            if let Some(position) = &state.position {
                if let Some(trade) = self.check_exit_conditions(&mut state, current_candle, &config) {
                    trades.push(trade);
                }
            }

            // Avaliar estratégia (se não tiver posição ou permite multiple positions)
            if state.position.is_none() {
                let context = self.build_context(history, &state, &strategy).await?;

                if let Some(signal) = strategy.evaluate(&context).await {
                    if let Some(trade) = self.execute_entry(&mut state, &signal, current_candle, &config) {
                        // Trade será finalizado quando fechar
                    }
                }
            } else {
                // Verificar sinal de saída
                let context = self.build_context(history, &state, &strategy).await?;

                if let Some(signal) = strategy.should_close_position(&context).await {
                    if let Some(trade) = self.execute_exit(&mut state, &signal, current_candle, &config) {
                        trades.push(trade);
                    }
                }
            }

            // Calcular equity e drawdown
            let equity = state.calculate_equity(current_candle.close);
            let drawdown = state.calculate_drawdown(equity);

            equity_curve.push(EquityPoint {
                timestamp: current_candle.close_time,
                equity,
                cash: state.cash,
                position_value: equity - state.cash,
            });

            drawdown_curve.push(DrawdownPoint {
                timestamp: current_candle.close_time,
                drawdown,
                drawdown_pct: drawdown / state.peak_equity * Decimal::from(100),
            });
        }

        // 5. Fechar posição aberta no final
        if let Some(_position) = &state.position {
            let last_candle = candles.last().unwrap();
            if let Some(trade) = self.force_close_position(&mut state, last_candle, &config) {
                trades.push(trade);
            }
        }

        // 6. Calcular métricas
        let final_equity = state.calculate_equity(candles.last().unwrap().close);
        let metrics = self.calculate_metrics(&trades, &equity_curve, config.initial_capital, final_equity);

        let execution_time_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            total_trades = trades.len(),
            total_return_pct = %metrics.total_return,
            win_rate = %metrics.win_rate,
            sharpe = %metrics.sharpe_ratio,
            max_drawdown = %metrics.max_drawdown,
            execution_time_ms = execution_time_ms,
            "Backtest completed"
        );

        Ok(BacktestResult {
            id: uuid::Uuid::new_v4().to_string(),
            config,
            metrics,
            trades,
            equity_curve,
            drawdown_curve,
            execution_time_ms,
            created_at: Utc::now(),
        })
    }

    async fn build_context(
        &self,
        candles: &[Candle],
        state: &BacktestState,
        strategy: &Arc<dyn Strategy>,
    ) -> AnalyticsResult<StrategyContext> {
        // Calcular indicadores
        let mut indicators = HashMap::new();
        for indicator_config in strategy.required_indicators() {
            let values = self.indicator_calculator.calculate(&indicator_config, candles)?;
            indicators.insert(indicator_config.name(), values);
        }

        // Buscar Fear & Greed para a data
        let current_date = candles.last().unwrap().close_time.date_naive();
        let fear_greed = self.fear_greed_repo.find_by_date(current_date).await.ok().flatten();

        Ok(StrategyContext {
            candles: candles.to_vec(),
            ticker: Some(Ticker {
                symbol: candles.last().unwrap().symbol.clone(),
                last_price: candles.last().unwrap().close,
                bid_price: candles.last().unwrap().close,
                ask_price: candles.last().unwrap().close,
                volume_24h: Decimal::ZERO,
                timestamp: candles.last().unwrap().close_time,
            }),
            current_position: state.position.clone().map(|p| p.into()),
            open_orders: Vec::new(),
            fear_greed,
            indicators,
            risk_config: RiskConfig::default(),
            available_balance: state.cash,
            timestamp: candles.last().unwrap().close_time,
        })
    }

    fn check_exit_conditions(
        &self,
        state: &mut BacktestState,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        let position = state.position.as_ref()?;

        // Check stop loss
        if let Some(sl) = position.stop_loss {
            let triggered = match position.side {
                PositionSide::Long => candle.low <= sl,
                PositionSide::Short => candle.high >= sl,
            };

            if triggered {
                return self.execute_stop_loss(state, sl, candle, config);
            }
        }

        // Check take profit
        if let Some(tp) = position.take_profit {
            let triggered = match position.side {
                PositionSide::Long => candle.high >= tp,
                PositionSide::Short => candle.low <= tp,
            };

            if triggered {
                return self.execute_take_profit(state, tp, candle, config);
            }
        }

        None
    }

    fn execute_entry(
        &self,
        state: &mut BacktestState,
        signal: &Signal,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        // Calcular preço de entrada com slippage
        let entry_price = match signal.side {
            OrderSide::Buy => candle.close * (Decimal::ONE + config.slippage_rate),
            OrderSide::Sell => candle.close * (Decimal::ONE - config.slippage_rate),
        };

        // Calcular quantidade
        let quantity = signal.quantity.unwrap_or_else(|| {
            let position_value = state.cash * Decimal::from_str("0.95").unwrap(); // 95% do cash
            position_value / entry_price
        });

        // Verificar se tem capital suficiente
        let cost = quantity * entry_price;
        let commission = cost * config.commission_rate;

        if cost + commission > state.cash {
            return None;
        }

        // Criar posição
        let position = SimulatedPosition {
            side: PositionSide::from(signal.side),
            entry_price,
            quantity,
            entry_time: candle.close_time,
            stop_loss: signal.stop_loss,
            take_profit: signal.take_profit,
            signal_id: signal.id.clone(),
        };

        state.cash -= cost + commission;
        state.position = Some(position);
        state.total_commission += commission;

        tracing::debug!(
            side = ?signal.side,
            entry_price = %entry_price,
            quantity = %quantity,
            commission = %commission,
            "Entry executed"
        );

        None // Trade será criado no exit
    }

    fn execute_exit(
        &self,
        state: &mut BacktestState,
        signal: &Signal,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        let position = state.position.take()?;

        // Calcular preço de saída com slippage
        let exit_price = match position.side {
            PositionSide::Long => candle.close * (Decimal::ONE - config.slippage_rate),
            PositionSide::Short => candle.close * (Decimal::ONE + config.slippage_rate),
        };

        self.close_position(state, position, exit_price, candle.close_time, "signal", config)
    }

    fn execute_stop_loss(
        &self,
        state: &mut BacktestState,
        stop_price: Decimal,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        let position = state.position.take()?;

        // Stop loss executa no preço do stop (pode haver gap)
        let exit_price = match position.side {
            PositionSide::Long => stop_price * (Decimal::ONE - config.slippage_rate),
            PositionSide::Short => stop_price * (Decimal::ONE + config.slippage_rate),
        };

        self.close_position(state, position, exit_price, candle.close_time, "stop_loss", config)
    }

    fn execute_take_profit(
        &self,
        state: &mut BacktestState,
        tp_price: Decimal,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        let position = state.position.take()?;

        let exit_price = match position.side {
            PositionSide::Long => tp_price * (Decimal::ONE - config.slippage_rate),
            PositionSide::Short => tp_price * (Decimal::ONE + config.slippage_rate),
        };

        self.close_position(state, position, exit_price, candle.close_time, "take_profit", config)
    }

    fn force_close_position(
        &self,
        state: &mut BacktestState,
        candle: &Candle,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        let position = state.position.take()?;

        let exit_price = match position.side {
            PositionSide::Long => candle.close * (Decimal::ONE - config.slippage_rate),
            PositionSide::Short => candle.close * (Decimal::ONE + config.slippage_rate),
        };

        self.close_position(state, position, exit_price, candle.close_time, "end_of_backtest", config)
    }

    fn close_position(
        &self,
        state: &mut BacktestState,
        position: SimulatedPosition,
        exit_price: Decimal,
        exit_time: DateTime<Utc>,
        exit_reason: &str,
        config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        // Calcular P&L
        let gross_pnl = match position.side {
            PositionSide::Long => (exit_price - position.entry_price) * position.quantity,
            PositionSide::Short => (position.entry_price - exit_price) * position.quantity,
        };

        let exit_value = position.quantity * exit_price;
        let commission = exit_value * config.commission_rate;
        let net_pnl = gross_pnl - commission;

        let return_pct = net_pnl / (position.entry_price * position.quantity) * Decimal::from(100);

        // Atualizar state
        state.cash += exit_value - commission;
        state.total_commission += commission;

        // Atualizar peak equity
        let current_equity = state.cash;
        if current_equity > state.peak_equity {
            state.peak_equity = current_equity;
        }

        let trade = BacktestTrade {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: config.symbol.clone(),
            side: position.side,
            entry_price: position.entry_price,
            exit_price,
            quantity: position.quantity,
            gross_pnl,
            commission: commission * Decimal::from(2), // entry + exit
            net_pnl,
            return_pct,
            entry_time: position.entry_time,
            exit_time,
            duration: exit_time - position.entry_time,
            exit_reason: exit_reason.to_string(),
            signal_id: Some(position.signal_id),
        };

        tracing::debug!(
            side = ?position.side,
            entry_price = %position.entry_price,
            exit_price = %exit_price,
            net_pnl = %net_pnl,
            return_pct = %return_pct,
            exit_reason = exit_reason,
            "Position closed"
        );

        Some(trade)
    }

    fn calculate_metrics(
        &self,
        trades: &[BacktestTrade],
        equity_curve: &[EquityPoint],
        initial_capital: Decimal,
        final_capital: Decimal,
    ) -> BacktestMetrics {
        // Total return
        let total_return = (final_capital - initial_capital) / initial_capital * Decimal::from(100);

        // Win rate
        let winners = trades.iter().filter(|t| t.net_pnl > Decimal::ZERO).count();
        let losers = trades.iter().filter(|t| t.net_pnl < Decimal::ZERO).count();
        let win_rate = if !trades.is_empty() {
            Decimal::from(winners) / Decimal::from(trades.len()) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Average win/loss
        let avg_win = if winners > 0 {
            trades.iter()
                .filter(|t| t.net_pnl > Decimal::ZERO)
                .map(|t| t.net_pnl)
                .sum::<Decimal>() / Decimal::from(winners)
        } else {
            Decimal::ZERO
        };

        let avg_loss = if losers > 0 {
            trades.iter()
                .filter(|t| t.net_pnl < Decimal::ZERO)
                .map(|t| t.net_pnl.abs())
                .sum::<Decimal>() / Decimal::from(losers)
        } else {
            Decimal::ZERO
        };

        // Profit factor
        let total_wins: Decimal = trades.iter()
            .filter(|t| t.net_pnl > Decimal::ZERO)
            .map(|t| t.net_pnl)
            .sum();
        let total_losses: Decimal = trades.iter()
            .filter(|t| t.net_pnl < Decimal::ZERO)
            .map(|t| t.net_pnl.abs())
            .sum();
        let profit_factor = if total_losses > Decimal::ZERO {
            total_wins / total_losses
        } else {
            Decimal::MAX
        };

        // Max drawdown
        let mut peak = initial_capital;
        let mut max_drawdown = Decimal::ZERO;
        let mut max_drawdown_duration = Duration::zero();
        let mut drawdown_start: Option<DateTime<Utc>> = None;

        for point in equity_curve {
            if point.equity > peak {
                peak = point.equity;
                drawdown_start = None;
            } else {
                let dd = (peak - point.equity) / peak * Decimal::from(100);
                if dd > max_drawdown {
                    max_drawdown = dd;
                }

                if drawdown_start.is_none() {
                    drawdown_start = Some(point.timestamp);
                } else {
                    let duration = point.timestamp - drawdown_start.unwrap();
                    if duration > max_drawdown_duration {
                        max_drawdown_duration = duration;
                    }
                }
            }
        }

        // Sharpe ratio
        let returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| {
                let r = (w[1].equity - w[0].equity) / w[0].equity;
                r.to_f64().unwrap_or(0.0)
            })
            .collect();

        let sharpe_ratio = if !returns.is_empty() {
            let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns.iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>() / returns.len() as f64;
            let std = variance.sqrt();

            if std > 0.0 {
                // Anualizar (assumindo daily)
                let annual_return = mean * 252.0;
                let annual_std = std * (252.0_f64).sqrt();
                Decimal::from_f64(annual_return / annual_std).unwrap_or(Decimal::ZERO)
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        // Sortino ratio
        let negative_returns: Vec<f64> = returns.iter()
            .filter(|r| **r < 0.0)
            .cloned()
            .collect();

        let sortino_ratio = if !negative_returns.is_empty() {
            let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let downside_variance: f64 = negative_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / negative_returns.len() as f64;
            let downside_std = downside_variance.sqrt();

            if downside_std > 0.0 {
                let annual_return = mean * 252.0;
                let annual_downside = downside_std * (252.0_f64).sqrt();
                Decimal::from_f64(annual_return / annual_downside).unwrap_or(Decimal::ZERO)
            } else {
                Decimal::MAX
            }
        } else {
            Decimal::MAX
        };

        // Calmar ratio
        let calmar_ratio = if max_drawdown > Decimal::ZERO {
            total_return / max_drawdown
        } else {
            Decimal::MAX
        };

        // Expectancy
        let expectancy = if !trades.is_empty() {
            let win_rate_dec = Decimal::from(winners) / Decimal::from(trades.len());
            win_rate_dec * avg_win - (Decimal::ONE - win_rate_dec) * avg_loss
        } else {
            Decimal::ZERO
        };

        BacktestMetrics {
            total_return,
            total_return_annualized: Decimal::ZERO, // TODO: calcular
            total_trades: trades.len() as u32,
            winning_trades: winners as u32,
            losing_trades: losers as u32,
            win_rate,
            avg_win,
            avg_loss,
            largest_win: trades.iter().map(|t| t.net_pnl).max().unwrap_or(Decimal::ZERO),
            largest_loss: trades.iter().map(|t| t.net_pnl).min().unwrap_or(Decimal::ZERO).abs(),
            profit_factor,
            expectancy,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_drawdown,
            max_drawdown_duration,
            avg_trade_duration: if !trades.is_empty() {
                Duration::seconds(
                    trades.iter()
                        .map(|t| t.duration.num_seconds())
                        .sum::<i64>() / trades.len() as i64
                )
            } else {
                Duration::zero()
            },
            total_commission: trades.iter().map(|t| t.commission).sum(),
        }
    }
}
```

## Estruturas de Resultado

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: String,
    pub config: BacktestConfig,
    pub metrics: BacktestMetrics,
    pub trades: Vec<BacktestTrade>,
    pub equity_curve: Vec<EquityPoint>,
    pub drawdown_curve: Vec<DrawdownPoint>,
    pub execution_time_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: Decimal,
    pub total_return_annualized: Decimal,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_duration: Duration,
    pub avg_trade_duration: Duration,
    pub total_commission: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub id: String,
    pub symbol: String,
    pub side: PositionSide,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub quantity: Decimal,
    pub gross_pnl: Decimal,
    pub commission: Decimal,
    pub net_pnl: Decimal,
    pub return_pct: Decimal,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub duration: Duration,
    pub exit_reason: String,
    pub signal_id: Option<SignalId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: Decimal,
    pub cash: Decimal,
    pub position_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownPoint {
    pub timestamp: DateTime<Utc>,
    pub drawdown: Decimal,
    pub drawdown_pct: Decimal,
}
```

## Estado do Backtest

```rust
struct BacktestState {
    cash: Decimal,
    position: Option<SimulatedPosition>,
    peak_equity: Decimal,
    total_commission: Decimal,
}

struct SimulatedPosition {
    side: PositionSide,
    entry_price: Decimal,
    quantity: Decimal,
    entry_time: DateTime<Utc>,
    stop_loss: Option<Decimal>,
    take_profit: Option<Decimal>,
    signal_id: SignalId,
}

impl BacktestState {
    fn new(initial_capital: Decimal) -> Self {
        Self {
            cash: initial_capital,
            position: None,
            peak_equity: initial_capital,
            total_commission: Decimal::ZERO,
        }
    }

    fn calculate_equity(&self, current_price: Decimal) -> Decimal {
        match &self.position {
            Some(pos) => {
                let position_value = pos.quantity * current_price;
                let unrealized_pnl = match pos.side {
                    PositionSide::Long => (current_price - pos.entry_price) * pos.quantity,
                    PositionSide::Short => (pos.entry_price - current_price) * pos.quantity,
                };
                self.cash + position_value + unrealized_pnl - position_value
            }
            None => self.cash,
        }
    }

    fn calculate_drawdown(&self, equity: Decimal) -> Decimal {
        if self.peak_equity > Decimal::ZERO {
            (self.peak_equity - equity).max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        }
    }

    fn update_market_price(&mut self, price: Decimal) {
        // Atualizar peak se equity aumentou
        let equity = self.calculate_equity(price);
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }
    }
}
```

## Otimização de Parâmetros

```rust
pub struct ParameterOptimizer {
    backtest_engine: Arc<BacktestEngine>,
}

#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub base_config: BacktestConfig,
    pub parameters: Vec<ParameterRange>,
    pub optimization_target: OptimizationTarget,
    pub max_iterations: usize,
}

#[derive(Debug, Clone)]
pub struct ParameterRange {
    pub name: String,
    pub min: Decimal,
    pub max: Decimal,
    pub step: Decimal,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizationTarget {
    TotalReturn,
    SharpeRatio,
    SortinoRatio,
    CalmarRatio,
    ProfitFactor,
}

impl ParameterOptimizer {
    /// Grid search optimization
    pub async fn optimize_grid(&self, config: OptimizationConfig) -> Vec<OptimizationResult> {
        let mut results = Vec::new();

        // Gerar todas as combinações de parâmetros
        let combinations = self.generate_combinations(&config.parameters);

        for (i, params) in combinations.iter().enumerate() {
            // Criar config com parâmetros atuais
            let mut test_config = config.base_config.clone();

            // Aplicar parâmetros à strategy_config
            let mut strategy_config = test_config.strategy_config.clone();
            for (name, value) in params {
                strategy_config[name] = serde_json::json!(value.to_string());
            }
            test_config.strategy_config = strategy_config;

            // Executar backtest
            match self.backtest_engine.run(test_config.clone()).await {
                Ok(result) => {
                    let score = self.calculate_score(&result.metrics, config.optimization_target);

                    results.push(OptimizationResult {
                        parameters: params.clone(),
                        metrics: result.metrics,
                        score,
                    });

                    tracing::info!(
                        iteration = i + 1,
                        total = combinations.len(),
                        score = %score,
                        "Optimization iteration complete"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        iteration = i + 1,
                        error = %e,
                        "Backtest failed for parameter combination"
                    );
                }
            }
        }

        // Ordenar por score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results
    }

    fn generate_combinations(&self, ranges: &[ParameterRange]) -> Vec<HashMap<String, Decimal>> {
        let mut combinations = vec![HashMap::new()];

        for range in ranges {
            let mut new_combinations = Vec::new();

            let mut value = range.min;
            while value <= range.max {
                for combo in &combinations {
                    let mut new_combo = combo.clone();
                    new_combo.insert(range.name.clone(), value);
                    new_combinations.push(new_combo);
                }
                value += range.step;
            }

            combinations = new_combinations;
        }

        combinations
    }

    fn calculate_score(&self, metrics: &BacktestMetrics, target: OptimizationTarget) -> Decimal {
        match target {
            OptimizationTarget::TotalReturn => metrics.total_return,
            OptimizationTarget::SharpeRatio => metrics.sharpe_ratio,
            OptimizationTarget::SortinoRatio => metrics.sortino_ratio,
            OptimizationTarget::CalmarRatio => metrics.calmar_ratio,
            OptimizationTarget::ProfitFactor => metrics.profit_factor,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub parameters: HashMap<String, Decimal>,
    pub metrics: BacktestMetrics,
    pub score: Decimal,
}
```

---

**Próximo**: [ADRs - Architecture Decision Records](../adr/README.md)
