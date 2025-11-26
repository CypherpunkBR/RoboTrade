# Deployment Guide

Guia de build, distribuição e deployment do RoboTrade.

## Build de Produção

### Pré-requisitos

- Rust toolchain (stable)
- Node.js 18+
- Dependências de sistema para Tauri (veja Getting Started)

### Build Completo

```bash
# Build release otimizado
cargo tauri build

# Output estará em:
# - macOS: target/release/bundle/macos/RoboTrade.app
# - Windows: target/release/bundle/msi/RoboTrade_x.x.x_x64.msi
# - Linux: target/release/bundle/deb/robotrade_x.x.x_amd64.deb
```

### Configuração de Release

```toml
# Cargo.toml
[profile.release]
lto = true           # Link-Time Optimization
codegen-units = 1    # Melhor otimização, build mais lento
panic = "abort"      # Menor binário
strip = true         # Remove símbolos de debug

[profile.release.package.robotrade-analytics]
opt-level = 3        # Máxima otimização para cálculos
```

## Build por Plataforma

### macOS

```bash
# Build para Intel
cargo tauri build --target x86_64-apple-darwin

# Build para Apple Silicon
cargo tauri build --target aarch64-apple-darwin

# Universal Binary (ambas arquiteturas)
cargo tauri build --target universal-apple-darwin
```

#### Code Signing (macOS)

```bash
# Requer Apple Developer Account
# Configure em tauri.conf.json:
{
  "tauri": {
    "bundle": {
      "macOS": {
        "signingIdentity": "Developer ID Application: Sua Empresa (TEAM_ID)",
        "providerShortName": "TEAM_ID"
      }
    }
  }
}

# Notarização (necessária para distribuição fora da App Store)
xcrun notarytool submit target/release/bundle/macos/RoboTrade.app \
  --apple-id "email@example.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait
```

### Windows

```bash
# Build para Windows (requer Windows ou cross-compilation)
cargo tauri build --target x86_64-pc-windows-msvc

# Gera:
# - .exe (standalone)
# - .msi (installer)
```

#### Code Signing (Windows)

```powershell
# Requer certificado de code signing
# Configure em tauri.conf.json:
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": "YOUR_CERTIFICATE_THUMBPRINT",
        "timestampUrl": "http://timestamp.digicert.com"
      }
    }
  }
}
```

### Linux

```bash
# AppImage (universal)
cargo tauri build --target x86_64-unknown-linux-gnu

# Gera:
# - .deb (Debian/Ubuntu)
# - .AppImage (universal)
# - .rpm (via cargo-rpm se configurado)
```

## Configuração do Tauri

### tauri.conf.json

```json
{
  "build": {
    "beforeBuildCommand": "pnpm build",
    "beforeDevCommand": "pnpm dev",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "RoboTrade",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "scope": ["$APPDATA/*", "$APPDATA/**"]
      },
      "path": {
        "all": true
      },
      "dialog": {
        "all": true
      },
      "notification": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "category": "Finance",
      "copyright": "Copyright 2024 CypherpunkBR",
      "dpiAware": true,
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "br.com.cypherpunk.robotrade",
      "longDescription": "Trading bot automatizado para criptomoedas",
      "shortDescription": "RoboTrade - Trading Automatizado",
      "targets": ["app", "dmg", "msi", "deb", "appimage"],
      "externalBin": [],
      "resources": ["resources/*"]
    },
    "security": {
      "csp": "default-src 'self'; connect-src 'self' https://api.binance.com wss://stream.binance.com"
    },
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.robotrade.app/{{target}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    },
    "windows": [
      {
        "title": "RoboTrade",
        "width": 1280,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

## Auto-Updater

### Configuração

```rust
// src-tauri/src/main.rs
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(not(debug_assertions))]
            {
                let handle = app.handle();
                tauri::async_runtime::spawn(async move {
                    check_for_updates(handle).await;
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_for_updates(app: tauri::AppHandle) {
    use tauri::updater::UpdateBuilder;

    match app.updater().check().await {
        Ok(update) => {
            if update.is_update_available() {
                // Mostra dialog para usuário
                if let Err(e) = update.download_and_install().await {
                    tracing::error!("Failed to install update: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Update check failed: {}", e);
        }
    }
}
```

### Servidor de Updates

Estrutura de arquivos no servidor:

```
releases.robotrade.app/
├── darwin-x86_64/
│   └── 1.0.0/
│       ├── RoboTrade.app.tar.gz
│       └── RoboTrade.app.tar.gz.sig
├── darwin-aarch64/
│   └── 1.0.0/
│       └── ...
├── linux-x86_64/
│   └── 1.0.0/
│       └── ...
└── windows-x86_64/
    └── 1.0.0/
        └── ...
```

Manifesto de update (`latest.json`):

```json
{
  "version": "1.1.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2024-01-15T00:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "url": "https://releases.robotrade.app/darwin-x86_64/1.1.0/RoboTrade.app.tar.gz",
      "signature": "BASE64_SIGNATURE"
    },
    "darwin-aarch64": {
      "url": "https://releases.robotrade.app/darwin-aarch64/1.1.0/RoboTrade.app.tar.gz",
      "signature": "BASE64_SIGNATURE"
    },
    "linux-x86_64": {
      "url": "https://releases.robotrade.app/linux-x86_64/1.1.0/robotrade.AppImage.tar.gz",
      "signature": "BASE64_SIGNATURE"
    },
    "windows-x86_64": {
      "url": "https://releases.robotrade.app/windows-x86_64/1.1.0/RoboTrade.msi.zip",
      "signature": "BASE64_SIGNATURE"
    }
  }
}
```

## CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-22.04'
        run: |
          sudo apt update
          sudo apt install -y libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev

      - name: Install frontend dependencies
        run: |
          cd src-tauri
          npm ci

      - name: Build
        run: |
          cargo tauri build --target ${{ matrix.target }}
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: release-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/bundle/**/*.dmg
            target/${{ matrix.target }}/release/bundle/**/*.app
            target/${{ matrix.target }}/release/bundle/**/*.deb
            target/${{ matrix.target }}/release/bundle/**/*.AppImage
            target/${{ matrix.target }}/release/bundle/**/*.msi

  publish:
    needs: release
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v3

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            release-*/**/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Distribuição

### Canais de Release

1. **Stable**: Versões testadas e aprovadas
2. **Beta**: Versões para early adopters
3. **Nightly**: Builds automáticos da main branch

### Versionamento

Seguimos [Semantic Versioning](https://semver.org/):

- **MAJOR**: Mudanças incompatíveis
- **MINOR**: Novas features compatíveis
- **PATCH**: Bug fixes compatíveis

```bash
# Bump version
cargo set-version 1.2.0

# Criar tag
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin v1.2.0
```

## Monitoramento Pós-Deploy

### Telemetria (Opt-in)

```rust
// Apenas se usuário consentir
pub async fn report_crash(error: &str) {
    if !user_opted_in() {
        return;
    }

    // Envia crash report anonimizado
    let report = CrashReport {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        error: sanitize_error(error),
        // NÃO inclui dados pessoais ou financeiros
    };

    // ...
}
```

### Health Checks

```rust
#[tauri::command]
pub async fn health_check() -> Result<HealthStatus, String> {
    let db_ok = check_database().await.is_ok();
    let api_ok = check_api_connectivity().await.is_ok();

    Ok(HealthStatus {
        database: db_ok,
        api: api_ok,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: get_uptime().as_secs(),
    })
}
```

## Rollback

### Processo de Rollback

1. Identifique a versão problemática
2. Remova do servidor de updates
3. Atualize `latest.json` para versão anterior
4. Comunique usuários afetados

```bash
# Reverter para versão anterior
# (usuários precisarão baixar manualmente ou esperar próximo update)

# No servidor de releases:
mv latest.json latest.json.broken
cp versions/1.0.0/latest.json latest.json
```

## Checklist de Release

- [ ] Todos os testes passando
- [ ] CHANGELOG atualizado
- [ ] Versão incrementada em Cargo.toml e tauri.conf.json
- [ ] Build testado em todas as plataformas
- [ ] Code signing funcionando
- [ ] Auto-updater testado
- [ ] Release notes escritas
- [ ] Tag criada no Git
- [ ] Artifacts uploaded
- [ ] Servidor de updates atualizado
- [ ] Comunicação enviada aos usuários (se necessário)
