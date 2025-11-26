# Comandos Disponíveis

Comandos automatizados para desenvolvimento, testes, segurança e releases no Banlek Uploader.

## Estrutura

Os comandos estão organizados por categoria em arquivos separados:

- **[basicos.md](./basicos.md)** - Comandos essenciais (install, dev, build)
- **[qualidade.md](./qualidade.md)** - Lint, format, padrões de código
- **[testes.md](./testes.md)** - Testes automatizados e cobertura
- **[seguranca.md](./seguranca.md)** - Validações de segurança do projeto
- **[debug.md](./debug.md)** - Ferramentas de debug e troubleshooting
- **[workflows.md](./workflows.md)** - Pipelines compostos (feature, hotfix, review)
- **[release.md](./release.md)** - Processo de release e versionamento
- **[utilitarios.md](./utilitarios.md)** - Limpeza e manutenção

## Início Rápido

### Desenvolvimento Diário
```bash
# Instalar dependências (primeira vez)
pnpm install

# Iniciar desenvolvimento
pnpm tauri:dev

# Antes de commit
pnpm format && pnpm lint && pnpm test
```

### Workflows Principais
```bash
# Pipeline completo de feature
.claude/commands.yaml workflow:feature

# Pipeline de hotfix urgente
.claude/commands.yaml workflow:hotfix

# Revisão de código
.claude/commands.yaml workflow:review
```

### Release
```bash
# 1. Preparar release
.claude/commands.yaml release:prepare

# 2. Atualizar CHANGELOG.md
.claude/commands.yaml release:changelog
# (editar CHANGELOG.md manualmente)

# 3. Build distribuição
.claude/commands.yaml release:build

# 4. Tag e publish
.claude/commands.yaml release:tag
```

## Comandos por Categoria

### Básicos
| Comando | Descrição |
|---------|-----------|
| `install` | Instala dependências |
| `dev` | Dev server (Vite + Tauri) |
| `build:web` | Build frontend |
| `build:tauri` | Build completo (.dmg/.app) |

### Qualidade
| Comando | Descrição |
|---------|-----------|
| `lint` | Executa linter (ESLint) |
| `lint:fix` | Linter com correções automáticas |
| `format` | Formata código (Prettier) |
| `format:check` | Verifica formatação |

### Testes
| Comando | Descrição |
|---------|-----------|
| `test` | Executa todos os testes |
| `test:watch` | Testes em watch mode |
| `test:coverage` | Cobertura de testes |
| `test:ui` | Interface visual (Vitest UI) |
| `test:regression` | Verifica regressão de cobertura |

### Segurança
| Comando | Descrição |
|---------|-----------|
| `check:security` | Valida tokens, IDs, deep links |
| `validate:ids` | Verifica hashId vs id numérico |
| `check:types` | Busca uso de 'any' |

### Debug
| Comando | Descrição |
|---------|-----------|
| `debug:upload` | Info sobre fluxo de upload |
| `debug:deep-links` | Info sobre deep links |
| `debug:monitor` | Info sobre monitor de pastas |
| `repo:map` | Estrutura do repositório |
| `check:coverage-delta` | Compara cobertura |
| `check:dependencies` | Verifica dependências |

### Workflows
| Comando | Descrição |
|---------|-----------|
| `workflow:feature` | Pipeline completo de feature |
| `workflow:hotfix` | Pipeline rápido de hotfix |
| `workflow:review` | Pipeline de revisão |

### Release
| Comando | Descrição |
|---------|-----------|
| `release:prepare` | Valida gates de qualidade |
| `release:changelog` | Gera changelog |
| `release:build` | Build para distribuição |
| `release:tag` | Cria tag e push |

### Utilitários
| Comando | Descrição |
|---------|-----------|
| `clean` | Limpa builds e cache |
| `clean:install` | Reinstala dependências |
| `help` | Lista todos os comandos |

## Executando Comandos

### Via pnpm (comandos básicos)
```bash
pnpm install
pnpm tauri:dev
pnpm build
pnpm lint
pnpm test
```

### Via commands.yaml (comandos customizados)
```bash
# Formato geral
.claude/commands.yaml <comando>

# Exemplos
.claude/commands.yaml workflow:feature
.claude/commands.yaml check:security
.claude/commands.yaml release:prepare
```

## Pipelines Recomendados

### Feature Development
```bash
# 1. Implementar código
vim src/components/NewFeature.tsx

# 2. Validar completo
.claude/commands.yaml workflow:feature

# 3. Se passar, commit
git add .
git commit -m "feat: nova feature X"
```

### Bug Fix
```bash
# 1. Reproduzir bug
# 2. Corrigir
vim src/services/problematic.service.ts

# 3. Adicionar teste de regressão
vim src/services/__tests__/problematic.service.test.ts

# 4. Validar
pnpm lint && pnpm test

# 5. Commit
git commit -m "fix: correção bug Y"
```

### Hotfix Crítico
```bash
# 1. Branch de hotfix
git checkout -b hotfix/critical-issue

# 2. Corrigir urgente
vim src/...

# 3. Pipeline rápido
.claude/commands.yaml workflow:hotfix

# 4. Deploy imediato
git commit -m "fix: correção crítica Z"
git push
# Release PATCH (v1.4.0 → v1.4.1)
```

### Code Review
```bash
# 1. Checkout do PR
git fetch origin pull/42/head:pr-42
git checkout pr-42

# 2. Revisar automaticamente
.claude/commands.yaml workflow:review

# 3. Revisar manualmente
# - Duplicações
# - Complexidade
# - Nomes descritivos

# 4. Aprovar ou solicitar mudanças
```

## Gates de Qualidade

Critérios que devem passar antes de avançar:

### Gate 1: Desenvolvimento
- ✅ `pnpm format:check`
- ✅ `pnpm lint`
- ✅ `pnpm test` (223/223)
- ✅ Cobertura >= 80%
- ✅ `pnpm build`

### Gate 2: Revisão
- ✅ Sem `any` injustificado
- ✅ Sem duplicações
- ✅ HashId/ID corretos
- ✅ Tokens no Keychain
- ✅ SRP respeitado

### Gate 3: Testes
- ✅ 223/223 testes passando
- ✅ Cobertura >= 80%
- ✅ Testes estáveis (não flaky)
- ✅ Edge cases cobertos

### Gate 4: QA
- ✅ Critérios de aceite atendidos
- ✅ Fluxos principais validados
- ✅ UX intuitiva
- ✅ Performance aceitável

### Gate 5: Release
- ✅ CHANGELOG atualizado
- ✅ Build Tauri sem erros
- ✅ Assinatura válida
- ✅ QA aprovou

## Métricas e KPIs

### Qualidade
- **Cobertura**: >= 80% (atual: 100%)
- **Linter Errors**: 0
- **TypeScript `any`**: 0

### Performance
- **Build Frontend**: < 3 min
- **Build Tauri**: < 10 min
- **Testes**: < 30 seg

### Release
- **Frequência**: 1-2/mês (MINOR)
- **Hotfixes**: < 1/mês
- **Tempo para Release**: 1-2 semanas
- **Tempo para Hotfix**: < 24 horas

## Referências

- [Agentes](./../agents/README.md) - Agentes especializados
- [CLAUDE.md](./../../CLAUDE.md) - Guia completo do projeto
- [CHANGELOG.md](./../../CHANGELOG.md) - Histórico de releases

## Ajuda

Para ajuda sobre um comando específico:
```bash
# Lista todos os comandos
.claude/commands.yaml help

# Ver documentação de categoria
cat .claude/commands/basicos.md
cat .claude/commands/testes.md
cat .claude/commands/release.md
```

## Contribuindo

Para adicionar novos comandos:

1. Escolha a categoria apropriada
2. Adicione documentação no arquivo `.md` correspondente
3. Inclua: descrição, uso, quando usar, exemplos
4. Atualize este README se necessário




