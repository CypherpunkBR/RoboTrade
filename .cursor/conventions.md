# Convenções de Código - RoboTrade

## Estilo Rust

### Formatação

```rust
// 2 espaços de indentação
pub fn my_function() {
  let value = 10;
  
  if value > 5 {
    println!("maior");
  }
}

// 100 caracteres por linha (máximo)
pub fn long_function_with_many_params(
  param1: String,
  param2: i32,
  param3: Decimal,
) -> Result<Value> {
  // ...
}
```

### Naming

```rust
// snake_case para funções e variáveis
fn calculate_profit_and_loss() -> Decimal {}
let total_balance = get_balance();

// PascalCase para structs e enums
struct TradingWorker {}
enum OrderType { Market, Limit }

// SCREAMING_SNAKE_CASE para constantes
const MAX_LEVERAGE: i32 = 125;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

### Imports

```rust
// Ordem: std → external → internal
use std::sync::Arc;
use std::collections::HashMap;

use tokio::time::Duration;
use serde::{Serialize, Deserialize};

use robotrade_core::entities::Order;
use crate::utils::format_price;
```

## Padrões de Código

### Result e Error Handling

```rust
// ✅ Sempre retorna Result em funções que podem falhar
pub async fn fetch_data() -> Result<Data, Error> {
    let response = http_client.get(url).await?;  // Propaga erro
    Ok(response.json().await?)
}

// ❌ NUNCA use unwrap() em produção
let value = may_fail().unwrap();  // PANIC se der erro!

// ✅ Use ? ou match
let value = may_fail()?;  // Retorna erro pro caller

// Ou handle explicitamente
let value = match may_fail() {
    Ok(v) => v,
    Err(e) => {
        error!("Falha: {}", e);
        return Err(e);
    }
};
```

### Decimal para Valores Financeiros

```rust
use rust_decimal::Decimal;
use std::str::FromStr;

// ✅ Sempre use Decimal
let price = Decimal::from_str("50123.456789")?;
let quantity = Decimal::from_str("0.1")?;
let total = price * quantity;

// ❌ NUNCA use f64 para dinheiro
let price: f64 = 50123.456789;  // IMPRECISÃO!
```

### Async Functions

```rust
// ✅ Async quando faz I/O
pub async fn get_balance() -> Result<Vec<Balance>> {
    self.http_client.get(url).await?
}

// ✅ Sync quando só processa
pub fn calculate_pnl(entry: Decimal, exit: Decimal) -> Decimal {
    exit - entry
}

// ✅ Use #[tokio::test] em testes async
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Logging

```rust
use tracing::{info, warn, error, debug};

// ✅ Níveis apropriados
info!("Ordem criada: {}", order_id);        // Informação normal
warn!("Saldo baixo: {}", balance);          // Atenção
error!("Falha ao conectar: {}", e);         // Erro crítico
debug!("Dados recebidos: {:?}", data);      // Debug

// ✅ Contexto suficiente
info!("Processing order {} for {}", order.id, order.symbol);

// ❌ Logging demais
debug!("Variável x: {}", x);  // Desnecessário
```

## Padrões Específicos do Projeto

### Símbolos sempre UPPERCASE

```rust
// ✅ Correto
let symbol = "BTCUSDT";

// ❌ Errado
let symbol = "btcusdt";  // Binance rejeita!
```

### Timestamps em UTC

```rust
use chrono::{DateTime, Utc};

// ✅ Sempre UTC
let now = Utc::now();
let timestamp: DateTime<Utc> = ...;

// ❌ NUNCA use local time
let now = Local::now();  // Bugs de timezone!
```

### Validação de Entrada

```rust
// ✅ Valida antes de usar
pub async fn set_leverage(symbol: &str, leverage: i32) -> Result<()> {
    // Valida símbolo
    if symbol.is_empty() {
        return Err(Error::InvalidSymbol);
    }
    
    // Valida leverage
    if leverage < 1 || leverage > 125 {
        return Err(Error::InvalidLeverage(leverage));
    }
    
    // Agora pode usar
    self.send_request(symbol, leverage).await
}
```

### Documentation

```rust
/// Calcula P&L realizado de um trade.
///
/// # Arguments
///
/// * `entry_price` - Preço de entrada
/// * `exit_price` - Preço de saída
/// * `quantity` - Quantidade tradada
/// * `side` - Long ou Short
///
/// # Returns
///
/// P&L em Decimal (positivo = lucro, negativo = prejuízo)
///
/// # Examples
///
/// ```
/// let pnl = calculate_pnl(
///     Decimal::from(50000),
///     Decimal::from(51000),
///     Decimal::from_str("0.1")?,
///     PositionSide::Long,
/// );
/// assert_eq!(pnl, Decimal::from(100));  // $1000 * 0.1 = $100
/// ```
pub fn calculate_pnl(
    entry_price: Decimal,
    exit_price: Decimal,
    quantity: Decimal,
    side: PositionSide,
) -> Decimal {
    let price_diff = match side {
        PositionSide::Long => exit_price - entry_price,
        PositionSide::Short => entry_price - exit_price,
    };
    
    price_diff * quantity
}
```

## TypeScript/React

### Components

```typescript
// PascalCase, um componente por arquivo
export function TradingChart({ symbol }: { symbol: string }) {
  const [data, setData] = useState<ChartData | null>(null);
  
  // Hooks no topo
  useEffect(() => {
    // ...
  }, [symbol]);
  
  // Handlers depois
  const handleClick = (price: number) => {
    // ...
  };
  
  // Render no final
  return (
    <div className="chart-container">
      {/* ... */}
    </div>
  );
}
```

### Types

```typescript
// Interfaces para shapes
interface OrderRequest {
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
}

// Types para unions
type OrderStatus = 'pending' | 'filled' | 'cancelled';

// Enums quando faz sentido
enum OrderType {
  Market = 'market',
  Limit = 'limit',
  Stop = 'stop',
}
```

### Async/Await

```typescript
// ✅ Try/catch em async
async function createOrder(request: OrderRequest) {
  try {
    const result = await invoke<Order>('create_order', { request });
    return result;
  } catch (error) {
    console.error('Falha ao criar ordem:', error);
    throw error;
  }
}

// ✅ Loading states
const [loading, setLoading] = useState(false);

async function handleSubmit() {
  setLoading(true);
  try {
    await createOrder(order);
  } finally {
    setLoading(false);
  }
}
```

## Database

### Queries Type-Safe

```rust
// ✅ Use sqlx macros (compile-time checked)
let trades = sqlx::query_as!(
    Trade,
    "SELECT * FROM trades WHERE symbol = ?",
    symbol
)
.fetch_all(&pool)
.await?;

// ❌ Evite queries string soltas
let result = sqlx::query("SELECT * FROM trades")
    .fetch_all(&pool)
    .await?;
```

### Transactions

```rust
// ✅ Use transaction quando precisa atomicidade
let mut tx = pool.begin().await?;

sqlx::query!("INSERT INTO orders ...")
    .execute(&mut *tx)
    .await?;

sqlx::query!("UPDATE positions ...")
    .execute(&mut *tx)
    .await?;

tx.commit().await?;  // Só comita se tudo deu certo
```

## Testes

### Arrange-Act-Assert

```rust
#[test]
fn test_calculate_pnl() {
    // Arrange
    let entry = Decimal::from(50000);
    let exit = Decimal::from(51000);
    let quantity = Decimal::from_str("0.1").unwrap();
    
    // Act
    let pnl = calculate_pnl(entry, exit, quantity, PositionSide::Long);
    
    // Assert
    assert_eq!(pnl, Decimal::from(100));
}
```

### Naming de Testes

```rust
// Formato: test_{o_que_testa}_{condição}_{resultado_esperado}

#[test]
fn test_set_leverage_with_invalid_value_returns_error() {
    // ...
}

#[test]
fn test_calculate_pnl_for_long_position_returns_positive() {
    // ...
}
```

### Mocks

```rust
// Use traits pra poder mockar
struct MockExchange {
    should_fail: bool,
}

#[async_trait]
impl ExchangeGateway for MockExchange {
    async fn place_order(&self, order: Order) -> Result<Order> {
        if self.should_fail {
            Err(Error::NetworkError)
        } else {
            Ok(order)
        }
    }
}
```

---

**Segue essas convenções e o código fica limpo e consistente! ✨**
