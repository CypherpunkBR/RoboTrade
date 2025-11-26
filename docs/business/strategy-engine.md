# Motor de Estratégias

Este documento descreve o motor de estratégias do RoboTrade, incluindo indicadores técnicos, estratégias de trading e geração de sinais.

## Visão Geral

O motor de estratégias é responsável por:
- Calcular indicadores técnicos
- Avaliar condições de mercado
- Gerar sinais de entrada e saída
- Gerenciar versões de estratégias
- Fornecer contexto para decisões

## Indicadores Técnicos

### Indicadores Implementados

| Indicador | Tipo | Parâmetros | Uso Típico |
|-----------|------|------------|------------|
| SMA | Tendência | period | Identificar tendência |
| EMA | Tendência | period | Tendência com peso recente |
| RSI | Momentum | period | Sobrecompra/sobrevenda |
| MACD | Momentum | fast, slow, signal | Cruzamentos |
| Bollinger Bands | Volatilidade | period, std_dev | Breakout, reversão |
| ATR | Volatilidade | period | Stop loss, position size |
| VWAP | Volume | - | Preço médio ponderado |

### Interface de Indicador

```rust
pub trait Indicator: Send + Sync {
    /// Nome do indicador
    fn name(&self) -> &str;

    /// Número mínimo de candles necessários
    fn min_candles(&self) -> usize;

    /// Calcula o indicador
    fn calculate(&self, candles: &[Candle]) -> IndicatorResult<Vec<IndicatorValue>>;

    /// Valida parâmetros
    fn validate_params(&self) -> Result<(), String>;
}

pub struct IndicatorValue {
    pub timestamp: DateTime<Utc>,
    pub value: Decimal,
    pub metadata: Option<serde_json::Value>,
}
```

### Implementações

#### Simple Moving Average (SMA)

```rust
pub struct SmaIndicator {
    period: usize,
}

impl Indicator for SmaIndicator {
    fn name(&self) -> &str { "SMA" }

    fn min_candles(&self) -> usize { self.period }

    fn calculate(&self, candles: &[Candle]) -> IndicatorResult<Vec<IndicatorValue>> {
        if candles.len() < self.period {
            return Err(AnalyticsError::InsufficientData {
                required: self.period,
                available: candles.len(),
            });
        }

        let mut results = Vec::new();

        for i in (self.period - 1)..candles.len() {
            let sum: Decimal = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();

            let sma = sum / Decimal::from(self.period);

            results.push(IndicatorValue {
                timestamp: candles[i].close_time,
                value: sma,
                metadata: None,
            });
        }

        Ok(results)
    }
}
```

#### Exponential Moving Average (EMA)

```rust
pub struct EmaIndicator {
    period: usize,
}

impl Indicator for EmaIndicator {
    fn name(&self) -> &str { "EMA" }

    fn min_candles(&self) -> usize { self.period }

    fn calculate(&self, candles: &[Candle]) -> IndicatorResult<Vec<IndicatorValue>> {
        if candles.len() < self.period {
            return Err(AnalyticsError::InsufficientData {
                required: self.period,
                available: candles.len(),
            });
        }

        let multiplier = Decimal::from(2) / (Decimal::from(self.period) + Decimal::ONE);
        let mut results = Vec::new();

        // Primeira EMA é SMA
        let initial_sma: Decimal = candles[..self.period]
            .iter()
            .map(|c| c.close)
            .sum::<Decimal>() / Decimal::from(self.period);

        results.push(IndicatorValue {
            timestamp: candles[self.period - 1].close_time,
            value: initial_sma,
            metadata: None,
        });

        let mut prev_ema = initial_sma;

        for candle in candles.iter().skip(self.period) {
            let ema = (candle.close - prev_ema) * multiplier + prev_ema;

            results.push(IndicatorValue {
                timestamp: candle.close_time,
                value: ema,
                metadata: None,
            });

            prev_ema = ema;
        }

        Ok(results)
    }
}
```

#### Relative Strength Index (RSI)

```rust
pub struct RsiIndicator {
    period: usize,
}

impl Indicator for RsiIndicator {
    fn name(&self) -> &str { "RSI" }

    fn min_candles(&self) -> usize { self.period + 1 }

    fn calculate(&self, candles: &[Candle]) -> IndicatorResult<Vec<IndicatorValue>> {
        if candles.len() < self.period + 1 {
            return Err(AnalyticsError::InsufficientData {
                required: self.period + 1,
                available: candles.len(),
            });
        }

        // Calcular mudanças de preço
        let changes: Vec<Decimal> = candles.windows(2)
            .map(|w| w[1].close - w[0].close)
            .collect();

        let mut gains: Vec<Decimal> = Vec::new();
        let mut losses: Vec<Decimal> = Vec::new();

        for change in &changes {
            if *change > Decimal::ZERO {
                gains.push(*change);
                losses.push(Decimal::ZERO);
            } else {
                gains.push(Decimal::ZERO);
                losses.push(change.abs());
            }
        }

        let mut results = Vec::new();

        // Primeira média
        let mut avg_gain: Decimal = gains[..self.period].iter().sum::<Decimal>()
            / Decimal::from(self.period);
        let mut avg_loss: Decimal = losses[..self.period].iter().sum::<Decimal>()
            / Decimal::from(self.period);

        for i in self.period..changes.len() {
            // Smooth averages
            avg_gain = (avg_gain * Decimal::from(self.period - 1) + gains[i])
                / Decimal::from(self.period);
            avg_loss = (avg_loss * Decimal::from(self.period - 1) + losses[i])
                / Decimal::from(self.period);

            let rsi = if avg_loss == Decimal::ZERO {
                Decimal::from(100)
            } else {
                let rs = avg_gain / avg_loss;
                Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs))
            };

            results.push(IndicatorValue {
                timestamp: candles[i + 1].close_time,
                value: rsi,
                metadata: None,
            });
        }

        Ok(results)
    }
}
```

#### Bollinger Bands

```rust
pub struct BollingerBandsIndicator {
    period: usize,
    std_dev: Decimal,
}

impl Indicator for BollingerBandsIndicator {
    fn name(&self) -> &str { "BB" }

    fn min_candles(&self) -> usize { self.period }

    fn calculate(&self, candles: &[Candle]) -> IndicatorResult<Vec<IndicatorValue>> {
        let sma = SmaIndicator { period: self.period };
        let sma_values = sma.calculate(candles)?;

        let mut results = Vec::new();

        for (i, sma_val) in sma_values.iter().enumerate() {
            let candle_index = self.period - 1 + i;
            let window = &candles[(candle_index + 1 - self.period)..=candle_index];

            // Calcular desvio padrão
            let mean = sma_val.value;
            let variance: Decimal = window
                .iter()
                .map(|c| (c.close - mean).powi(2))
                .sum::<Decimal>() / Decimal::from(self.period);

            let std = variance.sqrt().unwrap_or(Decimal::ZERO);

            let upper = mean + self.std_dev * std;
            let lower = mean - self.std_dev * std;

            results.push(IndicatorValue {
                timestamp: sma_val.timestamp,
                value: mean,
                metadata: Some(serde_json::json!({
                    "upper": upper.to_string(),
                    "middle": mean.to_string(),
                    "lower": lower.to_string(),
                    "bandwidth": ((upper - lower) / mean * Decimal::from(100)).to_string(),
                })),
            });
        }

        Ok(results)
    }
}
```

## Estratégias

### Interface de Estratégia

```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    /// ID único da estratégia
    fn id(&self) -> &str;

    /// Nome amigável
    fn name(&self) -> &str;

    /// Descrição
    fn description(&self) -> &str;

    /// Versão da estratégia
    fn version(&self) -> &str;

    /// Símbolos que opera
    fn symbols(&self) -> &[String];

    /// Timeframes necessários
    fn required_timeframes(&self) -> &[TimeFrame];

    /// Indicadores necessários
    fn required_indicators(&self) -> Vec<IndicatorConfig>;

    /// Mínimo de candles para operar
    fn min_candles_required(&self) -> usize;

    /// Avalia mercado e gera sinais de entrada
    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal>;

    /// Verifica se deve fechar posição
    async fn should_close_position(&self, context: &StrategyContext) -> Option<Signal>;

    /// Calcula stop loss
    fn calculate_stop_loss(&self, context: &StrategyContext, side: OrderSide) -> Option<Decimal>;

    /// Calcula take profit
    fn calculate_take_profit(&self, context: &StrategyContext, side: OrderSide) -> Option<Decimal>;

    /// Calcula tamanho da posição
    fn calculate_position_size(&self, context: &StrategyContext) -> Option<Decimal>;
}
```

### Contexto de Estratégia

```rust
pub struct StrategyContext {
    /// Candles recentes (mais antigo primeiro)
    pub candles: Vec<Candle>,

    /// Ticker atual
    pub ticker: Option<Ticker>,

    /// Posição atual (se houver)
    pub current_position: Option<Position>,

    /// Ordens abertas
    pub open_orders: Vec<Order>,

    /// Dados de Fear & Greed
    pub fear_greed: Option<FearGreedData>,

    /// Indicadores pré-calculados
    pub indicators: HashMap<String, Vec<IndicatorValue>>,

    /// Configurações de risco
    pub risk_config: RiskConfig,

    /// Saldo disponível
    pub available_balance: Decimal,

    /// Timestamp da avaliação
    pub timestamp: DateTime<Utc>,
}

impl StrategyContext {
    /// Retorna último valor de um indicador
    pub fn last_indicator_value(&self, indicator: &str) -> Option<&IndicatorValue> {
        self.indicators.get(indicator)?.last()
    }

    /// Retorna valores de um indicador para N períodos
    pub fn indicator_values(&self, indicator: &str, periods: usize) -> Option<&[IndicatorValue]> {
        let values = self.indicators.get(indicator)?;
        if values.len() >= periods {
            Some(&values[values.len() - periods..])
        } else {
            None
        }
    }

    /// Último candle
    pub fn last_candle(&self) -> Option<&Candle> {
        self.candles.last()
    }

    /// Preço atual
    pub fn current_price(&self) -> Option<Decimal> {
        self.ticker.as_ref().map(|t| t.last_price)
            .or_else(|| self.candles.last().map(|c| c.close))
    }
}
```

### Estratégia Fear & Greed Contrarian

```rust
pub struct FearGreedStrategy {
    id: String,
    config: FearGreedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedConfig {
    /// Comprar quando <= threshold
    pub buy_threshold: u32,

    /// Vender quando >= threshold
    pub sell_threshold: u32,

    /// Símbolos para operar
    pub symbols: Vec<String>,

    /// Tamanho da posição (% do capital)
    pub position_size_pct: Decimal,

    /// Usar stop loss
    pub use_stop_loss: bool,

    /// Distância do stop loss (%)
    pub stop_loss_pct: Decimal,

    /// Usar take profit
    pub use_take_profit: bool,

    /// Distância do take profit (%)
    pub take_profit_pct: Decimal,

    /// Cooldown entre sinais (horas)
    pub cooldown_hours: u32,
}

#[async_trait]
impl Strategy for FearGreedStrategy {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Fear & Greed Contrarian" }
    fn version(&self) -> &str { "1.0.0" }

    fn description(&self) -> &str {
        "Estratégia contrarian baseada no índice Fear & Greed. \
         Compra quando o mercado está em medo extremo (índice baixo) e \
         vende quando está em ganância extrema (índice alto)."
    }

    fn symbols(&self) -> &[String] { &self.config.symbols }

    fn required_timeframes(&self) -> &[TimeFrame] {
        &[TimeFrame::D1]
    }

    fn required_indicators(&self) -> Vec<IndicatorConfig> {
        vec![]  // Não usa indicadores técnicos
    }

    fn min_candles_required(&self) -> usize { 1 }

    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal> {
        let fg = context.fear_greed.as_ref()?;
        let current_price = context.current_price()?;

        // Não gerar sinal se já tem posição
        if context.current_position.is_some() {
            return None;
        }

        // Sinal de compra: medo extremo
        if fg.value <= self.config.buy_threshold {
            let stop_loss = self.calculate_stop_loss(context, OrderSide::Buy);
            let take_profit = self.calculate_take_profit(context, OrderSide::Buy);
            let quantity = self.calculate_position_size(context);

            return Some(Signal {
                id: SignalId::new(),
                strategy_id: self.id.clone(),
                strategy_version: self.version().to_string(),
                symbol: self.config.symbols[0].clone(),
                signal_type: SignalType::Entry,
                side: OrderSide::Buy,
                entry_price: current_price,
                stop_loss,
                take_profit,
                quantity,
                risk_reward_ratio: self.calculate_rr_ratio(current_price, stop_loss, take_profit),
                confidence: self.calculate_confidence(fg.value, true),
                status: SignalStatus::Pending,
                reason: format!(
                    "Fear & Greed Index at {} ({}) - Extreme Fear zone",
                    fg.value, fg.classification
                ),
                metadata: serde_json::json!({
                    "fear_greed_value": fg.value,
                    "fear_greed_classification": fg.classification,
                }),
                created_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::hours(24)),
            });
        }

        None
    }

    async fn should_close_position(&self, context: &StrategyContext) -> Option<Signal> {
        let fg = context.fear_greed.as_ref()?;
        let position = context.current_position.as_ref()?;
        let current_price = context.current_price()?;

        // Fechar long quando em ganância extrema
        if position.side == PositionSide::Long && fg.value >= self.config.sell_threshold {
            return Some(Signal {
                id: SignalId::new(),
                strategy_id: self.id.clone(),
                strategy_version: self.version().to_string(),
                symbol: position.symbol.clone(),
                signal_type: SignalType::Exit,
                side: OrderSide::Sell,
                entry_price: current_price,
                stop_loss: None,
                take_profit: None,
                quantity: Some(position.quantity),
                risk_reward_ratio: None,
                confidence: self.calculate_confidence(fg.value, false),
                status: SignalStatus::Pending,
                reason: format!(
                    "Fear & Greed Index at {} ({}) - Extreme Greed zone",
                    fg.value, fg.classification
                ),
                metadata: serde_json::json!({
                    "fear_greed_value": fg.value,
                    "position_pnl": position.unrealized_pnl,
                }),
                created_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::hours(24)),
            });
        }

        None
    }

    fn calculate_stop_loss(&self, context: &StrategyContext, side: OrderSide) -> Option<Decimal> {
        if !self.config.use_stop_loss {
            return None;
        }

        let price = context.current_price()?;
        let distance = price * self.config.stop_loss_pct / Decimal::from(100);

        match side {
            OrderSide::Buy => Some(price - distance),
            OrderSide::Sell => Some(price + distance),
        }
    }

    fn calculate_take_profit(&self, context: &StrategyContext, side: OrderSide) -> Option<Decimal> {
        if !self.config.use_take_profit {
            return None;
        }

        let price = context.current_price()?;
        let distance = price * self.config.take_profit_pct / Decimal::from(100);

        match side {
            OrderSide::Buy => Some(price + distance),
            OrderSide::Sell => Some(price - distance),
        }
    }

    fn calculate_position_size(&self, context: &StrategyContext) -> Option<Decimal> {
        let available = context.available_balance;
        let price = context.current_price()?;

        let notional = available * self.config.position_size_pct;
        let quantity = notional / price;

        Some(quantity)
    }
}

impl FearGreedStrategy {
    fn calculate_confidence(&self, fg_value: u32, is_buy: bool) -> Decimal {
        // Mais extremo = mais confiança
        let distance = if is_buy {
            self.config.buy_threshold.saturating_sub(fg_value) as f64
        } else {
            fg_value.saturating_sub(self.config.sell_threshold) as f64
        };

        let max_distance = 25.0; // Máximo de distância considerado
        let confidence = (0.5 + (distance / max_distance) * 0.5).min(1.0);

        Decimal::from_f64(confidence).unwrap_or(Decimal::from_str("0.5").unwrap())
    }

    fn calculate_rr_ratio(
        &self,
        entry: Decimal,
        stop_loss: Option<Decimal>,
        take_profit: Option<Decimal>,
    ) -> Option<Decimal> {
        let sl = stop_loss?;
        let tp = take_profit?;

        let risk = (entry - sl).abs();
        let reward = (tp - entry).abs();

        if risk == Decimal::ZERO {
            return None;
        }

        Some(reward / risk)
    }
}
```

### Estratégia RSI Reversal

```rust
pub struct RsiReversalStrategy {
    id: String,
    config: RsiReversalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiReversalConfig {
    pub rsi_period: usize,
    pub oversold_threshold: Decimal,
    pub overbought_threshold: Decimal,
    pub symbols: Vec<String>,
    pub timeframe: TimeFrame,
    pub position_size_pct: Decimal,
    pub stop_loss_atr_multiplier: Decimal,
    pub take_profit_atr_multiplier: Decimal,
    pub atr_period: usize,
}

#[async_trait]
impl Strategy for RsiReversalStrategy {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "RSI Reversal" }
    fn version(&self) -> &str { "1.0.0" }

    fn description(&self) -> &str {
        "Estratégia de reversão baseada em RSI. \
         Compra quando RSI está em sobrevenda e vende quando está em sobrecompra."
    }

    fn symbols(&self) -> &[String] { &self.config.symbols }

    fn required_timeframes(&self) -> &[TimeFrame] {
        &[self.config.timeframe]
    }

    fn required_indicators(&self) -> Vec<IndicatorConfig> {
        vec![
            IndicatorConfig::Rsi { period: self.config.rsi_period },
            IndicatorConfig::Atr { period: self.config.atr_period },
        ]
    }

    fn min_candles_required(&self) -> usize {
        self.config.rsi_period.max(self.config.atr_period) + 10
    }

    async fn evaluate(&self, context: &StrategyContext) -> Option<Signal> {
        if context.current_position.is_some() {
            return None;
        }

        let rsi = context.last_indicator_value("RSI")?.value;
        let atr = context.last_indicator_value("ATR")?.value;
        let current_price = context.current_price()?;

        // Compra em sobrevenda
        if rsi <= self.config.oversold_threshold {
            let stop_loss = Some(current_price - atr * self.config.stop_loss_atr_multiplier);
            let take_profit = Some(current_price + atr * self.config.take_profit_atr_multiplier);

            return Some(Signal {
                id: SignalId::new(),
                strategy_id: self.id.clone(),
                strategy_version: self.version().to_string(),
                symbol: context.candles.last()?.symbol.clone(),
                signal_type: SignalType::Entry,
                side: OrderSide::Buy,
                entry_price: current_price,
                stop_loss,
                take_profit,
                quantity: self.calculate_position_size(context),
                risk_reward_ratio: Some(
                    self.config.take_profit_atr_multiplier / self.config.stop_loss_atr_multiplier
                ),
                confidence: self.calculate_confidence(rsi, true),
                status: SignalStatus::Pending,
                reason: format!("RSI at {:.2} - Oversold condition", rsi),
                metadata: serde_json::json!({
                    "rsi": rsi.to_string(),
                    "atr": atr.to_string(),
                }),
                created_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::hours(4)),
            });
        }

        // Short em sobrecompra
        if rsi >= self.config.overbought_threshold {
            let stop_loss = Some(current_price + atr * self.config.stop_loss_atr_multiplier);
            let take_profit = Some(current_price - atr * self.config.take_profit_atr_multiplier);

            return Some(Signal {
                id: SignalId::new(),
                strategy_id: self.id.clone(),
                strategy_version: self.version().to_string(),
                symbol: context.candles.last()?.symbol.clone(),
                signal_type: SignalType::Entry,
                side: OrderSide::Sell,
                entry_price: current_price,
                stop_loss,
                take_profit,
                quantity: self.calculate_position_size(context),
                risk_reward_ratio: Some(
                    self.config.take_profit_atr_multiplier / self.config.stop_loss_atr_multiplier
                ),
                confidence: self.calculate_confidence(rsi, false),
                status: SignalStatus::Pending,
                reason: format!("RSI at {:.2} - Overbought condition", rsi),
                metadata: serde_json::json!({
                    "rsi": rsi.to_string(),
                    "atr": atr.to_string(),
                }),
                created_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::hours(4)),
            });
        }

        None
    }

    // ... implementar should_close_position, etc.
}
```

## Registry de Estratégias

```rust
pub struct StrategyRegistry {
    strategies: RwLock<HashMap<String, Arc<dyn Strategy>>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, strategy: Arc<dyn Strategy>) {
        let id = strategy.id().to_string();
        self.strategies.write().insert(id.clone(), strategy);
        tracing::info!(strategy_id = id, "Strategy registered");
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Strategy>> {
        self.strategies.read().get(id).cloned()
    }

    pub fn list(&self) -> Vec<StrategyInfo> {
        self.strategies
            .read()
            .values()
            .map(|s| StrategyInfo {
                id: s.id().to_string(),
                name: s.name().to_string(),
                description: s.description().to_string(),
                version: s.version().to_string(),
                symbols: s.symbols().to_vec(),
                timeframes: s.required_timeframes().to_vec(),
            })
            .collect()
    }

    pub fn load_from_config(&self, configs: &HashMap<String, StrategyConfig>) {
        for (strategy_id, config) in configs {
            if !config.enabled {
                continue;
            }

            match self.create_strategy(strategy_id, config) {
                Ok(strategy) => self.register(strategy),
                Err(e) => tracing::error!(
                    strategy_id = strategy_id,
                    error = %e,
                    "Failed to create strategy"
                ),
            }
        }
    }

    fn create_strategy(
        &self,
        strategy_id: &str,
        config: &StrategyConfig,
    ) -> Result<Arc<dyn Strategy>, AnalyticsError> {
        match strategy_id {
            "fear_greed_contrarian" => {
                let fg_config: FearGreedConfig = serde_json::from_value(config.params.clone())?;
                Ok(Arc::new(FearGreedStrategy {
                    id: strategy_id.to_string(),
                    config: fg_config,
                }))
            }
            "rsi_reversal" => {
                let rsi_config: RsiReversalConfig = serde_json::from_value(config.params.clone())?;
                Ok(Arc::new(RsiReversalStrategy {
                    id: strategy_id.to_string(),
                    config: rsi_config,
                }))
            }
            _ => Err(AnalyticsError::StrategyNotFound {
                strategy_id: strategy_id.to_string(),
            }),
        }
    }
}
```

## Engine de Avaliação

```rust
pub struct StrategyEngine {
    registry: Arc<StrategyRegistry>,
    candle_repo: Arc<dyn CandleRepository>,
    fear_greed_repo: Arc<dyn FearGreedRepository>,
    signal_repo: Arc<dyn SignalRepository>,
    position_repo: Arc<dyn PositionRepository>,
    indicator_calculator: Arc<IndicatorCalculator>,
    risk_config: RiskConfig,
}

impl StrategyEngine {
    /// Avalia todas as estratégias habilitadas
    pub async fn evaluate_all(&self) -> AnalyticsResult<Vec<Signal>> {
        let strategies = self.registry.list();
        let mut all_signals = Vec::new();

        for strategy_info in strategies {
            match self.evaluate_strategy(&strategy_info.id).await {
                Ok(signals) => all_signals.extend(signals),
                Err(e) => tracing::error!(
                    strategy_id = strategy_info.id,
                    error = %e,
                    "Strategy evaluation failed"
                ),
            }
        }

        Ok(all_signals)
    }

    /// Avalia uma estratégia específica
    pub async fn evaluate_strategy(&self, strategy_id: &str) -> AnalyticsResult<Vec<Signal>> {
        let strategy = self.registry.get(strategy_id)
            .ok_or(AnalyticsError::StrategyNotFound {
                strategy_id: strategy_id.to_string(),
            })?;

        let mut signals = Vec::new();

        for symbol in strategy.symbols() {
            let context = self.build_context(&strategy, symbol).await?;

            // Verificar se deve fechar posição existente
            if let Some(signal) = strategy.should_close_position(&context).await {
                self.save_signal(&signal).await?;
                signals.push(signal);
            }
            // Verificar entrada
            else if let Some(signal) = strategy.evaluate(&context).await {
                self.save_signal(&signal).await?;
                signals.push(signal);
            }
        }

        Ok(signals)
    }

    async fn build_context(
        &self,
        strategy: &Arc<dyn Strategy>,
        symbol: &str,
    ) -> AnalyticsResult<StrategyContext> {
        // Buscar candles
        let primary_tf = strategy.required_timeframes().first()
            .ok_or(AnalyticsError::InvalidStrategyConfig {
                message: "No timeframes configured".to_string(),
            })?;

        let candles = self.candle_repo
            .find_candles(symbol, *primary_tf, strategy.min_candles_required())
            .await?;

        // Calcular indicadores
        let mut indicators = HashMap::new();
        for indicator_config in strategy.required_indicators() {
            let values = self.indicator_calculator
                .calculate(&indicator_config, &candles)?;
            indicators.insert(indicator_config.name(), values);
        }

        // Buscar Fear & Greed
        let fear_greed = self.fear_greed_repo.get_latest().await.ok().flatten();

        // Buscar posição atual
        let current_position = self.position_repo
            .find_open_by_symbol(symbol)
            .await?;

        Ok(StrategyContext {
            candles,
            ticker: None, // TODO: implementar
            current_position,
            open_orders: Vec::new(),
            fear_greed,
            indicators,
            risk_config: self.risk_config.clone(),
            available_balance: Decimal::from(10000), // TODO: buscar saldo real
            timestamp: Utc::now(),
        })
    }

    async fn save_signal(&self, signal: &Signal) -> AnalyticsResult<()> {
        self.signal_repo.save(signal).await?;

        tracing::info!(
            signal_id = %signal.id,
            strategy_id = signal.strategy_id,
            symbol = signal.symbol,
            side = ?signal.side,
            entry_price = %signal.entry_price,
            confidence = %signal.confidence,
            "Signal generated"
        );

        Ok(())
    }
}
```

---

**Próximo**: [Execução de Ordens](./order-execution.md)
