# Quick Development Guide

## Setup Rápido

```bash
# 1. Clone
git clone https://github.com/USER/RoboTrade.git
cd RoboTrade

# 2. Instala dependências
npm install

# 3. Configura .env
cp .env.example .env
# Edita .env com tuas API keys

# 4. Roda em dev
npm run tauri dev
```

## Comandos Principais

```bash
# Build tudo
cargo build --workspace

# Testa tudo
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Coverage
cargo tarpaulin --workspace --all-features

# Build release
cargo build --workspace --release
```

## Adicionar Feature

### 1. Nova Estratégia

```bash
# Cria arquivo
touch crates/analytics/src/strategies/my_strategy.rs

# Implementa trait Strategy
# Adiciona testes
# Exporta no mod.rs
```

### 2. Novo Indicador

```bash
# Cria arquivo
touch crates/analytics/src/indicators/my_indicator.rs

# Implementa função
# Adiciona testes
# Exporta no mod.rs
```

### 3. Novo Comando Tauri

```rust
// 1. Em src-tauri/src/commands.rs
#[tauri::command]
pub async fn my_command(params: Params) -> Result<Output, String> {
    // implementação
}

// 2. Registra em src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ...
    my_command,
])
```

## Testing Workflow

```bash
# Testa enquanto desenvolve
cargo test -p robotrade-analytics -- --nocapture

# Watch mode (re-testa quando muda)
cargo watch -x "test -p robotrade-analytics"

# Coverage do que tu mexeu
cargo tarpaulin -p robotrade-analytics --out html
```

## Debugging

### Rust Backend
```rust
// Adiciona logs
use tracing::info;
info!("Debug: {:?}", value);

// Ou usa dbg!
dbg!(&my_value);
```

### Frontend
```typescript
// Console log
console.log('Debug:', value);

// Ou usa React DevTools (F12)
```

## Common Tasks

### Atualizar dependências

```bash
# Rust
cargo update

# Frontend
cd src-web && npm update
```

### Limpar build cache

```bash
# Rust
cargo clean

# Frontend
cd src-web && rm -rf node_modules && npm install
```

### Rodar benchmarks

```bash
cargo bench --workspace
```

### Gerar docs

```bash
cargo doc --workspace --open
```

---

**Fluxo rápido pra desenvolvimento produtivo! 🚀**
