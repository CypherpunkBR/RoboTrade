# Getting Started

Guia completo para configurar o ambiente de desenvolvimento do RoboTrade.

## Pré-requisitos

### Sistema Operacional

- **macOS** 12.0+ (Monterey ou superior)
- **Linux** Ubuntu 20.04+ / Fedora 35+ / Arch Linux
- **Windows** 10/11 com WSL2 (recomendado) ou nativo

### Ferramentas Necessárias

| Ferramenta | Versão Mínima | Instalação |
|------------|---------------|------------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| pnpm/npm | 8+ | `npm install -g pnpm` |
| Git | 2.30+ | [git-scm.com](https://git-scm.com/) |

### Dependências de Sistema

#### macOS

```bash
# Xcode Command Line Tools
xcode-select --install

# Homebrew (se não tiver)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# SQLite (geralmente já vem no macOS)
brew install sqlite
```

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    librsvg2-dev \
    libsqlite3-dev \
    pkg-config
```

#### Fedora

```bash
sudo dnf install -y \
    gcc \
    openssl-devel \
    gtk3-devel \
    webkit2gtk4.1-devel \
    librsvg2-devel \
    sqlite-devel
```

#### Arch Linux

```bash
sudo pacman -S \
    base-devel \
    openssl \
    gtk3 \
    webkit2gtk-4.1 \
    librsvg \
    sqlite
```

## Instalação

### 1. Clone o Repositório

```bash
git clone https://github.com/CypherpunkBR/RoboTrade.git
cd RoboTrade
```

### 2. Configure o Rust

```bash
# Instale rustup se ainda não tiver
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Atualize para versão estável mais recente
rustup update stable

# Adicione componentes úteis
rustup component add clippy rustfmt rust-analyzer
```

### 3. Instale Dependências do Frontend

```bash
cd src-tauri
pnpm install
# ou
npm install
```

### 4. Configure Variáveis de Ambiente

Crie um arquivo `.env` na raiz do projeto:

```bash
# Modo de desenvolvimento
RUST_LOG=debug,sqlx=warn,hyper=warn

# Caminho do banco de dados (opcional, usa default se não definido)
DATABASE_URL=sqlite:./data/robotrade.db

# Para testes com API real (opcional)
# BINANCE_API_KEY=sua_api_key_aqui
# BINANCE_SECRET_KEY=sua_secret_key_aqui
```

### 5. Inicialize o Banco de Dados

```bash
# Crie o diretório de dados
mkdir -p data

# Execute as migrations
cargo run -p robotrade-infra --bin migrate

# Ou via sqlx-cli (se instalado)
sqlx migrate run --source crates/infra/migrations
```

### 6. Verifique a Instalação

```bash
# Build completo
cargo build

# Rode os testes
cargo test

# Verifique linting
cargo clippy

# Inicie em modo desenvolvimento
cargo tauri dev
```

## Estrutura de Diretórios

```
RoboTrade/
├── .github/              # GitHub Actions workflows
├── crates/               # Workspace Rust
│   ├── core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types/   # Tipos primitivos
│   │   │   ├── traits/  # Traits base
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   ├── infra/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── database/ # Repositórios SQLite
│   │   │   ├── config/   # Sistema de configuração
│   │   │   └── telemetry/
│   │   ├── migrations/   # SQL migrations
│   │   └── Cargo.toml
│   ├── market_data/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── providers/ # Binance, etc
│   │   │   └── aggregator.rs
│   │   └── Cargo.toml
│   ├── analytics/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── indicators/ # SMA, EMA, RSI, etc
│   │   │   └── signals.rs
│   │   └── Cargo.toml
│   ├── trading_worker/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── worker.rs
│   │   │   ├── queue.rs
│   │   │   └── strategies/
│   │   └── Cargo.toml
│   ├── exchange_gateways/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── binance/
│   │   │   └── paper/
│   │   └── Cargo.toml
│   └── src-tauri/
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/ # Tauri commands
│       │   └── state.rs
│       ├── Cargo.toml
│       └── tauri.conf.json
├── src/                  # Frontend React (se separado)
├── docs/                 # Documentação
├── data/                 # Dados locais (gitignored)
├── Cargo.toml            # Workspace root
└── rustfmt.toml          # Configuração de formatação
```

## Fluxo de Desenvolvimento

### 1. Crie uma Branch

```bash
git checkout -b feature/nome-da-feature
# ou
git checkout -b fix/descricao-do-bug
```

### 2. Desenvolva

```bash
# Terminal 1: Rode o backend com hot-reload
cargo watch -x 'build -p robotrade-core'

# Terminal 2: Rode o frontend
cargo tauri dev
```

### 3. Teste

```bash
# Testes unitários
cargo test

# Testes de um módulo específico
cargo test -p robotrade-analytics indicators

# Com cobertura (requer cargo-tarpaulin)
cargo tarpaulin --out Html
```

### 4. Verifique Qualidade

```bash
# Formatação
cargo fmt --all

# Linting
cargo clippy -- -D warnings

# Verificação de tipos
cargo check --workspace
```

### 5. Commit e Push

```bash
git add .
git commit -m "feat: descrição da mudança"
git push origin feature/nome-da-feature
```

## Configuração do IDE

### VS Code

Instale as extensões recomendadas:

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension usernamehw.errorlens
code --install-extension tauri-apps.tauri-vscode
```

Configuração recomendada (`.vscode/settings.json`):

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.lens.run.enable": true,
  "rust-analyzer.lens.debug.enable": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "files.watcherExclude": {
    "**/target/**": true
  }
}
```

### JetBrains (RustRover/CLion)

1. Instale o plugin Rust
2. Configure `Settings > Languages & Frameworks > Rust > Clippy` para rodar no save
3. Habilite `Use rustfmt instead of built-in formatter`

## Troubleshooting

### Erro: "linker `cc` not found"

**macOS:**
```bash
xcode-select --install
```

**Linux:**
```bash
sudo apt install build-essential  # Ubuntu
sudo dnf install gcc              # Fedora
```

### Erro: "pkg-config not found"

**macOS:**
```bash
brew install pkg-config
```

**Linux:**
```bash
sudo apt install pkg-config
```

### Erro: "failed to run custom build command for `openssl-sys`"

**macOS:**
```bash
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

**Linux:**
```bash
sudo apt install libssl-dev  # Ubuntu
sudo dnf install openssl-devel  # Fedora
```

### Erro: "webkit2gtk not found"

**Ubuntu:**
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Fedora:**
```bash
sudo dnf install webkit2gtk4.1-devel
```

### Build muito lento

1. Use sccache para cache de compilação:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

2. Configure para usar mais threads:
   ```bash
   # Em .cargo/config.toml
   [build]
   jobs = 8  # Ajuste para seu número de cores
   ```

3. Use mold linker (Linux):
   ```bash
   sudo apt install mold
   # Em .cargo/config.toml
   [target.x86_64-unknown-linux-gnu]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

### Banco de dados corrompido

```bash
# Backup do banco atual
cp data/robotrade.db data/robotrade.db.bak

# Delete e recrie
rm data/robotrade.db
cargo run -p robotrade-infra --bin migrate
```

## Próximos Passos

- [Coding Standards](./coding-standards.md) - Aprenda os padrões de código
- [Testing Guide](./testing.md) - Saiba como escrever testes
- [Architecture Overview](../architecture/overview.md) - Entenda a arquitetura
