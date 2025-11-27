# Integração com Exchanges

Este documento descreve como integrar e buscar dados das APIs de exchanges de criptomoedas no RoboTrade.

## Status Atual

### Binance Futures ✅ COMPLETO
- **Status**: Produção
- **API Base**: `https://fapi.binance.com`
- **Testnet**: `https://testnet.binancefuture.com`
- **Documentação**: [Binance Futures API](https://binance-docs.github.io/apidocs/futures/en/)

**Funcionalidades**:
- ✅ Autenticação HMAC-SHA256
- ✅ Order execution (market, limit, stop, etc.)
- ✅ Position management
- ✅ Leverage adjustment
- ✅ Balance & account info
- ✅ Trade history
- ✅ Income history (P&L)
- ✅ Candles/OHLCV data
- ⚠️ WebSocket streams (planejado)

### Kraken Futures ⚠️ PARCIAL
- **Status**: Em desenvolvimento
- **API Base**: `https://futures.kraken.com/derivatives/api/v3`
- **Documentação**: [Kraken Futures API](https://docs.futures.kraken.com/)

**Funcionalidades**:
- ✅ Client structure
- ✅ Signature logic (Nonce + API-Sign)
- ⚠️ ExchangeGateway implementation (50%)
- ❌ WebSocket streams

**TODO**:
- Completar métodos de orders
- Implementar position management
- Adicionar balance queries
- Testar em testnet

### Binance Spot ❌ PLANEJADO
- **Status**: Não iniciado
- **API Base**: `https://api.binance.com`

### Kraken Spot ❌ PLANEJADO  
- **Status**: Não iniciado
- **API Base**: `https://api.kraken.com`

---

## Como Adicionar uma Nova Exchange

### 1. Criar Módulo

```bash
# Estrutura de diretório
crates/exchange_gateways/src/
├── binance/      # Exemplo existente
├── kraken/       # Exemplo parcial
└── nova_exchange/
    ├── mod.rs       # Re-exports públicos
    ├── client.rs    # Client principal
    ├── signer.rs    # Lógica de assinatura
    ├── models.rs    # Tipos específicos da API
    └── error.rs     # Erros específicos
```

### 2. Implementar Client

```rust
// crates/exchange_gateways/src/nova_exchange/client.rs
use reqwest::Client;
use std::sync::Arc;

pub struct NovaExchangeClient {
    api_key: String,
    secret_key: String,
    http_client: Client,
    base_url: String,
}

impl NovaExchangeClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://testnet.novaexchange.com"
        } else {
            "https://api.novaexchange.com"
        };
        
        Self {
            api_key,
            secret_key,
            http_client: Client::new(),
            base_url: base_url.to_string(),
        }
    }
}
```

### 3. Implementar Assinatura

Cada exchange tem seu próprio método de autenticação:

#### HMAC-SHA256 (Binance)
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn sign_request(&self, query_string: &str) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())?;
    mac.update(query_string.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    Ok(signature)
}
```

#### Nonce + Base64 (Kraken)
```rust
use hmac::{Hmac, Mac};
use sha2::Sha512;

pub fn sign_request(&self, endpoint: &str, nonce: &str, post_data: &str) -> Result<String> {
    let message = format!("{}{}{}", post_data, nonce, endpoint);
    let mut mac = Hmac::<Sha512>::new_from_slice(
        &base64::decode(&self.secret_key)?
    )?;
    mac.update(message.as_bytes());
    let signature = base64::encode(mac.finalize().into_bytes());
    Ok(signature)
}
```

### 4. Implementar ExchangeGateway Trait

```rust
use async_trait::async_trait;
use robotrade_core::traits::ExchangeGateway;

#[async_trait]
impl ExchangeGateway for NovaExchangeClient {
    async fn get_balance(&self) -> Result<Vec<Balance>> {
        let endpoint = "/api/v1/balance";
        let params = self.build_signed_params()?;
        
        let response = self.http_client
            .get(&format!("{}{}", self.base_url, endpoint))
            .header("X-API-KEY", &self.api_key)
            .query(&params)
            .send()
            .await?;
        
        // Parse response e converter para Balance
        todo!()
    }
    
    async fn place_order(&self, order: Order) -> Result<Order> {
        // Implementar lógica de order placement
        todo!()
    }
    
    // ... outros métodos do trait
}
```

### 5. Adicionar Rate Limiting

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct NovaExchangeClient {
    // ... outros campos
    rate_limiter: RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock
    >,
}

impl NovaExchangeClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        // Exemplo: 100 requests/min = ~1.67/sec
        let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
        let rate_limiter = RateLimiter::direct(quota);
        
        Self {
            api_key,
            secret_key,
            http_client: reqwest::Client::new(),
            rate_limiter,
            // ...
        }
    }
    
    async fn execute_request<T>(&self, request: reqwest::Request) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Aguardar rate limit
        self.rate_limiter.until_ready().await;
        
        // Executar request
        let response = self.http_client.execute(request).await?;
        Ok(response.json().await?)
    }
}
```

### 6. Implementar Error Handling

```rust
pub fn map_exchange_error(code: i32, msg: &str) -> Error {
    match code {
        1001 => Error::InvalidApiKey,
        1002 => Error::RateLimitExceeded,
        1003 => Error::InsufficientBalance,
        2001 => Error::InvalidOrder(msg.to_string()),
        _ => Error::ExchangeError(code, msg.to_string()),
    }
}
```

### 7. Adicionar ao ExchangeId Enum

```rust
// crates/core/src/entities/exchange.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExchangeId {
    BinanceFutures,
    BinanceSpot,
    KrakenFutures,
    KrakenSpot,
    NovaExchange,  // <-- Adicionar aqui
    Paper,
}
```

### 8. Adicionar Testes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_balance() {
        let client = NovaExchangeClient::new(
            "test_api_key".to_string(),
            "test_secret".to_string(),
            true,
        );
        
        // Mock ou testnet
        let balance = client.get_balance().await;
        assert!(balance.is_ok());
    }
    
    #[test]
    fn test_sign_request() {
        let client = NovaExchangeClient::new(
            "key".to_string(),
            "secret".to_string(),
            true,
        );
        
        let signature = client.sign_request("test_query").unwrap();
        assert!(!signature.is_empty());
    }
}
```

---

## Buscar Dados das APIs

### Binance Futures - Candles (OHLCV)

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;

let client = BinanceFuturesClient::new(
    api_key.clone(),
    secret_key.clone(),
    false, // produção
);

// Buscar últimos 100 candles de 1h
let candles = client.get_candles(
    "BTCUSDT",
    "1h",
    None, // start_time
    None, // end_time
    Some(100),
).await?;

// Salvar no banco
use robotrade_infra::repositories::SqliteCandleRepository;
let repo = SqliteCandleRepository::new(pool.clone());

for candle in candles {
    repo.save(&candle).await?;
}
```

### Binance Futures - Histórico de Trades

```rust
// Buscar trades do usuário
let trades = client.get_user_trades("BTCUSDT", Some(100)).await?;

for trade in trades {
    println!("Trade: {} @ {} = {}", 
        trade.quantity, 
        trade.price, 
        trade.realized_pnl
    );
}
```

### Binance Futures - Income History (P&L)

```rust
use chrono::{Utc, Duration};

let end = Utc::now();
let start = end - Duration::days(30); // Último mês

let income_history = client.get_income_history(
    Some("BTCUSDT"),
    Some(start),
    Some(end),
    Some(1000),
).await?;

for income in income_history {
    println!("{}: {} {} ({})", 
        income.timestamp,
        income.income,
        income.asset,
        income.income_type
    );
}
```

### Kraken Futures - Quando Implementado

```rust
// Exemplo futuro (quando completar implementação)
use robotrade_exchange_gateways::kraken::KrakenFuturesClient;

let client = KrakenFuturesClient::new(
    api_key,
    secret_key,
    false,
);

let candles = client.get_candles("PI_XBTUSD", "60", None, None, Some(100)).await?;
```

---

## WebSocket Integration (Planejado)

### Estrutura Proposta

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct ExchangeWebSocket {
    url: String,
    subscriptions: Vec<String>,
}

impl ExchangeWebSocket {
    pub async fn subscribe_klines(&mut self, symbol: &str, interval: &str) {
        let stream = format!("{}@kline_{}", symbol.to_lowercase(), interval);
        self.subscriptions.push(stream);
    }
    
    pub async fn connect(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        loop {
            match read.next().await {
                Some(Ok(Message::Text(text))) => {
                    self.handle_message(&text).await?;
                }
                Some(Ok(Message::Close(_))) => {
                    warn!("WebSocket closed, reconnecting...");
                    break;
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                None => break,
            }
        }
        
        Ok(())
    }
}
```

---

## Checklist de Integração

- [ ] Client struct criado
- [ ] Método de assinatura implementado
- [ ] ExchangeGateway trait implementado (todos os métodos)
- [ ] Rate limiting configurado
- [ ] Error handling específico
- [ ] Testes unitários
- [ ] Testes de integração (testnet)
- [ ] Documentação atualizada
- [ ] ExchangeId enum atualizado

---

## Próximos Passos

1. **Completar Kraken Futures**: Implementar métodos faltantes do ExchangeGateway
2. **Adicionar WebSocket**: Streams em tempo real para Binance e Kraken
3. **Binance Spot**: Integração com API spot da Binance
4. **Kraken Spot**: Integração com API spot da Kraken
5. **OKX/Bybit**: Se houver demanda

---

## Referências

- [Binance Futures API Docs](https://binance-docs.github.io/apidocs/futures/en/)
- [Kraken Futures API Docs](https://docs.futures.kraken.com/)
- [crates/exchange_gateways/](../crates/exchange_gateways/)
- [.claude/agents/specialists/exchange-gateway-specialist.md](../.claude/agents/specialists/exchange-gateway-specialist.md)
