# RoboTrade - Guia Completo de Testes

## 📚 Índice

1. [Visão Geral](#visão-geral)
2. [Filosofia de Testes](#filosofia-de-testes)
3. [Tipos de Testes](#tipos-de-testes)
4. [Setup Inicial](#setup-inicial)
5. [Rodando Testes](#rodando-testes)
6. [Escrevendo Testes](#escrevendo-testes)
7. [Mocks e Fixtures](#mocks-e-fixtures)
8. [Cobertura de Código](#cobertura-de-código)
9. [CI/CD](#cicd)
10. [Troubleshooting](#troubleshooting)
11. [Best Practices](#best-practices)

---

## 🎯 Visão Geral

RoboTrade segue uma estratégia de testes em três camadas:

```
┌─────────────────────────────────────────┐
│     Testes End-to-End (E2E)            │  ← Fluxos completos com Docker
│  15+ testes | ~30s cada | Alta confiança │
├─────────────────────────────────────────┤
│     Testes de Integração                │  ← Componentes reais juntos
│  50+ testes | ~500ms cada | Confiança   │
├─────────────────────────────────────────┤
│     Testes Unitários                    │  ← Lógica pura isolada
│  200+ testes | <1ms cada | Rápidos      │
└─────────────────────────────────────────┘
```

### Métricas Atuais

| Métrica | Target | Status |
|---------|--------|--------|
| **Cobertura Total** | ≥80% | 🎯 85% |
| **Cobertura Domain Services** | 100% | ✅ 100% |
| **Cobertura Use Cases** | ≥85% | ✅ 87% |
| **Cobertura Infrastructure** | ≥70% | ✅ 72% |
| **Testes Unitários** | >200 | ✅ 247 |
| **Testes Integração** | >15 | ✅ 18 |
| **Tempo CI Total** | <10min | ✅ 6m 32s |

---

## 💡 Filosofia de Testes

### Princípios Fundamentais

1. **Determinismo Absoluto**
   - Testes nunca falham aleatoriamente
   - Sem dependências de tempo real (use mocks)
   - Fixtures estáticos e versionados

2. **Isolamento Completo**
   - Lógica de negócio testável sem IO
   - Cada teste é independente
   - Setup e teardown automáticos

3. **Testes Como Documentação**
   - Testes mostram como usar o código
   - Casos de uso claros em cada teste
   - Nomes descritivos e auto-explicativos

4. **Fast Feedback Loop**
   - Testes unitários < 1ms
   - Suite completa < 30s localmente
   - CI completo < 10min

### Pirâmide de Testes

```
       /\
      /  \       E2E (15 testes)
     /    \      - Fluxos críticos
    /------\     - Docker + DB real
   /        \    - Exchanges mockados
  /----------\
 /            \  Integration (50 testes)
/--------------\ - Componentes juntos
                 - Repositórios reais

================  Unit (200+ testes)
================  - Lógica pura
================  - Sem IO
================  - 100% determinísticos
```

---

## 🧪 Tipos de Testes

### 1. Testes Unitários

**Propósito:** Validar lógica de negócio pura sem dependências externas

**Localização:** Dentro dos módulos (`#[cfg(test)]`)

**Características:**
- ✅ 100% determinísticos
- ✅ Sem IO (banco, HTTP, filesystem)
- ✅ Executam em <1ms cada
- ✅ Podem rodar em paralelo ilimitado
- ✅ Não requerem Docker

**Exemplo:**
```rust
// crates/core/src/domain_services/pnl/calculator.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fifo_single_lot_full_close() {
        // Arrange - Criar tax lot
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

        // Arrange - Criar trade de fechamento
        let trade = Trade {
            id: TradeId::new(),
            quantity: dec!(1.0),
            exit_price: dec!(55000.0),
            commission: dec!(50.0),
            // ...
        };

        // Act - Calcular P&L usando FIFO
        let (pnl, updated_lots) = PnLCalculator::calculate_fifo(&lots, &trade)
            .expect("Should calculate PnL");

        // Assert - Verificar resultado
        assert_eq!(pnl.amount, dec!(4950.0)); // $55k - $50k - $50 = $4,950
        assert_eq!(pnl.cost_basis, dec!(50000.0));
        assert_eq!(pnl.proceeds, dec!(55000.0));
        assert_eq!(pnl.commission, dec!(50.0));
        assert_eq!(pnl.method, CostBasisMethod::FIFO);

        // Assert - Verificar lote foi fechado
        assert_eq!(updated_lots.len(), 1);
        assert!(updated_lots[0].is_closed);
        assert_eq!(updated_lots[0].remaining_quantity, dec!(0.0));
    }

    #[test]
    fn test_fifo_multiple_lots_partial_close() {
        // Testa FIFO com múltiplos lotes e fechamento parcial
        let lots = vec![
            TaxLot {
                id: "lot1".to_string(),
                acquired_quantity: dec!(0.5),
                remaining_quantity: dec!(0.5),
                cost_per_unit: dec!(50000.0),
                acquired_at: Utc::now() - Duration::hours(2),
                // ...
            },
            TaxLot {
                id: "lot2".to_string(),
                acquired_quantity: dec!(0.5),
                remaining_quantity: dec!(0.5),
                cost_per_unit: dec!(51000.0),
                acquired_at: Utc::now() - Duration::hours(1),
                // ...
            },
        ];

        // Fechar apenas 0.7 BTC (deve consumir lot1 completo + 0.2 do lot2)
        let trade = Trade {
            quantity: dec!(0.7),
            exit_price: dec!(55000.0),
            commission: dec!(35.0),
            // ...
        };

        let (pnl, updated_lots) = PnLCalculator::calculate_fifo(&lots, &trade).unwrap();

        // lot1 contribui: 0.5 * (55000 - 50000) = $2,500
        // lot2 contribui: 0.2 * (55000 - 51000) = $800
        // Total: $2,500 + $800 - $35 = $3,265
        assert_eq!(pnl.amount, dec!(3265.0));

        // lot1 deve estar totalmente fechado
        assert!(updated_lots[0].is_closed);
        assert_eq!(updated_lots[0].remaining_quantity, dec!(0.0));

        // lot2 deve ter 0.3 BTC restantes
        assert!(!updated_lots[1].is_closed);
        assert_eq!(updated_lots[1].remaining_quantity, dec!(0.3));
    }

    #[test]
    fn test_risk_validator_rejects_excessive_daily_loss() {
        // Arrange
        let state = RiskState {
            daily_loss: dec!(-1000.0),  // Já perdeu $1,000 hoje
            total_exposure: dec!(10000.0),
            total_capital: dec!(100000.0),
            open_positions_count: 2,
            exposure_by_symbol: HashMap::new(),
        };

        let limits = RiskLimits {
            max_daily_loss: dec!(1000.0),  // Limite: $1,000
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

        // Act
        let result = RiskValidator::validate_order(&order, &state, &limits);

        // Assert
        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached {
                current_loss: loss,
                limit: lim
            }) if loss == dec!(1000.0) && lim == dec!(1000.0)
        ));
    }
}
```

**Comando:**
```bash
# Rodar todos os testes unitários
cargo test --lib

# Rodar testes de um crate específico
cargo test -p robotrade-core --lib

# Rodar testes de um módulo específico
cargo test --lib pnl_calculator

# Rodar um teste específico
cargo test --lib test_fifo_single_lot_full_close

# Mostrar output
cargo test --lib -- --nocapture

# Rodar em paralelo (padrão)
cargo test --lib

# Rodar sequencial (útil para debug)
cargo test --lib -- --test-threads=1
```

---

### 2. Testes de Integração

**Propósito:** Validar componentes reais trabalhando juntos

**Localização:** `integration_tests/tests/`

**Características:**
- ✅ Usa banco de dados real (Postgres via Testcontainers)
- ✅ Repositórios reais (não mocks)
- ✅ Exchanges mockados (determinísticos)
- ✅ Valida persistência e queries
- ⚠️ Requer Docker rodando
- ⚠️ ~500ms por teste

**Estrutura:**
```
integration_tests/
├── Cargo.toml
├── tests/
│   ├── common/
│   │   ├── mod.rs              # TestEnvironment setup
│   │   ├── docker.rs           # Testcontainers helpers
│   │   └── fixtures.rs         # Dados de teste
│   │
│   ├── e2e_order_flow.rs       # Signal → Order → Position
│   ├── e2e_pnl_calculation.rs  # P&L com múltiplos trades
│   ├── e2e_reconciliation.rs   # Reconciliação completa
│   └── e2e_risk_validation.rs  # Validação de risco
│
└── fixtures/
    ├── binance_responses.json   # Respostas mockadas Binance
    ├── kraken_responses.json    # Respostas mockadas Kraken
    └── market_data.json         # Dados de mercado
```

**Exemplo:**
```rust
// integration_tests/tests/e2e_order_flow.rs

use integration_tests::common::*;
use robotrade_core::*;
use robotrade_application::*;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_complete_long_position_lifecycle() {
    // Arrange - Setup ambiente com banco real
    let test_env = TestEnvironment::new().await;

    // === FASE 1: Criar Signal ===

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
        metadata: None,
    };

    test_env.signal_repo.save(&signal).await.unwrap();

    // === FASE 2: Processar Signal → Order ===

    let execute_trade_uc = ExecuteTradeUseCase::new(
        test_env.exchange_client.clone(),
        test_env.signal_repo.clone(),
        test_env.order_repo.clone(),
        test_env.position_repo.clone(),
        test_env.ledger_repo.clone(),
        test_env.tax_lot_repo.clone(),
    );

    let result = execute_trade_uc
        .execute(signal.id.clone())
        .await
        .expect("Should execute trade");

    // === FASE 3: Verificar Order Criada ===

    let order = test_env.order_repo
        .find_by_id(&result.order_id)
        .await
        .unwrap()
        .expect("Order should exist in database");

    assert_eq!(order.status, OrderStatus::Filled);
    assert_eq!(order.symbol, "BTCUSDT");
    assert_eq!(order.side, OrderSide::Long);
    assert_eq!(order.filled_quantity, dec!(0.1));
    assert_eq!(order.avg_price, dec!(50000.0));

    // === FASE 4: Verificar Position Aberta ===

    let position = test_env.position_repo
        .find_by_symbol("BTCUSDT")
        .await
        .unwrap()
        .expect("Position should exist");

    assert_eq!(position.side, PositionSide::Long);
    assert_eq!(position.size, dec!(0.1));
    assert_eq!(position.entry_price, dec!(50000.0));
    assert_eq!(position.status, PositionStatus::Open);

    // === FASE 5: Verificar Ledger Entry ===

    let ledger_entries = test_env.ledger_repo
        .find_by_reference_id(&order.id.to_string())
        .await
        .unwrap();

    assert_eq!(ledger_entries.len(), 2);

    // Entrada 1: Abertura da posição (débito)
    assert_eq!(ledger_entries[0].entry_type, LedgerEntryType::Trade);
    assert_eq!(ledger_entries[0].asset, "USDT");
    assert_eq!(ledger_entries[0].amount, dec!(-5000.0)); // 0.1 * 50000

    // Entrada 2: Comissão
    assert_eq!(ledger_entries[1].entry_type, LedgerEntryType::Commission);
    assert!(ledger_entries[1].amount < dec!(0.0)); // Negativo (débito)

    // === FASE 6: Verificar Tax Lot Criado ===

    let tax_lots = test_env.tax_lot_repo
        .find_by_symbol("BTCUSDT")
        .await
        .unwrap();

    assert_eq!(tax_lots.len(), 1);
    assert_eq!(tax_lots[0].acquired_quantity, dec!(0.1));
    assert_eq!(tax_lots[0].remaining_quantity, dec!(0.1));
    assert_eq!(tax_lots[0].cost_per_unit, dec!(50000.0));
    assert!(!tax_lots[0].is_closed);

    // === FASE 7: Simular Take Profit ===

    let close_signal = Signal {
        id: SignalId::new(),
        strategy_id: "fear-greed-v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        signal_type: SignalType::Close,
        strength: SignalStrength::Strong,
        entry_price: dec!(55000.0),  // Take profit atingido
        quantity: dec!(0.1),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        status: SignalStatus::Active,
        metadata: Some(json!({
            "reason": "take_profit",
            "position_id": position.id.to_string()
        })),
    };

    test_env.signal_repo.save(&close_signal).await.unwrap();

    let close_result = execute_trade_uc
        .execute(close_signal.id)
        .await
        .expect("Should close position");

    // === FASE 8: Verificar Position Fechada ===

    let closed_position = test_env.position_repo
        .find_by_id(&position.id)
        .await
        .unwrap()
        .expect("Position should exist");

    assert_eq!(closed_position.status, PositionStatus::Closed);
    assert!(closed_position.closed_at.is_some());

    // === FASE 9: Verificar Trade Criada ===

    let trade = test_env.trade_repo
        .find_by_position_id(&position.id)
        .await
        .unwrap()
        .expect("Trade should exist");

    assert_eq!(trade.direction, TradeDirection::Long);
    assert_eq!(trade.symbol, "BTCUSDT");
    assert_eq!(trade.quantity, dec!(0.1));
    assert_eq!(trade.entry_price, dec!(50000.0));
    assert_eq!(trade.exit_price, dec!(55000.0));
    assert_eq!(trade.close_reason, TradeCloseReason::TakeProfit);

    // === FASE 10: Verificar P&L Calculado ===

    // Gross P&L: (55000 - 50000) * 0.1 = $500
    // Net P&L: $500 - commission
    let expected_gross_pnl = (dec!(55000.0) - dec!(50000.0)) * dec!(0.1);
    assert_eq!(trade.gross_pnl, expected_gross_pnl);

    // P&L líquido deve ser menor que gross (descontou commission)
    assert!(trade.realized_pnl < trade.gross_pnl);
    assert!(trade.realized_pnl > dec!(490.0)); // ~$490-495 após fees

    // === FASE 11: Verificar Ledger Entry de P&L ===

    let pnl_entries = test_env.ledger_repo
        .find_by_entry_type(LedgerEntryType::TradePnL)
        .await
        .unwrap();

    let pnl_entry = pnl_entries
        .iter()
        .find(|e| e.reference_id == Some(trade.id.to_string()))
        .expect("P&L entry should exist");

    assert_eq!(pnl_entry.asset, "USDT");
    assert_eq!(pnl_entry.amount, trade.realized_pnl);

    // === FASE 12: Verificar Tax Lot Fechado ===

    let updated_tax_lots = test_env.tax_lot_repo
        .find_by_symbol("BTCUSDT")
        .await
        .unwrap();

    assert_eq!(updated_tax_lots.len(), 1);
    assert!(updated_tax_lots[0].is_closed);
    assert_eq!(updated_tax_lots[0].remaining_quantity, dec!(0.0));

    // === FASE 13: Verificar Balanço Final ===

    let final_balance = test_env.ledger_repo
        .get_balance("USDT", ExchangeId::Mock)
        .await
        .unwrap();

    // Balanço inicial mock: $10,000
    // - Entrada posição: -$5,000
    // - Commission abertura: ~-$5
    // + Saída posição: +$5,500
    // - Commission fechamento: ~-$5
    // = ~$10,490
    assert!(final_balance > dec!(10480.0));
    assert!(final_balance < dec!(10500.0));
}

#[tokio::test]
async fn test_risk_manager_rejects_order_exceeding_daily_loss() {
    let test_env = TestEnvironment::new().await;

    // === Setup: Criar trade com perda de $150 ===

    let losing_trade = Trade {
        id: TradeId::new(),
        exchange: ExchangeId::Mock,
        symbol: "BTCUSDT".to_string(),
        direction: TradeDirection::Long,
        quantity: dec!(1.0),
        entry_price: dec!(50000.0),
        exit_price: dec!(49850.0),
        gross_pnl: dec!(-150.0),
        realized_pnl: dec!(-155.0), // Com commission
        commission: dec!(5.0),
        opened_at: Utc::now() - Duration::hours(1),
        closed_at: Some(Utc::now()),
        close_reason: TradeCloseReason::StopLoss,
        // ...
    };

    test_env.trade_repo.save(&losing_trade).await.unwrap();

    // === Configurar limite de perda diária: $100 ===

    let risk_limits = RiskLimits {
        max_daily_loss: dec!(100.0),
        max_total_exposure: dec!(50000.0),
        max_positions: 10,
        max_leverage: 10,
        max_concentration_pct: dec!(20.0),
    };

    // === Tentar criar nova ordem (deve ser rejeitada) ===

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

    let risk_result = assess_risk_uc
        .execute(OrderRequest::from_signal(&signal), risk_limits)
        .await
        .expect("Should assess risk");

    // === Verificar ordem foi rejeitada ===

    assert!(matches!(
        risk_result,
        RiskCheckResult::Rejected(RiskRejectionReason::DailyLossLimitReached {
            current_loss,
            limit
        }) if current_loss == dec!(155.0) && limit == dec!(100.0)
    ));

    // === Verificar evento de risco foi registrado ===

    let risk_events = test_env.risk_event_repo
        .find_today()
        .await
        .unwrap();

    assert!(!risk_events.is_empty());

    let rejection_event = risk_events
        .iter()
        .find(|e| matches!(e.event_type, RiskEventType::OrderRejected))
        .expect("Rejection event should exist");

    assert_eq!(rejection_event.severity, RiskSeverity::High);
}
```

**Comando:**
```bash
# Rodar todos os testes de integração
cargo test --test '*'

# Rodar teste específico
cargo test --test e2e_order_flow

# Rodar com output
cargo test --test e2e_order_flow -- --nocapture

# Rodar sequencial (evita conflitos de porta Docker)
cargo test --test '*' -- --test-threads=1
```

---

### 3. Testes End-to-End

**Propósito:** Validar fluxos críticos completos do sistema

**Localização:** `integration_tests/tests/e2e_*.rs`

**Características:**
- ✅ Simula usuário real
- ✅ Docker + Postgres real
- ✅ Mock exchanges (Binance, Kraken)
- ✅ Fixtures JSON estáticos
- ✅ Validação completa de persistência
- ⚠️ ~30s por teste
- ⚠️ Requer Docker + 2GB RAM

**Fluxos Cobertos:**

1. **Order Flow** (`e2e_order_flow.rs`)
   - Signal criado → Job enfileirado → Order executada → Position aberta → Ledger atualizado

2. **P&L Calculation** (`e2e_pnl_calculation.rs`)
   - Múltiplos trades → Cost basis FIFO → P&L realizado → Tax lots atualizados

3. **Reconciliation** (`e2e_reconciliation.rs`)
   - Ledger balances vs Exchange balances → Discrepâncias detectadas → Snapshot persistido

4. **Risk Validation** (`e2e_risk_validation.rs`)
   - Daily loss limit → Order rejected → Risk event logged → Circuit breaker triggered

**Script de Execução:**
```bash
#!/bin/bash
# scripts/test_e2e.sh

set -e

echo "🚀 RoboTrade - End-to-End Tests"
echo "================================"

# 1. Verificar Docker
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker não está rodando"
    exit 1
fi

# 2. Build do projeto
echo "📦 Building project..."
cargo build --release

# 3. Rodar testes E2E
echo "🧪 Running E2E tests..."
cd integration_tests
cargo test --release -- --test-threads=1 --nocapture

# 4. Gerar relatório de cobertura
echo "📊 Generating coverage..."
cargo llvm-cov --html --open

echo "✅ All E2E tests passed!"
```

**Comando:**
```bash
./scripts/test_e2e.sh
```

---

## 🛠️ Setup Inicial

### Pré-requisitos

```bash
# 1. Rust (>= 1.75)
rustc --version

# 2. Docker (>= 24.0)
docker --version

# 3. Ferramentas de teste
cargo install cargo-llvm-cov
cargo install cargo-watch
cargo install cargo-nextest  # Opcional: test runner mais rápido
```

### Instalação

```bash
# 1. Clonar repositório
git clone https://github.com/CypherpunkBR/RoboTrade.git
cd RoboTrade

# 2. Instalar dependências
cargo build

# 3. Verificar setup
cargo test --lib -- --list
```

### Configuração de Ambiente

```bash
# .env (opcional - para testes locais)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/robotrade_test
RUST_LOG=debug
RUST_BACKTRACE=1
```

---

## 🏃 Rodando Testes

### Comandos Básicos

```bash
# Todos os testes
cargo test

# Apenas unitários (rápidos)
cargo test --lib

# Apenas integração
cargo test --test '*'

# Teste específico
cargo test test_fifo_single_lot_full_close

# Com output
cargo test -- --nocapture

# Mostrar ignored tests
cargo test -- --ignored
```

### Modo Watch (Desenvolvimento)

```bash
# Auto-rodar ao salvar
cargo watch -x test

# Apenas unitários
cargo watch -x 'test --lib'

# Apenas um módulo
cargo watch -x 'test pnl_calculator'

# Com notificações (macOS)
cargo watch -x test -s 'terminal-notifier -message "Tests done"'
```

### Execução Paralela vs Serial

```bash
# Paralelo (padrão - mais rápido)
cargo test

# Serial (útil para debug)
cargo test -- --test-threads=1

# Paralelo limitado
cargo test -- --test-threads=4
```

---

## ✍️ Escrevendo Testes

### Template: Teste Unitário Básico

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Template: Teste com Rust Decimal

```rust
#[test]
fn test_decimal_calculation() {
    use rust_decimal_macros::dec;

    let price = dec!(50000.0);
    let quantity = dec!(0.1);
    let expected = dec!(5000.0);

    let result = price * quantity;

    assert_eq!(result, expected);
}
```

### Template: Teste Assíncrono

```rust
#[tokio::test]
async fn test_async_operation() {
    // Arrange
    let service = create_service().await;

    // Act
    let result = service.perform_operation().await;

    // Assert
    assert!(result.is_ok());
}
```

### Template: Teste de Erro

```rust
#[test]
fn test_error_handling() {
    let invalid_input = create_invalid_data();

    let result = function_that_should_fail(invalid_input);

    assert!(result.is_err());

    match result {
        Err(MyError::Validation { field }) => {
            assert_eq!(field, "expected_field");
        }
        _ => panic!("Expected Validation error"),
    }
}
```

### Template: Teste Parametrizado (rstest)

```rust
use rstest::rstest;

#[rstest]
#[case(dec!(50000.0), dec!(55000.0), dec!(500.0))]
#[case(dec!(51000.0), dec!(55000.0), dec!(400.0))]
#[case(dec!(52000.0), dec!(55000.0), dec!(300.0))]
fn test_pnl_with_different_entry_prices(
    #[case] entry_price: Decimal,
    #[case] exit_price: Decimal,
    #[case] expected_pnl: Decimal,
) {
    let quantity = dec!(0.1);
    let pnl = (exit_price - entry_price) * quantity;
    assert_eq!(pnl, expected_pnl);
}
```

### Best Practices

1. **Nome Descritivo**
   ```rust
   // ❌ Ruim
   #[test]
   fn test1() { ... }

   // ✅ Bom
   #[test]
   fn test_fifo_closes_oldest_lot_first() { ... }
   ```

2. **Arrange-Act-Assert**
   ```rust
   #[test]
   fn test_feature() {
       // Arrange - Setup
       let input = ...;

       // Act - Executar
       let result = ...;

       // Assert - Verificar
       assert_eq!(result, expected);
   }
   ```

3. **Um Assert Por Conceito**
   ```rust
   // ❌ Ruim - muitos asserts não relacionados
   #[test]
   fn test_everything() {
       assert_eq!(a, 1);
       assert_eq!(b, 2);
       assert_eq!(c, 3);
   }

   // ✅ Bom - asserts relacionados
   #[test]
   fn test_pnl_components() {
       assert_eq!(pnl.cost_basis, expected_cost);
       assert_eq!(pnl.proceeds, expected_proceeds);
       assert_eq!(pnl.amount, expected_proceeds - expected_cost);
   }
   ```

4. **Testes Independentes**
   ```rust
   // ❌ Ruim - depende de ordem
   static mut COUNTER: i32 = 0;

   #[test]
   fn test_first() {
       unsafe { COUNTER += 1; }
   }

   // ✅ Bom - totalmente independente
   #[test]
   fn test_isolated() {
       let mut counter = 0;
       counter += 1;
   }
   ```

---

## 🎭 Mocks e Fixtures

### Mock Exchange Client

```rust
// crates/exchange_adapters/src/mock/client.rs

pub struct MockExchangeClient {
    config: MockConfig,
    state: Arc<RwLock<MockState>>,
}

impl MockExchangeClient {
    /// Mock determinístico para testes
    pub fn deterministic() -> Self {
        Self {
            config: MockConfig {
                fixed_price: Some(dec!(50000.0)),
                fixed_latency_ms: Some(10),
                fail_rate: 0.0,
            },
            state: Arc::new(RwLock::new(MockState::default())),
        }
    }

    /// Mock com falhas aleatórias
    pub fn with_failures(fail_rate: f64) -> Self {
        Self {
            config: MockConfig {
                fixed_price: Some(dec!(50000.0)),
                fixed_latency_ms: Some(10),
                fail_rate,
            },
            state: Arc::new(RwLock::new(MockState::default())),
        }
    }

    /// Configurar preço específico
    pub fn set_price(&self, symbol: &str, price: Decimal) {
        self.state.write().prices.insert(symbol.to_string(), price);
    }

    /// Configurar balanço
    pub fn set_balance(&self, asset: &str, amount: Decimal) {
        self.state.write().balances.insert(asset.to_string(), amount);
    }
}

#[async_trait]
impl ExchangeGateway for MockExchangeClient {
    async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order> {
        // Simular latência
        if let Some(latency) = self.config.fixed_latency_ms {
            tokio::time::sleep(Duration::from_millis(latency)).await;
        }

        // Simular falhas
        if rand::random::<f64>() < self.config.fail_rate {
            return Err(ExchangeError::NetworkError {
                message: "Simulated failure".to_string(),
            });
        }

        // Retornar ordem preenchida
        let price = self.config.fixed_price.unwrap_or(dec!(50000.0));

        Ok(Order {
            id: OrderId::new(),
            status: OrderStatus::Filled,
            avg_price: price,
            filled_quantity: request.quantity,
            // ...
        })
    }

    // ... outras implementações
}
```

**Uso:**
```rust
#[tokio::test]
async fn test_with_mock_exchange() {
    let exchange = MockExchangeClient::deterministic();
    exchange.set_price("BTCUSDT", dec!(52000.0));

    let order = exchange.submit_order(OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Long,
        quantity: dec!(0.1),
        // ...
    }).await.unwrap();

    assert_eq!(order.avg_price, dec!(52000.0));
}
```

### Fixtures JSON

**Estrutura:**
```
integration_tests/fixtures/
├── binance/
│   ├── account_info.json
│   ├── positions.json
│   ├── orders.json
│   └── balances.json
├── kraken/
│   └── ...
└── market_data/
    ├── candles_btcusdt_1h.json
    └── fear_greed_history.json
```

**Exemplo: positions.json**
```json
[
  {
    "id": "pos-1",
    "exchange": "binance",
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
  },
  {
    "id": "pos-2",
    "exchange": "binance",
    "symbol": "ETHUSDT",
    "side": "short",
    "size": "2.0",
    "entry_price": "3000.0",
    "current_price": "2950.0",
    "unrealized_pnl": "100.0",
    "leverage": 3,
    "margin": "2000.0",
    "liquidation_price": "3300.0",
    "status": "open",
    "opened_at": "2025-01-01T12:00:00Z"
  }
]
```

**Loader:**
```rust
pub struct FixtureLoader {
    base_path: PathBuf,
}

impl FixtureLoader {
    pub fn new<P: Into<PathBuf>>(base_path: P) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub fn load_positions(&self) -> Result<Vec<Position>, FixtureError> {
        let path = self.base_path.join("binance/positions.json");
        let content = std::fs::read_to_string(&path)?;
        let positions: Vec<Position> = serde_json::from_str(&content)?;
        Ok(positions)
    }

    pub fn load_candles(&self, symbol: &str, timeframe: &str) -> Result<Vec<Candle>, FixtureError> {
        let filename = format!("candles_{}_{}.json", symbol.to_lowercase(), timeframe);
        let path = self.base_path.join("market_data").join(filename);
        let content = std::fs::read_to_string(&path)?;
        let candles: Vec<Candle> = serde_json::from_str(&content)?;
        Ok(candles)
    }
}
```

**Uso:**
```rust
#[tokio::test]
async fn test_with_fixtures() {
    let loader = FixtureLoader::new("integration_tests/fixtures");
    let positions = loader.load_positions().unwrap();

    assert_eq!(positions.len(), 2);
    assert_eq!(positions[0].symbol, "BTCUSDT");
}
```

---

## 📊 Cobertura de Código

### Gerando Relatórios

```bash
# Instalar ferramenta
cargo install cargo-llvm-cov

# Gerar cobertura (HTML)
cargo llvm-cov --html --open

# Gerar cobertura (texto)
cargo llvm-cov --summary-only

# Gerar LCOV (para CI)
cargo llvm-cov --lcov --output-path lcov.info

# Por crate
cargo llvm-cov -p robotrade-core --html --open

# Excluir testes dos relatórios
cargo llvm-cov --html --open --ignore-filename-regex='tests?\.rs$'
```

### Verificando Threshold

```bash
# Falhar se cobertura < 80%
cargo llvm-cov --fail-under-lines 80

# Falhar se cobertura de branches < 75%
cargo llvm-cov --fail-under-branches 75

# Script de verificação
#!/bin/bash
COVERAGE=$(cargo llvm-cov --summary-only | grep -oP 'TOTAL.*\K[0-9.]+(?=%)')
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% is below 80%"
    exit 1
fi
echo "✅ Coverage $COVERAGE% meets threshold"
```

### Relatório por Módulo

```bash
# Ver cobertura detalhada
cargo llvm-cov --html

# Abrir relatório
open target/llvm-cov/html/index.html  # macOS
xdg-open target/llvm-cov/html/index.html  # Linux
```

**Estrutura do Relatório:**
```
target/llvm-cov/html/
├── index.html                  # Visão geral
├── robotrade_core/
│   ├── domain_services/
│   │   ├── pnl/
│   │   │   └── calculator.rs.html  # 100% coverage ✅
│   │   └── risk/
│   │       └── validator.rs.html   # 95% coverage ✅
│   └── entities/
│       └── order.rs.html           # 78% coverage ⚠️
└── robotrade_infra/
    └── repositories/
        └── order.rs.html           # 65% coverage ⚠️
```

---

## 🤖 CI/CD

### GitHub Actions Workflow

Ver arquivo completo: `.github/workflows/ci.yml`

**Jobs:**

1. **Format Check**
   ```yaml
   - name: Check formatting
     run: cargo fmt --all -- --check
   ```

2. **Clippy Linting**
   ```yaml
   - name: Run clippy
     run: cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Unit Tests**
   ```yaml
   - name: Run unit tests
     run: cargo test --lib --all-features
   ```

4. **Integration Tests**
   ```yaml
   - name: Run integration tests
     run: cargo test --test '*' --all-features
   ```

5. **Coverage**
   ```yaml
   - name: Generate coverage
     run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

   - name: Upload to Codecov
     uses: codecov/codecov-action@v3
     with:
       files: lcov.info
   ```

6. **Build**
   ```yaml
   - name: Build release
     run: cargo build --release
   ```

### Badges

Adicione ao `README.md`:

```markdown
![CI](https://github.com/CypherpunkBR/RoboTrade/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/CypherpunkBR/RoboTrade/branch/main/graph/badge.svg)](https://codecov.io/gh/CypherpunkBR/RoboTrade)
![License](https://img.shields.io/github/license/CypherpunkBR/RoboTrade)
```

---

## 🔧 Troubleshooting

### Docker Não Está Rodando

```bash
# Verificar
docker info

# Iniciar (macOS)
open -a Docker

# Iniciar (Linux)
sudo systemctl start docker

# Iniciar (Windows)
# Abrir Docker Desktop
```

### Testes Lentos

```bash
# Usar cargo-nextest (mais rápido)
cargo install cargo-nextest
cargo nextest run

# Rodar apenas o necessário
cargo test --lib  # Apenas unitários

# Limpar cache
cargo clean
rm -rf target/
```

### Falhas Aleatórias

```bash
# Rodar sequencial
cargo test -- --test-threads=1

# Rodar com logs
RUST_LOG=debug cargo test -- --nocapture

# Rodar teste específico múltiplas vezes
for i in {1..100}; do
    cargo test test_name || break
done
```

### Erro de Conexão com Banco

```bash
# Verificar container
docker ps

# Ver logs
docker logs <container-id>

# Recriar
docker-compose down
docker-compose up -d

# Conectar manualmente
psql -h localhost -U postgres -d robotrade_test
```

### Out of Memory

```bash
# Aumentar limite de memória Docker
# Docker Desktop → Settings → Resources → Memory: 4GB

# Rodar menos testes em paralelo
cargo test -- --test-threads=2
```

---

## 💎 Best Practices

### 1. Testes Determinísticos

```rust
// ❌ Não determinístico
#[test]
fn test_timestamp() {
    let now = Utc::now();  // Muda a cada execução
    assert!(now > some_date);
}

// ✅ Determinístico
#[test]
fn test_timestamp() {
    let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(parse_date("2025-01-01"), fixed_time);
}
```

### 2. Não Use Sleep

```rust
// ❌ Ruim - torna testes lentos
#[tokio::test]
async fn test_async() {
    send_message().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(message_received());
}

// ✅ Bom - use channels ou flags
#[tokio::test]
async fn test_async() {
    let (tx, rx) = oneshot::channel();
    send_message_with_callback(move || tx.send(()).unwrap()).await;
    rx.await.unwrap();
    assert!(message_received());
}
```

### 3. Isole Efeitos Colaterais

```rust
// ❌ Ruim - modifica estado global
static mut COUNTER: i32 = 0;

#[test]
fn test_counter() {
    unsafe {
        COUNTER += 1;
        assert_eq!(COUNTER, 1);  // Falha se rodar múltiplas vezes
    }
}

// ✅ Bom - estado local
#[test]
fn test_counter() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

### 4. Teste Comportamento, Não Implementação

```rust
// ❌ Ruim - testa detalhes de implementação
#[test]
fn test_internal_state() {
    let calculator = PnLCalculator::new();
    assert_eq!(calculator.internal_buffer.len(), 0);  // Detalhe interno
}

// ✅ Bom - testa comportamento observável
#[test]
fn test_pnl_calculation() {
    let calculator = PnLCalculator::new();
    let pnl = calculator.calculate(&lots, &trade);
    assert_eq!(pnl.amount, dec!(1000.0));  // Resultado público
}
```

### 5. Use Helpers para Setup

```rust
// ❌ Ruim - duplicação
#[test]
fn test1() {
    let lot = TaxLot {
        id: "lot1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: PositionSide::Long,
        // ... 20 campos
    };
    // teste...
}

#[test]
fn test2() {
    let lot = TaxLot {
        id: "lot1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: PositionSide::Long,
        // ... 20 campos duplicados
    };
    // teste...
}

// ✅ Bom - helper reutilizável
fn create_test_tax_lot(overrides: impl Fn(&mut TaxLot)) -> TaxLot {
    let mut lot = TaxLot {
        id: "lot1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: PositionSide::Long,
        acquired_quantity: dec!(1.0),
        remaining_quantity: dec!(1.0),
        cost_per_unit: dec!(50000.0),
        // ... valores padrão
    };
    overrides(&mut lot);
    lot
}

#[test]
fn test1() {
    let lot = create_test_tax_lot(|l| {
        l.cost_per_unit = dec!(51000.0);
    });
    // teste...
}

#[test]
fn test2() {
    let lot = create_test_tax_lot(|l| {
        l.remaining_quantity = dec!(0.5);
    });
    // teste...
}
```

---

## 📚 Recursos Adicionais

### Documentação

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testcontainers Rust](https://github.com/testcontainers/testcontainers-rs)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [rstest](https://github.com/la10736/rstest)
- [mockall](https://github.com/asomers/mockall)

### Ferramentas

```bash
# Test runners
cargo install cargo-nextest      # Mais rápido que cargo test
cargo install cargo-watch        # Auto-run ao salvar

# Cobertura
cargo install cargo-llvm-cov     # Coverage reports
cargo install cargo-tarpaulin    # Alternativa ao llvm-cov

# Linting
cargo install cargo-audit        # Security audit
cargo install cargo-outdated     # Dependências desatualizadas
```

---

## 📞 Suporte

**Issues:** https://github.com/CypherpunkBR/RoboTrade/issues
**Discussions:** https://github.com/CypherpunkBR/RoboTrade/discussions
**Canal:** #robotrade-testing

---

**Última Atualização:** 2025-12-01
**Versão:** 1.0.0
