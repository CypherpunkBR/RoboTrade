# Gestão de Risco

Este documento descreve as regras de gestão de risco do RoboTrade.

## Princípios

1. **Preservação de Capital** - O objetivo principal é proteger o capital
2. **Risco Definido** - Nunca entrar em trade sem saber o risco máximo
3. **Consistência** - Regras aplicadas uniformemente
4. **Fail Safe** - Na dúvida, não operar

## Limites de Risco

### Configuração

```toml
[risk]
# Tamanho máximo de posição (em USD)
max_position_size = "10000.00"

# Percentual máximo de risco por trade
max_risk_per_trade = "0.02"  # 2%

# Perda máxima diária
max_daily_loss = "500.00"

# Número máximo de ordens por minuto
max_orders_per_minute = 10

# Número máximo de posições abertas
max_open_positions = 5

# Alavancagem máxima
max_leverage = 10

# Exigir stop loss
require_stop_loss = true

# Distância mínima do stop loss (%)
min_stop_loss_distance = "0.5"

# Circuit breaker
circuit_breaker_enabled = true
circuit_breaker_max_losses = 5
circuit_breaker_cooldown_minutes = 60
```

### Hierarquia de Limites

```
┌─────────────────────────────────────────────────────────────────┐
│                     LIMITES DE CONTA                             │
│                                                                  │
│  max_daily_loss = $500                                          │
│  max_open_positions = 5                                         │
│  max_orders_per_minute = 10                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    LIMITES POR POSIÇÃO                           │
│                                                                  │
│  max_position_size = $10,000                                    │
│  max_leverage = 10x                                             │
│  require_stop_loss = true                                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     LIMITES POR TRADE                            │
│                                                                  │
│  max_risk_per_trade = 2%                                        │
│  min_stop_loss_distance = 0.5%                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Position Sizing

### Fórmula de Kelly Criterion

```rust
pub fn kelly_criterion(win_rate: Decimal, avg_win: Decimal, avg_loss: Decimal) -> Decimal {
    // f* = (bp - q) / b
    // b = avg_win / avg_loss
    // p = win_rate
    // q = 1 - win_rate

    let b = avg_win / avg_loss;
    let p = win_rate;
    let q = Decimal::ONE - win_rate;

    let kelly = (b * p - q) / b;

    // Kelly fracionado (metade para segurança)
    let half_kelly = kelly / Decimal::from(2);

    // Limitar entre 0 e 25%
    half_kelly.max(Decimal::ZERO).min(Decimal::from_str("0.25").unwrap())
}
```

### Position Sizing Baseado em Risco

```rust
pub struct PositionSizer {
    config: RiskConfig,
}

impl PositionSizer {
    /// Calcula tamanho da posição baseado no risco
    pub fn calculate_position_size(
        &self,
        account_equity: Decimal,
        entry_price: Decimal,
        stop_loss_price: Decimal,
    ) -> Decimal {
        // Risco em USD
        let risk_amount = account_equity * self.config.max_risk_per_trade;

        // Distância do stop em USD por unidade
        let stop_distance = (entry_price - stop_loss_price).abs();

        if stop_distance == Decimal::ZERO {
            return Decimal::ZERO;
        }

        // Quantidade = Risco / Distância do Stop
        let quantity = risk_amount / stop_distance;

        // Aplicar limite máximo de posição
        let max_quantity = self.config.max_position_size / entry_price;

        quantity.min(max_quantity)
    }

    /// Calcula tamanho com base em ATR
    pub fn calculate_position_size_atr(
        &self,
        account_equity: Decimal,
        entry_price: Decimal,
        atr: Decimal,
        atr_multiplier: Decimal,
    ) -> Decimal {
        let stop_distance = atr * atr_multiplier;
        let stop_loss = entry_price - stop_distance; // Para long

        self.calculate_position_size(account_equity, entry_price, stop_loss)
    }

    /// Calcula tamanho com base em percentual fixo
    pub fn calculate_position_size_fixed_pct(
        &self,
        account_equity: Decimal,
        entry_price: Decimal,
        position_pct: Decimal,
    ) -> Decimal {
        let notional = account_equity * position_pct;
        let quantity = notional / entry_price;

        // Aplicar limite máximo
        let max_quantity = self.config.max_position_size / entry_price;
        quantity.min(max_quantity)
    }
}
```

## Stop Loss

### Tipos de Stop Loss

```rust
pub enum StopLossType {
    /// Stop fixo em preço
    Fixed { price: Decimal },

    /// Stop em percentual do entry
    Percentage { pct: Decimal },

    /// Stop baseado em ATR
    Atr { multiplier: Decimal, atr: Decimal },

    /// Stop em suporte/resistência
    Technical { level: Decimal },

    /// Trailing stop
    Trailing { distance: Decimal, trail_type: TrailType },
}

pub enum TrailType {
    /// Distância fixa em USD
    Fixed,
    /// Distância percentual
    Percentage,
    /// Baseado em ATR
    Atr,
}

impl StopLossType {
    pub fn calculate_price(&self, entry_price: Decimal, side: OrderSide) -> Decimal {
        match self {
            StopLossType::Fixed { price } => *price,

            StopLossType::Percentage { pct } => {
                let distance = entry_price * *pct / Decimal::from(100);
                match side {
                    OrderSide::Buy => entry_price - distance,
                    OrderSide::Sell => entry_price + distance,
                }
            }

            StopLossType::Atr { multiplier, atr } => {
                let distance = *atr * *multiplier;
                match side {
                    OrderSide::Buy => entry_price - distance,
                    OrderSide::Sell => entry_price + distance,
                }
            }

            StopLossType::Technical { level } => *level,

            StopLossType::Trailing { distance, trail_type } => {
                // Trailing começa igual ao fixo
                match trail_type {
                    TrailType::Fixed => match side {
                        OrderSide::Buy => entry_price - *distance,
                        OrderSide::Sell => entry_price + *distance,
                    },
                    TrailType::Percentage => {
                        let dist = entry_price * *distance / Decimal::from(100);
                        match side {
                            OrderSide::Buy => entry_price - dist,
                            OrderSide::Sell => entry_price + dist,
                        }
                    }
                    TrailType::Atr => {
                        // distance é o ATR multiplicado
                        match side {
                            OrderSide::Buy => entry_price - *distance,
                            OrderSide::Sell => entry_price + *distance,
                        }
                    }
                }
            }
        }
    }
}
```

### Trailing Stop Manager

```rust
pub struct TrailingStopManager {
    positions: RwLock<HashMap<PositionId, TrailingStopState>>,
}

pub struct TrailingStopState {
    pub position_id: PositionId,
    pub side: PositionSide,
    pub trail_type: TrailType,
    pub distance: Decimal,
    pub highest_price: Decimal,  // Para long
    pub lowest_price: Decimal,   // Para short
    pub current_stop: Decimal,
}

impl TrailingStopManager {
    /// Atualiza trailing stop com novo preço
    pub async fn update(&self, position_id: &PositionId, current_price: Decimal) -> Option<Decimal> {
        let mut positions = self.positions.write().await;
        let state = positions.get_mut(position_id)?;

        let new_stop = match state.side {
            PositionSide::Long => {
                // Atualiza highest
                if current_price > state.highest_price {
                    state.highest_price = current_price;

                    // Calcula novo stop
                    let new_stop = self.calculate_stop(current_price, state);
                    if new_stop > state.current_stop {
                        state.current_stop = new_stop;
                        return Some(new_stop);
                    }
                }
                None
            }
            PositionSide::Short => {
                // Atualiza lowest
                if current_price < state.lowest_price {
                    state.lowest_price = current_price;

                    // Calcula novo stop
                    let new_stop = self.calculate_stop(current_price, state);
                    if new_stop < state.current_stop {
                        state.current_stop = new_stop;
                        return Some(new_stop);
                    }
                }
                None
            }
        };

        new_stop
    }

    fn calculate_stop(&self, price: Decimal, state: &TrailingStopState) -> Decimal {
        let distance = match state.trail_type {
            TrailType::Fixed => state.distance,
            TrailType::Percentage => price * state.distance / Decimal::from(100),
            TrailType::Atr => state.distance,
        };

        match state.side {
            PositionSide::Long => price - distance,
            PositionSide::Short => price + distance,
        }
    }
}
```

## Take Profit

### Estratégias de Take Profit

```rust
pub enum TakeProfitStrategy {
    /// Take profit fixo
    Fixed { price: Decimal },

    /// Baseado em Risk:Reward
    RiskReward { ratio: Decimal, stop_loss: Decimal },

    /// Múltiplos targets
    Scaled {
        targets: Vec<ScaledTarget>,
    },

    /// Trailing take profit
    Trailing { activation_pct: Decimal, trail_pct: Decimal },
}

pub struct ScaledTarget {
    /// Percentual da posição a fechar
    pub position_pct: Decimal,
    /// Preço target
    pub price: Decimal,
}

impl TakeProfitStrategy {
    pub fn calculate_prices(
        &self,
        entry_price: Decimal,
        side: OrderSide,
    ) -> Vec<(Decimal, Decimal)> {
        // Retorna (preço, percentual da posição)
        match self {
            TakeProfitStrategy::Fixed { price } => {
                vec![(*price, Decimal::from(100))]
            }

            TakeProfitStrategy::RiskReward { ratio, stop_loss } => {
                let risk = (entry_price - *stop_loss).abs();
                let reward = risk * *ratio;
                let tp = match side {
                    OrderSide::Buy => entry_price + reward,
                    OrderSide::Sell => entry_price - reward,
                };
                vec![(tp, Decimal::from(100))]
            }

            TakeProfitStrategy::Scaled { targets } => {
                targets.iter()
                    .map(|t| (t.price, t.position_pct))
                    .collect()
            }

            TakeProfitStrategy::Trailing { .. } => {
                // Trailing não tem preço fixo
                vec![]
            }
        }
    }
}
```

## Circuit Breaker

### Implementação

```rust
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    consecutive_losses: AtomicU32,
    total_losses_today: AtomicU32,
    daily_pnl: AtomicDecimal,
    cooldown_until: RwLock<Option<DateTime<Utc>>>,
    last_reset: RwLock<DateTime<Utc>>,
}

pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub max_consecutive_losses: u32,
    pub max_daily_losses: u32,
    pub max_daily_loss_amount: Decimal,
    pub cooldown_minutes: u32,
}

impl CircuitBreaker {
    /// Registra resultado de trade
    pub async fn record_trade(&self, pnl: Decimal) {
        // Reset diário se necessário
        self.check_daily_reset().await;

        if pnl < Decimal::ZERO {
            let consecutive = self.consecutive_losses.fetch_add(1, Ordering::SeqCst) + 1;
            let total = self.total_losses_today.fetch_add(1, Ordering::SeqCst) + 1;

            // Atualiza P&L diário
            self.daily_pnl.fetch_add(pnl);

            // Verifica triggers
            if consecutive >= self.config.max_consecutive_losses {
                self.trigger(CircuitBreakerReason::ConsecutiveLosses {
                    count: consecutive,
                    max: self.config.max_consecutive_losses,
                }).await;
            }

            if total >= self.config.max_daily_losses {
                self.trigger(CircuitBreakerReason::DailyLossCount {
                    count: total,
                    max: self.config.max_daily_losses,
                }).await;
            }

            if self.daily_pnl.load() <= -self.config.max_daily_loss_amount {
                self.trigger(CircuitBreakerReason::DailyLossAmount {
                    amount: self.daily_pnl.load(),
                    max: self.config.max_daily_loss_amount,
                }).await;
            }
        } else {
            // Reseta contador de perdas consecutivas
            self.consecutive_losses.store(0, Ordering::SeqCst);
            self.daily_pnl.fetch_add(pnl);
        }
    }

    /// Verifica se trading está permitido
    pub async fn is_trading_allowed(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        let cooldown = self.cooldown_until.read().await;
        match *cooldown {
            Some(until) => Utc::now() >= until,
            None => true,
        }
    }

    /// Trigger do circuit breaker
    async fn trigger(&self, reason: CircuitBreakerReason) {
        let cooldown_until = Utc::now() + Duration::minutes(self.config.cooldown_minutes as i64);

        {
            let mut cooldown = self.cooldown_until.write().await;
            *cooldown = Some(cooldown_until);
        }

        tracing::warn!(
            reason = ?reason,
            cooldown_until = %cooldown_until,
            "Circuit breaker triggered"
        );

        // TODO: Emitir evento e notificar usuário
    }

    async fn check_daily_reset(&self) {
        let mut last_reset = self.last_reset.write().await;
        let now = Utc::now();

        if now.date_naive() != last_reset.date_naive() {
            self.consecutive_losses.store(0, Ordering::SeqCst);
            self.total_losses_today.store(0, Ordering::SeqCst);
            self.daily_pnl.store(Decimal::ZERO);
            *last_reset = now;

            tracing::info!("Circuit breaker daily counters reset");
        }
    }
}

#[derive(Debug, Clone)]
pub enum CircuitBreakerReason {
    ConsecutiveLosses { count: u32, max: u32 },
    DailyLossCount { count: u32, max: u32 },
    DailyLossAmount { amount: Decimal, max: Decimal },
    ManualTrigger,
}
```

## Drawdown Management

### Cálculo de Drawdown

```rust
pub struct DrawdownCalculator {
    peak_equity: Decimal,
    current_equity: Decimal,
    max_drawdown: Decimal,
    max_drawdown_date: Option<DateTime<Utc>>,
}

impl DrawdownCalculator {
    /// Atualiza equity e calcula drawdown
    pub fn update(&mut self, equity: Decimal) -> DrawdownInfo {
        self.current_equity = equity;

        // Novo pico?
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        // Calcular drawdown atual
        let current_drawdown = if self.peak_equity > Decimal::ZERO {
            (self.peak_equity - equity) / self.peak_equity * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Atualizar máximo drawdown
        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
            self.max_drawdown_date = Some(Utc::now());
        }

        DrawdownInfo {
            current_drawdown,
            max_drawdown: self.max_drawdown,
            peak_equity: self.peak_equity,
            current_equity: self.current_equity,
        }
    }

    /// Verifica se drawdown excede limite
    pub fn exceeds_limit(&self, limit_pct: Decimal) -> bool {
        let current = (self.peak_equity - self.current_equity) / self.peak_equity * Decimal::from(100);
        current > limit_pct
    }
}

pub struct DrawdownInfo {
    pub current_drawdown: Decimal,
    pub max_drawdown: Decimal,
    pub peak_equity: Decimal,
    pub current_equity: Decimal,
}
```

## Risk Monitor

### Monitoramento em Tempo Real

```rust
pub struct RiskMonitor {
    config: RiskConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    drawdown_calculator: RwLock<DrawdownCalculator>,
    position_repo: Arc<dyn PositionRepository>,
    metrics: Arc<MetricsRegistry>,
}

impl RiskMonitor {
    /// Executa verificação de risco
    pub async fn check(&self, portfolio: &Portfolio) -> RiskCheckResult {
        let mut warnings = Vec::new();
        let mut violations = Vec::new();

        // 1. Verificar drawdown
        let dd_info = self.drawdown_calculator.write().await.update(portfolio.total_equity);

        if dd_info.current_drawdown > self.config.max_drawdown_warning_pct {
            warnings.push(RiskWarning::HighDrawdown {
                current: dd_info.current_drawdown,
                threshold: self.config.max_drawdown_warning_pct,
            });
        }

        if dd_info.current_drawdown > self.config.max_drawdown_limit_pct {
            violations.push(RiskViolation::DrawdownLimitExceeded {
                current: dd_info.current_drawdown,
                limit: self.config.max_drawdown_limit_pct,
            });
        }

        // 2. Verificar exposição total
        let total_exposure = portfolio.calculate_total_exposure();
        if total_exposure > self.config.max_total_exposure {
            violations.push(RiskViolation::ExposureLimitExceeded {
                current: total_exposure,
                limit: self.config.max_total_exposure,
            });
        }

        // 3. Verificar concentração
        for position in &portfolio.positions {
            let position_pct = position.notional_value / portfolio.total_equity * Decimal::from(100);
            if position_pct > self.config.max_position_concentration_pct {
                warnings.push(RiskWarning::HighConcentration {
                    symbol: position.symbol.clone(),
                    pct: position_pct,
                    threshold: self.config.max_position_concentration_pct,
                });
            }
        }

        // 4. Verificar circuit breaker
        if !self.circuit_breaker.is_trading_allowed().await {
            violations.push(RiskViolation::CircuitBreakerActive);
        }

        // Atualizar métricas
        self.metrics.trading().current_drawdown.set_f64(dd_info.current_drawdown.to_f64().unwrap());
        self.metrics.trading().max_drawdown.set_f64(dd_info.max_drawdown.to_f64().unwrap());

        RiskCheckResult {
            is_healthy: violations.is_empty(),
            warnings,
            violations,
            drawdown_info: dd_info,
            timestamp: Utc::now(),
        }
    }

    /// Executa ações baseadas em violações
    pub async fn enforce(&self, result: &RiskCheckResult) {
        if !result.is_healthy {
            for violation in &result.violations {
                match violation {
                    RiskViolation::DrawdownLimitExceeded { .. } => {
                        // Fechar todas as posições
                        self.close_all_positions("Drawdown limit exceeded").await;
                    }
                    RiskViolation::ExposureLimitExceeded { .. } => {
                        // Reduzir posições
                        self.reduce_exposure().await;
                    }
                    RiskViolation::CircuitBreakerActive => {
                        // Já tratado pelo circuit breaker
                    }
                }
            }
        }
    }

    async fn close_all_positions(&self, reason: &str) {
        tracing::warn!(reason = reason, "Closing all positions due to risk violation");

        let positions = self.position_repo.find_open().await.unwrap_or_default();

        for position in positions {
            // Enfileirar ordem de fechamento
            // TODO: implementar
        }
    }
}

pub struct RiskCheckResult {
    pub is_healthy: bool,
    pub warnings: Vec<RiskWarning>,
    pub violations: Vec<RiskViolation>,
    pub drawdown_info: DrawdownInfo,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum RiskWarning {
    HighDrawdown { current: Decimal, threshold: Decimal },
    HighConcentration { symbol: String, pct: Decimal, threshold: Decimal },
    ApproachingDailyLimit { current: Decimal, limit: Decimal },
}

#[derive(Debug, Clone)]
pub enum RiskViolation {
    DrawdownLimitExceeded { current: Decimal, limit: Decimal },
    ExposureLimitExceeded { current: Decimal, limit: Decimal },
    CircuitBreakerActive,
}
```

## Métricas de Risco

```rust
pub struct RiskMetrics {
    /// Sharpe Ratio (anualizado)
    pub sharpe_ratio: Decimal,

    /// Sortino Ratio (anualizado)
    pub sortino_ratio: Decimal,

    /// Calmar Ratio
    pub calmar_ratio: Decimal,

    /// Win Rate
    pub win_rate: Decimal,

    /// Profit Factor
    pub profit_factor: Decimal,

    /// Expectancy
    pub expectancy: Decimal,

    /// Average Win
    pub avg_win: Decimal,

    /// Average Loss
    pub avg_loss: Decimal,

    /// Max Consecutive Wins
    pub max_consecutive_wins: u32,

    /// Max Consecutive Losses
    pub max_consecutive_losses: u32,

    /// Recovery Factor
    pub recovery_factor: Decimal,
}

impl RiskMetrics {
    pub fn calculate(trades: &[Trade], risk_free_rate: Decimal) -> Self {
        let returns: Vec<Decimal> = trades.iter().map(|t| t.return_pct).collect();

        let sharpe = Self::calculate_sharpe(&returns, risk_free_rate);
        let sortino = Self::calculate_sortino(&returns, risk_free_rate);

        let wins: Vec<&Trade> = trades.iter().filter(|t| t.net_pnl > Decimal::ZERO).collect();
        let losses: Vec<&Trade> = trades.iter().filter(|t| t.net_pnl < Decimal::ZERO).collect();

        let win_rate = if !trades.is_empty() {
            Decimal::from(wins.len()) / Decimal::from(trades.len())
        } else {
            Decimal::ZERO
        };

        let avg_win = if !wins.is_empty() {
            wins.iter().map(|t| t.net_pnl).sum::<Decimal>() / Decimal::from(wins.len())
        } else {
            Decimal::ZERO
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().map(|t| t.net_pnl.abs()).sum::<Decimal>() / Decimal::from(losses.len())
        } else {
            Decimal::ZERO
        };

        let total_wins: Decimal = wins.iter().map(|t| t.net_pnl).sum();
        let total_losses: Decimal = losses.iter().map(|t| t.net_pnl.abs()).sum();

        let profit_factor = if total_losses > Decimal::ZERO {
            total_wins / total_losses
        } else {
            Decimal::MAX
        };

        let expectancy = win_rate * avg_win - (Decimal::ONE - win_rate) * avg_loss;

        Self {
            sharpe_ratio: sharpe,
            sortino_ratio: sortino,
            calmar_ratio: Decimal::ZERO, // Requer max drawdown
            win_rate,
            profit_factor,
            expectancy,
            avg_win,
            avg_loss,
            max_consecutive_wins: Self::max_consecutive(&trades, true),
            max_consecutive_losses: Self::max_consecutive(&trades, false),
            recovery_factor: Decimal::ZERO, // Requer max drawdown
        }
    }

    fn calculate_sharpe(returns: &[Decimal], risk_free_rate: Decimal) -> Decimal {
        if returns.is_empty() {
            return Decimal::ZERO;
        }

        let mean = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
        let variance: Decimal = returns
            .iter()
            .map(|r| (*r - mean).powi(2))
            .sum::<Decimal>() / Decimal::from(returns.len());

        let std_dev = variance.sqrt().unwrap_or(Decimal::ONE);

        if std_dev == Decimal::ZERO {
            return Decimal::ZERO;
        }

        // Anualizar (assumindo daily returns)
        let annual_mean = mean * Decimal::from(252);
        let annual_std = std_dev * Decimal::from(252).sqrt().unwrap();

        (annual_mean - risk_free_rate) / annual_std
    }

    fn calculate_sortino(returns: &[Decimal], risk_free_rate: Decimal) -> Decimal {
        if returns.is_empty() {
            return Decimal::ZERO;
        }

        let mean = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        // Downside deviation (apenas retornos negativos)
        let downside_returns: Vec<Decimal> = returns
            .iter()
            .filter(|r| **r < Decimal::ZERO)
            .cloned()
            .collect();

        if downside_returns.is_empty() {
            return Decimal::MAX;
        }

        let downside_variance: Decimal = downside_returns
            .iter()
            .map(|r| r.powi(2))
            .sum::<Decimal>() / Decimal::from(downside_returns.len());

        let downside_std = downside_variance.sqrt().unwrap_or(Decimal::ONE);

        if downside_std == Decimal::ZERO {
            return Decimal::MAX;
        }

        // Anualizar
        let annual_mean = mean * Decimal::from(252);
        let annual_downside = downside_std * Decimal::from(252).sqrt().unwrap();

        (annual_mean - risk_free_rate) / annual_downside
    }

    fn max_consecutive(trades: &[Trade], wins: bool) -> u32 {
        let mut max = 0u32;
        let mut current = 0u32;

        for trade in trades {
            let is_winner = trade.net_pnl > Decimal::ZERO;
            if is_winner == wins {
                current += 1;
                max = max.max(current);
            } else {
                current = 0;
            }
        }

        max
    }
}
```

---

**Próximo**: [Sistema de Filas](./queue-system.md)
