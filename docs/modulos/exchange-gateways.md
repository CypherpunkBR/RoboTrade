# Módulo Exchange Gateways

O crate `robotrade-exchange-gateways` fornece a integração com exchanges de criptomoedas, abstraindo as diferenças entre APIs para uma interface unificada.

## Estrutura de Diretórios

```
crates/exchange_gateways/src/
├── lib.rs              # Exportações e re-exportações
└── binance/            # Cliente Binance Futures
    ├── mod.rs          # Módulo principal
    ├── client.rs       # BinanceFuturesClient
    ├── models.rs       # Structs de request/response
    └── signer.rs       # Assinatura HMAC-SHA256
```

## Binance Futures Client

### Configuração

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;

// Testnet (desenvolvimento)
let client = BinanceFuturesClient::testnet(
    "api_key".to_string(),
    "api_secret".to_string(),
);

// Mainnet (produção)
let client = BinanceFuturesClient::mainnet(
    "api_key".to_string(),
    "api_secret".to_string(),
);
```

### URLs de API

| Ambiente | URL Base |
|----------|----------|
| Mainnet | `https://fapi.binance.com` |
| Testnet | `https://testnet.binancefuture.com` |

### Métodos Implementados

#### Verificação de Conexão

```rust
/// Verifica se a API está acessível
async fn ping(&self) -> ExchangeResult<bool>;

/// Obtém informações da conta
async fn get_account_info(&self) -> ExchangeResult<AccountInfo>;
```

#### Gestão de Ordens

```rust
/// Envia uma ordem para a exchange
async fn submit_order(&self, request: OrderRequest) -> ExchangeResult<Order>;

/// Cancela uma ordem existente
async fn cancel_order(&self, order_id: &OrderId) -> ExchangeResult<bool>;

/// Obtém status de uma ordem
async fn get_order(&self, order_id: &OrderId) -> ExchangeResult<Option<Order>>;

/// Lista ordens abertas
async fn get_open_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<Order>>;
```

#### Posições e Saldos

```rust
/// Obtém posições abertas
async fn get_positions(&self) -> ExchangeResult<Vec<Position>>;

/// Obtém saldos da conta
async fn get_balances(&self) -> ExchangeResult<HashMap<String, Decimal>>;

/// Define alavancagem para um símbolo
async fn set_leverage(&self, symbol: &str, leverage: u32) -> ExchangeResult<()>;
```

### Assinatura de Requests

Requests autenticados usam HMAC-SHA256:

```rust
// signer.rs
pub fn sign(api_secret: &str, query_string: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .expect("HMAC aceita chave de qualquer tamanho");
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

### Exemplo de Uso

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;
use robotrade_core::entities::{OrderRequest, OrderSide, OrderType};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceFuturesClient::testnet(
        std::env::var("BINANCE_API_KEY")?,
        std::env::var("BINANCE_API_SECRET")?,
    );

    // Verificar conexão
    if client.ping().await? {
        println!("Conectado à Binance!");
    }

    // Obter posições abertas
    let positions = client.get_positions().await?;
    for pos in positions {
        println!("{}: {} @ {}", pos.symbol, pos.quantity, pos.entry_price);
    }

    // Criar ordem de mercado
    let request = OrderRequest::market(
        "BTCUSDT",
        OrderSide::Buy,
        dec!(0.001),
    );

    let order = client.submit_order(request).await?;
    println!("Ordem criada: {}", order.id);

    Ok(())
}
```

### Modelos de Request/Response

```rust
/// Resposta de ordem da Binance
#[derive(Deserialize)]
pub struct BinanceOrderResponse {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: String,
    pub price: String,
    pub avg_price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cum_quote: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    pub stop_price: Option<String>,
    pub working_type: Option<String>,
    pub update_time: i64,
}

/// Resposta de posição da Binance
#[derive(Deserialize)]
pub struct BinancePositionResponse {
    pub symbol: String,
    pub position_amt: String,
    pub entry_price: String,
    pub mark_price: String,
    pub unrealized_profit: String,
    pub leverage: String,
    pub margin_type: String,
    pub isolated_margin: String,
    pub position_side: String,
}
```

### Configurações Padrão

| Parâmetro | Valor | Descrição |
|-----------|-------|-----------|
| Timeout | 30s | Tempo máximo para requests |
| Recv Window | 5000ms | Janela de recebimento |
| Retry | 3x | Tentativas em caso de erro |

### Tratamento de Erros

```rust
pub enum ExchangeError {
    /// Falha na conexão
    ConnectionFailed { exchange: ExchangeId, reason: String },

    /// Rate limit atingido
    RateLimited { exchange: ExchangeId, retry_after: Duration },

    /// Credenciais inválidas
    InvalidCredentials { exchange: ExchangeId },

    /// Saldo insuficiente
    InsufficientFunds { symbol: String, required: Decimal },

    /// Ordem não encontrada
    OrderNotFound { order_id: String },

    /// Símbolo inválido
    InvalidSymbol { symbol: String },

    /// Erro de assinatura
    SignatureError { message: String },
}
```

## Exchanges Planejadas

| Exchange | Status | Prioridade |
|----------|--------|------------|
| Binance Futures | ✅ Implementado | - |
| Kraken Pro Futures | 📋 Planejado | Alta |
| Bybit | 📋 Planejado | Média |
| OKX | 📋 Planejado | Baixa |

## Segurança

- **API Keys**: Devem ser armazenadas de forma segura (variáveis de ambiente ou keyring)
- **Assinatura**: Todos os requests autenticados são assinados com HMAC-SHA256
- **Testnet**: Sempre desenvolva e teste usando testnet antes de usar mainnet
- **Rate Limits**: O cliente respeita os rate limits da exchange

## Testando

```bash
# Configurar variáveis de ambiente
export BINANCE_API_KEY="sua_api_key"
export BINANCE_API_SECRET="seu_api_secret"
export BINANCE_TESTNET=true

# Rodar testes
cargo test -p robotrade-exchange-gateways
```
