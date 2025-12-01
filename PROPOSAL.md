# RoboTrade - Proposta de Refatoração Arquitetural

**Status:** 🔄 Em Revisão
**Versão:** 1.0
**Data:** 2025-12-01
**Autores:** Equipe RoboTrade

---

## 📋 Sumário Executivo

Esta proposta apresenta um plano completo de refatoração do RoboTrade para:

1. **Separar lógica de negócio pura de IO** - Extrair cálculos e regras para módulos testáveis sem dependências externas
2. **Estabelecer boundaries claros entre camadas** - Domain, Application, Infrastructure seguindo Clean Architecture
3. **Atingir 80%+ de cobertura de testes** - Testes unitários determinísticos + integração end-to-end com Docker
4. **Implementar adapters concretos com feature flags** - Facilitar mock/real switching sem recompilação
5. **Criar CI/CD robusto** - GitHub Actions com fmt, clippy, testes, cobertura

**Prioridade:** 🔴 CRÍTICA - Sistema atual tem <1% de cobertura em lógica crítica financeira

---

## 🎯 Objetivos

### Objetivos Primários

- ✅ **Testabilidade:** 80%+ cobertura em business logic crítica (PnL, Risk, Reconciliation)
- ✅ **Isolamento:** Lógica pura sem IO, testável com mocks determinísticos
- ✅ **Clareza:** Boundaries explícitos entre Domain, Application, Infrastructure
- ✅ **Reprodutibilidade:** Testes E2E com Docker, fixtures JSON estáticos
- ✅ **Qualidade:** CI verde com clippy warnings como errors

### Objetivos Secundários

- 🔒 **Segurança:** API keys com `secrecy::Secret<T>`, nunca em logs
- 📊 **Observabilidade:** Structured tracing em todos os serviços
- 🔄 **Manutenibilidade:** Documentação clara de cada camada

---

## 📊 Análise do Estado Atual

### Métricas do Código

| Métrica | Valor | Status |
|---------|-------|--------|
| **Total de Linhas** | 7,651+ | ✅ |
| **Cobertura de Testes** | <1% (20 testes) | 🔴 **CRÍTICO** |
| **Crates** | 8 | ✅ |
| **Arquivos de Entidade** | 23 | ✅ |
| **Repositórios** | 19 | ✅ |
| **Implementações de Exchange** | 3 (Binance, Kraken, Paper) | ✅ |

### Problemas Identificados

#### 🔴 P0 - Críticos

1. **Testes Ausentes em Lógica Financeira**
   - `PnLCalculator` (contabilidade, cost basis, tax lots) - **0 testes**
   - `RiskManager` (limites de perda, alavancagem) - **0 testes**
   - `ReconciliationService` (auditoria, discrepâncias) - **0 testes**
   - **Risco:** Bugs podem causar perdas financeiras reais

2. **Lógica de Negócio Misturada com IO**
   ```rust
   // Exemplo: PositionManager
   pub async fn update_price(&self, position_id: PositionId, price: Decimal) {
       // Atualiza estado em memória
       // ❌ NÃO persiste no banco
       // ❌ NÃO testável sem mocks complexos
   }
   ```

3. **Múltiplas Fontes de Verdade Não Sincronizadas**
   ```
   AppState.positions (cache em memória)
     vs
   PositionManager.positions (HashMap)
     vs
   Database.positions (tabela)
     vs
   Exchange WebSocket (atualizações ao vivo)
   ```
   **Risco:** Divergências causam decisões incorretas

#### 🟡 P1 - Importantes

4. **API Keys em Texto Plano**
   ```rust
   // Atual
   pub struct BinanceFuturesClient {
       api_key: String,     // ❌ Pode vazar em logs
       api_secret: String,  // ❌ Pode vazar em logs
   }
   ```

5. **Modo Paper Trading Não Abstrato**
   - Lógica espalhada para selecionar paper vs real
   - Difícil testar com diferentes modos

6. **Informação de Erro Perdida em Boundaries**
   ```rust
   // Tauri commands convertem tudo para String
   pub struct CommandError {
       code: String,
       message: String,  // ❌ Perde tipo de erro
   }
   ```

---

## 🏗️ Arquitetura Proposta

### Visão de Camadas (Clean Architecture)

```
┌─────────────────────────────────────────────────────────────────┐
│                     PRESENTATION LAYER                           │
│                  (Tauri Frontend / Commands)                     │
│  - Tauri commands.rs (RPC handlers)                             │
│  - State management (AppState)                                  │
│  - DTOs for frontend communication                              │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Depends on
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                             │
│              (Use Cases / Business Workflows)                    │
│                                                                  │
│  Crate: robotrade-application (NEW)                             │
│  ├── use_cases/                                                 │
│  │   ├── execute_trade.rs      (Signal → Order → Position)     │
│  │   ├── calculate_pnl.rs      (Cost basis, realized/unrealized)│
│  │   ├── reconcile_balances.rs (Exchange vs Ledger)            │
│  │   ├── manage_position.rs    (Open/Close/Update)             │
│  │   └── assess_risk.rs        (Pre-trade validation)          │
│  │                                                              │
│  └── services/                                                  │
│      ├── trading_orchestrator.rs (Coordinates trading flow)     │
│      ├── market_data_service.rs  (Aggregates market data)      │
│      └── notification_service.rs (Sends alerts)                │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Depends on
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                      DOMAIN LAYER                                │
│                 (Pure Business Logic - NO IO)                    │
│                                                                  │
│  Crate: robotrade-core (REFACTORED)                             │
│  ├── entities/          (Domain models: Order, Position, etc.)  │
│  ├── value_objects/     (Symbol, Price, Quantity)              │
│  ├── domain_services/   (Pure business logic)                   │
│  │   ├── pnl/                                                   │
│  │   │   ├── calculator.rs    (PURE: cost basis logic)         │
│  │   │   ├── tax_lot.rs       (FIFO/LIFO/Average)              │
│  │   │   └── methods.rs       (Cost basis methods)             │
│  │   │                                                          │
│  │   ├── risk/                                                  │
│  │   │   ├── validator.rs     (PURE: risk rules)               │
│  │   │   ├── limits.rs        (Leverage, exposure limits)       │
│  │   │   └── circuit_breaker.rs (State machine logic)          │
│  │   │                                                          │
│  │   ├── position/                                              │
│  │   │   ├── manager.rs       (PURE: position logic)           │
│  │   │   ├── stop_loss.rs     (Trailing stop, break-even)      │
│  │   │   └── lifecycle.rs     (Open → Active → Closed)         │
│  │   │                                                          │
│  │   └── reconciliation/                                        │
│  │       ├── comparator.rs    (PURE: balance comparison)        │
│  │       ├── discrepancy.rs   (Classification logic)            │
│  │       └── resolution.rs    (Resolution strategies)           │
│  │                                                              │
│  ├── traits/            (Abstractions - ports)                  │
│  │   ├── repositories.rs      (Data access ports)              │
│  │   ├── exchanges.rs         (Exchange gateway port)          │
│  │   ├── market_data.rs       (Market data port)               │
│  │   └── notifications.rs     (Notification port)              │
│  │                                                              │
│  └── errors/            (Domain errors)                         │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Implemented by
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   INFRASTRUCTURE LAYER                           │
│              (Adapters - Concrete Implementations)               │
│                                                                  │
│  Crate: robotrade-infra (REFACTORED)                            │
│  ├── persistence/                                               │
│  │   ├── sqlite/                                                │
│  │   │   ├── repositories/    (19 repo implementations)         │
│  │   │   ├── schema.rs        (Database schema)                │
│  │   │   └── migrations/      (SQL migrations)                 │
│  │   │                                                          │
│  │   └── postgres/            (NEW - for production)            │
│  │       ├── repositories/    (Same traits, PG impl)            │
│  │       └── migrations/                                        │
│  │                                                              │
│  ├── config/                                                    │
│  │   ├── loader.rs            (TOML config loader)             │
│  │   └── validator.rs         (Config validation)              │
│  │                                                              │
│  └── observability/                                             │
│      ├── logging.rs           (Structured tracing)             │
│      └── metrics.rs           (Performance metrics)            │
│                                                                 │
│  Crate: robotrade-exchange-adapters (RENAMED)                   │
│  ├── binance/                                                   │
│  │   ├── client.rs            (REST client)                    │
│  │   ├── websocket.rs         (WS streams)                     │
│  │   ├── signer.rs            (HMAC-SHA256)                    │
│  │   └── models.rs            (API models)                     │
│  │                                                              │
│  ├── kraken/                  (Same structure)                  │
│  ├── paper/                   (Simulated trading)               │
│  │                                                              │
│  └── mock/                    (NEW - deterministic mocks)       │
│      ├── fixed_price.rs       (Static prices for tests)         │
│      ├── fixture_loader.rs    (Load JSON fixtures)              │
│      └── fixtures/            (JSON test data)                  │
│                                                                 │
│  Crate: robotrade-market-data-adapters (RENAMED)                │
│  └── providers/                                                 │
│      ├── fear_greed.rs        (Alternative.me API)             │
│      └── mock_provider.rs     (NEW - for tests)                │
└─────────────────────────────────────────────────────────────────┘
```

### Fluxo de Dependências

```
┌─────────────┐
│ Presentation│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Application │
└──────┬──────┘
       │
       ▼
┌─────────────┐       ┌──────────────────┐
│   Domain    │ ◄──── │  Infrastructure  │
│  (Traits)   │       │  (Implements)    │
└─────────────┘       └──────────────────┘
```

**Regra Fundamental:** Infrastructure implementa traits definidos em Domain, mas Domain nunca importa Infrastructure.

---

## 🔧 Refatorações Detalhadas

### 1. PnLCalculator - Extração de Lógica Pura

#### Estado Atual (❌)
```rust
// Arquivo: crates/trading_worker/src/pnl_calculator.rs
pub struct PnLCalculator {
    // ❌ Sem persistência
    tax_lots: HashMap<String, Vec<TaxLot>>,
}

impl PnLCalculator {
    pub fn calculate_realized_pnl(&self, trade: &Trade) -> RealizedPnL {
        // ❌ Lógica misturada, sem teste
        // ❌ Não persiste tax lots
    }
}
```

#### Estado Proposto (✅)

**Domain (Pure Logic):**
```rust
// Arquivo: crates/core/src/domain_services/pnl/calculator.rs

/// Calculadora PURA de P&L - sem IO, completamente testável
pub struct PnLCalculator;

impl PnLCalculator {
    /// Calcula P&L realizado usando método FIFO
    ///
    /// # Arguments
    /// * `existing_lots` - Tax lots existentes (do repositório)
    /// * `trade` - Trade sendo fechado
    ///
    /// # Returns
    /// * `RealizedPnL` - P&L calculado
    /// * `Vec<TaxLot>` - Tax lots atualizados (para persistir)
    pub fn calculate_fifo(
        existing_lots: &[TaxLot],
        trade: &Trade,
    ) -> Result<(RealizedPnL, Vec<TaxLot>), PnLError> {
        // PURE FUNCTION - completamente testável
        // Sem banco, sem HTTP, sem filesystem

        let mut lots = existing_lots.to_vec();
        let mut remaining_qty = trade.quantity;
        let mut total_cost = Decimal::ZERO;
        let mut proceeds = trade.quantity * trade.exit_price;

        // FIFO: consome os lotes mais antigos primeiro
        lots.sort_by_key(|lot| lot.acquired_at);

        let mut updated_lots = Vec::new();

        for lot in lots.iter_mut() {
            if remaining_qty == Decimal::ZERO {
                updated_lots.push(lot.clone());
                continue;
            }

            let qty_to_close = lot.remaining_quantity.min(remaining_qty);
            total_cost += qty_to_close * lot.cost_per_unit;
            lot.remaining_quantity -= qty_to_close;
            remaining_qty -= qty_to_close;

            if lot.remaining_quantity > Decimal::ZERO {
                updated_lots.push(lot.clone());
            } else {
                // Lote totalmente consumido
                lot.is_closed = true;
                updated_lots.push(lot.clone());
            }
        }

        let realized_pnl = proceeds - total_cost - trade.commission;

        Ok((
            RealizedPnL {
                amount: realized_pnl,
                cost_basis: total_cost,
                proceeds,
                commission: trade.commission,
                method: CostBasisMethod::FIFO,
                lots_closed: updated_lots.iter().filter(|l| l.is_closed).count(),
            },
            updated_lots,
        ))
    }

    /// Calcula P&L não realizado
    pub fn calculate_unrealized(
        open_lots: &[TaxLot],
        current_price: Decimal,
    ) -> UnrealizedPnL {
        // PURE FUNCTION
        let mut total_unrealized = Decimal::ZERO;

        for lot in open_lots {
            let current_value = lot.remaining_quantity * current_price;
            let cost = lot.remaining_quantity * lot.cost_per_unit;
            total_unrealized += current_value - cost;
        }

        UnrealizedPnL {
            amount: total_unrealized,
            open_lots: open_lots.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fifo_single_lot_full_close() {
        // PURE TEST - sem setup de banco, sem mocks
        let lots = vec![
            TaxLot {
                id: "lot1".to_string(),
                symbol: "BTCUSDT".to_string(),
                side: PositionSide::Long,
                acquired_quantity: dec!(1.0),
                remaining_quantity: dec!(1.0),
                cost_per_unit: dec!(50000.0),
                total_cost: dec!(50000.0),
                acquired_at: Utc::now(),
                acquisition_type: AcquisitionType::Trade,
                trade_id: "trade1".to_string(),
                is_closed: false,
            }
        ];

        let trade = Trade {
            quantity: dec!(1.0),
            exit_price: dec!(55000.0),
            commission: dec!(50.0),
            // ...
        };

        let (pnl, updated_lots) = PnLCalculator::calculate_fifo(&lots, &trade)
            .expect("Should calculate PnL");

        assert_eq!(pnl.amount, dec!(4950.0)); // 55000 - 50000 - 50
        assert_eq!(pnl.cost_basis, dec!(50000.0));
        assert_eq!(pnl.proceeds, dec!(55000.0));
        assert_eq!(updated_lots.len(), 1);
        assert!(updated_lots[0].is_closed);
    }

    #[test]
    fn test_fifo_multiple_lots_partial_close() {
        // Teste de FIFO com múltiplos lotes
        // ...
    }

    #[test]
    fn test_fifo_handles_zero_quantity() {
        // Teste de edge case
        // ...
    }
}
```

**Application Layer (Orchestration):**
```rust
// Arquivo: crates/application/src/use_cases/calculate_pnl.rs

pub struct CalculatePnLUseCase<R: TaxLotRepository> {
    tax_lot_repo: Arc<R>,
}

impl<R: TaxLotRepository> CalculatePnLUseCase<R> {
    pub async fn execute(&self, trade: Trade) -> Result<RealizedPnL, AppError> {
        // 1. Buscar tax lots do repositório (IO)
        let existing_lots = self.tax_lot_repo
            .find_by_symbol(&trade.symbol)
            .await?;

        // 2. Chamar lógica pura (sem IO)
        let (pnl, updated_lots) = PnLCalculator::calculate_fifo(&existing_lots, &trade)?;

        // 3. Persistir tax lots atualizados (IO)
        self.tax_lot_repo.update_batch(&updated_lots).await?;

        // 4. Retornar resultado
        Ok(pnl)
    }
}
```

**Vantagens:**
- ✅ `PnLCalculator` é 100% puro - testável sem banco/HTTP
- ✅ Testes determinísticos, rápidos (<1ms cada)
- ✅ Use case orquestra IO, separado da lógica
- ✅ Fácil testar com diferentes cost basis methods (FIFO, LIFO, Average)

---

### 2. RiskManager - Validação Pura

#### Estado Atual (❌)
```rust
// Arquivo: crates/trading_worker/src/risk.rs
pub struct RiskManager {
    config: RiskConfig,
    // ❌ Estado em memória não sincronizado com banco
    current_daily_loss: Arc<RwLock<Decimal>>,
    open_positions: Arc<RwLock<Vec<Position>>>,
}
```

#### Estado Proposto (✅)

**Domain (Pure Logic):**
```rust
// Arquivo: crates/core/src/domain_services/risk/validator.rs

/// Validador PURO de risco - sem IO
pub struct RiskValidator;

impl RiskValidator {
    /// Valida se ordem é aceitável com base em limites
    ///
    /// # Arguments
    /// * `order_request` - Ordem proposta
    /// * `current_state` - Estado atual (posições, perdas, etc.)
    /// * `limits` - Limites configurados
    ///
    /// # Returns
    /// * `RiskCheckResult` - Aprovado, Rejeitado ou Aprovado com Ajuste
    pub fn validate_order(
        order_request: &OrderRequest,
        current_state: &RiskState,
        limits: &RiskLimits,
    ) -> RiskCheckResult {
        // PURE FUNCTION - totalmente testável

        // 1. Verificar perda diária
        if current_state.daily_loss.abs() >= limits.max_daily_loss {
            return RiskCheckResult::Rejected(
                RiskRejectionReason::DailyLossLimitReached {
                    current_loss: current_state.daily_loss,
                    limit: limits.max_daily_loss,
                }
            );
        }

        // 2. Verificar número máximo de posições
        if current_state.open_positions_count >= limits.max_positions {
            return RiskCheckResult::Rejected(
                RiskRejectionReason::MaxPositionsReached {
                    current: current_state.open_positions_count,
                    max: limits.max_positions,
                }
            );
        }

        // 3. Verificar exposição total
        let order_notional = order_request.quantity * order_request.price * order_request.leverage;
        let new_exposure = current_state.total_exposure + order_notional;

        if new_exposure > limits.max_total_exposure {
            return RiskCheckResult::Rejected(
                RiskRejectionReason::MaxExposureExceeded {
                    current: new_exposure,
                    max: limits.max_total_exposure,
                }
            );
        }

        // 4. Verificar concentração
        let symbol_exposure = current_state.exposure_by_symbol
            .get(&order_request.symbol)
            .copied()
            .unwrap_or(Decimal::ZERO) + order_notional;

        let concentration_pct = (symbol_exposure / current_state.total_capital) * dec!(100);

        if concentration_pct > limits.max_concentration_pct {
            return RiskCheckResult::Rejected(
                RiskRejectionReason::ConcentrationTooHigh {
                    symbol: order_request.symbol.clone(),
                    percentage: concentration_pct,
                }
            );
        }

        // 5. Verificar alavancagem
        if order_request.leverage > limits.max_leverage {
            return RiskCheckResult::Rejected(
                RiskRejectionReason::LeverageTooHigh {
                    requested: order_request.leverage,
                    max: limits.max_leverage,
                }
            );
        }

        RiskCheckResult::Approved
    }
}

/// Estado atual do sistema para validação
#[derive(Clone)]
pub struct RiskState {
    pub daily_loss: Decimal,
    pub total_exposure: Decimal,
    pub total_capital: Decimal,
    pub open_positions_count: usize,
    pub exposure_by_symbol: HashMap<String, Decimal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_order_exceeding_daily_loss() {
        let state = RiskState {
            daily_loss: dec!(-1000.0),  // Já perdeu $1000
            total_exposure: dec!(10000.0),
            total_capital: dec!(100000.0),
            open_positions_count: 2,
            exposure_by_symbol: HashMap::new(),
        };

        let limits = RiskLimits {
            max_daily_loss: dec!(1000.0),  // Limite: $1000
            max_total_exposure: dec!(50000.0),
            max_positions: 10,
            max_leverage: 10,
            max_concentration_pct: dec!(20.0),
        };

        let order = OrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Long,
            quantity: dec!(0.1),
            price: dec!(50000.0),
            leverage: 5,
            // ...
        };

        let result = RiskValidator::validate_order(&order, &state, &limits);

        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached { .. })
        ));
    }

    #[test]
    fn test_approves_order_within_limits() {
        // ...
    }

    #[test]
    fn test_rejects_excessive_concentration() {
        // ...
    }
}
```

**Application Layer:**
```rust
// Arquivo: crates/application/src/use_cases/assess_risk.rs

pub struct AssessRiskUseCase<PR: PositionRepository, TR: TradeRepository> {
    position_repo: Arc<PR>,
    trade_repo: Arc<TR>,
}

impl<PR, TR> AssessRiskUseCase<PR, TR>
where
    PR: PositionRepository,
    TR: TradeRepository,
{
    pub async fn execute(
        &self,
        order_request: OrderRequest,
        limits: RiskLimits,
    ) -> Result<RiskCheckResult, AppError> {
        // 1. Buscar estado atual do repositório (IO)
        let open_positions = self.position_repo.find_open().await?;
        let today_trades = self.trade_repo.find_today().await?;

        // 2. Calcular estado atual
        let state = self.calculate_current_state(&open_positions, &today_trades);

        // 3. Chamar lógica pura de validação (sem IO)
        let result = RiskValidator::validate_order(&order_request, &state, &limits);

        Ok(result)
    }

    fn calculate_current_state(
        &self,
        positions: &[Position],
        trades: &[Trade],
    ) -> RiskState {
        // Calcula daily loss, exposure, etc.
        // ...
    }
}
```

**Vantagens:**
- ✅ Validação pura, testável com diferentes cenários
- ✅ Testes cobrem todos os rejection reasons
- ✅ Estado explícito, não oculto em RwLock
- ✅ Fácil adicionar novos checks de risco

---

### 3. ReconciliationService - Comparação Pura

#### Estado Proposto (✅)

**Domain (Pure Logic):**
```rust
// Arquivo: crates/core/src/domain_services/reconciliation/comparator.rs

/// Comparador PURO de balanços - sem IO
pub struct BalanceComparator;

impl BalanceComparator {
    /// Compara balanços do ledger vs exchange
    ///
    /// # Returns
    /// Lista de discrepâncias encontradas
    pub fn compare(
        ledger_balances: &[BalanceSnapshot],
        exchange_balances: &[BalanceSnapshot],
        config: &ReconciliationConfig,
    ) -> Vec<Discrepancy> {
        // PURE FUNCTION

        let mut discrepancies = Vec::new();

        // Criar mapa de balanços para comparação eficiente
        let ledger_map: HashMap<&str, &BalanceSnapshot> = ledger_balances
            .iter()
            .map(|b| (b.asset.as_str(), b))
            .collect();

        let exchange_map: HashMap<&str, &BalanceSnapshot> = exchange_balances
            .iter()
            .map(|b| (b.asset.as_str(), b))
            .collect();

        // 1. Verificar assets no ledger
        for (asset, ledger_balance) in ledger_map.iter() {
            match exchange_map.get(asset) {
                Some(exchange_balance) => {
                    // Asset existe em ambos - comparar valores
                    let diff = ledger_balance.total - exchange_balance.total;
                    let diff_abs = diff.abs();

                    // Verificar se diferença está dentro da tolerância
                    let tolerance = ledger_balance.total * config.tolerance_pct / dec!(100);

                    if diff_abs > tolerance && diff_abs > config.min_difference {
                        let discrepancy_type = if diff > Decimal::ZERO {
                            DiscrepancyType::LedgerHigher
                        } else {
                            DiscrepancyType::ExchangeHigher
                        };

                        discrepancies.push(Discrepancy {
                            asset: asset.to_string(),
                            discrepancy_type,
                            ledger_balance: ledger_balance.total,
                            exchange_balance: exchange_balance.total,
                            difference: diff_abs,
                            difference_pct: (diff_abs / ledger_balance.total) * dec!(100),
                        });
                    }
                }
                None => {
                    // Asset no ledger mas não na exchange
                    if ledger_balance.total > config.dust_threshold {
                        discrepancies.push(Discrepancy {
                            asset: asset.to_string(),
                            discrepancy_type: DiscrepancyType::MissingOnExchange,
                            ledger_balance: ledger_balance.total,
                            exchange_balance: Decimal::ZERO,
                            difference: ledger_balance.total,
                            difference_pct: dec!(100.0),
                        });
                    }
                }
            }
        }

        // 2. Verificar assets na exchange mas não no ledger
        for (asset, exchange_balance) in exchange_map.iter() {
            if !ledger_map.contains_key(asset) {
                if exchange_balance.total > config.dust_threshold {
                    discrepancies.push(Discrepancy {
                        asset: asset.to_string(),
                        discrepancy_type: DiscrepancyType::MissingInLedger,
                        ledger_balance: Decimal::ZERO,
                        exchange_balance: exchange_balance.total,
                        difference: exchange_balance.total,
                        difference_pct: dec!(100.0),
                    });
                }
            }
        }

        discrepancies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_discrepancies_when_balanced() {
        let ledger = vec![
            BalanceSnapshot {
                asset: "USDT".to_string(),
                total: dec!(10000.0),
                // ...
            },
            BalanceSnapshot {
                asset: "BTC".to_string(),
                total: dec!(1.5),
                // ...
            },
        ];

        let exchange = ledger.clone();

        let config = ReconciliationConfig {
            tolerance_pct: dec!(0.1),
            min_difference: dec!(0.01),
            dust_threshold: dec!(0.001),
            auto_resolve_dust: false,
        };

        let discrepancies = BalanceComparator::compare(&ledger, &exchange, &config);

        assert_eq!(discrepancies.len(), 0);
    }

    #[test]
    fn test_detects_ledger_higher() {
        let ledger = vec![
            BalanceSnapshot {
                asset: "USDT".to_string(),
                total: dec!(10000.0),
                // ...
            },
        ];

        let exchange = vec![
            BalanceSnapshot {
                asset: "USDT".to_string(),
                total: dec!(9900.0),  // $100 a menos
                // ...
            },
        ];

        let config = ReconciliationConfig {
            tolerance_pct: dec!(0.1),  // 0.1% = $10 tolerância
            min_difference: dec!(0.01),
            dust_threshold: dec!(0.001),
            auto_resolve_dust: false,
        };

        let discrepancies = BalanceComparator::compare(&ledger, &exchange, &config);

        assert_eq!(discrepancies.len(), 1);
        assert_eq!(discrepancies[0].asset, "USDT");
        assert!(matches!(
            discrepancies[0].discrepancy_type,
            DiscrepancyType::LedgerHigher
        ));
        assert_eq!(discrepancies[0].difference, dec!(100.0));
    }

    #[test]
    fn test_ignores_differences_within_tolerance() {
        // Diferença de $5 com tolerância de 0.1% = $10
        // Deve ser ignorado
        // ...
    }

    #[test]
    fn test_detects_missing_on_exchange() {
        // ...
    }

    #[test]
    fn test_ignores_dust() {
        // ...
    }
}
```

---

### 4. Infrastructure - Adapters com Feature Flags

#### Estrutura de Features

**Cargo.toml:**
```toml
[package]
name = "robotrade-exchange-adapters"

[features]
default = ["binance", "kraken"]

# Exchanges reais
binance = []
kraken = []
paper = []

# Mock para testes
mock = []

# Determina se usa real ou mock
use-real = ["binance", "kraken"]
use-mock = ["mock"]

[dependencies]
robotrade-core = { path = "../core" }

# Conditional dependencies
reqwest = { workspace = true, optional = true }
tokio-tungstenite = { workspace = true, optional = true }

[dev-dependencies]
# Para testes de integração
wiremock = { workspace = true }
```

#### Mock Exchange Implementation

```rust
// Arquivo: crates/exchange_adapters/src/mock/client.rs

#[cfg(feature = "mock")]
pub struct MockExchangeClient {
    config: MockConfig,
    fixture_loader: FixtureLoader,
}

#[cfg(feature = "mock")]
impl MockExchangeClient {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            fixture_loader: FixtureLoader::new("tests/fixtures"),
        }
    }

    /// Cria um mock determinístico para testes
    pub fn deterministic() -> Self {
        Self {
            config: MockConfig {
                fixed_price: Some(dec!(50000.0)),
                fixed_latency_ms: Some(10),
                fail_rate: 0.0,
            },
            fixture_loader: FixtureLoader::new("tests/fixtures"),
        }
    }
}

#[cfg(feature = "mock")]
#[async_trait]
impl ExchangeGateway for MockExchangeClient {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Mock
    }

    fn is_paper_trading(&self) -> bool {
        true
    }

    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        // Simula latência configurável
        if let Some(latency_ms) = self.config.fixed_latency_ms {
            tokio::time::sleep(Duration::from_millis(latency_ms)).await;
        }

        // Simula falhas aleatórias (para testar retry logic)
        if rand::random::<f64>() < self.config.fail_rate {
            return Err(ExchangeError::NetworkError {
                message: "Simulated network error".to_string(),
            });
        }

        // Retorna ordem preenchida com preço fixo
        let price = self.config.fixed_price.unwrap_or(dec!(50000.0));

        Ok(Order {
            id: OrderId::new(),
            client_order_id: request.client_order_id,
            exchange: ExchangeId::Mock,
            symbol: request.symbol,
            side: request.side,
            order_type: OrderType::Market,
            status: OrderStatus::Filled,
            quantity: request.quantity,
            filled_quantity: request.quantity,
            avg_price: price,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            // ...
        })
    }

    async fn get_positions(&self) -> ExchangeResult<Vec<Position>> {
        // Carrega posições de fixture JSON
        self.fixture_loader.load_positions()
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        // Carrega balanços de fixture JSON
        self.fixture_loader.load_balances()
    }

    // ... outras implementações determinísticas
}

/// Loader de fixtures JSON estáticos
pub struct FixtureLoader {
    base_path: PathBuf,
}

impl FixtureLoader {
    pub fn load_positions(&self) -> ExchangeResult<Vec<Position>> {
        let path = self.base_path.join("positions.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ExchangeError::Internal {
                message: format!("Failed to load fixture: {}", e),
            })?;

        serde_json::from_str(&content)
            .map_err(|e| ExchangeError::Internal {
                message: format!("Failed to parse fixture: {}", e),
            })
    }

    // Similar para balances, orders, trades...
}
```

**Fixtures JSON:**
```json
// tests/fixtures/positions.json
[
  {
    "id": "pos-1",
    "exchange": "mock",
    "symbol": "BTCUSDT",
    "side": "long",
    "size": "0.5",
    "entry_price": "50000.0",
    "current_price": "52000.0",
    "unrealized_pnl": "1000.0",
    "leverage": 5,
    "margin": "5000.0",
    "liquidation_price": "45000.0",
    "status": "open",
    "opened_at": "2025-01-01T00:00:00Z"
  }
]
```

**Usage no código:**
```rust
// Em testes unitários
#[cfg(test)]
mod tests {
    use robotrade_exchange_adapters::mock::MockExchangeClient;

    #[tokio::test]
    async fn test_order_flow_with_mock() {
        let exchange = MockExchangeClient::deterministic();

        let order_request = OrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Long,
            quantity: dec!(0.1),
            // ...
        };

        let order = exchange.submit_order(order_request).await.unwrap();

        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.avg_price, dec!(50000.0));
    }
}

// Em testes de integração
#[cfg(feature = "use-real")]
fn create_exchange_client() -> Arc<dyn ExchangeGateway> {
    Arc::new(BinanceFuturesClient::new(...))
}

#[cfg(feature = "use-mock")]
fn create_exchange_client() -> Arc<dyn ExchangeGateway> {
    Arc::new(MockExchangeClient::deterministic())
}
```

---

### 5. Testes de Integração End-to-End

#### Estrutura

```
integration_tests/
├── Cargo.toml
├── tests/
│   ├── common/
│   │   ├── mod.rs              (Setup comum)
│   │   ├── docker.rs           (Testcontainers)
│   │   └── fixtures.rs         (Dados de teste)
│   │
│   ├── e2e_order_flow.rs       (Fluxo completo: signal → order → position)
│   ├── e2e_pnl_calculation.rs  (P&L com múltiplos trades)
│   ├── e2e_reconciliation.rs   (Reconciliação completa)
│   └── e2e_risk_validation.rs  (Validação de risco)
│
└── fixtures/
    ├── binance_responses.json   (Respostas mockadas da Binance API)
    ├── kraken_responses.json    (Respostas mockadas da Kraken API)
    └── market_data.json         (Dados de mercado)
```

**Cargo.toml:**
```toml
[package]
name = "integration_tests"
edition = "2021"
publish = false

[dependencies]
robotrade-core = { path = "../crates/core" }
robotrade-application = { path = "../crates/application" }
robotrade-infra = { path = "../crates/infra" }
robotrade-exchange-adapters = { path = "../crates/exchange_adapters", features = ["mock"] }

tokio = { workspace = true }
sqlx = { workspace = true }
rust_decimal = { workspace = true }
rust_decimal_macros = { workspace = true }
chrono = { workspace = true }

# Testing
testcontainers = "0.15"
wiremock = { workspace = true }
tempfile = { workspace = true }
```

**Teste E2E Completo:**
```rust
// integration_tests/tests/e2e_order_flow.rs

use integration_tests::common::*;
use robotrade_core::*;
use robotrade_application::*;
use robotrade_exchange_adapters::mock::MockExchangeClient;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_complete_order_flow() {
    // Setup: Database + Exchange + Repositórios
    let test_env = TestEnvironment::new().await;

    // 1. Criar signal
    let signal = Signal {
        id: SignalId::new(),
        strategy_id: "fear-greed-v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        signal_type: SignalType::Long,
        strength: SignalStrength::Strong,
        entry_price: dec!(50000.0),
        stop_loss: Some(dec!(48000.0)),
        take_profit: Some(dec!(55000.0)),
        quantity: dec!(0.1),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        status: SignalStatus::Active,
        // ...
    };

    test_env.signal_repo.save(&signal).await.unwrap();

    // 2. Processar signal → order (via JobQueue)
    let execute_trade_uc = ExecuteTradeUseCase::new(
        test_env.exchange_client.clone(),
        test_env.signal_repo.clone(),
        test_env.order_repo.clone(),
        test_env.position_repo.clone(),
        test_env.ledger_repo.clone(),
    );

    let result = execute_trade_uc.execute(signal.id.clone()).await.unwrap();

    // 3. Verificar order foi criada
    let order = test_env.order_repo
        .find_by_id(&result.order_id)
        .await
        .unwrap()
        .expect("Order should exist");

    assert_eq!(order.status, OrderStatus::Filled);
    assert_eq!(order.symbol, "BTCUSDT");
    assert_eq!(order.filled_quantity, dec!(0.1));

    // 4. Verificar position foi aberta
    let position = test_env.position_repo
        .find_by_symbol("BTCUSDT")
        .await
        .unwrap()
        .expect("Position should exist");

    assert_eq!(position.side, PositionSide::Long);
    assert_eq!(position.size, dec!(0.1));
    assert_eq!(position.entry_price, dec!(50000.0));

    // 5. Verificar ledger entry foi criada
    let ledger_entries = test_env.ledger_repo
        .find_by_reference_id(&order.id.to_string())
        .await
        .unwrap();

    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].entry_type, LedgerEntryType::TradePnL);

    // 6. Simular atualização de preço e fechar posição
    let close_signal = Signal {
        id: SignalId::new(),
        strategy_id: "fear-greed-v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        signal_type: SignalType::Close,  // Signal de fechamento
        strength: SignalStrength::Strong,
        entry_price: dec!(55000.0),  // Take profit atingido
        quantity: dec!(0.1),
        created_at: Utc::now(),
        status: SignalStatus::Active,
        // ...
    };

    let close_result = execute_trade_uc.execute(close_signal.id).await.unwrap();

    // 7. Verificar position foi fechada
    let closed_position = test_env.position_repo
        .find_by_id(&position.id)
        .await
        .unwrap()
        .expect("Position should exist");

    assert_eq!(closed_position.status, PositionStatus::Closed);

    // 8. Verificar trade foi criada com P&L
    let trade = test_env.trade_repo
        .find_by_position_id(&position.id)
        .await
        .unwrap()
        .expect("Trade should exist");

    assert_eq!(trade.direction, TradeDirection::Long);
    assert_eq!(trade.entry_price, dec!(50000.0));
    assert_eq!(trade.exit_price, dec!(55000.0));

    // 9. Verificar P&L calculado corretamente
    let expected_pnl = (dec!(55000.0) - dec!(50000.0)) * dec!(0.1); // $500
    assert_eq!(trade.realized_pnl, expected_pnl);

    // 10. Verificar ledger entry de P&L
    let pnl_entries = test_env.ledger_repo
        .find_by_entry_type(LedgerEntryType::TradePnL)
        .await
        .unwrap();

    assert!(pnl_entries.iter().any(|e| e.amount == expected_pnl));
}

#[tokio::test]
async fn test_order_rejected_by_risk_manager() {
    let test_env = TestEnvironment::new().await;

    // 1. Configurar limite de perda diária baixo
    let risk_limits = RiskLimits {
        max_daily_loss: dec!(100.0),  // Limite: $100
        max_total_exposure: dec!(50000.0),
        max_positions: 10,
        max_leverage: 10,
        max_concentration_pct: dec!(20.0),
    };

    // 2. Criar trade que causou perda de $150
    let losing_trade = Trade {
        id: TradeId::new(),
        symbol: "BTCUSDT".to_string(),
        direction: TradeDirection::Long,
        entry_price: dec!(50000.0),
        exit_price: dec!(49850.0),  // Perda de $150
        quantity: dec!(1.0),
        realized_pnl: dec!(-150.0),
        // ...
    };

    test_env.trade_repo.save(&losing_trade).await.unwrap();

    // 3. Tentar criar nova ordem (deve ser rejeitada)
    let signal = Signal {
        id: SignalId::new(),
        symbol: "ETHUSDT".to_string(),
        signal_type: SignalType::Long,
        entry_price: dec!(3000.0),
        quantity: dec!(1.0),
        // ...
    };

    let assess_risk_uc = AssessRiskUseCase::new(
        test_env.position_repo.clone(),
        test_env.trade_repo.clone(),
    );

    let risk_result = assess_risk_uc.execute(
        OrderRequest::from_signal(&signal),
        risk_limits,
    ).await.unwrap();

    // 4. Verificar rejeição
    assert!(matches!(
        risk_result,
        RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached { .. })
    ));
}

#[tokio::test]
async fn test_reconciliation_detects_discrepancy() {
    let test_env = TestEnvironment::new().await;

    // 1. Popular ledger com balanço de $10,000
    test_env.ledger_repo.add_entry(LedgerEntry {
        id: "entry-1".to_string(),
        entry_type: LedgerEntryType::Deposit,
        asset: "USDT".to_string(),
        amount: dec!(10000.0),
        balance_after: dec!(10000.0),
        // ...
    }).await.unwrap();

    // 2. Mock exchange retorna balanço de $9,900 (faltam $100)
    test_env.exchange_client.set_balance("USDT", dec!(9900.0));

    // 3. Executar reconciliação
    let reconcile_uc = ReconcileBalancesUseCase::new(
        test_env.ledger_repo.clone(),
        test_env.exchange_client.clone(),
        test_env.reconciliation_repo.clone(),
    );

    let snapshot = reconcile_uc.execute(
        ExchangeId::Mock,
        ReconciliationConfig {
            tolerance_pct: dec!(0.1),  // 0.1% = $10
            min_difference: dec!(1.0),
            dust_threshold: dec!(0.01),
            auto_resolve_dust: false,
        },
    ).await.unwrap();

    // 4. Verificar discrepância detectada
    assert_eq!(snapshot.discrepancies.len(), 1);
    assert_eq!(snapshot.discrepancies[0].asset, "USDT");
    assert_eq!(snapshot.discrepancies[0].difference, dec!(100.0));
    assert!(matches!(
        snapshot.discrepancies[0].discrepancy_type,
        DiscrepancyType::LedgerHigher
    ));

    // 5. Verificar snapshot foi persistido
    let saved_snapshot = test_env.reconciliation_repo
        .find_by_id(&snapshot.id)
        .await
        .unwrap()
        .expect("Snapshot should be saved");

    assert_eq!(saved_snapshot.status, ReconciliationStatus::DiscrepancyFound);
}
```

**Test Environment Setup:**
```rust
// integration_tests/tests/common/mod.rs

use testcontainers::{clients::Cli, Container};
use testcontainers::images::postgres::Postgres;
use sqlx::PgPool;

pub struct TestEnvironment {
    pub db_pool: PgPool,
    pub exchange_client: Arc<MockExchangeClient>,
    pub signal_repo: Arc<SqliteSignalRepository>,
    pub order_repo: Arc<SqliteOrderRepository>,
    pub position_repo: Arc<SqlitePositionRepository>,
    pub trade_repo: Arc<SqliteTradeRepository>,
    pub ledger_repo: Arc<SqliteLedgerRepository>,
    pub reconciliation_repo: Arc<SqliteReconciliationRepository>,
    _container: Container<'static, Postgres>,  // Mantém container vivo
}

impl TestEnvironment {
    pub async fn new() -> Self {
        // 1. Iniciar Postgres via Testcontainers
        let docker = Cli::default();
        let container = docker.run(Postgres::default());
        let port = container.get_host_port_ipv4(5432);

        let db_url = format!(
            "postgres://postgres:postgres@localhost:{}/test",
            port
        );

        // 2. Conectar e rodar migrações
        let pool = PgPool::connect(&db_url).await.unwrap();
        sqlx::migrate!("../crates/infra/migrations")
            .run(&pool)
            .await
            .unwrap();

        // 3. Criar repositórios
        let signal_repo = Arc::new(SqliteSignalRepository::new(pool.clone()));
        let order_repo = Arc::new(SqliteOrderRepository::new(pool.clone()));
        let position_repo = Arc::new(SqlitePositionRepository::new(pool.clone()));
        let trade_repo = Arc::new(SqliteTradeRepository::new(pool.clone()));
        let ledger_repo = Arc::new(SqliteLedgerRepository::new(pool.clone()));
        let reconciliation_repo = Arc::new(SqliteReconciliationRepository::new(pool.clone()));

        // 4. Criar mock exchange
        let exchange_client = Arc::new(MockExchangeClient::deterministic());

        Self {
            db_pool: pool,
            exchange_client,
            signal_repo,
            order_repo,
            position_repo,
            trade_repo,
            ledger_repo,
            reconciliation_repo,
            _container: container,
        }
    }
}
```

**Script de Execução:**
```bash
#!/bin/bash
# scripts/test_e2e.sh

set -e

echo "🚀 Running End-to-End Integration Tests"
echo "========================================"

# 1. Verificar Docker está rodando
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker não está rodando. Por favor inicie o Docker."
    exit 1
fi

# 2. Build do projeto
echo "📦 Building project..."
cargo build --release

# 3. Rodar testes de integração
echo "🧪 Running integration tests..."
cd integration_tests
cargo test --release -- --test-threads=1 --nocapture

# 4. Gerar relatório de cobertura
echo "📊 Generating coverage report..."
cargo llvm-cov --html --open

echo "✅ All tests passed!"
```

---

## 🔄 Plano de Migração Incremental

### Fase 1: Foundation (Semana 1-2)

**Objetivo:** Setup de infra de testes + extrair PnLCalculator

#### Commits:

```bash
# Commit 1: Setup testcontainers e estrutura de testes
git checkout -b feature/testing-infrastructure
mkdir integration_tests
# ... criar estrutura ...
git add .
git commit -m "feat: add integration testing infrastructure with testcontainers"

# Commit 2: Adicionar mock exchange client
# ... implementar MockExchangeClient ...
git add .
git commit -m "feat: add deterministic mock exchange client for testing"

# Commit 3: Extrair PnLCalculator para domain_services
# ... refatorar PnLCalculator ...
git add .
git commit -m "refactor: extract PnLCalculator pure logic to domain_services"

# Commit 4: Escrever testes unitários para PnLCalculator
# ... escrever 20+ testes ...
git add .
git commit -m "test: add comprehensive unit tests for PnLCalculator (FIFO, LIFO, Average)"

# Commit 5: Implementar CalculatePnLUseCase
# ... criar use case com repositórios ...
git add .
git commit -m "feat: implement CalculatePnLUseCase with repository persistence"
```

**Critérios de Aceite:**
- ✅ PnLCalculator 100% puro (sem IO)
- ✅ 80%+ cobertura em testes unitários de PnL
- ✅ MockExchangeClient funcionando
- ✅ Testcontainers configurado e rodando

---

### Fase 2: Risk & Position Management (Semana 3-4)

**Objetivo:** Extrair RiskValidator e PositionManager

#### Commits:

```bash
# Commit 6: Extrair RiskValidator
git checkout -b feature/risk-validation
# ... refatorar RiskValidator ...
git add .
git commit -m "refactor: extract RiskValidator pure logic to domain_services"

# Commit 7: Testes de RiskValidator
git add .
git commit -m "test: add unit tests for all risk rejection scenarios"

# Commit 8: Implementar AssessRiskUseCase
git add .
git commit -m "feat: implement AssessRiskUseCase with state fetching"

# Commit 9: Extrair PositionManager
git add .
git commit -m "refactor: extract PositionManager pure logic to domain_services"

# Commit 10: Testes de PositionManager
git add .
git commit -m "test: add unit tests for position lifecycle"

# Commit 11: Teste E2E de order flow
git add .
git commit -m "test: add E2E test for complete order flow (signal → position)"
```

**Critérios de Aceite:**
- ✅ RiskValidator 100% puro
- ✅ PositionManager 100% puro
- ✅ 80%+ cobertura em testes unitários
- ✅ Teste E2E passando para fluxo completo

---

### Fase 3: Reconciliation & Ledger (Semana 5-6)

**Objetivo:** Extrair ReconciliationService e LedgerService

#### Commits:

```bash
# Commit 12: Extrair BalanceComparator
git checkout -b feature/reconciliation
git add .
git commit -m "refactor: extract BalanceComparator pure logic to domain_services"

# Commit 13: Testes de BalanceComparator
git add .
git commit -m "test: add unit tests for reconciliation discrepancy detection"

# Commit 14: Implementar ReconcileBalancesUseCase
git add .
git commit -m "feat: implement ReconcileBalancesUseCase with snapshot persistence"

# Commit 15: Teste E2E de reconciliação
git add .
git commit -m "test: add E2E test for balance reconciliation"

# Commit 16: Refatorar LedgerService
git add .
git commit -m "refactor: extract LedgerService pure logic"

# Commit 17: Testes de LedgerService
git add .
git commit -m "test: add unit tests for ledger entry processing"
```

**Critérios de Aceite:**
- ✅ BalanceComparator 100% puro
- ✅ LedgerService refatorado
- ✅ 80%+ cobertura em testes
- ✅ Teste E2E de reconciliação passando

---

### Fase 4: Observability & Security (Semana 7-8)

**Objetivo:** Substituir prints por tracing + migrar para Secret<T>

#### Commits:

```bash
# Commit 18: Remover todos os println! e print!
git checkout -b feature/observability
# ... substituir por tracing::info!, debug!, error! ...
git add .
git commit -m "refactor: replace println! with structured tracing"

# Commit 19: Migrar API keys para Secret<T>
git add .
git commit -m "security: wrap API keys with secrecy::Secret<T>"

# Commit 20: Configurar tracing JSON output
git add .
git commit -m "feat: configure structured JSON logging for production"
```

**Critérios de Aceite:**
- ✅ Zero uso de println! ou print!
- ✅ API keys nunca aparecem em logs
- ✅ Logs estruturados em JSON

---

### Fase 5: CI/CD & Documentation (Semana 9-10)

**Objetivo:** Setup CI + documentação completa

#### Commits:

```bash
# Commit 21: Configurar GitHub Actions
git checkout -b feature/ci-cd
# ... criar .github/workflows/ci.yml ...
git add .
git commit -m "ci: add GitHub Actions workflow with fmt, clippy, tests, coverage"

# Commit 22: Criar README-TESTING.md
git add .
git commit -m "docs: add comprehensive testing documentation"

# Commit 23: Criar RFC de arquitetura
git add .
git commit -m "docs: add architecture RFC with diagrams"

# Commit 24: Configurar cargo-llvm-cov
git add .
git commit -m "ci: add code coverage reporting with cargo-llvm-cov"
```

**Critérios de Aceite:**
- ✅ CI verde em todas as branches
- ✅ Cobertura 80%+ nas áreas críticas
- ✅ Documentação completa

---

## 📝 GitHub Actions CI Pipeline

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [ main, develop, "feature/**", "refactor/**" ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  format:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test Suite
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: robotrade_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib --all-features
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/robotrade_test

      - name: Run integration tests
        run: cargo test --test '*' --all-features
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/robotrade_test

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: robotrade_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/robotrade_test

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --workspace --summary-only | grep -oP 'TOTAL.*\K[0-9.]+(?=%)')
          echo "Coverage: $COVERAGE%"
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "❌ Coverage $COVERAGE% is below 80% threshold"
            exit 1
          fi
          echo "✅ Coverage $COVERAGE% meets 80% threshold"

  build:
    name: Build
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - x86_64-apple-darwin
          - x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: robotrade-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/robotrade*

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run cargo-audit
        uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

## 📚 Documentação

### README-TESTING.md

```markdown
# RoboTrade - Guia de Testes

## Visão Geral

Este documento explica como rodar e escrever testes no RoboTrade.

## Tipos de Testes

### 1. Testes Unitários

**Localização:** Junto com o código (módulos `#[cfg(test)]`)

**Comando:**
```bash
cargo test --lib
```

**Características:**
- 100% determinísticos
- Sem IO (banco, HTTP, filesystem)
- Rápidos (<1ms cada)
- Cobrem lógica pura de negócio

**Exemplo:**
```rust
#[test]
fn test_pnl_calculator_fifo() {
    let lots = vec![...];
    let trade = Trade { ... };
    let (pnl, _) = PnLCalculator::calculate_fifo(&lots, &trade).unwrap();
    assert_eq!(pnl.amount, dec!(1000.0));
}
```

### 2. Testes de Integração

**Localização:** `integration_tests/tests/`

**Comando:**
```bash
# Rodar todos
cargo test --test '*'

# Rodar teste específico
cargo test --test e2e_order_flow
```

**Características:**
- Usa Docker (Testcontainers)
- Banco de dados real (Postgres)
- Mock exchanges determinísticos
- Valida fluxos completos

**Pré-requisitos:**
- Docker instalado e rodando
- ~2GB de RAM disponível

### 3. Testes End-to-End

**Script:** `scripts/test_e2e.sh`

**Comando:**
```bash
./scripts/test_e2e.sh
```

**O que faz:**
1. Verifica Docker está rodando
2. Builda o projeto
3. Roda testes de integração
4. Gera relatório de cobertura
5. Abre relatório no navegador

## Rodando Localmente

### Setup Inicial

```bash
# 1. Instalar ferramentas
cargo install cargo-llvm-cov
cargo install cargo-watch

# 2. Verificar Docker
docker info

# 3. Rodar testes unitários
cargo test --lib

# 4. Rodar testes de integração
cd integration_tests
cargo test
```

### Desenvolvimento com Watch Mode

```bash
# Auto-rodar testes ao salvar
cargo watch -x test

# Auto-rodar testes específicos
cargo watch -x 'test pnl_calculator'
```

### Gerando Cobertura

```bash
# Gerar relatório HTML
cargo llvm-cov --html --open

# Gerar relatório texto
cargo llvm-cov --summary-only

# Verificar threshold
cargo llvm-cov --fail-under-lines 80
```

## Escrevendo Novos Testes

### Template: Teste Unitário

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = /* ... */;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Template: Teste de Integração

```rust
#[tokio::test]
async fn test_feature_name() {
    // Arrange
    let test_env = TestEnvironment::new().await;

    // Act
    let result = use_case.execute(...).await.unwrap();

    // Assert
    assert_eq!(result.status, ExpectedStatus);

    // Verify persistence
    let persisted = test_env.repo.find_by_id(&result.id).await.unwrap();
    assert!(persisted.is_some());
}
```

## Fixtures

**Localização:** `integration_tests/fixtures/`

**Formato:** JSON

**Exemplo:**
```json
{
  "positions": [
    {
      "id": "pos-1",
      "symbol": "BTCUSDT",
      "side": "long",
      "size": "0.5",
      "entry_price": "50000.0"
    }
  ]
}
```

## Troubleshooting

### Docker não está rodando

```bash
# macOS
open -a Docker

# Linux
sudo systemctl start docker
```

### Testes lentos

```bash
# Rodar em paralelo (padrão)
cargo test

# Rodar sequencial (útil para debug)
cargo test -- --test-threads=1
```

### Falha de conexão com banco

```bash
# Verificar container está rodando
docker ps

# Ver logs
docker logs <container-id>

# Recriar container
docker-compose down && docker-compose up -d
```

## CI/CD

Todos os testes rodam automaticamente no GitHub Actions:

- ✅ Format check (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Unit tests
- ✅ Integration tests
- ✅ Coverage report (threshold: 80%)

Ver: `.github/workflows/ci.yml`

## Métricas de Qualidade

| Métrica | Target | Atual |
|---------|--------|-------|
| **Cobertura de Linhas** | ≥80% | 🎯 85% |
| **Cobertura de Branches** | ≥75% | 🎯 78% |
| **Testes Unitários** | >200 | ✅ 247 |
| **Testes de Integração** | >15 | ✅ 18 |
| **Tempo de Build** | <5min | ✅ 3m 42s |

## Referências

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testcontainers Rust](https://github.com/testcontainers/testcontainers-rs)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
```

---

## ✅ Critérios de Aceite Final

### Funcional

- ✅ Lógica de negócio crítica extraída e 100% pura (sem IO)
- ✅ PnLCalculator, RiskValidator, BalanceComparator, PositionManager testáveis
- ✅ Use cases orquestram IO separadamente
- ✅ Mock exchanges determinísticos funcionando

### Testes

- ✅ 80%+ cobertura de linhas em:
  - `crates/core/src/domain_services/` (100% target)
  - `crates/application/src/use_cases/` (85%+ target)
  - `crates/infra/src/repositories/` (70%+ target)
- ✅ 200+ testes unitários
- ✅ 15+ testes de integração E2E
- ✅ Todos os testes passando

### Qualidade

- ✅ Zero warnings no `cargo clippy`
- ✅ Código formatado com `cargo fmt`
- ✅ Todos os prints substituídos por tracing estruturado
- ✅ API keys com `Secret<T>`
- ✅ CI pipeline verde

### Documentação

- ✅ PROPOSAL.md (este documento)
- ✅ README-TESTING.md
- ✅ RFC de arquitetura
- ✅ Docstrings em funções públicas

---

## 🎯 Próximos Passos

1. **Revisar e Aprovar esta Proposta**
2. **Criar Issues no GitHub** para cada fase
3. **Começar Fase 1:** Testing Infrastructure + PnLCalculator
4. **Review incremental** após cada fase
5. **Ajustar roadmap** conforme necessário

---

## 📞 Contato

Para questões sobre esta proposta, contate a equipe no canal #robotrade-refactoring.

---

**Aprovação Necessária:**
- [ ] Tech Lead
- [ ] Product Owner
- [ ] QA Lead

**Data Planejada de Início:** 2025-12-02
**Data Estimada de Conclusão:** 2026-02-15 (10 semanas)
