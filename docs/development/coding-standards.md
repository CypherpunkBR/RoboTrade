# Coding Standards

Padrões de código e convenções do projeto RoboTrade.

## Formatação

### Rustfmt

Todo código deve ser formatado com `rustfmt`. Configuração em `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
```

Execute antes de cada commit:

```bash
cargo fmt --all
```

### Clippy

Usamos clippy com configurações estritas. Execute:

```bash
cargo clippy -- -D warnings
```

Lints habilitados no `Cargo.toml` do workspace:

```toml
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
# Exceções justificadas
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
```

## Nomenclatura

### Arquivos e Módulos

```
snake_case.rs           # Arquivos
mod market_data;        # Módulos
```

### Tipos

```rust
// Structs: PascalCase
pub struct OrderRequest { ... }
pub struct BinanceGateway { ... }

// Enums: PascalCase, variantes também
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

// Traits: PascalCase, geralmente adjetivos ou verbos
pub trait Executable { ... }
pub trait Repository { ... }
```

### Funções e Variáveis

```rust
// Funções: snake_case
pub fn calculate_sma(prices: &[Decimal]) -> Decimal { ... }
pub async fn fetch_candles(symbol: &str) -> Result<Vec<Candlestick>> { ... }

// Variáveis: snake_case
let order_count = 0;
let mut price_buffer = Vec::new();

// Constantes: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

### Lifetimes

```rust
// Nomes descritivos quando não óbvio
fn process<'request, 'config>(
    req: &'request Request,
    cfg: &'config Config,
) -> &'request str { ... }

// 'a quando óbvio
fn first<'a>(items: &'a [Item]) -> &'a Item { ... }
```

## Organização de Código

### Estrutura de Módulo

```rust
// lib.rs ou mod.rs

// 1. Módulos públicos primeiro
pub mod types;
pub mod traits;
pub mod error;

// 2. Módulos internos
mod internal;

// 3. Re-exports para API pública conveniente
pub use types::{Order, Position, Candlestick};
pub use traits::ExchangeGateway;
pub use error::{Error, Result};

// 4. Prelude opcional para imports comuns
pub mod prelude {
    pub use crate::{Order, Position, Error, Result};
}
```

### Estrutura de Arquivo

```rust
// 1. Documentação do módulo
//! # Module Name
//!
//! Descrição do módulo e exemplos de uso.

// 2. Imports agrupados
use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{Error, Result};
use crate::types::Order;

// 3. Constantes
const MAX_BATCH_SIZE: usize = 100;

// 4. Type aliases
type OrderMap = HashMap<String, Order>;

// 5. Structs
#[derive(Debug, Clone)]
pub struct OrderManager {
    orders: Arc<RwLock<OrderMap>>,
}

// 6. Implementações
impl OrderManager {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for OrderManager {
    fn default() -> Self {
        Self::new()
    }
}

// 7. Traits implementations
impl std::fmt::Display for OrderManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OrderManager")
    }
}

// 8. Testes no final
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_manager_creation() {
        let manager = OrderManager::new();
        // ...
    }
}
```

## Documentação

### Doc Comments

```rust
/// Calcula a média móvel simples de uma série de preços.
///
/// # Arguments
///
/// * `prices` - Slice de preços ordenados do mais antigo ao mais recente
/// * `period` - Número de períodos para a média
///
/// # Returns
///
/// Retorna `Some(Decimal)` com a média se houver dados suficientes,
/// ou `None` se `prices.len() < period`.
///
/// # Examples
///
/// ```
/// use robotrade_analytics::calculate_sma;
/// use rust_decimal_macros::dec;
///
/// let prices = vec![dec!(100), dec!(102), dec!(104), dec!(103), dec!(105)];
/// let sma = calculate_sma(&prices, 3);
/// assert_eq!(sma, Some(dec!(104)));
/// ```
///
/// # Panics
///
/// Não faz panic em nenhuma condição.
pub fn calculate_sma(prices: &[Decimal], period: usize) -> Option<Decimal> {
    if prices.len() < period {
        return None;
    }

    let sum: Decimal = prices.iter().rev().take(period).sum();
    Some(sum / Decimal::from(period))
}
```

### Módulos e Crates

```rust
//! # RoboTrade Analytics
//!
//! Biblioteca de indicadores técnicos e análise de mercado.
//!
//! ## Features
//!
//! - Indicadores de tendência (SMA, EMA, MACD)
//! - Indicadores de momento (RSI, Stochastic)
//! - Indicadores de volatilidade (Bollinger Bands, ATR)
//!
//! ## Quick Start
//!
//! ```rust
//! use robotrade_analytics::indicators::{Sma, Indicator};
//!
//! let mut sma = Sma::new(20);
//! sma.update(100.0);
//! // ...
//! ```
```

## Error Handling

### Tipos de Erro

```rust
use thiserror::Error;

/// Erros do módulo de market data
#[derive(Error, Debug)]
pub enum MarketDataError {
    #[error("Failed to fetch candles for {symbol}: {source}")]
    FetchError {
        symbol: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimitExceeded { retry_after_secs: u64 },

    #[error("Invalid timeframe: {0}")]
    InvalidTimeframe(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

// Type alias para Result
pub type Result<T> = std::result::Result<T, MarketDataError>;
```

### Propagação

```rust
// Prefer ? operator
pub async fn get_candles(&self, symbol: &str) -> Result<Vec<Candlestick>> {
    let raw = self.client
        .fetch(symbol)
        .await
        .map_err(|e| MarketDataError::FetchError {
            symbol: symbol.to_string(),
            source: e,
        })?;

    let candles = parse_candles(raw)?;

    self.cache.store(&candles).await?;

    Ok(candles)
}

// Evite .unwrap() em código de produção
// Use .expect() com mensagem clara quando garantido
let config = Config::load()
    .expect("Config file must exist and be valid");
```

### Logging de Erros

```rust
pub async fn process_order(&self, order: Order) -> Result<()> {
    match self.execute(order).await {
        Ok(result) => {
            tracing::info!(
                order_id = %order.id,
                "Order executed successfully"
            );
            Ok(result)
        }
        Err(e) => {
            tracing::error!(
                order_id = %order.id,
                error = %e,
                "Failed to execute order"
            );
            Err(e)
        }
    }
}
```

## Async/Await

### Funções Async

```rust
// Prefira async fn quando possível
pub async fn fetch_data(&self) -> Result<Data> {
    self.client.get().await
}

// Use impl Future para casos complexos
pub fn complex_operation(&self) -> impl Future<Output = Result<Data>> + '_ {
    async move {
        // ...
    }
}
```

### Concorrência

```rust
use tokio::join;
use futures::future::try_join_all;

// Operações independentes em paralelo
pub async fn fetch_all_symbols(&self, symbols: &[String]) -> Result<Vec<Data>> {
    let futures: Vec<_> = symbols
        .iter()
        .map(|s| self.fetch_symbol(s))
        .collect();

    try_join_all(futures).await
}

// Join de operações diferentes
pub async fn initialize(&self) -> Result<()> {
    let (config, db, cache) = join!(
        self.load_config(),
        self.connect_db(),
        self.init_cache(),
    );

    config?;
    db?;
    cache?;

    Ok(())
}
```

### Cancelamento

```rust
use tokio::select;
use tokio_util::sync::CancellationToken;

pub async fn run_with_cancel(&self, cancel: CancellationToken) -> Result<()> {
    loop {
        select! {
            _ = cancel.cancelled() => {
                tracing::info!("Operation cancelled");
                break;
            }
            result = self.do_work() => {
                result?;
            }
        }
    }
    Ok(())
}
```

## Testes

### Estrutura

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Helper functions no topo
    fn create_test_order() -> Order {
        Order {
            id: "test-123".to_string(),
            // ...
        }
    }

    // Testes agrupados por funcionalidade
    mod order_validation {
        use super::*;

        #[test]
        fn rejects_negative_quantity() {
            let order = Order {
                quantity: dec!(-1),
                ..create_test_order()
            };

            assert!(matches!(
                validate_order(&order),
                Err(ValidationError::NegativeQuantity)
            ));
        }

        #[test]
        fn accepts_valid_order() {
            let order = create_test_order();
            assert!(validate_order(&order).is_ok());
        }
    }

    mod order_execution {
        use super::*;

        #[tokio::test]
        async fn executes_market_order() {
            // ...
        }
    }
}
```

### Nomes de Teste

```rust
// Formato: test_<função>_<cenário>_<resultado_esperado>
#[test]
fn calculate_sma_with_insufficient_data_returns_none() { ... }

#[test]
fn validate_order_with_negative_price_returns_error() { ... }

#[tokio::test]
async fn fetch_candles_with_invalid_symbol_returns_not_found() { ... }
```

## Performance

### Alocações

```rust
// Evite alocações desnecessárias
// Ruim
fn process(items: Vec<String>) -> Vec<String> {
    items.into_iter().map(|s| s.to_uppercase()).collect()
}

// Bom - reusa alocação
fn process(items: &mut [String]) {
    for item in items {
        item.make_ascii_uppercase();
    }
}

// Use &str quando não precisa ownership
fn validate(name: &str) -> bool { ... }

// Use Cow para flexibilidade
use std::borrow::Cow;
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)
    }
}
```

### Clone

```rust
// Evite .clone() desnecessário
// Ruim
let orders: Vec<Order> = self.orders.clone();
for order in orders { ... }

// Bom - empresta referência
for order in &self.orders { ... }

// Use Arc para compartilhamento
use std::sync::Arc;
let shared_data = Arc::new(expensive_data);
let cloned = Arc::clone(&shared_data);  // Cheap!
```

## Segurança

### Dados Sensíveis

```rust
// Nunca log API keys ou secrets
tracing::info!(
    api_key = "[REDACTED]",  // Não logue o valor real
    "Connecting to exchange"
);

// Use tipos que não implementam Debug/Display
pub struct ApiSecret(String);

impl ApiSecret {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

// Não derive Debug para tipos com secrets
// #[derive(Debug)]  // NÃO!
pub struct Credentials {
    api_key: String,
    secret: ApiSecret,
}
```

### Validação de Input

```rust
// Sempre valide input externo
pub fn process_symbol(symbol: &str) -> Result<Symbol> {
    // Sanitize
    let symbol = symbol.trim().to_uppercase();

    // Validate format
    if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(Error::InvalidSymbol(symbol));
    }

    // Validate length
    if symbol.len() > 20 {
        return Err(Error::SymbolTooLong(symbol));
    }

    Ok(Symbol(symbol))
}
```

## Checklist de Code Review

- [ ] Código formatado com `cargo fmt`
- [ ] Sem warnings de `cargo clippy`
- [ ] Testes passando
- [ ] Documentação para APIs públicas
- [ ] Erros tratados adequadamente (sem unwrap em produção)
- [ ] Logs estruturados com contexto
- [ ] Sem secrets hardcoded ou em logs
- [ ] Performance considerada (alocações, clones)
- [ ] Async/await usado corretamente
- [ ] Cancelamento graceful implementado onde aplicável
