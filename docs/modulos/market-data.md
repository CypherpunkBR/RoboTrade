# Módulo Market Data

O crate `robotrade-market-data` fornece providers para coleta de dados de mercado de fontes externas.

## Estrutura de Diretórios

```
crates/market_data/src/
├── lib.rs                  # Exportações públicas
└── providers/
    ├── mod.rs
    └── alternative_me.rs   # Provider Fear & Greed (alternative.me)
```

## Provider Fear & Greed Index

### Fonte de Dados

O índice Fear & Greed é coletado da API pública do [Alternative.me](https://alternative.me/crypto/fear-and-greed-index/).

### Configuração

```rust
use robotrade_market_data::providers::AlternativeMeFearGreedProvider;

let provider = AlternativeMeFearGreedProvider::new();
```

### Métodos

```rust
/// Obtém o valor atual do índice
async fn fetch_current(&self) -> MarketDataResult<FearGreedData>;

/// Obtém histórico dos últimos N dias
async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>>;

/// Verifica se a API está acessível
async fn health_check(&self) -> bool;
```

### Exemplo de Uso

```rust
use robotrade_market_data::providers::AlternativeMeFearGreedProvider;
use robotrade_core::traits::FearGreedProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = AlternativeMeFearGreedProvider::new();

    // Verificar saúde da API
    if !provider.health_check().await {
        eprintln!("API Fear & Greed indisponível");
        return Ok(());
    }

    // Obter valor atual
    let current = provider.fetch_current().await?;
    println!(
        "Fear & Greed atual: {} ({})",
        current.value,
        current.classification
    );

    // Obter histórico de 30 dias
    let history = provider.fetch_history(30).await?;
    for data in history {
        println!(
            "{}: {} - {}",
            data.timestamp.format("%Y-%m-%d"),
            data.value,
            data.classification
        );
    }

    Ok(())
}
```

### Classificação do Índice

| Faixa | Classificação | Significado |
|-------|---------------|-------------|
| 0-24 | Extreme Fear | Medo extremo - possível oportunidade de compra |
| 25-44 | Fear | Medo - mercado pessimista |
| 45-55 | Neutral | Neutro - mercado indeciso |
| 56-75 | Greed | Ganância - mercado otimista |
| 76-100 | Extreme Greed | Ganância extrema - possível topo |

### Resposta da API

```rust
/// Dados do índice Fear & Greed
pub struct FearGreedData {
    /// Valor do índice (0-100)
    pub value: u8,

    /// Classificação textual
    pub classification: FearGreedClassification,

    /// Timestamp da medição
    pub timestamp: DateTime<Utc>,
}

/// Classificação do índice
pub enum FearGreedClassification {
    ExtremeFear,
    Fear,
    Neutral,
    Greed,
    ExtremeGreed,
}
```

### Tratamento de Erros

```rust
pub enum MarketDataError {
    /// Falha na conexão com provider
    ConnectionFailed { provider: String, reason: String },

    /// Dados inválidos recebidos
    InvalidData { provider: String, reason: String },

    /// Rate limit atingido
    RateLimited { provider: String, retry_after: Duration },

    /// Provider não disponível
    ProviderUnavailable { provider: String },

    /// Erro ao parsear resposta
    ParseError { message: String },
}
```

## Providers Planejados

### Binance Market Data

```rust
// Futuro: BinanceMarketDataProvider
pub trait MarketDataProvider: Send + Sync {
    /// Obtém ticker atual
    async fn get_ticker(&self, symbol: &str) -> MarketDataResult<Ticker>;

    /// Obtém candles históricos
    async fn get_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
        limit: u32,
    ) -> MarketDataResult<Vec<Candle>>;

    /// Obtém orderbook
    async fn get_orderbook(&self, symbol: &str, depth: u32) -> MarketDataResult<Orderbook>;
}
```

### WebSocket Stream

```rust
// Futuro: Stream de dados em tempo real
pub trait MarketDataStream: Send + Sync {
    /// Subscreve a atualizações de ticker
    async fn subscribe_ticker(&self, symbol: &str) -> MarketDataResult<Receiver<Ticker>>;

    /// Subscreve a atualizações de candles
    async fn subscribe_candles(
        &self,
        symbol: &str,
        timeframe: TimeFrame,
    ) -> MarketDataResult<Receiver<Candle>>;

    /// Subscreve a atualizações de trades
    async fn subscribe_trades(&self, symbol: &str) -> MarketDataResult<Receiver<Trade>>;
}
```

## Roadmap

| Feature | Descrição | Status |
|---------|-----------|--------|
| Fear & Greed (Alternative.me) | Índice de sentimento | ✅ Implementado |
| Binance REST Candles | Candles históricos | 📋 Planejado |
| Binance WebSocket | Dados em tempo real | 📋 Planejado |
| CoinGecko | Dados de mercado | 📋 Planejado |
| On-chain Metrics | Dados on-chain | 📋 Planejado |

## Cache

O provider implementa cache interno para evitar requests desnecessários:

```rust
// Configuração de cache (futuro)
let provider = AlternativeMeFearGreedProvider::new()
    .with_cache_duration(Duration::from_secs(300)); // 5 minutos
```

## Testes

```rust
#[tokio::test]
async fn test_fetch_current_fear_greed() {
    let provider = AlternativeMeFearGreedProvider::new();
    let data = provider.fetch_current().await.unwrap();

    assert!(data.value <= 100);
    assert!(data.timestamp <= Utc::now());
}

#[tokio::test]
async fn test_fetch_history() {
    let provider = AlternativeMeFearGreedProvider::new();
    let history = provider.fetch_history(7).await.unwrap();

    assert_eq!(history.len(), 7);
}
```
