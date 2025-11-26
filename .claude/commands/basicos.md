# Comandos Básicos

Comandos essenciais para desenvolvimento diário.

## install

**Descrição:** Instala dependências do projeto

**Uso:**
```bash
pnpm install
```

**Quando usar:**
- Primeira vez configurando o projeto
- Após atualizar package.json
- Após fazer pull de novas dependências
- Quando node_modules está corrompido

---

## dev

**Descrição:** Inicia dev server (Vite + Tauri)

**Uso:**
```bash
pnpm tauri:dev
```

**O que faz:**
- Inicia Vite dev server (frontend)
- Compila e executa app Tauri
- Hot reload ativo
- DevTools disponível (Cmd+Opt+I)

**Quando usar:**
- Desenvolvimento diário
- Testar features no app real
- Debug com DevTools

---

## build:web

**Descrição:** Build do frontend (Vite)

**Uso:**
```bash
pnpm build
```

**Output:**
- Arquivos estáticos em `dist/`

**Quando usar:**
- Validar que frontend compila
- Antes de build Tauri
- CI/CD pipeline

---

## build:tauri

**Descrição:** Build completo (DMG/.app para macOS)

**Uso:**
```bash
pnpm tauri:build
```

**Output:**
- `src-tauri/target/release/bundle/dmg/` - Instalador .dmg
- `src-tauri/target/release/bundle/macos/` - App bundle .app

**Tempo:** ~10 minutos

**Quando usar:**
- Preparar release
- Testar instalador
- QA em máquina limpa

**Notas:**
- Requer certificado Apple Distribution
- Assinatura de código automática se configurada

