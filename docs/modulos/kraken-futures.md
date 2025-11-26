# Módulo Kraken Futures

O módulo `kraken` dentro de `robotrade-exchange-gateways` fornece integração com a API REST da Kraken Futures para trading de contratos perpétuos e futuros.

## Estrutura de Diretórios

```
crates/exchange_gateways/src/kraken/
├── mod.rs          # Exportações públicas
├── client.rs       # KrakenFuturesClient
├── models.rs       # Modelos de dados
└── signer.rs       # Assinatura de requisições
```

## Ambientes

| Ambiente | URL | Uso |
|----------|-----|-----|
| Demo | `https://demo-futures.kraken.com` | Testes e desenvolvimento |
| Produção | `https://futures.kraken.com` | Trading real |

## Autenticação

A API da Kraken Futures utiliza um sistema de autenticação baseado em:

- **APIKey**: Chave pública da API
- **Nonce**: Timestamp em milissegundos (incrementa a cada requisição)
- **Authent**: Assinatura HMAC-SHA-512

### Algoritmo de Assinatura

```rust
// 1. Concatenar: postData + nonce + endpointPath
let concat = format!("{}{}{}", post_data, nonce, endpoint_path);

// 2. SHA-256 hash
let sha256_hash = Sha256::digest(concat.as_bytes());

// 3. Decodificar API secret de base64
let decoded_secret = base64::decode(api_secret)?;

// 4. HMAC-SHA-512 usando o secret decodificado
let mut mac = HmacSha512::new_from_slice(&decoded_secret)?;
mac.update(&sha256_hash);

// 5. Codificar resultado em base64
base64::encode(mac.finalize().into_bytes())
```

## Uso

### Criação do Cliente

```rust
use robotrade_exchange_gateways::KrakenFuturesClient;

// Cliente para ambiente demo (testnet)
let client = KrakenFuturesClient::demo("sua_api_key", "seu_api_secret");

// Cliente para produção
let client = KrakenFuturesClient::mainnet("sua_api_key", "seu_api_secret");
```

### Dados de Mercado (Públicos)

```rust
// Listar instrumentos disponíveis
let instruments = client.get_instruments().await?;
println!("Contratos disponíveis: {:?}", instruments.len());

// Buscar tickers
let tickers = client.get_tickers(Some(&["PI_XBTUSD", "PI_ETHUSD"])).await?;
for ticker in tickers {
    println!("{}: bid={:?}, ask={:?}", ticker.symbol, ticker.bid, ticker.ask);
}

// Buscar orderbook
let orderbook = client.get_orderbook("PI_XBTUSD").await?;
println!("Melhor bid: {:?}", orderbook.bids.first());
println!("Melhor ask: {:?}", orderbook.asks.first());

// Buscar candles
use robotrade_exchange_gateways::kraken::CandleInterval;

let candles = client.get_candles(
    "PI_XBTUSD",
    CandleInterval::H1,
    None,
    None,
).await?;
```

### Informações da Conta (Privados)

```rust
// Buscar informações da conta
let account = client.get_account().await?;
if let Some(flex) = account.flex {
    println!("Portfolio value: {:?}", flex.portfolio_value);
    println!("Available margin: {:?}", flex.available_margin);
}

// Buscar saldos (via trait ExchangeGateway)
use robotrade_core::traits::ExchangeGateway;

let balances = client.get_balances().await?;
for balance in balances {
    println!("{}: free={}, locked={}", balance.asset, balance.free, balance.locked);
}
```

### Posições

```rust
// Buscar posições abertas
let positions = client.get_open_positions().await?;
for pos in positions {
    println!(
        "{} {} {} @ {}",
        pos.symbol, pos.side, pos.size, pos.price
    );
}

// Via trait ExchangeGateway
let positions = client.get_positions().await?;
```

### Ordens

```rust
use robotrade_exchange_gateways::kraken::NewOrderRequest;
use rust_decimal::Decimal;

// Ordem de mercado
let request = NewOrderRequest::market("PI_XBTUSD", "buy", Decimal::from(1));
let status = client.send_order(&request).await?;
println!("Ordem enviada: {:?}", status.order_id);

// Ordem limite
let request = NewOrderRequest::limit(
    "PI_XBTUSD",
    "sell",
    Decimal::from(1),
    Decimal::from(50000),
).with_reduce_only(true);
let status = client.send_order(&request).await?;

// Cancelar ordem
let cancel_status = client.cancel_order("order_id_aqui").await?;
println!("Status: {}", cancel_status.status);

// Listar ordens abertas
let open_orders = client.get_open_orders().await?;
```

### Via Trait ExchangeGateway

```rust
use robotrade_core::traits::ExchangeGateway;
use robotrade_core::entities::{OrderRequest, OrderSide, OrderType, TimeInForce};
use rust_decimal::Decimal;

// Enviar ordem via interface unificada
let request = OrderRequest {
    symbol: "PI_XBTUSD".to_string(),
    side: OrderSide::Buy,
    order_type: OrderType::Market,
    quantity: Decimal::from(1),
    price: None,
    stop_price: None,
    time_in_force: TimeInForce::GTC,
    reduce_only: false,
};

let order = client.submit_order(request).await?;

// Verificar se é paper trading
if client.is_paper_trading() {
    println!("Usando ambiente demo");
}

// Ping para verificar conectividade
client.ping().await?;
```

## Símbolos de Contratos

A Kraken Futures usa nomenclatura própria para contratos:

| Símbolo | Descrição |
|---------|-----------|
| `PI_XBTUSD` | Perpétuo BTC/USD |
| `PI_ETHUSD` | Perpétuo ETH/USD |
| `PF_XBTUSD` | Futuro BTC/USD |
| `FI_XBTUSD_YYMMDD` | Futuro BTC/USD com vencimento |

## Tipos de Ordem

| Tipo | Código API | Descrição |
|------|------------|-----------|
| Limit | `lmt` | Ordem limite |
| Market | `mkt` | Ordem a mercado |
| Stop | `stp` | Stop order |
| Take Profit | `take_profit` | Take profit order |
| IOC | `ioc` | Immediate or cancel |

## Tratamento de Erros

```rust
use robotrade_core::error::ExchangeError;

match client.send_order(&request).await {
    Ok(status) => println!("Ordem enviada: {:?}", status),
    Err(ExchangeError::InsufficientBalance { .. }) => {
        println!("Saldo insuficiente");
    }
    Err(ExchangeError::ApiError { message, .. }) => {
        println!("Erro da API: {}", message);
    }
    Err(e) => println!("Outro erro: {:?}", e),
}
```

## Diferenças da Binance

| Aspecto | Kraken | Binance |
|---------|--------|---------|
| Alavancagem | Dinâmica por tamanho | Configurável por símbolo |
| Assinatura | HMAC-SHA-512 | HMAC-SHA-256 |
| Headers auth | APIKey, Nonce, Authent | X-MBX-APIKEY + signature param |
| Símbolos | PI_XBTUSD | BTCUSDT |
| Ambiente teste | demo-futures.kraken.com | testnet.binancefuture.com |

## Limitações Conhecidas

1. **Sem endpoint de ordem individual**: Para buscar status de uma ordem específica, é necessário listar todas as ordens abertas
2. **Alavancagem não configurável**: Kraken usa margem dinâmica baseada no tamanho da posição
3. **Símbolos diferentes**: Usar nomenclatura Kraken (PI_XBTUSD) ao invés de BTCUSDT

## Testes

```bash
# Rodar testes do módulo
cargo test --package robotrade-exchange-gateways

# Testes específicos da Kraken
cargo test kraken
```

## Links

- [Kraken Futures API Docs](https://docs.kraken.com/api/docs/guides/futures-rest/)
- [Demo Account](https://demo-futures.kraken.com)
