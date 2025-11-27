# RoboTrade - Documentação Técnica

Documentação completa do projeto RoboTrade - trading bot automatizado em Rust + Tauri + React.

## Estrutura da Documentação

```
docs/
├── README.md                       # Este arquivo
├── ROADMAP.md                      # Roadmap de desenvolvimento
│
├── architecture/                   # Documentação de Arquitetura
│   ├── overview.md                # Visão geral do sistema
│   ├── modules.md                 # Detalhamento dos módulos
│   ├── data-flow.md               # Fluxo de dados
│   ├── database-schema.md         # Schema do SQLite
│   ├── error-handling.md          # Tratamento de erros
│   ├── security.md                # Segurança e modos de trading
│   ├── config-system.md           # Sistema de configuração
│   └── telemetry.md               # Métricas e logging
│
├── business/                       # Regras de Negócio
│   ├── market-data.md             # Coleta de dados de mercado
│   ├── strategy-engine.md         # Engine de estratégias
│   ├── order-execution.md         # Execução de ordens
│   ├── risk-management.md         # Gestão de risco
│   ├── queue-system.md            # Sistema de filas
│   └── backtesting.md             # Motor de backtesting
│
├── adr/                            # Architecture Decision Records
│   ├── README.md                  # Índice e template
│   ├── ADR-001-workspace-structure.md
│   ├── ADR-002-sqlite-choice.md
│   ├── ADR-003-worker-queue-design.md
│   ├── ADR-004-strategy-versioning.md
│   ├── ADR-005-live-vs-paper-mode.md
│   ├── ADR-006-telemetry-design.md
│   ├── ADR-007-decimal-precision.md
│   └── ADR-008-async-runtime.md
│
├── api/                            # Documentação da API
│   ├── README.md                  # Visão geral
│   ├── tauri-commands.md          # Comandos IPC
│   ├── dtos.md                    # Data Transfer Objects
│   ├── events.md                  # Sistema de eventos
│   └── error-codes.md             # Códigos de erro
│
└── development/                    # Guias de Desenvolvimento
    ├── README.md                  # Índice
    ├── getting-started.md         # Setup inicial
    ├── coding-standards.md        # Padrões de código
    ├── testing.md                 # Guia de testes
    ├── deployment.md              # Build e distribuição
    └── contributing.md            # Guia de contribuição
```

## Quick Links

### Começando

| Documento | Descrição |
|-----------|-----------|
| [Architecture Overview](./architecture/overview.md) | Visão geral da arquitetura do sistema |
| [Getting Started](./development/getting-started.md) | Setup do ambiente de desenvolvimento |
| [Coding Standards](./development/coding-standards.md) | Padrões e convenções de código |

### Arquitetura

| Documento | Descrição |
|-----------|-----------|
| [Modules](./architecture/modules.md) | Detalhamento de cada crate |
| [Data Flow](./architecture/data-flow.md) | Fluxo de dados no sistema |
| [Database Schema](./architecture/database-schema.md) | Schema completo do SQLite |
| [Security](./architecture/security.md) | Live vs Paper mode, segurança |

### Regras de Negócio

| Documento | Descrição |
|-----------|-----------|
| [Strategy Engine](./business/strategy-engine.md) | Indicadores e estratégias |
| [Order Execution](./business/order-execution.md) | Pipeline de execução de ordens |
| [Risk Management](./business/risk-management.md) | Gestão de risco e limites |
| [Backtesting](./business/backtesting.md) | Motor de simulação |

### API

| Documento | Descrição |
|-----------|-----------|
| [Tauri Commands](./api/tauri-commands.md) | Referência dos comandos IPC |
| [DTOs](./api/dtos.md) | Estruturas de dados |
| [Events](./api/events.md) | Sistema de eventos assíncronos |
| [Error Codes](./api/error-codes.md) | Códigos de erro e tratamento |

### Decisões Arquiteturais (ADRs)

| ADR | Título |
|-----|--------|
| [ADR-001](./adr/ADR-001-workspace-structure.md) | Estrutura do Workspace |
| [ADR-002](./adr/ADR-002-sqlite-choice.md) | Escolha do SQLite |
| [ADR-003](./adr/ADR-003-worker-queue-design.md) | Design do Sistema de Filas |
| [ADR-004](./adr/ADR-004-strategy-versioning.md) | Versionamento de Estratégias |
| [ADR-005](./adr/ADR-005-live-vs-paper-mode.md) | Modo Live vs Paper |
| [ADR-006](./adr/ADR-006-telemetry-design.md) | Design de Telemetria |
| [ADR-007](./adr/ADR-007-decimal-precision.md) | Precisão Decimal |
| [ADR-008](./adr/ADR-008-async-runtime.md) | Runtime Assíncrono |

## Princípios do Projeto

### Separação de Responsabilidades

```
┌──────────────────────────────────────────────────────────────┐
│                    Frontend (React)                          │
│                  Apenas UI - Zero lógica                     │
└──────────────────────────────────────────────────────────────┘
                              │
                              │ IPC (Tauri)
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                    Backend (Rust)                            │
│            100% da lógica de negócio e estado               │
│                                                              │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌─────────────────┐  │
│  │  core   │ │ analytics│ │market_data│ │ trading_worker │  │
│  └─────────┘ └──────────┘ └──────────┘ └─────────────────┘  │
│                           │                                  │
│  ┌─────────────────────┐  │  ┌────────────────────────────┐ │
│  │ exchange_gateways   │◄─┼──│         infra              │ │
│  └─────────────────────┘  │  └────────────────────────────┘ │
│                           │                                  │
│                           ▼                                  │
│                    ┌──────────────┐                         │
│                    │   SQLite     │                         │
│                    └──────────────┘                         │
└──────────────────────────────────────────────────────────────┘
```

### Segurança por Design

- **Paper trading é o padrão** - Live trading requer confirmação explícita
- **API keys protegidas** - Armazenadas no keyring do sistema, nunca expostas
- **Validação independente** - Backend não confia em dados do frontend
- **Limites de risco** - Circuit breakers e limites configuráveis

### Type Safety

- `rust_decimal::Decimal` para todos os valores financeiros
- Tipos distintos para estados diferentes (Live vs Paper)
- Validação em compile-time quando possível

## Como Navegar

| Eu quero... | Veja... |
|-------------|---------|
| Entender a arquitetura geral | [Architecture Overview](./architecture/overview.md) |
| Configurar ambiente de dev | [Getting Started](./development/getting-started.md) |
| Entender uma decisão técnica | [ADRs](./adr/README.md) |
| Implementar uma estratégia | [Strategy Engine](./business/strategy-engine.md) |
| Integrar com o backend | [Tauri Commands](./api/tauri-commands.md) |
| Rodar um backtest | [Backtesting](./business/backtesting.md) |
| Contribuir com código | [Contributing](./development/contributing.md) |

## Stack Tecnológico

| Camada | Tecnologia | Uso |
|--------|------------|-----|
| Backend | Rust 1.75+ | Lógica de negócio |
| Async Runtime | Tokio | Concorrência |
| Desktop Framework | Tauri 2.0 | IPC e window management |
| Frontend | React 18 + TypeScript | Interface do usuário |
| Database | SQLite + sqlx | Persistência local |
| Serialização | serde + JSON | Comunicação |
| Logging | tracing | Observabilidade |
| HTTP Client | reqwest | APIs REST |
| WebSocket | tokio-tungstenite | Streams de dados |

## Convenções

### Nomenclatura

- **Arquivos**: kebab-case (`strategy-engine.md`)
- **ADRs**: Prefixo `ADR-XXX-` (`ADR-001-workspace-structure.md`)
- **Código Rust**: snake_case para funções/variáveis, PascalCase para tipos
- **TypeScript**: camelCase para variáveis, PascalCase para tipos/interfaces

### Idioma

- Títulos e explicações em **português**
- Código e termos técnicos em **inglês**
- Commits e PRs em **inglês**

---

**Versão do Projeto**: 0.1.0
**Última Atualização**: 2025-11-27
