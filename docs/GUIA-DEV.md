# 🛠️ Guia do Desenvolvedor - RoboTrade

E aí, dev! Quer contribuir pro RoboTrade ou entender como funciona por dentro? Esse guia é pra ti.

---

## 🏗️ Arquitetura - Como tá organizado

O projeto usa **Cargo Workspace** com 7 crates separados. Pensa como se fosse um **bolo de camadas**:

```
         🎨 src-tauri (App Tauri - Interface)
              ↓ usa tudo abaixo
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    🔧 trading_worker  📊 analytics  🔌 exchange_gateways  📡 market_data
              ↓              ↓              ↓              ↓
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
              🗄️ infra (Database, Config, Logging)
              ↓
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
              💎 core (Entidades, Traits, Errors)
              FUNDAÇÃO - NÃO DEPENDE DE NINGUÉM!
```

### A Regra de Ouro 🥇

**`core` não depende de NENHUM outro crate interno!**

Por quê? Porque `core` é a **fundação**. Todo mundo usa ele, mas ele não usa ninguém. Assim:
- Sem dependências circulares
- Fácil de testar
- Reutilizável

### O que cada crate faz

#### 🏆 robotrade-core (Fundação)
**Responsabilidade:** Define as "peças do jogo"

- **Entidades:** Candle, Order, Position, Trade, Signal, etc
- **Traits:** Contratos que outros devem seguir (ExchangeGateway, Strategy, Repository)
- **Erros:** Tipos de erro do domínio

**Não tem:** Lógica de negócio, acesso a banco, HTTP, nada!

**Exemplo:**
```rust
// Define O QUE é uma ordem
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    // ...
}

// Define o CONTRATO de uma exchange
pub trait ExchangeGateway {
    async fn place_order(&self, order: Order) -> Result<Order>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
}
```

#### 🗄️ robotrade-infra (Infraestrutura)
**Responsabilidade:** "Encanamento" do sistema

- **Config:** Lê/salva arquivos TOML
- **Database:** SQLite com sqlx
- **Logging:** tracing + arquivos de log
- **Repositories:** Implementações de acesso a dados

**Não tem:** Lógica de trading, integração com exchange

**Exemplo:**
```rust
// Implementa como SALVAR candles no banco
pub struct SqliteCandleRepository {
    pool: SqlitePool,
}

impl CandleRepository for SqliteCandleRepository {
    async fn save(&self, candle: &Candle) -> Result<()> {
        sqlx::query!(
            "INSERT INTO candles (...) VALUES (...)",
            candle.symbol, candle.open, ...
        ).execute(&self.pool).await?;
        Ok(())
    }
}
```

#### 🔌 robotrade-exchange-gateways (Exchanges)
**Responsabilidade:** Falar com as exchanges

- **Binance:** Cliente completo da API
- **Kraken:** Cliente completo da API
- **Autenticação:** HMAC-SHA256, Nonce, etc
- **WebSocket:** Streams de dados em tempo real

**Implementa:** Trait `ExchangeGateway` do core

**Exemplo:**
```rust
pub struct BinanceFuturesClient {
    api_key: String,
    secret_key: String,
    http_client: reqwest::Client,
}

impl ExchangeGateway for BinanceFuturesClient {
    async fn place_order(&self, order: Order) -> Result<Order> {
        // Monta request
        let params = self.build_params(&order)?;
        let signature = self.sign(params)?;
        
        // Envia pra Binance
        let response = self.http_client
            .post("https://fapi.binance.com/fapi/v1/order")
            .query(&params)
            .send()
            .await?;
        
        // Converte resposta pra Order do core
        Ok(response.json().await?)
    }
}
```

#### 📡 robotrade-market-data (Dados de Mercado)
**Responsabilidade:** Buscar dados externos

- **Fear & Greed Index:** Sentimento do mercado
- **Candles:** Dados históricos OHLCV
- **Tickers:** Preços em tempo real

**Implementa:** Traits `MarketDataProvider` e `FearGreedProvider`

#### 📊 robotrade-analytics (Análise e Estratégias)
**Responsabilidade:** "Cérebro" do sistema

- **Indicators:** SMA, EMA, RSI, MACD, Bollinger, ATR
- **Strategies:** Fear & Greed (+ outros que tu criar)
- **Backtest Engine:** Testa estratégias em histórico

**Implementa:** Trait `Strategy`

**Exemplo de Indicador:**
```rust
pub fn rsi(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    // Calcula RSI
    // Retorna valores de 0-100
}

// Usar:
let closes: Vec<Decimal> = candles.iter().map(|c| c.close).collect();
let rsi_values = rsi(&closes, 14);

if rsi_values.last() < 30 {
    // Oversold! Pode comprar
}
```

#### 🔧 robotrade-trading-worker (Motor de Trading)
**Responsabilidade:** Executa trades automaticamente

- **Job Queue:** Fila assíncrona de tarefas
- **Position Manager:** Gerencia posições abertas
- **Risk Manager:** Valida se pode fazer trade
- **Circuit Breaker:** Para tudo se perder demais
- **Scheduler:** Agenda coleta de dados e avaliação de estratégias

**Exemplo:**
```rust
// Avalia se pode fazer um trade
pub struct RiskManager {
    config: RiskConfig,
}

impl RiskManager {
    pub fn can_open_position(&self, signal: &Signal, account: &Account) -> bool {
        // Já tem muitas posições?
        if account.open_positions >= self.config.max_positions {
            return false;
        }
        
        // Vai estourar o tamanho máximo?
        let position_value = signal.size * signal.price;
        if position_value > account.equity * self.config.max_position_size_pct {
            return false;
        }
        
        // Perdeu demais hoje?
        if account.daily_pnl < -self.config.max_daily_loss_pct * account.equity {
            return false;  // Circuit Breaker!
        }
        
        true
    }
}
```

#### 🎨 src-tauri (Aplicação)
**Responsabilidade:** Junta tudo e expõe pro frontend

- **Comandos Tauri:** IPC entre Rust e React
- **State Management:** Estado global thread-safe
- **Services:** Orquestra os outros crates

---

## 🔧 Como Adicionar Features

### Adicionar uma nova Estratégia

**1. Cria o arquivo:**
```bash
touch crates/analytics/src/strategies/minha_estrategia.rs
```

**2. Implementa o trait Strategy:**
```rust
use async_trait::async_trait;
use robotrade_core::{entities::*, traits::Strategy};

pub struct MinhaEstrategia {
    // Tuas configs
    pub buy_threshold: Decimal,
    pub sell_threshold: Decimal,
}

#[async_trait]
impl Strategy for MinhaEstrategia {
    async fn evaluate(&self, candles: &[Candle]) -> Result<Option<Signal>> {
        // Precisa de dados suficientes
        if candles.len() < 20 {
            return Ok(None);
        }
        
        // TUA LÓGICA AQUI!
        // Exemplo: RSI
        let closes: Vec<Decimal> = candles.iter().map(|c| c.close).collect();
        let rsi = calculate_rsi(&closes, 14);
        let current_rsi = rsi.last().unwrap();
        
        // Sinal de COMPRA se RSI < 30 (oversold)
        if *current_rsi < Decimal::from(30) {
            return Ok(Some(Signal {
                signal_type: SignalType::Long,
                strength: SignalStrength::Medium,
                indicator_value: Some(*current_rsi),
                timestamp: Utc::now(),
                metadata: None,
            }));
        }
        
        // Sinal de VENDA se RSI > 70 (overbought)
        if *current_rsi > Decimal::from(70) {
            return Ok(Some(Signal {
                signal_type: SignalType::Short,
                strength: SignalStrength::Medium,
                indicator_value: Some(*current_rsi),
                timestamp: Utc::now(),
                metadata: None,
            }));
        }
        
        Ok(None)  // Sem sinal
    }
    
    async fn should_close_position(
        &self,
        position: &Position,
        current_price: Decimal,
    ) -> Result<bool> {
        // Lógica de SAÍDA
        // Pode usar SL/TP automático ou lógica customizada
        Ok(false)
    }
    
    fn name(&self) -> &str {
        "Minha Estratégia RSI"
    }
    
    fn config(&self) -> &dyn std::any::Any {
        self
    }
}
```

**3. Adiciona ao módulo:**
```rust
// crates/analytics/src/strategies/mod.rs
pub mod fear_greed;
pub mod minha_estrategia;  // <-- Adiciona isso

pub use fear_greed::FearGreedStrategy;
pub use minha_estrategia::MinhaEstrategia;  // <-- E isso
```

**4. Testa:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_minha_estrategia() {
        let strategy = MinhaEstrategia {
            buy_threshold: Decimal::from(30),
            sell_threshold: Decimal::from(70),
        };
        
        // Cria candles fake pra testar
        let candles = create_test_candles();
        
        let signal = strategy.evaluate(&candles).await.unwrap();
        assert!(signal.is_some());
    }
}
```

**5. Usa no worker:**
```rust
// src-tauri/src/trading_worker.rs
let minha_estrategia = MinhaEstrategia::new();
worker.add_strategy(Box::new(minha_estrategia));
```

Pronto! Tua estratégia já tá rodando!

### Adicionar um novo Indicador

**1. Cria o arquivo:**
```bash
touch crates/analytics/src/indicators/meu_indicador.rs
```

**2. Implementa a função:**
```rust
use rust_decimal::Decimal;

/// Calcula teu indicador maluco
pub fn meu_indicador(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    prices
        .windows(period)
        .map(|window| {
            // TUA LÓGICA AQUI
            let sum: Decimal = window.iter().sum();
            sum / Decimal::from(period)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_meu_indicador() {
        let prices = vec![
            Decimal::from(100),
            Decimal::from(105),
            Decimal::from(110),
        ];
        
        let result = meu_indicador(&prices, 2);
        assert_eq!(result.len(), 2);
    }
}
```

**3. Adiciona ao módulo:**
```rust
// crates/analytics/src/indicators/mod.rs
pub mod sma;
pub mod ema;
pub mod rsi;
pub mod meu_indicador;  // <-- Adiciona

pub use meu_indicador::meu_indicador;
```

**4. Usa onde quiser:**
```rust
let closes: Vec<Decimal> = candles.iter().map(|c| c.close).collect();
let valores = meu_indicador(&closes, 20);
```

### Adicionar um novo Comando Tauri

**1. Adiciona a função:**
```rust
// src-tauri/src/commands.rs

#[tauri::command]
pub async fn meu_comando(
    symbol: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("Executando meu comando para {}", symbol);
    
    // TUA LÓGICA AQUI
    let result = fazer_alguma_coisa(&symbol).await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}
```

**2. Registra no builder:**
```rust
// src-tauri/src/lib.rs

tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... comandos existentes
        meu_comando,  // <-- Adiciona aqui
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

**3. Usa no frontend:**
```typescript
// src-web/src/...
import { invoke } from '@tauri-apps/api/core';

const resultado = await invoke<string>('meu_comando', { 
    symbol: 'BTCUSDT' 
});

console.log(resultado);
```

Pronto! Agora o frontend pode chamar teu comando Rust!

---

## 🧪 Como Testar

### Testes Unitários

**Testar tudo:**
```bash
cargo test
```

**Testar um crate específico:**
```bash
cargo test -p robotrade-core
cargo test -p robotrade-analytics
```

**Testar uma função específica:**
```bash
cargo test test_calculate_pnl
```

**Ver o output (println!, dbg!):**
```bash
cargo test -- --nocapture
```

### Testes com Mock

Quando tu precisa mockar uma exchange:

```rust
use robotrade_core::traits::ExchangeGateway;
use async_trait::async_trait;

// Cria um mock
struct MockExchange {
    should_fail: bool,
}

#[async_trait]
impl ExchangeGateway for MockExchange {
    async fn place_order(&self, order: Order) -> Result<Order> {
        if self.should_fail {
            return Err(Error::ExchangeError(1001, "Mock error".into()));
        }
        Ok(order)
    }
    
    // ... outros métodos
}

// Usa no teste
#[tokio::test]
async fn test_com_mock() {
    let mock = MockExchange { should_fail: false };
    let result = mock.place_order(test_order()).await;
    assert!(result.is_ok());
}
```

### Coverage (Cobertura de Código)

```bash
# Instala tarpaulin
cargo install cargo-tarpaulin

# Roda coverage
cargo tarpaulin --workspace --all-features

# Gera HTML
cargo tarpaulin --workspace --all-features --out html
```

Abre `tarpaulin-report.html` no browser pra ver quais linhas tão cobertas!

**Meta:** >= 70% de cobertura (ideal: >= 80%)

---

## 🐛 Como Debugar

### Logs são teus amigos

O sistema usa `tracing` pra logging. Tá em TODO canto:

```rust
use tracing::{info, warn, error, debug};

info!("Executando ordem {}", order_id);
warn!("Saldo baixo: {}", balance);
error!("Falha ao conectar: {}", e);
debug!("Dados recebidos: {:?}", data);
```

**Controlar nível de log:**
```bash
# .env
RUST_LOG=debug  # Mostra tudo
RUST_LOG=info   # Padrão
RUST_LOG=warn   # Só avisos e erros
```

**Logs vão pra:**
- Console (quando roda)
- Arquivo: `~/.local/share/robotrade/logs/robotrade.log`

### Debugar com VSCode

**1. Instala a extensão:** rust-analyzer

**2. Adiciona configuração:**
```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug RoboTrade",
            "cargo": {
                "args": ["build", "--bin=robotrade", "--package=robotrade"]
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

**3. Coloca breakpoints** no código

**4. F5** pra debugar!

### Debugar o Frontend

No navegador (quando roda `npm run tauri dev`):
- **F12** → abre DevTools
- **Console** → vê logs do React
- **Network** → vê chamadas ao backend
- **Components** → usa React DevTools

---

## 📊 Como fazer Backtest

### 1. Busca dados históricos

```rust
use robotrade_exchange_gateways::binance::BinanceFuturesClient;

let client = BinanceFuturesClient::new(api_key, secret, false);

// Busca 1 ano de candles de 1h
let candles = client.get_candles(
    "BTCUSDT",
    "1h",
    Some(DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")?),
    Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?),
    None,
).await?;
```

### 2. Configura o backtest

```rust
use robotrade_analytics::backtest::{BacktestEngine, BacktestConfig};

let config = BacktestConfig {
    initial_capital: Decimal::from(10000),        // Começa com $10k
    position_size_pct: Decimal::from_str("0.1")?, // 10% por trade
    trading_fee_pct: Decimal::from_str("0.0004")?,// 0.04% fee
    slippage_pct: Decimal::from_str("0.0005")?,   // 0.05% slippage
    default_stop_loss_pct: Some(Decimal::from_str("0.02")?), // 2% SL
    default_take_profit_pct: Some(Decimal::from_str("0.04")?), // 4% TP
    leverage: 1,  // Sem alavancagem no backtest (conservador)
};
```

### 3. Roda o backtest

```rust
use robotrade_analytics::strategies::FearGreedStrategy;

let strategy = FearGreedStrategy::new(/* config */);
let engine = BacktestEngine::new(config, Box::new(strategy));

let result = engine.run(&candles).await?;

println!("📊 Resultados:");
println!("Capital Final: ${}", result.final_equity);
println!("Retorno Total: {:.2}%", result.metrics.total_return_pct);
println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct);
println!("Win Rate: {:.2}%", result.metrics.win_rate_pct);
println!("Total Trades: {}", result.metrics.total_trades);
```

### 4. Analisa os resultados

**Boa estratégia tem:**
- ✅ Sharpe Ratio >= 1.0 (idealmente >= 2.0)
- ✅ Max Drawdown <= 20%
- ✅ Win Rate >= 40%
- ✅ Profit Factor >= 1.5
- ✅ Total Trades >= 30 (significância estatística)

**Red flags:**
- 🚩 Win Rate > 80% (provável overfitting)
- 🚩 Drawdown > 30% (risco muito alto)
- 🚩 Poucas trades (< 20)
- 🚩 Performance perfeita (100% win rate = bugado ou overfitted)

---

## 🔨 Ferramentas de Desenvolvimento

### Linting (Clippy)

```bash
# Básico
cargo clippy

# Rigoroso (usa no CI)
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all -W clippy::pedantic -W clippy::nursery
```

**Clippy vai reclamar de:**
- Código não idiomático
- Possíveis bugs
- Performance issues
- Complexidade alta

**Fixa automaticamente:**
```bash
cargo clippy --fix --allow-dirty --allow-staged
```

### Formatação (rustfmt)

```bash
# Formata tudo
cargo fmt --all

# Só checa (não modifica)
cargo fmt --all -- --check
```

**Configuração:** Tá no `rustfmt.toml` (2 espaços, 100 chars por linha)

### Security Audit

```bash
# Verifica vulnerabilidades conhecidas
cargo audit

# Verifica licenças e advisories
cargo deny check
```

**Roda isso antes de cada release!**

### Dependency Analysis

```bash
# Acha dependências não usadas
cargo machete

# Acha versões duplicadas
cargo tree --duplicates

# Mostra árvore completa
cargo tree
```

### Benchmarks

```bash
# Roda todos benchmarks
cargo bench

# Benchmark específico
cargo bench --bench backtest_benchmark
```

**Adiciona benchmark:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_minha_funcao(c: &mut Criterion) {
    c.bench_function("minha_funcao", |b| {
        b.iter(|| {
            minha_funcao(black_box(100))
        });
    });
}

criterion_group!(benches, benchmark_minha_funcao);
criterion_main!(benches);
```

---

## 📁 Estrutura de Arquivos - Onde cada coisa fica

```
RoboTrade/
│
├── crates/                    # Workspace de bibliotecas Rust
│   ├── core/                  # 💎 Fundação (entities, traits, errors)
│   │   ├── src/
│   │   │   ├── entities/      # Order, Position, Trade, etc
│   │   │   ├── traits.rs      # Interfaces (ExchangeGateway, Strategy, etc)
│   │   │   ├── error.rs       # Tipos de erro
│   │   │   └── dto/           # Data Transfer Objects
│   │   └── Cargo.toml
│   │
│   ├── infra/                 # 🗄️ Infraestrutura
│   │   ├── src/
│   │   │   ├── config/        # Leitura de TOML
│   │   │   ├── database/      # SQLite setup e schema
│   │   │   ├── repositories/  # Implementações de Repository
│   │   │   └── logging/       # Setup de logs
│   │   └── Cargo.toml
│   │
│   ├── exchange_gateways/     # 🔌 Clientes de Exchange
│   │   ├── src/
│   │   │   ├── binance/       # Cliente Binance completo
│   │   │   ├── kraken/        # Cliente Kraken completo
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── market_data/           # 📡 Providers de dados
│   │   ├── src/
│   │   │   └── providers/     # Fear & Greed, etc
│   │   └── Cargo.toml
│   │
│   ├── analytics/             # 📊 Análise e Estratégias
│   │   ├── src/
│   │   │   ├── indicators/    # SMA, EMA, RSI, MACD, etc
│   │   │   ├── strategies/    # FearGreedStrategy, etc
│   │   │   └── backtest/      # Motor de backtest
│   │   └── Cargo.toml
│   │
│   └── trading_worker/        # 🔧 Motor de Trading Automático
│       ├── src/
│       │   ├── collector.rs   # Coleta dados
│       │   ├── scheduler.rs   # Agenda tarefas
│       │   ├── job_queue.rs   # Fila de jobs
│       │   ├── position_manager.rs  # Gerencia posições
│       │   ├── risk.rs        # Risk Manager
│       │   └── circuit_breaker.rs   # Circuit Breaker
│       └── Cargo.toml
│
├── src-tauri/                 # 🎨 Aplicação Tauri
│   ├── src/
│   │   ├── lib.rs             # Entry point
│   │   ├── commands.rs        # Comandos IPC (1300+ linhas!)
│   │   ├── state.rs           # Estado global
│   │   ├── exchange_service.rs # Orquestra exchanges
│   │   ├── market_data_service.rs # Orquestra market data
│   │   └── trading_worker.rs  # Worker principal
│   ├── Cargo.toml
│   └── tauri.conf.json        # Config do Tauri
│
├── src-web/                   # ⚛️ Frontend React/TypeScript
│   ├── src/
│   │   ├── components/        # Componentes reutilizáveis
│   │   │   ├── TradingChart.tsx      # Gráfico de trading
│   │   │   ├── PriceAlerts.tsx       # Sistema de alertas
│   │   │   ├── FearGreedGauge.tsx    # Medidor F&G
│   │   │   └── ...
│   │   ├── contexts/          # React Contexts
│   │   │   └── GlobalFilterContext.tsx
│   │   ├── pages/             # Páginas da app
│   │   │   ├── Dashboard.tsx  # Tela principal
│   │   │   ├── Trading.tsx    # Trading manual
│   │   │   ├── Charts.tsx     # Gráficos e alertas
│   │   │   ├── History.tsx    # Histórico
│   │   │   ├── Reports.tsx    # Relatórios visuais
│   │   │   └── Settings.tsx   # Configurações
│   │   ├── lib/               # Utilitários
│   │   └── types/             # TypeScript types
│   └── package.json
│
├── docs/                      # 📚 Documentação
│   ├── GUIA-COMPLETO.md       # Este guia!
│   ├── GUIA-DEV.md            # Guia do dev
│   ├── architecture/          # Docs de arquitetura
│   ├── business/              # Lógica de negócio
│   ├── api/                   # API reference
│   └── ...
│
├── .claude/                   # 🤖 Config do Claude AI
│   ├── agents/                # Agentes especializados
│   └── commands/              # Comandos documentados
│
├── .cursor/                   # 🎯 Config do Cursor IDE
│   ├── project-overview.md
│   ├── rules
│   └── ...
│
├── .env                       # 🔑 Credenciais (NÃO COMMITAR!)
├── config.toml                # ⚙️ Configurações do usuário
├── Cargo.toml                 # 📦 Workspace manifest
└── package.json               # 📦 Scripts do projeto
```

---

## 🚦 Workflow de Desenvolvimento

### 1. Pega uma issue

Vai no GitHub Issues e pega algo pra fazer.

### 2. Cria uma branch

```bash
git checkout -b feature/minha-feature
```

### 3. Implementa

- Escreve o código
- Adiciona testes
- Roda `cargo fmt`
- Roda `cargo clippy`
- Roda `cargo test`

### 4. Commita

```bash
git add .
git commit -m "feat: adiciona minha feature incrível

- Implementa X
- Adiciona testes para Y
- Atualiza docs"
```

**Formato:** [Conventional Commits](https://www.conventionalcommits.org/)
- `feat:` = nova feature
- `fix:` = correção de bug
- `chore:` = manutenção
- `docs:` = documentação
- `test:` = testes

### 5. Push e Pull Request

```bash
git push origin feature/minha-feature
```

Abre PR no GitHub com:
- Título descritivo
- Descrição do que mudou
- Screenshots (se mudou UI)
- Como testar

### 6. CI/CD valida

GitHub Actions roda automaticamente:
- ✅ Formatting check
- ✅ Clippy
- ✅ Tests
- ✅ Build
- ✅ Security audit

Se tudo passar, tá pronto pra merge!

---

## 🎯 Convenções de Código

### Naming

**Rust:**
- `snake_case` pra variáveis e funções
- `PascalCase` pra structs e enums
- `SCREAMING_SNAKE_CASE` pra constantes

```rust
const MAX_LEVERAGE: i32 = 125;

struct TradingConfig {
    default_size: Decimal,
}

fn calculate_pnl(entry: Decimal, exit: Decimal) -> Decimal {
    exit - entry
}
```

**TypeScript:**
- `camelCase` pra variáveis e funções
- `PascalCase` pra componentes e tipos
- `SCREAMING_SNAKE_CASE` pra constantes

```typescript
const MAX_RETRIES = 3;

interface OrderRequest {
    symbol: string;
    quantity: number;
}

function createOrder(request: OrderRequest) {
    // ...
}
```

### Erros

**Sempre usa Result:**
```rust
// ❌ Ruim - pode dar panic
pub fn divide(a: i32, b: i32) -> i32 {
    a / b  // Panic se b = 0!
}

// ✅ Bom - retorna erro
pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("Cannot divide by zero".into());
    }
    Ok(a / b)
}
```

**Propaga erros com `?`:**
```rust
pub async fn buscar_preco(symbol: &str) -> Result<Decimal> {
    let client = criar_client()?;  // Se der erro, retorna
    let ticker = client.get_ticker(symbol).await?;  // Propaga erro
    Ok(ticker.price)
}
```

### Async/Await

**Sempre async quando faz I/O:**
```rust
// ✅ Bom - async pra HTTP
pub async fn fetch_data() -> Result<Data> {
    let response = reqwest::get("https://api.com").await?;
    Ok(response.json().await?)
}
```

**Spawn tasks pra paralelizar:**
```rust
let task1 = tokio::spawn(async { buscar_binance().await });
let task2 = tokio::spawn(async { buscar_kraken().await });

let (result1, result2) = tokio::join!(task1, task2);
```

---

## 📚 Referências Úteis

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)

### Crates
- [tokio](https://docs.rs/tokio/) - Async runtime
- [sqlx](https://docs.rs/sqlx/) - Database
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [serde](https://docs.rs/serde/) - Serialização
- [tracing](https://docs.rs/tracing/) - Logging

### Tauri
- [Tauri Docs](https://tauri.app/v1/guides/)
- [Tauri API](https://tauri.app/v1/api/)

### Trading
- [Binance Futures API](https://binance-docs.github.io/apidocs/futures/en/)
- [Kraken Futures API](https://docs.futures.kraken.com/)
- [Investopedia](https://www.investopedia.com/) - Conceitos de trading

---

## 🤝 Contribuindo

Quer ajudar? Massa! Aqui vai como:

### 1. Issues Boas pra Começar
- Label: `good first issue`
- Label: `help wanted`

### 2. Áreas que Precisam de Ajuda
- Mais estratégias de trading
- Mais indicadores técnicos
- Testes de integração
- Documentação
- UI/UX improvements
- Performance optimizations

### 3. Código de Conduta
- Respeita todo mundo
- Crítica construtiva
- Sem spam
- Sem código malicioso

### 4. Processo de Review
1. Abre PR
2. CI roda checks
3. Reviewer analisa código
4. Feedback/aprovação
5. Merge!

---

Agora tu sabe como DESENVOLVER no RoboTrade! 🚀

Bora codar! 💻
