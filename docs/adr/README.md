# Architecture Decision Records (ADRs)

Este diretório contém os registros de decisões arquiteturais do projeto RoboTrade.

## O que é uma ADR?

Uma Architecture Decision Record (ADR) é um documento que captura uma decisão arquitetural importante junto com seu contexto e consequências.

## Formato

Cada ADR segue o formato:
- **Título**: Descrição curta da decisão
- **Status**: Proposta, Aceita, Deprecada, Substituída
- **Contexto**: Situação que levou à decisão
- **Decisão**: A decisão tomada
- **Consequências**: Impactos positivos e negativos

## Índice de ADRs

| ID | Título | Status | Data |
|----|--------|--------|------|
| [ADR-001](./ADR-001-workspace-structure.md) | Estrutura do Workspace | Aceita | 2024-01 |
| [ADR-002](./ADR-002-sqlite-choice.md) | Escolha do SQLite | Aceita | 2024-01 |
| [ADR-003](./ADR-003-worker-queue-design.md) | Design do Sistema de Filas | Aceita | 2024-01 |
| [ADR-004](./ADR-004-strategy-versioning.md) | Versionamento de Estratégias | Aceita | 2024-01 |
| [ADR-005](./ADR-005-live-vs-paper-mode.md) | Modo Live vs Paper | Aceita | 2024-01 |
| [ADR-006](./ADR-006-telemetry-design.md) | Design de Telemetria | Aceita | 2024-01 |
| [ADR-007](./ADR-007-decimal-precision.md) | Precisão Decimal | Aceita | 2024-01 |
| [ADR-008](./ADR-008-async-runtime.md) | Runtime Assíncrono | Aceita | 2024-01 |

## Como Adicionar uma Nova ADR

1. Criar arquivo com formato `ADR-XXX-titulo-kebab-case.md`
2. Usar o template abaixo
3. Adicionar entrada neste índice
4. Submeter PR para review

## Template

```markdown
# ADR-XXX: Título da Decisão

## Status

[Proposta | Aceita | Deprecada | Substituída por ADR-YYY]

## Contexto

Descreva a situação e o problema que está sendo resolvido.

## Decisão

Descreva a decisão tomada.

## Consequências

### Positivas
- ...

### Negativas
- ...

### Neutras
- ...

## Alternativas Consideradas

### Alternativa 1
- Descrição
- Prós
- Contras
- Motivo da rejeição

## Referências

- Links relevantes
```
