# Execução de Ordens

Este documento descreve as regras de negócio para execução de ordens no RoboTrade.

## Visão Geral

O fluxo de execução de ordens segue um pipeline rigoroso:

```
Signal → Validação → Risk Check → Job Queue → Execution → Confirmation
```

## Tipos de Ordem

### Ordens Suportadas

| Tipo | Descrição | Uso |
|------|-----------|-----|
| MARKET | Execução imediata ao preço de mercado | Entrada/saída rápida |
| LIMIT | Execução apenas no preço especificado ou melhor | Entrada com preço definido |
| STOP_MARKET | Market order quando preço atinge trigger | Stop loss |
| STOP_LIMIT | Limit order quando preço atinge trigger | Stop loss com preço mínimo |
| TAKE_PROFIT_MARKET | Market order quando preço atinge target | Take profit |

### Estrutura de Ordem

```rust
pub struct OrderRequest {
    /// Símbolo do par
    pub symbol: String,

    /// Lado da ordem
    pub side: OrderSide,

    /// Tipo de ordem
    pub order_type: OrderType,

    /// Quantidade
    pub quantity: Decimal,

    /// Preço (para LIMIT orders)
    pub price: Option<Decimal>,

    /// Preço de trigger (para STOP orders)
    pub stop_price: Option<Decimal>,

    /// Se é reduce-only
    pub reduce_only: bool,

    /// ID do cliente
    pub client_order_id: Option<String>,

    /// ID do sinal que gerou a ordem
    pub signal_id: Option<SignalId>,

    /// Time in force
    pub time_in_force: TimeInForce,
}

pub enum OrderSide {
    Buy,
    Sell,
}

pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TakeProfitMarket,
}

pub enum TimeInForce {
    GTC,  // Good Till Cancelled
    IOC,  // Immediate Or Cancel
    FOK,  // Fill Or Kill
    GTX,  // Good Till Crossing (Post Only)
}
```

## Pipeline de Execução

### 1. Recebimento do Sinal

```rust
pub struct SignalHandler {
    signal_repo: Arc<dyn SignalRepository>,
    order_service: Arc<OrderService>,
}

impl SignalHandler {
    pub async fn handle_signal(&self, signal: Signal) -> WorkerResult<()> {
        // 1. Atualizar status do sinal
        self.signal_repo.update_status(&signal.id, SignalStatus::Processing).await?;

        // 2. Criar request de ordem
        let order_request = OrderRequest {
            symbol: signal.symbol.clone(),
            side: signal.side,
            order_type: OrderType::Market,
            quantity: signal.quantity.unwrap_or_default(),
            price: None,
            stop_price: None,
            reduce_only: signal.signal_type == SignalType::Exit,
            client_order_id: Some(format!("signal-{}", signal.id)),
            signal_id: Some(signal.id.clone()),
            time_in_force: TimeInForce::GTC,
        };

        // 3. Enviar para serviço de ordens
        match self.order_service.place_order(order_request).await {
            Ok(order) => {
                self.signal_repo.update_status(&signal.id, SignalStatus::Executed).await?;
                self.signal_repo.link_order(&signal.id, &order.id).await?;

                // 4. Criar ordens de SL/TP se configuradas
                if let Some(sl) = signal.stop_loss {
                    self.order_service.place_stop_loss(&order, sl).await?;
                }
                if let Some(tp) = signal.take_profit {
                    self.order_service.place_take_profit(&order, tp).await?;
                }

                Ok(())
            }
            Err(e) => {
                self.signal_repo.update_status(&signal.id, SignalStatus::Failed).await?;
                Err(e)
            }
        }
    }
}
```

### 2. Validação

```rust
pub struct OrderValidator {
    symbol_repo: Arc<dyn SymbolRepository>,
}

impl OrderValidator {
    pub fn validate(&self, request: &OrderRequest) -> ValidationResult<()> {
        // Validar símbolo
        let symbol_info = self.symbol_repo.get(&request.symbol)?;
        if symbol_info.status != SymbolStatus::Trading {
            return Err(ValidationError::SymbolNotTrading(request.symbol.clone()));
        }

        // Validar quantidade
        self.validate_quantity(request, &symbol_info)?;

        // Validar preço
        self.validate_price(request, &symbol_info)?;

        // Validar tipo de ordem
        self.validate_order_type(request)?;

        Ok(())
    }

    fn validate_quantity(
        &self,
        request: &OrderRequest,
        symbol: &SymbolInfo,
    ) -> ValidationResult<()> {
        // Quantidade mínima
        if request.quantity < symbol.min_quantity {
            return Err(ValidationError::QuantityTooSmall {
                min: symbol.min_quantity,
                actual: request.quantity,
            });
        }

        // Quantidade máxima
        if let Some(max) = symbol.max_quantity {
            if request.quantity > max {
                return Err(ValidationError::QuantityTooLarge {
                    max,
                    actual: request.quantity,
                });
            }
        }

        // Step size
        if !is_multiple_of(request.quantity, symbol.step_size) {
            return Err(ValidationError::InvalidQuantityStep {
                step: symbol.step_size,
                actual: request.quantity,
            });
        }

        Ok(())
    }

    fn validate_price(
        &self,
        request: &OrderRequest,
        symbol: &SymbolInfo,
    ) -> ValidationResult<()> {
        if let Some(price) = request.price {
            // Preço positivo
            if price <= Decimal::ZERO {
                return Err(ValidationError::InvalidPrice("Price must be positive".into()));
            }

            // Tick size
            if !is_multiple_of(price, symbol.tick_size) {
                return Err(ValidationError::InvalidPriceTick {
                    tick: symbol.tick_size,
                    actual: price,
                });
            }

            // Notional mínimo
            if let Some(min_notional) = symbol.min_notional {
                let notional = price * request.quantity;
                if notional < min_notional {
                    return Err(ValidationError::NotionalTooSmall {
                        min: min_notional,
                        actual: notional,
                    });
                }
            }
        }

        Ok(())
    }
}
```

### 3. Verificação de Risco

```rust
pub struct RiskChecker {
    config: RiskConfig,
    position_repo: Arc<dyn PositionRepository>,
    trade_repo: Arc<dyn TradeRepository>,
    daily_pnl: Arc<AtomicDecimal>,
}

impl RiskChecker {
    pub async fn check(&self, request: &OrderRequest, portfolio: &Portfolio) -> RiskResult<()> {
        // 1. Verificar tamanho máximo de posição
        self.check_position_size(request, portfolio)?;

        // 2. Verificar risco por trade
        self.check_risk_per_trade(request, portfolio)?;

        // 3. Verificar perda diária
        self.check_daily_loss()?;

        // 4. Verificar número de posições
        self.check_max_positions(request).await?;

        // 5. Verificar rate limit
        self.check_order_rate_limit()?;

        // 6. Verificar stop loss (se obrigatório)
        self.check_stop_loss_required(request)?;

        // 7. Verificar circuit breaker
        self.check_circuit_breaker()?;

        Ok(())
    }

    fn check_position_size(&self, request: &OrderRequest, portfolio: &Portfolio) -> RiskResult<()> {
        let current_price = portfolio.get_price(&request.symbol)?;
        let position_value = request.quantity * current_price;

        if position_value > self.config.max_position_size {
            return Err(RiskError::PositionSizeExceeded {
                max: self.config.max_position_size,
                requested: position_value,
            });
        }

        Ok(())
    }

    fn check_risk_per_trade(&self, request: &OrderRequest, portfolio: &Portfolio) -> RiskResult<()> {
        // Calcular risco baseado no stop loss
        let risk_amount = match &request.stop_loss {
            Some(sl) => {
                let entry = request.price.unwrap_or(portfolio.get_price(&request.symbol)?);
                (entry - sl).abs() * request.quantity
            }
            None => {
                // Sem SL, considera risco total
                request.quantity * portfolio.get_price(&request.symbol)?
            }
        };

        let max_risk = portfolio.total_equity * self.config.max_risk_per_trade;

        if risk_amount > max_risk {
            return Err(RiskError::RiskPerTradeExceeded {
                max: max_risk,
                actual: risk_amount,
            });
        }

        Ok(())
    }

    fn check_daily_loss(&self) -> RiskResult<()> {
        let current_pnl = self.daily_pnl.load();

        if current_pnl < -self.config.max_daily_loss {
            return Err(RiskError::DailyLossLimitReached {
                limit: self.config.max_daily_loss,
                current: current_pnl,
            });
        }

        Ok(())
    }

    async fn check_max_positions(&self, request: &OrderRequest) -> RiskResult<()> {
        // Ignorar se é reduce only
        if request.reduce_only {
            return Ok(());
        }

        let open_positions = self.position_repo.find_open().await?.len();

        if open_positions >= self.config.max_open_positions as usize {
            return Err(RiskError::MaxPositionsReached {
                max: self.config.max_open_positions,
                current: open_positions as u32,
            });
        }

        Ok(())
    }

    fn check_circuit_breaker(&self) -> RiskResult<()> {
        if self.circuit_breaker.is_triggered() {
            return Err(RiskError::CircuitBreakerActive {
                cooldown_until: self.circuit_breaker.cooldown_until(),
            });
        }

        Ok(())
    }
}
```

### 4. Enfileiramento

```rust
pub struct OrderService {
    validator: Arc<OrderValidator>,
    risk_checker: Arc<RiskChecker>,
    job_queue: Arc<JobQueue>,
    config: TradingConfig,
}

impl OrderService {
    pub async fn place_order(&self, request: OrderRequest) -> WorkerResult<JobId> {
        // 1. Validar
        self.validator.validate(&request)?;

        // 2. Buscar portfolio
        let portfolio = self.get_portfolio().await?;

        // 3. Verificar risco
        self.risk_checker.check(&request, &portfolio).await?;

        // 4. Criar job
        let job = Job::new(
            JobPayload::PlaceOrder(PlaceOrderPayload::from(request)),
            JobPriority::High,
        );

        // 5. Enfileirar
        let job_id = self.job_queue.enqueue(job).await?;

        tracing::info!(
            job_id = %job_id,
            symbol = request.symbol,
            side = ?request.side,
            quantity = %request.quantity,
            "Order enqueued"
        );

        Ok(job_id)
    }

    pub async fn place_stop_loss(&self, order: &Order, stop_price: Decimal) -> WorkerResult<JobId> {
        let request = OrderRequest {
            symbol: order.symbol.clone(),
            side: order.side.opposite(),
            order_type: OrderType::StopMarket,
            quantity: order.filled_quantity,
            price: None,
            stop_price: Some(stop_price),
            reduce_only: true,
            client_order_id: Some(format!("{}-sl", order.id)),
            signal_id: None,
            time_in_force: TimeInForce::GTC,
        };

        self.place_order(request).await
    }

    pub async fn place_take_profit(&self, order: &Order, take_profit: Decimal) -> WorkerResult<JobId> {
        let request = OrderRequest {
            symbol: order.symbol.clone(),
            side: order.side.opposite(),
            order_type: OrderType::TakeProfitMarket,
            quantity: order.filled_quantity,
            price: None,
            stop_price: Some(take_profit),
            reduce_only: true,
            client_order_id: Some(format!("{}-tp", order.id)),
            signal_id: None,
            time_in_force: TimeInForce::GTC,
        };

        self.place_order(request).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> WorkerResult<JobId> {
        let job = Job::new(
            JobPayload::CancelOrder(CancelOrderPayload {
                order_id: order_id.to_string(),
            }),
            JobPriority::Critical,
        );

        self.job_queue.enqueue(job).await
    }
}
```

### 5. Execução

```rust
pub struct OrderExecutor<G: ExchangeGateway> {
    gateway: Arc<G>,
    order_repo: Arc<dyn OrderRepository>,
    position_repo: Arc<dyn PositionRepository>,
    metrics: Arc<MetricsRegistry>,
}

impl<G: ExchangeGateway> OrderExecutor<G> {
    pub async fn execute(&self, payload: PlaceOrderPayload) -> WorkerResult<Order> {
        let start = Instant::now();

        // Converter payload para OrderRequest
        let request = OrderRequest::from(payload);

        tracing::info!(
            symbol = request.symbol,
            side = ?request.side,
            order_type = ?request.order_type,
            quantity = %request.quantity,
            "Executing order"
        );

        // Submeter ordem
        let order = self.gateway.submit_order(request.clone()).await?;

        // Salvar ordem no banco
        self.order_repo.save(&order).await?;

        // Atualizar posição se preenchida
        if order.status == OrderStatus::Filled {
            self.update_position(&order).await?;
        }

        // Métricas
        let latency = start.elapsed().as_millis() as f64;
        self.metrics.exchange().latency.observe(latency);
        self.metrics.exchange().orders_submitted.inc();

        if order.status == OrderStatus::Filled {
            self.metrics.exchange().orders_filled.inc();
        }

        tracing::info!(
            order_id = %order.id,
            status = ?order.status,
            filled_quantity = %order.filled_quantity,
            average_price = ?order.average_price,
            latency_ms = latency,
            "Order executed"
        );

        Ok(order)
    }

    async fn update_position(&self, order: &Order) -> WorkerResult<()> {
        let existing = self.position_repo.find_open_by_symbol(&order.symbol).await?;

        match existing {
            Some(mut position) => {
                // Atualizar posição existente
                if order.side == position.side.to_order_side() {
                    // Aumentando posição
                    position.add_to_position(order.filled_quantity, order.average_price.unwrap());
                } else {
                    // Reduzindo ou fechando posição
                    position.reduce_position(order.filled_quantity, order.average_price.unwrap());

                    if position.quantity == Decimal::ZERO {
                        position.status = PositionStatus::Closed;
                        position.closed_at = Some(Utc::now());

                        // Criar trade record
                        self.create_trade(&position, order).await?;
                    }
                }

                self.position_repo.save(&position).await?;
            }
            None if !order.reduce_only => {
                // Nova posição
                let position = Position {
                    id: PositionId::new(),
                    exchange: self.gateway.exchange_id(),
                    symbol: order.symbol.clone(),
                    side: PositionSide::from(order.side),
                    status: PositionStatus::Open,
                    quantity: order.filled_quantity,
                    entry_price: order.average_price.unwrap(),
                    current_price: order.average_price,
                    leverage: 1,
                    margin_type: MarginType::Isolated,
                    unrealized_pnl: None,
                    realized_pnl: Some(Decimal::ZERO),
                    stop_loss: None,
                    take_profit: None,
                    signal_id: order.signal_id.clone(),
                    entry_order_id: Some(order.id.clone()),
                    exit_order_id: None,
                    opened_at: Utc::now(),
                    closed_at: None,
                    metadata: None,
                };

                self.position_repo.save(&position).await?;
            }
            _ => {
                // Reduce only mas sem posição existente - ignorar
                tracing::warn!(
                    order_id = %order.id,
                    "Reduce only order without existing position"
                );
            }
        }

        Ok(())
    }

    async fn create_trade(&self, position: &Position, exit_order: &Order) -> WorkerResult<()> {
        let exit_price = exit_order.average_price.unwrap();
        let gross_pnl = match position.side {
            PositionSide::Long => (exit_price - position.entry_price) * position.quantity,
            PositionSide::Short => (position.entry_price - exit_price) * position.quantity,
        };

        let fees = Decimal::ZERO; // TODO: calcular fees reais
        let net_pnl = gross_pnl - fees;
        let return_pct = (net_pnl / (position.entry_price * position.quantity)) * Decimal::from(100);

        let trade = Trade {
            id: TradeId::new(),
            position_id: Some(position.id.clone()),
            exchange: position.exchange,
            symbol: position.symbol.clone(),
            side: position.side,
            entry_order_id: position.entry_order_id.clone(),
            exit_order_id: Some(exit_order.id.clone()),
            quantity: position.quantity,
            entry_price: position.entry_price,
            exit_price,
            gross_pnl,
            fees,
            net_pnl,
            return_pct,
            duration_seconds: Some(
                (Utc::now() - position.opened_at).num_seconds() as u64
            ),
            strategy_id: None, // TODO: vincular
            signal_id: position.signal_id.clone(),
            metadata: None,
            opened_at: position.opened_at,
            closed_at: Utc::now(),
        };

        self.trade_repo.save(&trade).await?;

        // Métricas
        if net_pnl > Decimal::ZERO {
            self.metrics.trading().winning_trades.inc();
        } else {
            self.metrics.trading().losing_trades.inc();
        }
        self.metrics.trading().total_trades.inc();

        tracing::info!(
            trade_id = %trade.id,
            symbol = trade.symbol,
            side = ?trade.side,
            entry_price = %trade.entry_price,
            exit_price = %trade.exit_price,
            net_pnl = %trade.net_pnl,
            return_pct = %trade.return_pct,
            "Trade closed"
        );

        Ok(())
    }
}
```

## Ciclo de Vida da Ordem

```
                              ┌─────────────┐
                              │   PENDING   │
                              └──────┬──────┘
                                     │
                           ┌─────────┴─────────┐
                           ▼                   ▼
                    ┌─────────────┐     ┌─────────────┐
                    │     NEW     │     │  REJECTED   │
                    └──────┬──────┘     └─────────────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
       ┌─────────────┐          ┌─────────────────┐
       │   FILLED    │          │PARTIALLY_FILLED │
       └─────────────┘          └────────┬────────┘
                                         │
                              ┌──────────┴──────────┐
                              ▼                     ▼
                       ┌─────────────┐       ┌─────────────┐
                       │   FILLED    │       │  CANCELLED  │
                       └─────────────┘       └─────────────┘

                              ┌─────────────┐
                              │   EXPIRED   │
                              └─────────────┘
```

### Status de Ordem

```rust
pub enum OrderStatus {
    /// Ordem criada mas não enviada
    Pending,

    /// Ordem aceita pela exchange
    New,

    /// Ordem parcialmente preenchida
    PartiallyFilled,

    /// Ordem totalmente preenchida
    Filled,

    /// Ordem cancelada
    Cancelled,

    /// Ordem rejeitada pela exchange
    Rejected,

    /// Ordem expirada (IOC, FOK, etc)
    Expired,
}
```

## Ordens Compostas

### OCO (One Cancels Other)

```rust
pub struct OcoOrder {
    /// Stop loss order
    pub stop_loss: OrderRequest,

    /// Take profit order
    pub take_profit: OrderRequest,
}

impl OrderService {
    pub async fn place_oco(&self, position: &Position, oco: OcoOrder) -> WorkerResult<(JobId, JobId)> {
        // Validar que ambas são reduce_only
        assert!(oco.stop_loss.reduce_only);
        assert!(oco.take_profit.reduce_only);

        // Validar que quantidade combina
        assert_eq!(oco.stop_loss.quantity, position.quantity);
        assert_eq!(oco.take_profit.quantity, position.quantity);

        // Enfileirar ambas
        let sl_job = self.place_order(oco.stop_loss).await?;
        let tp_job = self.place_order(oco.take_profit).await?;

        // Registrar link OCO para cancelamento automático
        self.oco_registry.register(&sl_job, &tp_job).await;

        Ok((sl_job, tp_job))
    }

    /// Chamado quando uma ordem OCO é preenchida
    pub async fn handle_oco_fill(&self, filled_order_id: &str) -> WorkerResult<()> {
        if let Some(other_order_id) = self.oco_registry.get_linked(filled_order_id).await {
            // Cancelar a outra ordem
            self.cancel_order(&other_order_id).await?;
        }

        Ok(())
    }
}
```

### Bracket Order

```rust
pub struct BracketOrder {
    /// Ordem de entrada
    pub entry: OrderRequest,

    /// Stop loss
    pub stop_loss_price: Decimal,

    /// Take profit
    pub take_profit_price: Decimal,
}

impl OrderService {
    pub async fn place_bracket_order(&self, bracket: BracketOrder) -> WorkerResult<JobId> {
        // 1. Enviar ordem de entrada
        let entry_job_id = self.place_order(bracket.entry.clone()).await?;

        // 2. Registrar para criar SL/TP quando entrada preencher
        self.bracket_registry.register(
            &entry_job_id,
            bracket.stop_loss_price,
            bracket.take_profit_price,
        ).await;

        Ok(entry_job_id)
    }

    /// Chamado quando ordem de entrada é preenchida
    pub async fn handle_bracket_fill(&self, entry_order: &Order) -> WorkerResult<()> {
        if let Some(bracket) = self.bracket_registry.get(&entry_order.id).await {
            // Criar ordens SL/TP
            self.place_stop_loss(entry_order, bracket.stop_loss_price).await?;
            self.place_take_profit(entry_order, bracket.take_profit_price).await?;
        }

        Ok(())
    }
}
```

## Sincronização de Estado

```rust
pub struct StateSyncService {
    gateway: Arc<dyn ExchangeGateway>,
    order_repo: Arc<dyn OrderRepository>,
    position_repo: Arc<dyn PositionRepository>,
    balance_repo: Arc<dyn BalanceRepository>,
}

impl StateSyncService {
    /// Sincroniza ordens abertas com a exchange
    pub async fn sync_orders(&self) -> WorkerResult<SyncResult> {
        let exchange_orders = self.gateway.get_open_orders(None).await?;
        let local_orders = self.order_repo.find_active().await?;

        let mut synced = 0;
        let mut cancelled = 0;

        // Atualizar ordens locais
        for local in &local_orders {
            let exchange_order = exchange_orders.iter()
                .find(|o| o.id == local.id || o.client_order_id == local.client_order_id);

            match exchange_order {
                Some(eo) if eo.status != local.status => {
                    // Atualizar status
                    let mut updated = local.clone();
                    updated.status = eo.status;
                    updated.filled_quantity = eo.filled_quantity;
                    updated.average_price = eo.average_price;
                    self.order_repo.save(&updated).await?;
                    synced += 1;
                }
                None if local.status == OrderStatus::New => {
                    // Ordem desapareceu - marcar como cancelada
                    let mut updated = local.clone();
                    updated.status = OrderStatus::Cancelled;
                    self.order_repo.save(&updated).await?;
                    cancelled += 1;
                }
                _ => {}
            }
        }

        Ok(SyncResult { synced, cancelled })
    }

    /// Sincroniza posições com a exchange
    pub async fn sync_positions(&self) -> WorkerResult<SyncResult> {
        let exchange_positions = self.gateway.get_positions().await?;
        let local_positions = self.position_repo.find_open().await?;

        let mut synced = 0;
        let mut closed = 0;

        for local in &local_positions {
            let exchange_pos = exchange_positions.iter()
                .find(|p| p.symbol == local.symbol && p.side == local.side);

            match exchange_pos {
                Some(ep) => {
                    // Atualizar posição
                    let mut updated = local.clone();
                    updated.quantity = ep.quantity;
                    updated.current_price = ep.current_price;
                    updated.unrealized_pnl = ep.unrealized_pnl;
                    updated.liquidation_price = ep.liquidation_price;
                    self.position_repo.save(&updated).await?;
                    synced += 1;
                }
                None => {
                    // Posição fechada na exchange
                    let mut updated = local.clone();
                    updated.status = PositionStatus::Closed;
                    updated.closed_at = Some(Utc::now());
                    updated.quantity = Decimal::ZERO;
                    self.position_repo.save(&updated).await?;
                    closed += 1;
                }
            }
        }

        Ok(SyncResult { synced, closed })
    }

    /// Sincroniza saldos
    pub async fn sync_balances(&self) -> WorkerResult<()> {
        let balances = self.gateway.get_balances().await?;

        for balance in balances {
            self.balance_repo.save(&balance).await?;
        }

        Ok(())
    }
}
```

---

**Próximo**: [Gestão de Risco](./risk-management.md)
