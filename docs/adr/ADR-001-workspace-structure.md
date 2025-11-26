# ADR-001: Estrutura do Workspace

## Status

Aceita

## Contexto

O RoboTrade é uma aplicação complexa que engloba múltiplos domínios: coleta de dados de mercado, análise técnica, execução de ordens, gerenciamento de risco, persistência de dados e interface gráfica. A organização do código precisa:

1. Permitir desenvolvimento paralelo de diferentes componentes
2. Facilitar testes isolados de cada módulo
3. Permitir reuso de código entre diferentes contextos (CLI, Tauri, testes)
4. Manter tempos de compilação aceitáveis
5. Separar claramente as responsabilidades de cada componente

## Decisão

Adotamos uma estrutura de **Cargo Workspace** com múltiplos crates organizados por domínio:

```
robotrade/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Tipos fundamentais e traits
│   ├── infra/              # Implementações de infraestrutura
│   ├── market_data/        # Coleta e processamento de dados
│   ├── analytics/          # Indicadores e análise técnica
│   ├── trading_worker/     # Execução de estratégias
│   ├── exchange_gateways/  # Integrações com exchanges
│   └── src-tauri/          # Aplicação desktop Tauri
```

### Hierarquia de Dependências

```
                    ┌─────────────┐
                    │  src-tauri  │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│ trading_worker  │ │ market_data │ │    analytics    │
└────────┬────────┘ └──────┬──────┘ └────────┬────────┘
         │                 │                 │
         │      ┌──────────┼──────────┐      │
         │      │          │          │      │
         ▼      ▼          ▼          ▼      ▼
    ┌────────────────────────────────────────────┐
    │           exchange_gateways                │
    └────────────────────┬───────────────────────┘
                         │
              ┌──────────┴──────────┐
              │                     │
              ▼                     ▼
        ┌──────────┐          ┌──────────┐
        │   core   │          │  infra   │
        └──────────┘          └──────────┘
```

### Responsabilidades de Cada Crate

| Crate | Responsabilidade | Dependências Externas Principais |
|-------|------------------|----------------------------------|
| `core` | Tipos primitivos, enums, traits base | `rust_decimal`, `chrono`, `serde` |
| `infra` | Database, config, logging, error handling | `sqlx`, `tokio`, `tracing`, `keyring` |
| `market_data` | Providers de dados, agregação, cache | `reqwest`, `tokio-tungstenite` |
| `analytics` | Indicadores técnicos, sinais | (apenas dependências internas) |
| `exchange_gateways` | Abstração de exchanges, order routing | `reqwest`, `hmac`, `sha2` |
| `trading_worker` | Execução de estratégias, job queue | `tokio`, canais async |
| `src-tauri` | GUI, comandos IPC, estado da aplicação | `tauri`, `specta` |

## Consequências

### Positivas

- **Compilação incremental eficiente**: Alterações em um crate não recompilam os outros
- **Fronteiras claras**: Cada crate tem uma API pública bem definida
- **Testabilidade**: Cada crate pode ser testado isoladamente com mocks
- **Reusabilidade**: Crates como `core` e `analytics` podem ser usados em outros projetos
- **Paralelismo de build**: Cargo compila crates independentes em paralelo
- **Separação de features**: Features específicas ficam no crate apropriado

### Negativas

- **Complexidade inicial**: Mais arquivos de configuração para gerenciar
- **Overhead de refatoração**: Mover tipos entre crates requer mudanças em múltiplos Cargo.toml
- **Possíveis ciclos**: Necessidade de cuidado para evitar dependências circulares
- **Boilerplate de re-exports**: Às vezes necessário re-exportar tipos para conveniência

### Neutras

- Cada crate tem seu próprio versionamento semântico (embora usemos versão unificada)
- Documentação pode ser gerada por crate ou para o workspace inteiro

## Alternativas Consideradas

### Alternativa 1: Monólito com Módulos

- **Descrição**: Um único crate com módulos internos
- **Prós**: Simplicidade, sem overhead de workspace
- **Contras**: Compilação lenta, acoplamento implícito, difícil testar isoladamente
- **Motivo da rejeição**: Não escala bem com a complexidade do projeto

### Alternativa 2: Workspace Plano

- **Descrição**: Crates na raiz sem diretório `crates/`
- **Prós**: Caminhos mais curtos
- **Contras**: Mistura crates com outros diretórios (docs, scripts, etc.)
- **Motivo da rejeição**: Organização menos clara, especialmente com Tauri

### Alternativa 3: Monorepo com Múltiplos Binários

- **Descrição**: Crates separados para CLI, server, GUI
- **Prós**: Flexibilidade de deploy
- **Contras**: Duplicação de código de bootstrap, complexidade de coordenação
- **Motivo da rejeição**: Inicialmente não necessitamos de múltiplos binários

## Referências

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Tauri Project Structure](https://tauri.app/v1/guides/getting-started/setup/)
- [Rust API Guidelines - Crate Organization](https://rust-lang.github.io/api-guidelines/naming.html)
