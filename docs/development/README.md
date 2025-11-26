# Guias de Desenvolvimento

Esta seção contém toda a documentação necessária para desenvolvedores que trabalham no RoboTrade.

## Índice

| Documento | Descrição |
|-----------|-----------|
| [Getting Started](./getting-started.md) | Setup inicial do ambiente de desenvolvimento |
| [Coding Standards](./coding-standards.md) | Padrões de código e convenções |
| [Testing Guide](./testing.md) | Estratégias e práticas de teste |
| [Deployment](./deployment.md) | Build e distribuição da aplicação |
| [Contributing](./contributing.md) | Guia de contribuição |

## Quick Start

```bash
# 1. Clone o repositório
git clone https://github.com/CypherpunkBR/RoboTrade.git
cd RoboTrade

# 2. Instale as dependências do Rust
rustup update stable

# 3. Instale as dependências do frontend
cd src-tauri
npm install

# 4. Rode em modo desenvolvimento
cargo tauri dev
```

## Estrutura do Projeto

```
RoboTrade/
├── crates/               # Workspace Rust
│   ├── core/            # Tipos e traits fundamentais
│   ├── infra/           # Infraestrutura (DB, config, logs)
│   ├── market_data/     # Coleta de dados de mercado
│   ├── analytics/       # Indicadores técnicos
│   ├── trading_worker/  # Execução de estratégias
│   ├── exchange_gateways/ # APIs de exchanges
│   └── src-tauri/       # Aplicação Tauri
├── docs/                # Documentação
├── migrations/          # Migrations SQLite
└── tests/               # Testes de integração
```

## Comandos Úteis

### Build

```bash
# Build todos os crates
cargo build

# Build release otimizado
cargo build --release

# Build apenas um crate
cargo build -p robotrade-core
```

### Testes

```bash
# Rodar todos os testes
cargo test

# Testes com output
cargo test -- --nocapture

# Testes de um crate específico
cargo test -p robotrade-analytics

# Testes de integração
cargo test --test '*'
```

### Qualidade de Código

```bash
# Linting
cargo clippy -- -D warnings

# Formatação
cargo fmt --all

# Verificar formatação
cargo fmt --all -- --check
```

### Documentação

```bash
# Gerar documentação
cargo doc --no-deps --open

# Documentação de todos os crates
cargo doc --workspace --no-deps
```

## Ambiente de Desenvolvimento

### VS Code Extensions Recomendadas

- rust-analyzer
- Even Better TOML
- Error Lens
- GitLens
- Tauri

### .vscode/settings.json Recomendado

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Links Úteis

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [sqlx Documentation](https://docs.rs/sqlx/)
