# Comandos de Release

Comandos para gerenciar o processo de release e versionamento.

## release:prepare

**Descrição:** Prepara release validando todos os gates de qualidade

**Uso:**
```bash
.claude/commands.yaml release:prepare
```

**Gates Validados:**
```
Gate 1/5: Formatação
  → pnpm format:check
  ✅ Formatação OK

Gate 2/5: Linter
  → pnpm lint
  ✅ Linter OK

Gate 3/5: Testes
  → pnpm test
  ✅ Testes OK (223/223)

Gate 4/5: Build Frontend
  → pnpm build
  ✅ Build OK

Gate 5/5: Segurança
  → .claude/commands.yaml check:security
  ✅ Segurança OK

🎉 Todos os gates passaram! Release pronto.

Próximos passos:
  1. Atualizar CHANGELOG.md
  2. Bumpar versão (package.json, tauri.conf.json, Cargo.toml)
  3. Executar: .claude/commands.yaml release:build
  4. QA valida build
  5. Executar: .claude/commands.yaml release:tag
```

**Quando usar:**
- Antes de qualquer release
- Validar que projeto está estável
- CI/CD antes de tag

**Exit codes:**
- `0` - Todos os gates passaram, pronto para release
- `1` - Algum gate falhou, não pode fazer release

**Tempo:** ~1-2 minutos

---

## release:changelog

**Descrição:** Gera changelog baseado em commits desde última tag

**Uso:**
```bash
.claude/commands.yaml release:changelog
```

**Output:**
```
# Mudanças desde v1.4.0

- feat: adicionar filtro por data (a1b2c3d)
- fix: corrigir upload de vídeos grandes (d4e5f6g)
- docs: atualizar guia de instalação (g7h8i9j)

⚠️  Organize em categorias no CHANGELOG.md:
  - Added (features novas)
  - Changed (mudanças)
  - Fixed (correções)
  - Security (segurança)
```

**Como usar:**
```bash
# 1. Gerar lista de commits
.claude/commands.yaml release:changelog

# 2. Copiar output

# 3. Editar CHANGELOG.md
vim CHANGELOG.md

# 4. Organizar em formato Keep a Changelog:
```

**Formato Keep a Changelog:**
```markdown
# Changelog

## [1.5.0] - 2025-10-18

### Added
- Filtro de fotos por data
- Suporte a upload de pastas inteiras

### Changed
- Melhorado performance de upload em 30%

### Fixed
- Corrigido upload de vídeos grandes (>500MB)
- Corrigido deep links com múltiplos arquivos

### Security
- Validação adicional em deep links
```

**Quando usar:**
- Antes de criar tag
- Preparar release notes
- Documentar mudanças

---

## release:build

**Descrição:** Build Tauri completo para distribuição

**Uso:**
```bash
.claude/commands.yaml release:build
```

**O que faz:**
```
📦 Building Tauri...

1. Compila frontend (Vite)
2. Compila backend (Rust)
3. Cria bundle macOS
4. Assina código (Apple Distribution)
5. Gera instalador DMG

✅ Build concluído!

Outputs:
  - DMG: src-tauri/target/release/bundle/dmg/
  - App: src-tauri/target/release/bundle/macos/

Verificar:
  - Assinatura válida (codesign)
  - Instalador abre sem erros
  - App funciona em máquina limpa
```

**Outputs:**
- **DMG**: `Banlek Uploader_1.5.0_aarch64.dmg`
- **App Bundle**: `Banlek Uploader.app`

**Tempo:** ~10 minutos

**Quando usar:**
- Após validar gates
- Após atualizar changelog
- Antes de QA validar

**Requisitos:**
- Certificado Apple Distribution configurado
- Xcode command line tools instalados
- Rust toolchain atualizado

**Verificar build:**
```bash
# Verificar assinatura
codesign -vvv --deep --strict "src-tauri/target/release/bundle/macos/Banlek Uploader.app"

# Verificar tamanho
du -sh src-tauri/target/release/bundle/dmg/*.dmg

# Testar instalador
open src-tauri/target/release/bundle/dmg/*.dmg
```

---

## release:tag

**Descrição:** Cria tag de versão e push (interativo)

**Uso:**
```bash
.claude/commands.yaml release:tag
```

**Fluxo interativo:**
```
📌 Criando tag para versão: v1.5.0

Arquivos a commitar:
  - CHANGELOG.md
  - package.json
  - src-tauri/tauri.conf.json
  - src-tauri/Cargo.toml

Continuar? (y/n) _
```

**Se confirmar (y):**
```bash
git add CHANGELOG.md package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore(release): v1.5.0"
git tag v1.5.0

✅ Tag criada: v1.5.0

Para publicar:
  git push
  git push --tags
```

**Quando usar:**
- Após build e QA aprovarem
- Como último passo do release
- Criar ponto de release no git

**⚠️ Importante:**
- Sempre sincronizar versão em 3 arquivos
- Seguir semver (MAJOR.MINOR.PATCH)
- Tag format: `v1.5.0` (com prefixo 'v')
- Verificar que CHANGELOG está atualizado

---

## Semver: Escolher Versão

### MAJOR (1.x.x → 2.0.0)
**Breaking changes**
- Mudanças incompatíveis na API
- Remoção de features
- Mudança de estrutura de dados

**Exemplos:**
- Remover suporte a formato de arquivo
- Mudar estrutura de armazenamento local
- Alterar API de comandos Tauri

### MINOR (x.1.x → x.2.0)
**Features novas (backward compatible)**
- Adicionar nova funcionalidade
- Melhorias sem quebrar compatibilidade
- Novas APIs opcionais

**Exemplos:**
- Adicionar filtro por data
- Suporte a novo formato de arquivo
- Nova preferência de usuário

### PATCH (x.x.1 → x.x.2)
**Bugfixes e correções**
- Corrigir bugs
- Melhorias de performance
- Correções de segurança
- Atualizações de documentação

**Exemplos:**
- Corrigir upload de vídeos grandes
- Corrigir deep links
- Melhorar performance

---

## Pipeline Completo de Release

### 1. Preparação
```bash
# Validar gates
.claude/commands.yaml release:prepare

# Gerar lista de commits
.claude/commands.yaml release:changelog

# Editar CHANGELOG.md
vim CHANGELOG.md
# Organizar em: Added, Changed, Fixed, Security
```

### 2. Versionamento
```bash
# Decidir tipo de release: MAJOR, MINOR ou PATCH

# Editar versões manualmente:
vim package.json                    # "version": "1.5.0"
vim src-tauri/tauri.conf.json       # "version": "1.5.0"
vim src-tauri/Cargo.toml            # version = "1.5.0"
```

### 3. Build
```bash
# Build para distribuição
.claude/commands.yaml release:build

# Tempo: ~10 minutos
# Output: DMG + App bundle
```

### 4. Validação QA
```bash
# QA instala e testa em máquina limpa
# - Instalador funciona
# - App abre sem erros
# - Features principais OK
# - Nenhum bug crítico
```

### 5. Tag e Publish
```bash
# Criar tag
.claude/commands.yaml release:tag
# Confirmar: y

# Push
git push
git push --tags

# Criar GitHub Release
# - Upload do DMG
# - Release notes do CHANGELOG
```

### 6. Comunicação
```bash
# Notificar stakeholders
# Atualizar documentação pública
# Anunciar release
```

---

## Rollback de Release

Se release tem problema crítico:

### 1. Remover tag
```bash
# Local
git tag -d v1.5.0

# Remote
git push --delete origin v1.5.0
```

### 2. Reverter commits
```bash
git revert HEAD~3..HEAD
```

### 3. Comunicar
- Notificar usuários
- Documentar problema
- Planejar hotfix

### 4. Hotfix
```bash
# Corrigir problema
# Testar extensivamente
# Release como PATCH (v1.5.1)
```

---

## Checklist de Release

Antes de fazer release:

**Preparação**
- [ ] Todos os gates passaram
- [ ] CHANGELOG atualizado
- [ ] Versão sincronizada (3 arquivos)
- [ ] Semver correto

**Build**
- [ ] Build Tauri concluído
- [ ] DMG e App criados
- [ ] Assinatura válida
- [ ] Tamanho razoável (<100MB)

**Validação**
- [ ] QA testou instalador
- [ ] App funciona em máquina limpa
- [ ] Features principais OK
- [ ] Nenhum bug crítico

**Publicação**
- [ ] Tag criada
- [ ] Push realizado
- [ ] GitHub Release criado
- [ ] DMG disponível para download

**Comunicação**
- [ ] Stakeholders notificados
- [ ] Docs atualizadas
- [ ] Release anunciado




