# RoboTrade - Roadmap

Roadmap de desenvolvimento do RoboTrade - trading bot automatizado em Rust + Tauri + React.

## Visão Geral

O RoboTrade é desenvolvido em fases incrementais, cada uma adicionando funcionalidades completas e testadas.

```
┌─────────────────────────────────────────────────────────────────────┐
│                           ROADMAP                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Phase 1: Foundation ✓        Phase 2: Trading ◐      Phase 3: UI  │
│  ═══════════════════          ══════════════════      ════════════ │
│  ✓ Core types                 ✓ Position tracking     □ Dashboard  │
│  ✓ Database schema            ✓ Risk management       □ Charts     │
│  ✓ Config system              ✓ Circuit breaker       □ Settings   │
│  ✓ Binance client             ✓ Job queue             □ Alerts     │
│  ✓ Kraken client              □ Paper trading         □ History    │
│  ✓ Fear & Greed provider      □ Order execution                    │
│                                                                     │
│  Phase 4: Advanced ◐          Phase 5: Polish         Phase 6:     │
│  ════════════════════         ═══════════════════     Production   │
│  ✓ Backtest engine            □ Performance           ════════════ │
│  ✓ All indicators             ✓ Documentation         □ Security   │
│  ✓ Fear & Greed strategy      □ Error handling        □ Deployment │
│  □ Multiple strategies        □ Notifications         □ Updates    │
│  □ Portfolio mgmt                                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Progresso Atual: ~70% da Fundação Completa

| Módulo | Completude | Notas |
|--------|------------|-------|
| robotrade-core | 100% | Entidades, DTOs, erros, traits |
| robotrade-analytics | 100% | 6 indicadores + backtest engine |
| robotrade-trading-worker | 100% | Scheduler, risk, circuit breaker |
| robotrade-exchange-gateways | 66% | Binance + Kraken (falta Paper) |
| robotrade-infra | 69% | Config + DB + 4/13 repositórios |
| robotrade-market-data | 33% | Apenas Fear & Greed provider |

---

## Phase 1: Foundation ✓ (Completa)

**Objetivo**: Estabelecer a base do sistema com tipos, infraestrutura e conexões básicas.

### Concluído ✓

- [x] Estrutura do workspace Cargo (6 crates)
- [x] Tipos core (Candlestick, Order, Position, Trade, Signal, Strategy, etc.)
- [x] Traits fundamentais (ExchangeGateway, Repository, Strategy)
- [x] Sistema de erros hierárquico com thiserror
- [x] Schema do banco de dados SQLite (35+ tabelas, 18 views, 20+ triggers)
- [x] Migrations com sqlx (3 arquivos de migração)
- [x] Sistema de configuração TOML
- [x] Cliente Binance Futures (REST) - 870 linhas
- [x] Cliente Kraken Futures (REST) - 776 linhas
- [x] Estrutura de documentação completa (45+ arquivos .md)
- [x] Provider Fear & Greed Index funcional
- [x] Database row types (842 linhas de FromRow structs)
- [x] 4 repositórios SQLite implementados (Candle, FearGreed, Exchange, Symbol)

### Pendente

- [ ] Cache em memória para market data
- [ ] Rate limiter para API calls
- [ ] WebSocket Binance para streams
- [ ] 9 repositórios adicionais (Account, Order, Position, Trade, Signal, Strategy, Risk, Job, Notification)

### Entregáveis

- Crates `core`, `infra`, `exchange_gateways` funcionais
- Banco de dados inicializado com migrations
- Conexão com Binance e Kraken APIs
- Provider Fear & Greed operacional

---

## Phase 2: Trading Engine ◐ (Em Progresso)

**Objetivo**: Implementar execução de ordens e gestão de posições.

### Concluído ✓

- [x] Position Manager (491 linhas)
  - [x] Abrir posições
  - [x] Fechar posições (total/parcial)
  - [x] Eventos de posição (PositionEvent)
  - [x] Stop Loss / Take Profit

- [x] Risk Management (522 linhas)
  - [x] Limites de posição
  - [x] Daily loss limit
  - [x] Max leverage
  - [x] RiskCheckResult e RiskRejectionReason

- [x] Circuit Breaker (495 linhas)
  - [x] Estados: Closed, Open, HalfOpen
  - [x] Auto-recovery com half-open state
  - [x] CircuitBreakerEvent

- [x] Job Queue (567 linhas)
  - [x] JobType, JobPayload
  - [x] JobPriority (Low, Normal, High)
  - [x] FIFO + priority handling

- [x] Scheduler (8,718 linhas)
  - [x] Event-driven architecture
  - [x] SchedulerEvent enum
  - [x] Periodic task execution

- [x] Data Collector (4,259 linhas)
  - [x] Market data aggregation

### Pendente

- [ ] Order Manager
  - [ ] Criar ordens de mercado e limitadas
  - [ ] Cancelar ordens
  - [ ] Rastrear status de ordens
  - [ ] Histórico de ordens

- [ ] Paper Trading
  - [ ] Simulador de execução local
  - [ ] Balanço virtual
  - [ ] Histórico de trades paper

### Entregáveis

- Crate `trading_worker` ✅ FUNCIONAL
- ⚠️ Falta: Paper trading client
- ⚠️ Falta: Order execution pipeline

---

## Phase 3: Strategy Engine ✓ (Completa)

**Objetivo**: Criar sistema de estratégias configuráveis.

### Concluído ✓

- [x] Indicadores Técnicos (todos com testes)
  - [x] SMA (Simple Moving Average)
  - [x] EMA (Exponential Moving Average)
  - [x] RSI (Relative Strength Index)
  - [x] MACD (Moving Average Convergence Divergence)
  - [x] Bollinger Bands
  - [x] ATR (Average True Range)

- [x] Strategy Framework
  - [x] Trait Strategy definida
  - [x] Trait Indicator & CandleIndicator
  - [x] Helpers: extract_close_prices, extract_high_prices, etc.

- [x] Fear & Greed Strategy (377 linhas)
  - [x] Integração com índice Alternative.me
  - [x] Regras de entrada/saída
  - [x] Parâmetros configuráveis

- [x] Backtest Engine (635 linhas)
  - [x] Simulador de mercado
  - [x] Equity curve tracking
  - [x] Drawdown calculation
  - [x] Trade statistics

### Pendente

- [ ] Strategy Versioning
  - [ ] Histórico de versões
  - [ ] Comparação de performance
  - [ ] Rollback

### Entregáveis

- Crate `analytics` ✅ FUNCIONAL
- 6 indicadores técnicos completos
- Fear & Greed strategy implementada
- Backtest engine operacional

---

## Phase 4: Backtesting ✓ (Parcialmente Completa)

**Objetivo**: Motor de backtesting para validação de estratégias.

### Concluído ✓

- [x] Backtest Engine (635 linhas)
  - [x] Simulador de mercado
  - [x] Execução de estratégias em dados históricos
  - [x] Cálculo de métricas

- [x] Métricas
  - [x] Total Return
  - [x] Max Drawdown
  - [x] Win Rate
  - [x] Equity curve
  - [x] Trade statistics

- [x] Visualização (dados estruturados)
  - [x] Equity curve (JSON)
  - [x] Trade list (JSON)

### Pendente

- [ ] Métricas Avançadas
  - [ ] Sharpe Ratio
  - [ ] Sortino Ratio
  - [ ] Profit Factor

- [ ] Otimização
  - [ ] Grid search de parâmetros
  - [ ] Walk-forward analysis
  - [ ] Monte Carlo simulation

- [ ] UI de Visualização
  - [ ] Gráfico de equity curve
  - [ ] Drawdown chart visual

### Entregáveis

- Backtest engine ✅ FUNCIONAL
- Suite básica de métricas ✅
- ⚠️ Falta: métricas avançadas (Sharpe, Sortino)
- ⚠️ Falta: otimização de parâmetros

---

## Phase 5: User Interface

**Objetivo**: Interface desktop completa com Tauri + React.

### Tarefas

- [ ] Dashboard
  - [ ] Visão geral do portfolio
  - [ ] Posições abertas
  - [ ] Ordens pendentes
  - [ ] P&L do dia

- [ ] Market Data
  - [ ] Lista de símbolos
  - [ ] Gráfico de candlesticks
  - [ ] Order book (opcional)

- [ ] Trading
  - [ ] Formulário de ordem
  - [ ] Confirmação visual
  - [ ] Histórico de trades

- [ ] Strategies
  - [ ] Lista de estratégias
  - [ ] Configuração
  - [ ] Ativar/desativar
  - [ ] Performance

- [ ] Settings
  - [ ] Configuração geral
  - [ ] API credentials
  - [ ] Notificações
  - [ ] Tema (light/dark)

### Entregáveis

- Aplicação Tauri funcional
- UI React completa
- Comandos IPC implementados
- Notificações do sistema

---

## Phase 6: Live Trading

**Objetivo**: Suporte a trading real com todas as salvaguardas.

### Tarefas

- [ ] Live Mode
  - [ ] Ativação segura
  - [ ] Confirmações visuais
  - [ ] Indicadores de modo

- [ ] Security
  - [ ] Keyring para credentials
  - [ ] Rate limiting
  - [ ] IP whitelist reminder

- [ ] Monitoring
  - [ ] Logs estruturados
  - [ ] Métricas em tempo real
  - [ ] Alertas

- [ ] Safeguards
  - [ ] Limites de perda
  - [ ] Pausar em erros
  - [ ] Rollback de estratégia

### Entregáveis

- Modo live funcional e seguro
- Sistema de alertas
- Logging completo
- Documentação de segurança

---

## Phase 7: Polish & Release

**Objetivo**: Preparar para release público.

### Tarefas

- [ ] Performance
  - [ ] Profiling e otimização
  - [ ] Redução de memória
  - [ ] Startup time

- [ ] Quality
  - [ ] Testes E2E
  - [ ] Code review completo
  - [ ] Security audit

- [ ] Documentation
  - [ ] User guide
  - [ ] API reference
  - [ ] FAQ

- [ ] Distribution
  - [ ] Build para Windows/macOS/Linux
  - [ ] Auto-updater
  - [ ] Installer

### Entregáveis

- Aplicação otimizada
- Testes abrangentes
- Documentação completa
- Builds para todas as plataformas

---

## Futuro (Backlog)

Features consideradas para versões futuras:

### Multi-Exchange

- [ ] Suporte a outras exchanges (Bybit, OKX, Kraken)
- [ ] Arbitragem entre exchanges
- [ ] Smart order routing
- [ ] Agregação de orderbook

### Advanced Strategies

- [ ] Machine Learning integration
- [ ] Portfolio rebalancing
- [ ] Dollar Cost Averaging automático
- [ ] Grid trading
- [ ] Sentiment analysis

### Social Features

- [ ] Compartilhar estratégias
- [ ] Leaderboard
- [ ] Copy trading

### Mobile & Notifications

- [ ] App iOS/Android (Tauri Mobile)
- [ ] Notificações push
- [ ] Telegram bot
- [ ] Webhooks

### Advanced Analytics

- [ ] On-chain metrics
- [ ] Funding rate analysis
- [ ] Whale movements tracking
- [ ] Correlation analysis

---

## Exchanges Suportadas

| Exchange | Status | Tipo | Prioridade |
|----------|--------|------|------------|
| Binance Futures | ✅ Implementado | Perpetual | - |
| Kraken Futures | ✅ Implementado | Perpetual | - |
| Paper Trading | 📋 Pendente | Simulador | Alta |
| Binance Spot | 📋 Planejado | Spot | Média |
| Bybit | 📋 Planejado | Perpetual | Baixa |
| OKX | 📋 Planejado | Perpetual | Baixa |

---

## Stack Tecnológico

| Camada | Tecnologia | Uso |
|--------|------------|-----|
| Backend | Rust 1.75+ | 100% da lógica de negócio |
| Async Runtime | Tokio | Concorrência |
| Desktop Framework | Tauri 2.0 | IPC e window management |
| Frontend | React 18 + TypeScript | Interface do usuário |
| UI Components | shadcn/ui + Tailwind | Componentes e estilo |
| Database | SQLite + sqlx | Persistência local |
| Charts | TradingView Lightweight | Gráficos de mercado |
| Logging | tracing | Observabilidade |

---

## Priorização

As prioridades são definidas por:

1. **Valor para o usuário**: Features que habilitam uso real
2. **Dependências técnicas**: Ordem lógica de implementação
3. **Risco**: Features críticas primeiro para validar arquitetura
4. **Esforço vs. Impacto**: Quick wins quando possível

---

## Como Contribuir

Quer ajudar? Veja:

1. Issues marcadas como `good first issue`
2. Features no roadmap marcadas como `help wanted`
3. Documentação e testes sempre bem-vindos

Consulte [Contributing Guide](./development/contributing.md) para detalhes.

---

**Última atualização**: 2025-11-27
**Versão do projeto**: 0.1.0

---

## Resumo de Progresso por Módulo

| Crate | Linhas de Código | Status | Completude |
|-------|------------------|--------|------------|
| robotrade-core | ~18,000+ | ✅ Pronto | 100% |
| robotrade-analytics | ~1,200 | ✅ Pronto | 100% |
| robotrade-trading-worker | ~14,000 | ✅ Pronto | 100% |
| robotrade-exchange-gateways | ~2,200 | ◐ Parcial | 66% |
| robotrade-infra | ~1,500+ | ◐ Parcial | 69% |
| robotrade-market-data | ~300 | ◐ Mínimo | 33% |

### Próximas Prioridades (Tier 1)

1. **Implementar Paper Trading Client** - Necessário para testes seguros
2. **Implementar repositórios pendentes** - 9 repositórios faltando
3. **Adicionar Binance WebSocket provider** - Market data em tempo real
