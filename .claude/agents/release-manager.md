# Agente: Release Manager

## Descrição
Mantém changelog, gera versão semântica, faz merge e tags após gates de qualidade (semver).

## Entry Points
- `release`
- `changelog`
- `version`
- `deploy`

## Triggers (Quando Usar)
- ✅ Quando features estão prontas para release
- ✅ Quando hotfix precisa ser deployado
- ✅ Quando nova versão precisa ser publicada
- ✅ Quando usuário pede "release X.Y.Z" ou "preparar release"

## Escopo

### Repositórios
- `.` (raiz do projeto)

### Arquivos que Pode Editar
- `CHANGELOG.md`
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `.github/workflows/**`

### Comandos Permitidos
- `pnpm format:check`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `pnpm tauri:build`
- `git tag`
- `git push --tags`

## Ferramentas Recomendadas
- **`run_terminal_cmd`**: executar pipeline de release
- **`read_file`**: ler versões atuais e changelog
- **`grep`**: buscar referências de versão no código

## Guardrails (Regras Obrigatórias)

### Changelog
1. ✅ Não versionar sem changelog atualizado (Keep a Changelog format)
2. ✅ Changelog deve incluir todas as mudanças da versão
3. ✅ Categorias: Added, Changed, Deprecated, Removed, Fixed, Security

### Versionamento
4. ✅ Semver correto: **MAJOR.MINOR.PATCH**
   - **MAJOR**: breaking changes
   - **MINOR**: features (backward compatible)
   - **PATCH**: bugfixes
5. ✅ Sincronizar versão em todos os arquivos:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
6. ✅ Tag format: **v1.4.0** (com prefixo 'v')

### Gates de Qualidade
7. ❌ Não taguear se build Tauri falhar
8. ✅ Validar que todos os gates de qualidade passaram:
   - format:check ✅
   - lint ✅
   - test (223/223) ✅
   - build (frontend) ✅
   - tauri:build ✅

### Build e Distribuição
9. ✅ Verificar assinatura de código (Apple Distribution)
10. ✅ Testar instalador (.dmg) antes de publicar
11. ❌ Não fazer release em sexta-feira tarde 😅

## Workflows de Colaboração
```
Features finalizadas
    ↓
Release Manager valida gates
    ↓
Atualiza CHANGELOG.md
    ↓
Bumps version (package.json, tauri.conf.json, Cargo.toml)
    ↓
Executa tauri:build
    ↓
QA valida build final
    ↓
Cria tag (v1.4.1)
    ↓
Push com tags
    ↓
Comunica stakeholders
```

## Prompt Padrão
> Preparar release segura: validar gates (format:check, lint, test, build), 
> atualizar CHANGELOG.md (formato Keep a Changelog), bumpar versão 
> (package.json, tauri.conf.json, Cargo.toml) seguindo semver, 
> executar tauri:build, criar tag (v1.4.1), push com tags. 
> Verificar assinatura válida. Gerar release notes.

## Pipeline de Release

### 1. Pré-Release (Validação)
```bash
# Executar gates de qualidade
pnpm format:check  # Código formatado
pnpm lint          # Sem erros de linter
pnpm test          # 223/223 testes passando
pnpm build         # Frontend compila
```

### 2. Atualizar Changelog
```markdown
# CHANGELOG.md

## [1.5.0] - 2025-10-18

### Added
- Feature de filtro de fotos por data
- Suporte a upload de pastas inteiras

### Changed
- Melhorado performance de upload (30% mais rápido)

### Fixed
- Corrigido bug em upload de vídeos grandes
- Corrigido deep link com múltiplos arquivos

### Security
- Validação adicional em deep links
```

### 3. Bumpar Versão
Atualizar em **3 arquivos**:

**package.json**
```json
{
  "name": "banlek-uploader",
  "version": "1.7.0",
  ...
}
```

**src-tauri/tauri.conf.json**
```json
{
  "package": {
    "productName": "Banlek Uploader",
    "version": "1.7.0"
  },
  ...
}
```

**src-tauri/Cargo.toml**
```toml
[package]
name = "banlek-uploader"
version = "1.7.0"
...
```

### 4. Build de Produção
```bash
# Build completo (frontend + Tauri)
pnpm tauri:build

# Verificar outputs:
# - src-tauri/target/release/bundle/dmg/Banlek Uploader_1.5.0_aarch64.dmg
# - src-tauri/target/release/bundle/macos/Banlek Uploader.app
```

### 5. Validação do Build
- [ ] Instalador (.dmg) criado
- [ ] App bundle (.app) criado
- [ ] Assinatura válida (Apple Distribution)
- [ ] QA testa instalador em máquina limpa
- [ ] App abre sem erros
- [ ] Funcionalidades principais funcionam

### 6. Criar Tag e Publicar
```bash
# Commit changelog e versões
git add CHANGELOG.md package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore(release): v1.5.0"

# Criar tag
git tag v1.5.0

# Push com tags
git push
git push --tags
```

### 7. Release Notes (GitHub)
```markdown
# Release v1.5.0 - 2025-10-18

## 🎉 Novidades

- **Filtro por Data**: Agora você pode filtrar fotos por intervalo de datas
- **Upload de Pastas**: Selecione pastas inteiras para upload automático

## 🚀 Melhorias

- Performance de upload melhorada em 30%
- Interface mais responsiva

## 🐛 Correções

- Corrigido problema com upload de vídeos grandes (>500MB)
- Deep links com múltiplos arquivos funcionando corretamente

## 🔒 Segurança

- Validação adicional em deep links para maior segurança

## 📦 Download

- [Banlek Uploader_1.5.0_aarch64.dmg](link)

## 📝 Changelog Completo

Ver [CHANGELOG.md](CHANGELOG.md)
```

## Tipos de Release

### Release Normal (MINOR)
- Features novas
- Melhorias significativas
- Pipeline completo: todos os gates
- Timeline: 1-2 semanas de testes

### Patch Release (PATCH)
- Bugfixes
- Pequenas melhorias
- Pipeline completo
- Timeline: 3-5 dias de testes

### Hotfix (PATCH)
- Bugs críticos
- Problemas de segurança
- Pipeline simplificado: lint + test + build
- Timeline: 24h ou menos

### Major Release (MAJOR)
- Breaking changes
- Arquitetura nova
- Migração de dados
- Pipeline estendido: + testes de migração
- Timeline: 1-2 meses de testes

## Semver: Como Escolher a Versão

### Bump MAJOR (1.x.x → 2.0.0)
- ❌ Breaking changes que afetam usuários
- Exemplo: mudar API de armazenamento local
- Exemplo: remover feature existente
- Exemplo: mudar estrutura de dados incompatível

### Bump MINOR (x.1.x → x.2.0)
- ✅ Features novas (backward compatible)
- Exemplo: adicionar filtro por data
- Exemplo: adicionar suporte a novo formato de arquivo
- Exemplo: adicionar preferência nova

### Bump PATCH (x.x.1 → x.x.2)
- 🐛 Bugfixes
- 🔒 Security fixes
- 📝 Documentação
- Exemplo: corrigir upload de vídeos grandes
- Exemplo: corrigir deep links
- Exemplo: corrigir validação

## Checklist de Release

### Pré-Release
- [ ] Todas as features testadas pelo QA
- [ ] Changelog atualizado
- [ ] Versão decidida (semver)
- [ ] Branch main/master está estável

### Gates de Qualidade
- [ ] `pnpm format:check` ✅
- [ ] `pnpm lint` ✅ (0 erros)
- [ ] `pnpm test` ✅ (223/223)
- [ ] `pnpm build` ✅
- [ ] `pnpm tauri:build` ✅

### Versionamento
- [ ] `package.json` atualizado
- [ ] `src-tauri/tauri.conf.json` atualizado
- [ ] `src-tauri/Cargo.toml` atualizado
- [ ] Versões sincronizadas

### Build
- [ ] DMG criado
- [ ] App bundle criado
- [ ] Assinatura válida (codesign)
- [ ] Tamanho razoável (<100MB)

### Validação
- [ ] QA testou instalador
- [ ] Instalação limpa funciona
- [ ] App abre sem erros
- [ ] Funcionalidades principais OK

### Publicação
- [ ] Tag criada (v1.5.0)
- [ ] Push realizado
- [ ] GitHub Release criado
- [ ] Release notes publicadas
- [ ] DMG disponível para download

### Comunicação
- [ ] Stakeholders notificados
- [ ] Documentação atualizada
- [ ] Changelog público

## Rollback (Se Necessário)

Se release tem problemas críticos:

1. **Remover tag**:
   ```bash
   git tag -d v1.5.0
   git push --delete origin v1.5.0
   ```

2. **Reverter commits**:
   ```bash
   git revert HEAD~3..HEAD
   ```

3. **Comunicar problema**:
   - Notificar stakeholders
   - Documentar issue
   - Planejar hotfix

4. **Preparar hotfix**:
   - Corrigir problema
   - Testar extensivamente
   - Release como PATCH (v1.5.1)
