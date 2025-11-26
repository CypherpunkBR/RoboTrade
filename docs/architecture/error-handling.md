# Tratamento de Erros

Este documento descreve a estratégia de tratamento de erros do RoboTrade.

## Princípios

1. **Erros tipados por domínio** - Cada módulo tem seus próprios tipos de erro
2. **Informação sem exposição** - Erros devem ser informativos mas não expor dados sensíveis
3. **Recuperação quando possível** - Retry automático para erros transientes
4. **Logging estruturado** - Todos os erros são logados com contexto
5. **Propagação controlada** - Erros são transformados ao cruzar camadas

## Hierarquia de Erros

```
┌─────────────────────────────────────────────────────────────────┐
│                         AppError                                 │
│                   (Erro de nível aplicação)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ MarketData  │  │  Analytics  │  │   Worker    │             │
│  │   Error     │  │   Error     │  │   Error     │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Exchange   │  │   Infra     │  │   Config    │             │
│  │   Error     │  │   Error     │  │   Error     │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Tipos de Erro por Domínio

### Core Errors (`robotrade-core`)

```rust
use thiserror::Error;

/// Erro de resultado genérico
pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid value for {field}: {message}")]
    ValidationError { field: String, message: String },

    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Operation not allowed: {reason}")]
    NotAllowed { reason: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}
```

### Market Data Errors

```rust
pub type MarketDataResult<T> = Result<T, MarketDataError>;

#[derive(Error, Debug)]
pub enum MarketDataError {
    #[error("Provider {provider} is unavailable: {reason}")]
    ProviderUnavailable { provider: String, reason: String },

    #[error("Rate limit exceeded for {provider}, retry after {retry_after_ms}ms")]
    RateLimitExceeded { provider: String, retry_after_ms: u64 },

    #[error("Invalid response from {provider}: {message}")]
    InvalidResponse { provider: String, message: String },

    #[error("Symbol {symbol} not found on {provider}")]
    SymbolNotFound { symbol: String, provider: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Timeout fetching data from {provider}")]
    Timeout { provider: String },

    #[error("Data normalization failed: {0}")]
    NormalizationError(String),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Infra(#[from] InfraError),
}
```

### Exchange Errors

```rust
pub type ExchangeResult<T> = Result<T, ExchangeError>;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Order rejected: {reason}")]
    OrderRejected { reason: String },

    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    #[error("Position not found: {symbol}")]
    PositionNotFound { symbol: String },

    #[error("Rate limit exceeded, retry after {retry_after_ms}ms")]
    RateLimitExceeded { retry_after_ms: u64 },

    #[error("Exchange API error ({code}): {message}")]
    ApiError { code: i32, message: String },

    #[error("Invalid order parameters: {message}")]
    InvalidOrder { message: String },

    #[error("Symbol {symbol} not tradeable")]
    SymbolNotTradeable { symbol: String },

    #[error("Leverage {leverage} not allowed for {symbol}")]
    InvalidLeverage { symbol: String, leverage: u32 },

    #[error("Network error communicating with exchange")]
    NetworkError(#[from] reqwest::Error),

    #[error("Timeout waiting for exchange response")]
    Timeout,

    #[error("Exchange is in maintenance mode")]
    MaintenanceMode,

    #[error("Paper trading simulation error: {0}")]
    PaperTradingError(String),

    #[error(transparent)]
    Core(#[from] CoreError),
}
```

### Analytics Errors

```rust
pub type AnalyticsResult<T> = Result<T, AnalyticsError>;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Insufficient data for calculation: need {required}, have {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Invalid indicator parameters: {message}")]
    InvalidParameters { message: String },

    #[error("Strategy {strategy_id} not found")]
    StrategyNotFound { strategy_id: String },

    #[error("Strategy configuration invalid: {message}")]
    InvalidStrategyConfig { message: String },

    #[error("Backtest failed: {reason}")]
    BacktestFailed { reason: String },

    #[error("Calculation overflow in {indicator}")]
    CalculationOverflow { indicator: String },

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Infra(#[from] InfraError),
}
```

### Worker Errors

```rust
pub type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Job {job_id} not found")]
    JobNotFound { job_id: String },

    #[error("Job {job_id} already completed")]
    JobAlreadyCompleted { job_id: String },

    #[error("Job execution failed after {retries} retries: {message}")]
    JobExecutionFailed { job_id: String, retries: u32, message: String },

    #[error("Queue is full, max capacity: {max_capacity}")]
    QueueFull { max_capacity: usize },

    #[error("Worker is not running")]
    WorkerNotRunning,

    #[error("Invalid job payload: {message}")]
    InvalidPayload { message: String },

    #[error(transparent)]
    Exchange(#[from] ExchangeError),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Infra(#[from] InfraError),
}
```

### Infra Errors

```rust
pub type InfraResult<T> = Result<T, InfraError>;

#[derive(Error, Debug)]
pub enum InfraError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationError(String),

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: String },

    #[error("Invalid configuration value for {key}: {message}")]
    InvalidConfigValue { key: String, message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Environment variable {var} not set")]
    EnvVarNotSet { var: String },
}
```

## Erro de Aplicação (Tauri)

```rust
pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Initialization failed: {0}")]
    InitializationError(String),

    #[error("State not initialized")]
    StateNotInitialized,

    #[error(transparent)]
    MarketData(#[from] MarketDataError),

    #[error(transparent)]
    Exchange(#[from] ExchangeError),

    #[error(transparent)]
    Analytics(#[from] AnalyticsError),

    #[error(transparent)]
    Worker(#[from] WorkerError),

    #[error(transparent)]
    Infra(#[from] InfraError),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Conversão para String (comandos Tauri retornam Result<T, String>)
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_safe_string()
    }
}

impl AppError {
    /// Converte para string segura (sem dados sensíveis)
    pub fn to_safe_string(&self) -> String {
        match self {
            // Erros que podem conter dados sensíveis
            AppError::Exchange(ExchangeError::AuthenticationError { .. }) => {
                "Authentication failed. Please check your API credentials.".to_string()
            }
            AppError::Infra(InfraError::KeyringError(_)) => {
                "Failed to access secure storage.".to_string()
            }
            // Outros erros podem ser expostos
            _ => self.to_string(),
        }
    }
}
```

## Padrões de Tratamento

### 1. Propagação com Contexto

```rust
use anyhow::{Context, Result};

async fn fetch_and_save_candles(
    provider: &dyn MarketDataProvider,
    repo: &dyn CandleRepository,
    symbol: &str,
) -> Result<()> {
    let candles = provider
        .fetch_candles(symbol, TimeFrame::H1, 100)
        .await
        .context(format!("Failed to fetch candles for {}", symbol))?;

    repo.save_candles(symbol, TimeFrame::H1, &candles)
        .await
        .context("Failed to save candles to database")?;

    Ok(())
}
```

### 2. Retry com Backoff

```rust
use tokio::time::{sleep, Duration};

async fn with_retry<T, E, F, Fut>(
    operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(e);
                }

                tracing::warn!(
                    attempt = attempt + 1,
                    max_retries = max_retries,
                    error = %e,
                    delay_ms = delay.as_millis(),
                    "Operation failed, retrying"
                );

                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}
```

### 3. Erro Recuperável vs Fatal

```rust
impl MarketDataError {
    /// Verifica se o erro é recuperável (pode ser retentado)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            MarketDataError::NetworkError(_)
                | MarketDataError::Timeout { .. }
                | MarketDataError::RateLimitExceeded { .. }
                | MarketDataError::ProviderUnavailable { .. }
        )
    }

    /// Sugere delay antes de retry
    pub fn suggested_retry_delay(&self) -> Option<Duration> {
        match self {
            MarketDataError::RateLimitExceeded { retry_after_ms, .. } => {
                Some(Duration::from_millis(*retry_after_ms))
            }
            MarketDataError::Timeout { .. } => Some(Duration::from_secs(5)),
            MarketDataError::NetworkError(_) => Some(Duration::from_secs(2)),
            _ => None,
        }
    }
}
```

### 4. Logging Estruturado de Erros

```rust
fn log_error<E: std::error::Error>(error: &E, context: &str) {
    tracing::error!(
        error.message = %error,
        error.source = ?error.source(),
        error.chain = ?error_chain(error),
        context = context,
        "Operation failed"
    );
}

fn error_chain<E: std::error::Error>(error: &E) -> Vec<String> {
    let mut chain = vec![error.to_string()];
    let mut current = error.source();
    while let Some(source) = current {
        chain.push(source.to_string());
        current = source.source();
    }
    chain
}
```

## Tratamento em Comandos Tauri

```rust
#[tauri::command]
pub async fn fetch_candles(
    state: tauri::State<'_, AppState>,
    symbol: String,
    timeframe: String,
    limit: usize,
) -> Result<Vec<CandleDto>, String> {
    // Validação de entrada
    let timeframe = TimeFrame::from_str(&timeframe)
        .map_err(|_| format!("Invalid timeframe: {}", timeframe))?;

    // Operação
    let candles = state
        .candle_repo
        .find_candles(&symbol, timeframe, limit)
        .await
        .map_err(|e| {
            tracing::error!(
                symbol = %symbol,
                timeframe = %timeframe,
                error = %e,
                "Failed to fetch candles"
            );
            e.to_string()
        })?;

    // Conversão para DTO
    Ok(candles.into_iter().map(CandleDto::from).collect())
}
```

## Validação de Entrada

```rust
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct OrderRequest {
    #[validate(length(min = 1, max = 20))]
    pub symbol: String,

    pub side: OrderSide,

    pub order_type: OrderType,

    #[validate(custom = "validate_quantity")]
    pub quantity: Decimal,

    #[validate(custom = "validate_price")]
    pub price: Option<Decimal>,
}

fn validate_quantity(quantity: &Decimal) -> Result<(), validator::ValidationError> {
    if *quantity <= Decimal::ZERO {
        return Err(validator::ValidationError::new("quantity_positive"));
    }
    if *quantity > Decimal::from(1_000_000) {
        return Err(validator::ValidationError::new("quantity_max"));
    }
    Ok(())
}

fn validate_price(price: &Decimal) -> Result<(), validator::ValidationError> {
    if *price <= Decimal::ZERO {
        return Err(validator::ValidationError::new("price_positive"));
    }
    Ok(())
}

// Uso
pub fn validate_order_request(request: &OrderRequest) -> CoreResult<()> {
    request.validate().map_err(|e| CoreError::ValidationError {
        field: "order".to_string(),
        message: e.to_string(),
    })
}
```

## Error Boundaries no Frontend

```typescript
// hooks/useCommand.ts
import { invoke } from '@tauri-apps/api/tauri';

interface CommandResult<T> {
  data: T | null;
  error: string | null;
  loading: boolean;
}

export function useCommand<T>(
  command: string,
  args?: Record<string, unknown>
): CommandResult<T> {
  const [result, setResult] = useState<CommandResult<T>>({
    data: null,
    error: null,
    loading: true,
  });

  useEffect(() => {
    invoke<T>(command, args)
      .then((data) => setResult({ data, error: null, loading: false }))
      .catch((error) => setResult({ data: null, error: String(error), loading: false }));
  }, [command, JSON.stringify(args)]);

  return result;
}

// Componente
function CandlesList({ symbol, timeframe }: Props) {
  const { data, error, loading } = useCommand<CandleDto[]>('fetch_candles', {
    symbol,
    timeframe,
    limit: 100,
  });

  if (loading) return <Spinner />;
  if (error) return <ErrorMessage message={error} />;
  if (!data) return <EmptyState />;

  return <CandlesTable candles={data} />;
}
```

## Métricas de Erros

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ErrorMetrics {
    pub market_data_errors: AtomicU64,
    pub exchange_errors: AtomicU64,
    pub worker_errors: AtomicU64,
    pub database_errors: AtomicU64,
}

impl ErrorMetrics {
    pub fn record_error(&self, error: &AppError) {
        match error {
            AppError::MarketData(_) => {
                self.market_data_errors.fetch_add(1, Ordering::Relaxed);
            }
            AppError::Exchange(_) => {
                self.exchange_errors.fetch_add(1, Ordering::Relaxed);
            }
            AppError::Worker(_) => {
                self.worker_errors.fetch_add(1, Ordering::Relaxed);
            }
            AppError::Infra(InfraError::DatabaseError(_)) => {
                self.database_errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn get_summary(&self) -> ErrorSummary {
        ErrorSummary {
            market_data: self.market_data_errors.load(Ordering::Relaxed),
            exchange: self.exchange_errors.load(Ordering::Relaxed),
            worker: self.worker_errors.load(Ordering::Relaxed),
            database: self.database_errors.load(Ordering::Relaxed),
        }
    }
}
```

---

**Próximo**: [Segurança](./security.md)
