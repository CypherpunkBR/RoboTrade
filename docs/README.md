# RoboTrade - Documentação Técnica

Esta pasta contém toda a documentação técnica, de negócio e decisões arquiteturais do projeto RoboTrade.

## Estrutura da Documentação

```
docs/
├── README.md                    # Este arquivo
├── ROADMAP.md                   # Roadmap de desenvolvimento
│
├── arquitetura/                 # Documentação de arquitetura
│   └── README.md                # Visão geral e diagrama de camadas
│
├── modulos/                     # Documentação dos módulos
│   ├── core.md                  # Entidades, traits, erros, DTOs
│   ├── exchange-gateways.md     # Integração com exchanges
│   ├── market-data.md           # Providers de dados de mercado
│   ├── analytics.md             # Backtest e estratégias
│   ├── trading-worker.md        # Scheduler e workers
│   ├── infra.md                 # Configuração, DB, logging
│   └── tauri-app.md             # Aplicação desktop
│
├── adr/                         # Architecture Decision Records (TODO)
├── api/                         # Documentação de APIs (TODO)
└── development/                 # Guias de desenvolvimento (TODO)
```

## Documentação dos Módulos

| Módulo | Descrição |
|--------|-----------|
| [Core](./modulos/core.md) | Entidades, traits, erros e DTOs - fundação do sistema |
| [Exchange Gateways](./modulos/exchange-gateways.md) | Clientes de exchanges (Binance Futures) |
| [Market Data](./modulos/market-data.md) | Providers de dados (Fear & Greed Index) |
| [Analytics](./modulos/analytics.md) | Engine de backtest e estratégias |
| [Trading Worker](./modulos/trading-worker.md) | Scheduler event-driven e workers |
| [Infra](./modulos/infra.md) | Configuração TOML, SQLite, logging |
| [Tauri App](./modulos/tauri-app.md) | Aplicação desktop com system tray |

## Princípios do Projeto

### Separação de Responsabilidades

1. **Rust** - 100% da lógica de negócio, cálculos, validações e persistência
2. **Tauri** - Ponte entre backend Rust e frontend React
3. **React** - Apenas interface visual, sem lógica de negócio
4. **SQLite** - Banco de dados local centralizado

### Segurança por Design

- Modo paper trading é o padrão
- Modo live requer confirmação explícita e múltiplas validações
- API keys nunca são expostas ao frontend
- Validação independente de dados vindos do React

### Modularidade

Cada crate Rust é independente e reutilizável:
- `core` não depende de nenhum outro crate interno
- Dependências fluem sempre do core para as pontas
- Traits definem contratos, implementações são intercambiáveis

## Como Navegar

- **Novo no projeto?** Comece por [architecture/overview.md](./architecture/overview.md)
- **Quer entender as regras de negócio?** Veja [business/](./business/)
- **Precisa entender uma decisão técnica?** Consulte [adr/](./adr/)
- **Vai desenvolver?** Leia [development/getting-started.md](./development/getting-started.md)

## Convenções

### Nomenclatura de Arquivos

- Usar kebab-case para nomes de arquivos
- Prefixo ADR-XXX para Architecture Decision Records
- Extensão .md para todos os documentos

### Formato dos Documentos

- Títulos em português
- Código e termos técnicos em inglês
- Diagramas em ASCII art ou Mermaid quando necessário

## Versionamento

A documentação acompanha a versão do projeto. Alterações significativas devem ser refletidas nos documentos correspondentes.

---

**Última atualização**: 2024-01
**Versão do projeto**: 0.1.0
