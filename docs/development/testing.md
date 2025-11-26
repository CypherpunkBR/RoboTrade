# Testing Guide

Guia completo de testes para o RoboTrade.

## Filosofia de Testes

O RoboTrade segue uma pirâmide de testes com foco em:

1. **Testes Unitários** (maioria): Testam funções e módulos isolados
2. **Testes de Integração**: Testam interação entre componentes
3. **Testes E2E**: Testam fluxos completos (menos frequentes)

```
        ╱╲
       ╱  ╲
      ╱ E2E╲         ← Poucos, lentos, alto valor
     ╱──────╲
    ╱        ╲
   ╱Integration╲     ← Moderados, testam integração
  ╱────────────╲
 ╱              ╲
╱  Unit Tests    ╲   ← Muitos, rápidos, isolados
╲________________╱
```

## Executando Testes

### Comandos Básicos

```bash
# Todos os testes
cargo test

# Testes de um crate específico
cargo test -p robotrade-analytics

# Teste específico
cargo test calculate_sma

# Com output (println! visível)
cargo test -- --nocapture

# Testes ignorados
cargo test -- --ignored

# Testes em paralelo (padrão) vs sequencial
cargo test -- --test-threads=1
```

### Filtragem

```bash
# Testes que contêm "order"
cargo test order

# Testes de um módulo
cargo test indicators::sma

# Excluir testes
cargo test -- --skip slow_test
```

### Cobertura

```bash
# Instale tarpaulin
cargo install cargo-tarpaulin

# Gere relatório HTML
cargo tarpaulin --out Html --output-dir coverage

# Com threshold mínimo
cargo tarpaulin --fail-under 80
```

## Testes Unitários

### Estrutura Básica

```rust
// Em src/indicators/sma.rs

pub fn calculate_sma(prices: &[Decimal], period: usize) -> Option<Decimal> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let sum: Decimal = prices.iter().rev().take(period).sum();
    Some(sum / Decimal::from(period))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn returns_none_for_insufficient_data() {
        let prices = vec![dec!(100), dec!(101)];
        assert_eq!(calculate_sma(&prices, 5), None);
    }

    #[test]
    fn returns_none_for_zero_period() {
        let prices = vec![dec!(100), dec!(101), dec!(102)];
        assert_eq!(calculate_sma(&prices, 0), None);
    }

    #[test]
    fn calculates_correct_average() {
        let prices = vec![dec!(100), dec!(102), dec!(104)];
        let result = calculate_sma(&prices, 3);
        assert_eq!(result, Some(dec!(102)));
    }

    #[test]
    fn uses_most_recent_values() {
        let prices = vec![dec!(90), dec!(100), dec!(102), dec!(104)];
        // SMA(3) deve usar [100, 102, 104], não incluir 90
        let result = calculate_sma(&prices, 3);
        assert_eq!(result, Some(dec!(102)));
    }
}
```

### Testando Erros

```rust
use crate::error::ValidationError;

pub fn validate_quantity(qty: Decimal) -> Result<(), ValidationError> {
    if qty <= Decimal::ZERO {
        return Err(ValidationError::NonPositiveQuantity(qty));
    }
    if qty > dec!(1000000) {
        return Err(ValidationError::QuantityTooLarge(qty));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_quantity() {
        let result = validate_quantity(dec!(0));
        assert!(matches!(
            result,
            Err(ValidationError::NonPositiveQuantity(_))
        ));
    }

    #[test]
    fn rejects_negative_quantity() {
        let result = validate_quantity(dec!(-5));
        assert!(matches!(
            result,
            Err(ValidationError::NonPositiveQuantity(qty)) if qty == dec!(-5)
        ));
    }

    #[test]
    fn rejects_excessive_quantity() {
        let result = validate_quantity(dec!(2000000));
        assert!(matches!(
            result,
            Err(ValidationError::QuantityTooLarge(_))
        ));
    }

    #[test]
    fn accepts_valid_quantity() {
        assert!(validate_quantity(dec!(100)).is_ok());
        assert!(validate_quantity(dec!(0.001)).is_ok());
        assert!(validate_quantity(dec!(999999)).is_ok());
    }
}
```

### Testando Async

```rust
use tokio;

pub async fn fetch_price(symbol: &str) -> Result<Decimal, Error> {
    // ... implementação
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetches_valid_symbol() {
        let price = fetch_price("BTCUSDT").await;
        assert!(price.is_ok());
        assert!(price.unwrap() > dec!(0));
    }

    #[tokio::test]
    async fn returns_error_for_invalid_symbol() {
        let result = fetch_price("INVALID").await;
        assert!(matches!(result, Err(Error::SymbolNotFound(_))));
    }
}
```

## Mocks e Test Doubles

### Trait Mocking com mockall

```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait ExchangeGateway: Send + Sync {
    async fn get_price(&self, symbol: &str) -> Result<Decimal, Error>;
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn strategy_places_order_on_signal() {
        let mut mock_gateway = MockExchangeGateway::new();

        // Configure expectativas
        mock_gateway
            .expect_get_price()
            .with(eq("BTCUSDT"))
            .times(1)
            .returning(|_| Ok(dec!(50000)));

        mock_gateway
            .expect_place_order()
            .times(1)
            .returning(|_| Ok(OrderResponse::filled()));

        let strategy = Strategy::new(Box::new(mock_gateway));
        let result = strategy.execute().await;

        assert!(result.is_ok());
    }
}
```

### Test Doubles Manuais

```rust
/// Gateway de teste que simula comportamento
pub struct TestGateway {
    prices: HashMap<String, Decimal>,
    orders: Vec<OrderRequest>,
    should_fail: bool,
}

impl TestGateway {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
            orders: Vec::new(),
            should_fail: false,
        }
    }

    pub fn with_price(mut self, symbol: &str, price: Decimal) -> Self {
        self.prices.insert(symbol.to_string(), price);
        self
    }

    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn orders(&self) -> &[OrderRequest] {
        &self.orders
    }
}

#[async_trait]
impl ExchangeGateway for TestGateway {
    async fn get_price(&self, symbol: &str) -> Result<Decimal, Error> {
        if self.should_fail {
            return Err(Error::ConnectionFailed);
        }
        self.prices
            .get(symbol)
            .copied()
            .ok_or(Error::SymbolNotFound(symbol.to_string()))
    }

    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, Error> {
        if self.should_fail {
            return Err(Error::OrderRejected);
        }
        // Armazena para verificação posterior
        // (necessitaria interior mutability em código real)
        Ok(OrderResponse::filled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handles_gateway_failure() {
        let gateway = TestGateway::new().failing();
        let strategy = Strategy::new(Box::new(gateway));

        let result = strategy.execute().await;

        assert!(matches!(result, Err(Error::ConnectionFailed)));
    }
}
```

## Testes de Integração

### Estrutura

```
tests/
├── common/
│   └── mod.rs          # Helpers compartilhados
├── market_data_test.rs
├── order_execution_test.rs
└── backtest_test.rs
```

### Helper Module

```rust
// tests/common/mod.rs

use robotrade_infra::database::Database;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn setup_test_db() -> Database {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    sqlx::migrate!("./crates/infra/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    Database::new(pool)
}

pub fn test_candles() -> Vec<Candlestick> {
    vec![
        Candlestick {
            open_time: Utc::now() - Duration::hours(2),
            open: dec!(100),
            high: dec!(105),
            low: dec!(99),
            close: dec!(103),
            volume: dec!(1000),
            ..Default::default()
        },
        // ... mais candles
    ]
}
```

### Teste de Integração

```rust
// tests/market_data_test.rs

mod common;

use common::setup_test_db;
use robotrade_market_data::MarketDataService;

#[tokio::test]
async fn stores_and_retrieves_candles() {
    let db = setup_test_db().await;
    let service = MarketDataService::new(db);

    let candles = common::test_candles();

    // Store
    service.store_candles("BTCUSDT", &candles).await.unwrap();

    // Retrieve
    let retrieved = service
        .get_candles("BTCUSDT", 100)
        .await
        .unwrap();

    assert_eq!(retrieved.len(), candles.len());
    assert_eq!(retrieved[0].close, candles[0].close);
}

#[tokio::test]
async fn handles_concurrent_writes() {
    let db = setup_test_db().await;
    let service = Arc::new(MarketDataService::new(db));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let svc = Arc::clone(&service);
            tokio::spawn(async move {
                let candles = common::test_candles();
                svc.store_candles(&format!("SYM{}", i), &candles).await
            })
        })
        .collect();

    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## Property-Based Testing

### Com proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn sma_always_within_range(
        prices in prop::collection::vec(1.0f64..1000.0, 5..100),
        period in 1usize..=5
    ) {
        let prices: Vec<Decimal> = prices
            .iter()
            .map(|&p| Decimal::from_f64(p).unwrap())
            .collect();

        if let Some(sma) = calculate_sma(&prices, period) {
            let min = prices.iter().min().unwrap();
            let max = prices.iter().max().unwrap();

            prop_assert!(sma >= *min);
            prop_assert!(sma <= *max);
        }
    }

    #[test]
    fn order_validation_never_panics(
        quantity in any::<f64>(),
        price in any::<f64>()
    ) {
        let qty = Decimal::from_f64(quantity).unwrap_or_default();
        let prc = Decimal::from_f64(price).unwrap_or_default();

        // Não deve panic, apenas retornar Ok ou Err
        let _ = validate_order(qty, prc);
    }
}
```

## Snapshot Testing

### Com insta

```rust
use insta::assert_json_snapshot;

#[test]
fn order_response_serialization() {
    let response = OrderResponse {
        order_id: "12345".to_string(),
        status: OrderStatus::Filled,
        filled_quantity: dec!(1.5),
        average_price: dec!(50000),
        fees: dec!(0.001),
    };

    assert_json_snapshot!(response);
}

// Gera arquivo de snapshot em tests/snapshots/
// Rode `cargo insta review` para aprovar mudanças
```

## Benchmarks

### Com criterion

```rust
// benches/indicators.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use robotrade_analytics::indicators::calculate_sma;

fn benchmark_sma(c: &mut Criterion) {
    let prices: Vec<Decimal> = (0..1000)
        .map(|i| Decimal::from(i))
        .collect();

    c.bench_function("sma_1000_period_20", |b| {
        b.iter(|| calculate_sma(black_box(&prices), black_box(20)))
    });
}

fn benchmark_sma_various_periods(c: &mut Criterion) {
    let prices: Vec<Decimal> = (0..1000)
        .map(|i| Decimal::from(i))
        .collect();

    let mut group = c.benchmark_group("SMA");

    for period in [5, 20, 50, 100, 200] {
        group.bench_with_input(
            format!("period_{}", period),
            &period,
            |b, &period| {
                b.iter(|| calculate_sma(black_box(&prices), black_box(period)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_sma, benchmark_sma_various_periods);
criterion_main!(benches);
```

Execute com:

```bash
cargo bench
```

## Testes de Performance

### Testes com Timeout

```rust
#[tokio::test]
#[timeout(1000)]  // Falha se demorar mais de 1 segundo
async fn should_complete_quickly() {
    let result = fast_operation().await;
    assert!(result.is_ok());
}
```

### Testes de Carga

```rust
#[tokio::test]
async fn handles_high_throughput() {
    let service = Arc::new(OrderService::new());

    let start = Instant::now();
    let order_count = 10000;

    let handles: Vec<_> = (0..order_count)
        .map(|i| {
            let svc = Arc::clone(&service);
            tokio::spawn(async move {
                svc.process_order(create_test_order(i)).await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;
    let duration = start.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();

    assert!(
        success_count as f64 / order_count as f64 > 0.99,
        "Success rate below 99%"
    );

    assert!(
        duration.as_secs() < 10,
        "Processing took too long: {:?}",
        duration
    );

    println!(
        "Processed {} orders in {:?} ({:.0} orders/sec)",
        order_count,
        duration,
        order_count as f64 / duration.as_secs_f64()
    );
}
```

## CI/CD Testing

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt --all -- --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Boas Práticas

### Nomenclatura de Testes

```rust
// Formato: test_<função>_<cenário>_<resultado>

#[test]
fn validate_order_with_negative_quantity_returns_error() { ... }

#[test]
fn calculate_sma_with_exact_period_data_returns_average() { ... }

#[tokio::test]
async fn fetch_candles_when_api_unavailable_returns_cached() { ... }
```

### Isolamento

```rust
// Cada teste deve ser independente
// Evite estado compartilhado entre testes

#[test]
fn test_a() {
    let service = create_fresh_service();
    // ...
}

#[test]
fn test_b() {
    let service = create_fresh_service();  // Nova instância!
    // ...
}
```

### Assertions Claras

```rust
// Ruim
assert!(result.is_ok());
assert!(value > 0);

// Bom
assert!(result.is_ok(), "Expected Ok, got {:?}", result);
assert!(value > 0, "Expected positive value, got {}", value);

// Melhor - use pretty_assertions para diffs claros
use pretty_assertions::assert_eq;
assert_eq!(actual, expected);
```

### Testes Determinísticos

```rust
// Evite depender de tempo real
// Ruim
let now = Utc::now();

// Bom - injete o tempo
fn process_with_time(data: Data, now: DateTime<Utc>) { ... }

#[test]
fn test_process() {
    let fixed_time = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
    process_with_time(data, fixed_time);
}
```
