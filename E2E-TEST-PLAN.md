# 📋 Plano Completo de Testes E2E - RoboTrade

> **Documento Gerado:** 2025-12-01  
> **Versão:** 1.0  
> **Status:** Em Implementação

## Visão Geral

Este documento descreve o plano completo de testes end-to-end (E2E) para o RoboTrade, cobrindo fluxos críticos desde ingestão de market data até reconciliação financeira.

## Objetivos

- ✅ Validar fluxo completo: Market Data → Orders → Fills → Ledger → PnL → Reconciliation
- ✅ Garantir determinismo total (seeds fixos, timestamps controláveis)
- ✅ Executar em CI (GitHub Actions) sem interação humana
- ✅ Capturar artifacts (logs, DB snapshots, coverage)
- ✅ Tempo máximo: 5 minutos por suite completa

## Stack Tecnológico

- **Database:** SQLite (test.sqlite3)
- **Mock Exchange:** Axum (Rust) com fixtures JSON
- **Test Runner:** cargo-nextest
- **Coverage:** cargo-tarpaulin
- **UI Testing:** Playwright (opcional)
- **CI/CD:** GitHub Actions

## Estrutura de Diretórios

```
tests/
├── e2e/
│   ├── common/
│   │   ├── mod.rs              # Test utilities
│   │   ├── setup.rs            # DB setup helper
│   │   ├── assertions.rs       # SQL assertion helpers
│   │   └── clock.rs            # Mock clock
│   ├── test_trade_flow.rs      # Main trade flow tests
│   ├── test_partial_fills.rs   # Partial fill scenarios
│   ├── test_reconciliation.rs  # Reconciliation tests
│   ├── test_idempotency.rs     # Worker restart tests
│   └── test_performance.rs     # High-throughput tests
└── fixtures/
    ├── db/                     # SQL seed files
    ├── exchange/               # Mock exchange responses
    └── scenarios/              # Complete test scenarios
```

## Casos de Teste Principais

### 1. E2E-001: Trade Full Fill - Happy Path
- **Prioridade:** P0 (Crítico)
- **Duração:** 15s
- **Objetivo:** Validar fluxo completo de uma ordem preenchida totalmente

### 2. E2E-002: Partial Fills - Realized/Unrealized PnL
- **Prioridade:** P0 (Crítico)
- **Duração:** 20s
- **Objetivo:** Validar cálculo de PnL em fills parciais

### 3. E2E-003: Reconciliation - Missing Ledger Entry
- **Prioridade:** P1 (Alta)
- **Duração:** 10s
- **Objetivo:** Detectar e corrigir discrepâncias no ledger

### 4. E2E-004: Worker Restart Idempotency
- **Prioridade:** P0 (Crítico)
- **Duração:** 30s
- **Objetivo:** Garantir idempotência em caso de crash

### 5. E2E-005: High-Throughput Performance
- **Prioridade:** P2 (Média)
- **Duração:** 60s
- **Objetivo:** Validar processamento de alto volume

## Scripts de Orquestração

- `scripts/e2e_setup.sh` - Preparação do ambiente
- `scripts/e2e_run.sh` - Execução dos testes
- `scripts/e2e_teardown.sh` - Limpeza e coleta de artifacts
- `scripts/apply_migrations_sqlite.sh` - Aplicação de migrations

## Critérios de Aceite

- [ ] Todos os 5 cenários principais passam consistentemente
- [ ] Ledger entries criadas para 100% dos fills
- [ ] PnL calculado corretamente (realized + unrealized)
- [ ] Reconciliação detecta discrepâncias
- [ ] Worker reinicia sem perda/duplicação de dados
- [ ] High-throughput: 200 fills em < 30s
- [ ] Determinismo: 100% (mesma seed = mesmo resultado)
- [ ] Suite completa < 5 minutos
- [ ] Coverage > 70%

## Roadmap de Implementação

### Fase 1: Fundação (Atual)
- [x] Documentação do plano
- [ ] MockExchangeServer básico
- [ ] TestEnvironment e helpers
- [ ] Caso 1 (happy path)

### Fase 2: Expansão
- [ ] Casos 2-4
- [ ] Fixtures complexas
- [ ] MockClock
- [ ] GitHub Actions

### Fase 3: Otimização
- [ ] Paralelização
- [ ] Caso 5 (performance)
- [ ] Coverage > 70%

### Fase 4: UI
- [ ] Testes Playwright
- [ ] Integração Tauri headless

## Referências

- Documento completo: Ver arquivos em `docs/testing/`
- Mock Exchange: `crates/exchange_gateways/src/mock/`
- Fixtures: `tests/fixtures/`
