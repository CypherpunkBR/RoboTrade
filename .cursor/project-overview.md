# RoboTrade - Project Overview

## Descrição

RoboTrade é um robô de trading automatizado para futuros de criptomoedas, construído inteiramente em Rust com frontend desktop Tauri 2.0. O projeto foca em Binance Futures (testnet e produção) com planos para suporte a Kraken Pro Futures.

## Stack Tecnológica

- **Linguagem**: Rust (edition 2021)
- **Runtime Async**: Tokio (full features)
- **Database**: SQLite com sqlx (compile-time checked queries)
- **Frontend**: Tauri 2.0 + React (TypeScript)
- **HTTP Client**: reqwest
- **Decimal Precision**: rust_decimal (para valores financeiros)
- **Logging**: tracing + tracing-subscriber

## Arquitetura

### Workspace de 7 Crates

```
crates/
├── core/              # Entidades, traits, erros (SEM deps internas)
├── infra/             # Config, logging, SQLite, repositories
├── market_data/       # Providers de dados de mercado
├── exchange_gateways/ # Clientes de APIs (Binance, Kraken)
├── analytics/         # Indicadores, backtest engine
├── trading_worker/    # Execução de trades, position management
src-tauri/             # App Tauri - integra todos os crates
```

### Layered Architecture

```
src-tauri (Frontend Integration)
    ↓
trading_worker, analytics, exchange_gateways, market_data
    ↓
infra (Config, Database, Logging)
    ↓
core (Entities, Traits, Errors) ← BASE (sem dependências internas)
```

## Features Principais

### ✅ Implementado
- Integração completa com Binance Futures
- Execução de ordens (market, limit, stop, etc.)
- Gerenciamento de posições com SL/TP
- Backtest engine completo com métricas
- Estratégia Fear & Greed
- Indicadores técnicos (SMA, EMA, RSI, MACD, Bollinger, ATR)
- Repositórios SQLite (candles, fear_greed)
- Configuração via TOML
- Logging estruturado

### ⚠️ Parcial
- Kraken Futures (estrutura pronta, métodos a completar)
- WebSocket streams (planejado)
- Repositories (4/13 implementados)

### ❌ Planejado
- Monthly P&L aggregation
- WebSocket real-time data
- Binance Spot API
- Kraken Spot API
- TradeRepository, PositionRepository, OrderRepository
- Frontend UI completo

## Versão Atual

**v0.1.0** - Foundation phase (~70% complete)

## Comandos Principais

```bash
# Build
cargo build --workspace

# Tests
cargo test --workspace --all-features

# Quality
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Coverage
cargo tarpaulin --workspace --all-features

# Release
cargo build --workspace --release --all-features
```

## Estrutura de Arquivos

- `/crates/*` - Bibliotecas Rust do workspace
- `/src-tauri` - Aplicação Tauri (backend Rust + frontend React)
- `/docs` - Documentação do projeto
- `/.claude` - Configuração para Claude AI
- `/.cursor` - Configuração para Cursor IDE (este diretório)
- `/tests` - Testes de integração
- `CLAUDE.md` - Guia para Claude Code
- `CHANGELOG.md` - Histórico de mudanças

## Links Úteis

- [Architecture Overview](../docs/architecture/overview.md)
- [Development Guide](../docs/development/getting-started.md)
- [Coding Standards](../docs/development/coding-standards.md)
- [ROADMAP](../docs/ROADMAP.md)

---

**Projeto**: RoboTrade
**Versão**: 0.1.0
**Licença**: MIT
**Última Atualização**: 2025-11-27
