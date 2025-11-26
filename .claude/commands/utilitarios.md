# Comandos Utilitários

Comandos auxiliares para manutenção e limpeza.

## clean

**Descrição:** Limpa arquivos de build e cache

**Uso:**
```bash
.claude/commands.yaml clean
```

**O que remove:**
- `dist/` - Build do frontend (Vite)
- `src-tauri/target/` - Build do Rust/Tauri
- `node_modules/.vite/` - Cache do Vite

**Output:**
```
🧹 Limpando arquivos temporários...
✅ Limpeza concluída
```

**Quando usar:**
- Build está com problemas estranhos
- Liberar espaço em disco
- Antes de commit (para não commitar builds)
- Reset do ambiente de desenvolvimento

**Espaço liberado:** ~2-5 GB (dependendo da quantidade de builds)

**Nota:** Não remove `node_modules/` completo, apenas cache. Para reinstalar dependências use `clean:install`.

---

## clean:install

**Descrição:** Limpa e reinstala dependências

**Uso:**
```bash
.claude/commands.yaml clean:install
```

**O que faz:**
```
🧹 Limpando node_modules...
  → rm -rf node_modules/
  → rm -rf src-tauri/target/

📦 Reinstalando dependências...
  → pnpm install

✅ Dependências reinstaladas
```

**Quando usar:**
- `node_modules/` está corrompido
- Dependências estão inconsistentes
- Erro estranho que não faz sentido
- Após atualizar pnpm/node
- "Tentou desligar e ligar de novo?" 😅

**Tempo:** 2-5 minutos

**Espaço temporariamente liberado:** ~500MB - 1GB

**Comandos equivalentes:**
```bash
# Manual
rm -rf node_modules/
rm -rf src-tauri/target/
pnpm install
```

---

## help

**Descrição:** Exibe ajuda sobre comandos disponíveis

**Uso:**
```bash
.claude/commands.yaml help
```

**Output:**
```
📚 Comandos Disponíveis - Banlek Uploader

=== BÁSICOS ===
  install          - Instalar dependências
  dev              - Dev server (Vite + Tauri)
  build:web        - Build frontend
  build:tauri      - Build completo (.dmg/.app)

=== QUALIDADE ===
  lint             - Linter
  lint:fix         - Linter + correções
  format           - Formatar código
  format:check     - Verificar formatação

=== TESTES ===
  test             - Executar testes
  test:watch       - Testes em watch mode
  test:coverage    - Cobertura
  test:ui          - Interface visual
  test:regression  - Verificar regressão

=== SEGURANÇA ===
  check:security   - Validar regras de segurança
  validate:ids     - Validar hashId vs id
  check:types      - Verificar uso de 'any'

=== DEBUG ===
  debug:upload     - Info sobre upload
  debug:deep-links - Info sobre deep links
  debug:monitor    - Info sobre monitor

=== WORKFLOWS ===
  workflow:feature - Pipeline completo de feature
  workflow:hotfix  - Pipeline rápido de hotfix
  workflow:review  - Pipeline de revisão

=== RELEASE ===
  release:prepare  - Validar gates de qualidade
  release:changelog- Gerar changelog
  release:build    - Build para distribuição
  release:tag      - Criar tag e push

Para mais detalhes, ver: .claude/commands.yaml
```

**Quando usar:**
- Esqueceu nome de um comando
- Explorar comandos disponíveis
- Onboarding de novos devs
- Referência rápida

---

## Outros Comandos Úteis

### Verificar versões
```bash
# Versão do Node
node --version

# Versão do pnpm
pnpm --version

# Versão do Rust
rustc --version

# Versão do Tauri CLI
pnpm tauri --version
```

### Informações do sistema
```bash
# Info do projeto
cat package.json | grep version

# Info do Tauri
cat src-tauri/tauri.conf.json | grep version

# Info do Cargo
cat src-tauri/Cargo.toml | grep version
```

### Git útil
```bash
# Status
git status

# Log recente
git log --oneline -10

# Branches
git branch -a

# Tags
git tag -l

# Última tag
git describe --tags --abbrev=0
```

### Análise de espaço
```bash
# Tamanho de pastas
du -sh node_modules/
du -sh src-tauri/target/
du -sh dist/

# Tamanho total do projeto
du -sh .
```

### Processos em execução
```bash
# Ver processos Node/Tauri
ps aux | grep node
ps aux | grep tauri

# Matar processo na porta 1420 (Vite padrão)
lsof -ti:1420 | xargs kill -9
```

---

## Troubleshooting Comum

### "Cannot find module"
```bash
# Reinstalar dependências
.claude/commands.yaml clean:install
```

### "Port already in use"
```bash
# Matar processo na porta
lsof -ti:1420 | xargs kill -9

# Ou mudar porta no vite.config.ts
```

### "Build failed" inexplicável
```bash
# Limpar tudo e tentar novamente
.claude/commands.yaml clean
pnpm install
pnpm tauri:build
```

### "Git merge conflict"
```bash
# Ver arquivos em conflito
git status | grep "both modified"

# Resolver manualmente e depois:
git add .
git commit -m "resolve merge conflicts"
```

### "Out of disk space"
```bash
# Liberar espaço
.claude/commands.yaml clean
rm -rf ~/.pnpm-store/_tmp/  # Cache do pnpm
rm -rf ~/Library/Caches/pnpm/  # Cache adicional
```

---

## Scripts Personalizados

Você pode adicionar seus próprios scripts úteis:

### package.json
```json
{
  "scripts": {
    "clean": "rm -rf dist/ src-tauri/target/ node_modules/.vite/",
    "clean:all": "rm -rf dist/ src-tauri/target/ node_modules/",
    "fresh": "pnpm clean:all && pnpm install",
    "pre-commit": "pnpm format && pnpm lint && pnpm test",
    "check:all": "pnpm format:check && pnpm lint && pnpm test && pnpm build"
  }
}
```

### Criar aliases no shell
```bash
# Adicionar ao ~/.zshrc ou ~/.bashrc
alias bd="cd ~/Dev/Banlek/banlek-uploader"
alias bdev="cd ~/Dev/Banlek/banlek-uploader && pnpm tauri:dev"
alias btest="cd ~/Dev/Banlek/banlek-uploader && pnpm test"
```

---

## Manutenção Recomendada

### Diariamente
- Executar testes antes de commit
- Validar formatação e lint

### Semanalmente
- Limpar cache (`clean`)
- Verificar dependências desatualizadas

### Mensalmente
- Atualizar dependências (minor/patch)
- Verificar vulnerabilidades
- Limpar git (branches antigas)

### Antes de Release
- Executar `clean:install` para ambiente limpo
- Validar todos os gates
- Build completo de produção
- QA em máquina limpa




