# 📚 Índice da Documentação - RoboTrade

Bem-vindo à documentação completa do RoboTrade! Aqui tu encontra TUDO sobre o sistema.

---

## 🎯 Pra Começar (Usuários)

**Leia nessa ordem se tu é novo:**

1. **[📖 GUIA-COMPLETO.md](GUIA-COMPLETO.md)** ⭐ **COMECE AQUI!**
   - O que é o RoboTrade
   - Como instalar e configurar
   - Como usar cada tela (Dashboard, Trading, Charts, etc)
   - Dicas e boas práticas
   - Glossário de termos
   - **~300 linhas de português informal e acessível**

2. **[❓ FAQ.md](FAQ.md)**
   - Perguntas frequentes
   - Troubleshooting de problemas comuns
   - Dicas de segurança
   - Como reportar bugs
   - **~200 linhas de respostas práticas**

3. **[⚙️ development/getting-started.md](development/getting-started.md)**
   - Setup técnico detalhado
   - Dependências por sistema operacional
   - Configuração de ambiente
   - Troubleshooting de instalação

---

## 🛠️ Pra Desenvolvedores

**Se tu quer contribuir ou entender o código:**

1. **[🛠️ GUIA-DEV.md](GUIA-DEV.md)** ⭐ **COMECE AQUI!**
   - Arquitetura explicada
   - Como adicionar features (estratégias, indicadores, exchanges)
   - Workflow de desenvolvimento
   - Convenções de código
   - Como testar e fazer backtest
   - **~400 linhas de explicação prática**

2. **[🏗️ architecture/overview.md](architecture/overview.md)**
   - Visão geral da arquitetura
   - Workspace de 7 crates
   - Layered architecture
   - Fluxo de dados
   - Design patterns

3. **[📋 development/coding-standards.md](development/coding-standards.md)**
   - Convenções de código Rust
   - Naming conventions
   - Error handling patterns
   - Testing guidelines

4. **[🗺️ ROADMAP.md](ROADMAP.md)**
   - Features implementadas vs planejadas
   - Progresso por módulo
   - Prioridades (Tier 1)
   - Backlog futuro

---

## 🔧 Features Específicas

### Trading e Exchanges

- **[🔌 exchange-integration.md](exchange-integration.md)** ⭐ **NOVO!**
  - Como integrar Binance Futures/Spot
  - Como integrar Kraken Futures/Spot
  - Autenticação (HMAC, Nonce)
  - Rate limiting
  - WebSocket patterns
  - Exemplos de código

- **[💰 pnl-tracking.md](pnl-tracking.md)** ⭐ **NOVO!**
  - Sistema de rastreamento de P&L
  - Capacidades atuais (TradeStats, income history)
  - Limitações (monthly aggregation, repositories)
  - Queries SQL úteis
  - Roadmap de implementação

- **[📡 data-population.md](data-population.md)** ⭐ **NOVO!**
  - Como buscar e popular dados das exchanges
  - Workflows de backfill
  - Sincronização contínua
  - Scheduled jobs
  - Validação de dados

### Business Logic

- **[📊 business/strategy-engine.md](business/strategy-engine.md)**
  - Como funcionam as estratégias
  - Trait Strategy
  - Indicadores disponíveis
  - Como criar nova estratégia

- **[📈 business/backtesting.md](business/backtesting.md)**
  - Backtest engine
  - Configuração de backtest
  - Métricas de performance
  - Como interpretar resultados

- **[💼 business/order-execution.md](business/order-execution.md)**
  - Pipeline de execução de ordens
  - Tipos de ordem
  - Validações
  - Error handling

- **[⚠️ business/risk-management.md](business/risk-management.md)**
  - Risk Manager
  - Circuit Breaker
  - Limites e validações
  - Position sizing

### Arquitetura Técnica

- **[📐 architecture/modules.md](architecture/modules.md)**
  - Detalhamento de cada crate (942 linhas!)
  - Responsabilidades
  - Principais structs e traits
  - Exemplos de código

- **[🗄️ architecture/database-schema.md](architecture/database-schema.md)**
  - Schema SQLite completo
  - Tabelas e índices
  - Migrações
  - Queries otimizadas

- **[🔄 architecture/data-flow.md](architecture/data-flow.md)**
  - Fluxo de dados no sistema
  - Trading manual vs automático
  - WebSocket integration
  - Caching strategies

- **[❗ architecture/error-handling.md](architecture/error-handling.md)**
  - Estratégia de errors
  - Tipos de erro
  - Propagação
  - Recovery

### API Reference

- **[📡 api/README.md](api/README.md)**
  - Overview da API
  - Comandos Tauri (60+)
  - DTOs
  - Events

- **[⚡ api/tauri-commands.md](api/tauri-commands.md)**
  - Lista completa de comandos IPC
  - Assinaturas e tipos
  - Exemplos de uso

- **[📦 api/dtos.md](api/dtos.md)**
  - Data Transfer Objects
  - Estruturas de dados
  - Serialização

### ADRs (Architecture Decision Records)

- **[📝 adr/README.md](adr/README.md)** - Template e índice
- **[ADR-001](adr/ADR-001-workspace-structure.md)** - Por que 7 crates?
- **[ADR-002](adr/ADR-002-sqlite-choice.md)** - Por que SQLite?
- **[ADR-003](adr/ADR-003-worker-queue-design.md)** - Design da fila de jobs
- **[ADR-004](adr/ADR-004-strategy-versioning.md)** - Versionamento de estratégias
- **[ADR-005](adr/ADR-005-live-vs-paper-mode.md)** - Paper vs Live mode
- **[ADR-006](adr/ADR-006-telemetry-design.md)** - Design de telemetria
- **[ADR-007](adr/ADR-007-decimal-precision.md)** - Por que rust_decimal?
- **[ADR-008](adr/ADR-008-async-runtime.md)** - Por que Tokio?

---

## 📊 Documentação por Crate

### core
- **[💎 modulos/core.md](modulos/core.md)**
  - Entidades (20+)
  - Traits (10+)
  - Errors
  - DTOs

### infra
- **[🗄️ modulos/infra.md](modulos/infra.md)**
  - Config system (TOML)
  - Database (SQLite)
  - Repositories (8 implementados)
  - Logging (tracing)

### exchange_gateways
- **[🔌 modulos/exchange-gateways.md](modulos/exchange-gateways.md)**
  - BinanceFuturesClient
  - KrakenFuturesClient
  - WebSocket integration
  - Autenticação

### market_data
- **[📡 modulos/market-data.md](modulos/market-data.md)**
  - MarketDataProvider
  - Fear & Greed Provider
  - Data normalization

### analytics
- **[📊 modulos/analytics.md](modulos/analytics.md)**
  - Indicators (SMA, EMA, RSI, MACD, BB, ATR)
  - Strategies (Fear & Greed)
  - Backtest engine

### trading_worker
- **[🔧 modulos/trading-worker.md](modulos/trading-worker.md)**
  - Worker principal
  - Risk Manager
  - Circuit Breaker
  - Job Queue
  - Sync Engine
  - Ledger Service

### src-tauri
- **[🎨 modulos/tauri-app.md](modulos/tauri-app.md)**
  - Comandos IPC (60+)
  - State management
  - Services
  - Frontend integration

---

## 📖 Guias de Desenvolvimento

- **[🚀 development/getting-started.md](development/getting-started.md)** - Setup inicial
- **[📝 development/coding-standards.md](development/coding-standards.md)** - Padrões de código
- **[🧪 development/testing.md](development/testing.md)** - Como testar
- **[🚢 development/deployment.md](development/deployment.md)** - Deploy e build
- **[🤝 development/contributing.md](development/contributing.md)** - Como contribuir

---

## 🎯 Documentação por Tarefa

### "Quero fazer trading manual"
→ Leia: [GUIA-COMPLETO.md](GUIA-COMPLETO.md) → seção "Trading"

### "Quero criar uma estratégia"
→ Leia: [GUIA-DEV.md](GUIA-DEV.md) → seção "Adicionar uma nova Estratégia"
→ Depois: [business/strategy-engine.md](business/strategy-engine.md)

### "Quero integrar uma nova exchange"
→ Leia: [exchange-integration.md](exchange-integration.md)
→ Depois: [GUIA-DEV.md](GUIA-DEV.md) → seção "Adicionar Exchange"

### "Quero fazer backtest"
→ Leia: [business/backtesting.md](business/backtesting.md)
→ Depois: [GUIA-DEV.md](GUIA-DEV.md) → seção "Como fazer Backtest"

### "Quero ver P&L por mês/moeda"
→ Leia: [pnl-tracking.md](pnl-tracking.md)

### "Quero popular dados históricos"
→ Leia: [data-population.md](data-population.md)

### "Tô com erro X"
→ Leia: [FAQ.md](FAQ.md) → seção "Erros e Problemas"

### "Como contribuo?"
→ Leia: [development/contributing.md](development/contributing.md)

---

## 📊 Estatísticas da Documentação

- **Total de arquivos:** 48 markdown files
- **Total de linhas:** ~35.000+ linhas
- **Idioma:** Português (guias) + English (código)
- **Última atualização:** 2025-11-27
- **Completude:** ~75% (excelente pra v0.1.0!)

---

## 🗺️ Navegação Rápida

| Se tu quer... | Vai aqui |
|---------------|----------|
| 🎯 Começar a usar | [GUIA-COMPLETO.md](GUIA-COMPLETO.md) |
| 🛠️ Desenvolver | [GUIA-DEV.md](GUIA-DEV.md) |
| ❓ Tirar dúvida | [FAQ.md](FAQ.md) |
| 🏗️ Entender arquitetura | [architecture/overview.md](architecture/overview.md) |
| 📊 Ver roadmap | [ROADMAP.md](ROADMAP.md) |
| 🔌 Integrar exchange | [exchange-integration.md](exchange-integration.md) |
| 💰 Entender P&L | [pnl-tracking.md](pnl-tracking.md) |
| 📡 Popular dados | [data-population.md](data-population.md) |
| 🧪 Fazer testes | [development/testing.md](development/testing.md) |
| 🤝 Contribuir | [development/contributing.md](development/contributing.md) |

---

## 💡 Dica

**Perdido?** Começa pelo [GUIA-COMPLETO.md](GUIA-COMPLETO.md). Ele tem tudo que tu precisa saber pra usar o sistema, explicado de um jeito simples e direto.

**Quer desenvolver?** Vai pro [GUIA-DEV.md](GUIA-DEV.md). Tem exemplos de código e explica passo a passo como adicionar features.

---

**Boa leitura! 📖✨**
