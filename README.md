# 🤖 RoboTrade

> Robô de trading automatizado para futuros de criptomoedas - Feito com Rust 🦀 + Tauri 2.0

[![Quality Gates](https://github.com/CypherpunkBR/RoboTrade/workflows/Quality%20Gates/badge.svg)](https://github.com/CypherpunkBR/RoboTrade/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🎯 O que é isso?

RoboTrade é uma **plataforma desktop completa** pra fazer trading de futuros de criptomoedas. Tu pode:

- 📊 **Tradear manualmente** com interface gráfica intuitiva
- 🤖 **Deixar o robô operar sozinho** com tuas estratégias
- 📈 **Analisar performance** com relatórios visuais detalhados
- ⚠️ **Gerenciar risco** automaticamente (circuit breaker, stop loss, etc)
- 🎮 **Testar sem risco** no paper trading antes de usar dinheiro real

**Suporta:** Binance Futures + Kraken Futures (testnet e produção)

---

## ✨ Features

### ✅ Já Funciona (Use Agora!)

- **Trading Manual Completo**
  - Ordens: Market, Limit, Stop Market, Stop Limit
  - Alavancagem configurável (1-125x)
  - Stop Loss e Take Profit automáticos
  - Trailing Stop
  - Multi-exchange (Binance + Kraken)

- **Análise de Mercado**
  - Gráficos de candlestick interativos
  - Indicadores técnicos (SMA, EMA, RSI, MACD, Bollinger Bands, ATR)
  - Fear & Greed Index em tempo real
  - Sistema de alertas de preço (6 tipos de condição)

- **Gestão de Risco**
  - Risk Manager automático
  - Circuit Breaker (para tudo se perder demais)
  - Limites de perda diária
  - Tamanho máximo por posição
  - Número máximo de posições simultâneas

- **Relatórios e Histórico**
  - Histórico completo de trades e execuções
  - Estatísticas detalhadas (win rate, profit factor, sharpe ratio)
  - Gráficos de PnL ao longo do tempo
  - Equity curve
  - PnL por símbolo
  - Exportação de dados

- **Paper Trading**
  - Simula trades sem gastar dinheiro
  - Testnet das exchanges (Binance + Kraken)
  - Perfeito pra aprender!

- **Sistema Avançado**
  - WebSocket para dados em tempo real
  - Ledger system (contabilidade dupla)
  - P&L calculator com cost basis
  - Reconciliação exchange vs ledger interno
  - Sincronização histórica de dados
  - SQLite database local
  - Configuração via TOML
  - Logging estruturado

### ⏳ Em Desenvolvimento

- Trading automático end-to-end (infraestrutura pronta, falta ativar)
- Interface de backtest no frontend
- Monthly P&L aggregation tables
- Mais estratégias (RSI, MACD, Grid Trading)

### ❌ Planejado

- Binance Spot API
- Kraken Spot API
- Notificações desktop/Telegram
- Machine Learning strategies
- Mobile app

---

## 🚀 Quick Start

### 1. Requisitos

- **Rust** 1.70+ ([instalar](https://rustup.rs/))
- **Node.js** 18+ ([instalar](https://nodejs.org/))
- **API Keys** da Binance/Kraken ([como conseguir](docs/GUIA-COMPLETO.md#credenciais-da-exchange))

### 2. Instalação

```bash
# Clone o repositório
git clone https://github.com/CypherpunkBR/RoboTrade.git
cd RoboTrade

# Instala dependências
npm install

# Configura credenciais
cp .env.example .env
# Edita .env com tuas API keys
```

### 3. Configurar .env

```bash
# Binance Futures
BINANCE_API_KEY=tua_api_key
BINANCE_API_SECRET=teu_secret
BINANCE_TESTNET=true  # true = testnet (seguro), false = real

# Kraken Futures
KRAKEN_FUTURES_API_KEY=tua_api_key
KRAKEN_FUTURES_API_SECRET=teu_secret
KRAKEN_FUTURES_DEMO=true  # true = demo (seguro), false = real

# Logging (opcional)
RUST_LOG=info
```

### 4. Rodar

```bash
# Modo desenvolvimento (hot reload)
npm run tauri dev

# Build produção
npm run tauri build
```

**Pronto!** 🎉 O app vai abrir e tu já pode começar a tradear!

---

## 📚 Documentação

### Pra Usuários
- **[📖 Guia Completo](docs/GUIA-COMPLETO.md)** - Como usar o RoboTrade (português informal)
- **[❓ FAQ](docs/FAQ.md)** - Perguntas frequentes
- **[⚙️ Configuração](docs/development/getting-started.md)** - Setup detalhado

### Pra Desenvolvedores
- **[🛠️ Guia do Dev](docs/GUIA-DEV.md)** - Como contribuir e desenvolver
- **[🏗️ Arquitetura](docs/architecture/overview.md)** - Como o sistema funciona
- **[📋 Coding Standards](docs/development/coding-standards.md)** - Convenções de código
- **[🗺️ Roadmap](docs/ROADMAP.md)** - O que tá planejado

### Features Específicas
- **[🔌 Exchange Integration](docs/exchange-integration.md)** - Como integrar exchanges
- **[💰 P&L Tracking](docs/pnl-tracking.md)** - Sistema de lucro e prejuízo
- **[📡 Data Population](docs/data-population.md)** - Como popular dados das APIs

---

## 🏗️ Arquitetura

### Workspace de 7 Crates Rust

```
┌─────────────────────────────────────┐
│      src-tauri (Tauri App)          │
│   Frontend Integration Layer        │
└─────────────────────────────────────┘
             ↓ usa
┌──────────┬──────────┬─────────┬──────────┐
│ trading_ │analytics │exchange_│market_   │
│ worker   │          │gateways │data      │
└──────────┴──────────┴─────────┴──────────┘
             ↓ usa
┌─────────────────────────────────────┐
│      infra (Database, Config)       │
└─────────────────────────────────────┘
             ↓ usa
┌─────────────────────────────────────┐
│  core (Entities, Traits, Errors)    │
│      FUNDAÇÃO - Sem dependências    │
└─────────────────────────────────────┘
```

**Princípio:** Layered architecture com separação clara de responsabilidades.

### Stack Tecnológica

- **Backend:** Rust 🦀 (async com Tokio)
- **Frontend:** React + TypeScript
- **Desktop:** Tauri 2.0
- **Database:** SQLite (sqlx com compile-time checks)
- **Charts:** lightweight-charts + Recharts
- **Precisão Financeira:** rust_decimal (28 casas decimais!)

---

## 🎮 Screenshots

### Dashboard
![Dashboard](docs/screenshots/dashboard.png)
*Visão geral: saldos, posições, ordens, Fear & Greed*

### Trading
![Trading](docs/screenshots/trading.png)
*Gráfico interativo + criação de ordens + gestão de risco*

### Relatórios
![Reports](docs/screenshots/reports.png)
*Análise visual de performance com gráficos*

---

## 🧪 Testes e Qualidade

### CI/CD Automático

- ✅ Formatação (rustfmt)
- ✅ Linting rigoroso (clippy: all + pedantic + nursery)
- ✅ Testes unitários e de integração
- ✅ Coverage com tarpaulin → Codecov
- ✅ Security audit (cargo-audit + cargo-deny)
- ✅ Dependency analysis
- ✅ Build release

### Quality Gates

```bash
# Formata código
cargo fmt --all

# Lint rigoroso
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Testa tudo
cargo test --workspace --all-features

# Coverage
cargo tarpaulin --workspace --all-features

# Build release
cargo build --workspace --release
```

**Meta de Coverage:** >= 70% (ideal: >= 80%)

---

## 🤝 Contribuindo

Contribuições são muito bem-vindas! 🎉

### Como Contribuir

1. Fork o projeto
2. Cria uma branch (`git checkout -b feature/minha-feature`)
3. Implementa tua feature + testes
4. Commita (`git commit -m 'feat: add minha feature'`)
5. Push (`git push origin feature/minha-feature`)
6. Abre um Pull Request

### Áreas que Precisam de Ajuda

- 📊 Novas estratégias de trading
- 📈 Novos indicadores técnicos
- 🧪 Mais testes
- 📖 Documentação e tutoriais
- 🎨 UI/UX improvements
- 🐛 Bug fixes

### Código de Conduta

- Respeita todo mundo
- Crítica construtiva
- Sem spam ou propaganda
- Sem código malicioso

---

## ⚠️ Disclaimer

**IMPORTANTE - LEIA ANTES DE USAR:**

- ✋ Este software é fornecido **"como está"**, sem garantias
- 📉 **Tu pode perder dinheiro** fazendo trading
- 🎰 Trading de futuros com alavancagem é **extremamente arriscado**
- 💸 Nunca investe mais do que pode perder
- 🚫 **Não é conselho financeiro** - DYOR (Do Your Own Research)
- 👨‍💻 Os desenvolvedores **não se responsabilizam** por perdas financeiras

**Use por sua conta e risco!**

---

## 📄 Licença

MIT License - Veja [LICENSE](LICENSE) para detalhes.

---

## 🙏 Agradecimentos

- [Tauri](https://tauri.app/) - Framework desktop incrível
- [Rust](https://www.rust-lang.org/) - Linguagem robusta e rápida
- [Binance](https://www.binance.com/) e [Kraken](https://www.kraken.com/) - APIs excelentes
- Comunidade open source 💙

---

## 📞 Contato e Suporte

- **Issues:** [GitHub Issues](https://github.com/CypherpunkBR/RoboTrade/issues)
- **Documentação:** [/docs](docs/)
- **Guia Completo:** [GUIA-COMPLETO.md](docs/GUIA-COMPLETO.md)
- **FAQ:** [FAQ.md](docs/FAQ.md)

---

**Feito com 💙 pela comunidade CypherpunkBR**

**Boas trades! 🚀📈**
