# Workflows Compostos

Comandos que executam múltiplos passos em sequência.

## workflow:feature

**Descrição:** Pipeline completo para desenvolver nova feature

**Uso:**
```bash
.claude/commands.yaml workflow:feature
```

**Pipeline:**
```
Etapa 1/6: Formatação
  → pnpm format:check

Etapa 2/6: Linter
  → pnpm lint

Etapa 3/6: Verificação de tipos (any)
  → .claude/commands.yaml check:types

Etapa 4/6: Segurança
  → .claude/commands.yaml check:security

Etapa 5/6: Testes
  → pnpm test

Etapa 6/6: Build
  → pnpm build

✅ Feature pronta para revisão!

Próximos passos:
  1. Revisor valida código
  2. Tester valida cobertura
  3. QA valida no app
```

**Quando usar:**
- Antes de commit de nova feature
- Antes de abrir PR
- Validação completa local
- CI/CD pipeline

**Tempo estimado:** 1-2 minutos

**Se falhar:**
- Etapa 1: Execute `pnpm format`
- Etapa 2: Execute `pnpm lint:fix` e corrija manualmente
- Etapa 3: Remova `any`, use tipos específicos
- Etapa 4: Corrija violações de segurança
- Etapa 5: Corrija testes quebrados
- Etapa 6: Corrija erros de build

---

## workflow:hotfix

**Descrição:** Pipeline rápido para correções urgentes

**Uso:**
```bash
.claude/commands.yaml workflow:hotfix
```

**Pipeline (simplificado):**
```
Etapa 1/4: Linter rápido
  → pnpm lint

Etapa 2/4: Testes
  → pnpm test

Etapa 3/4: Build
  → pnpm build

Etapa 4/4: Build Tauri
  → pnpm tauri:build

✅ Hotfix pronto!

⚠️  Lembrete: atualizar CHANGELOG.md e fazer PATCH release
```

**Quando usar:**
- Bug crítico em produção
- Correção de segurança urgente
- Situação emergencial

**Diferença do workflow:feature:**
- ❌ Sem formatação (mais rápido)
- ❌ Sem check de tipos
- ❌ Sem check de segurança completo
- ✅ Foco em estabilidade mínima
- ✅ Build completo incluído

**Timeline:** < 24 horas da identificação ao deploy

**⚠️ Importante:**
- Hotfix pula algumas validações para velocidade
- Após hotfix, fazer revisão completa na próxima release
- Sempre atualizar CHANGELOG.md
- Fazer PATCH release (ex: 1.4.0 → 1.4.1)

---

## workflow:review

**Descrição:** Pipeline de revisão de código

**Uso:**
```bash
.claude/commands.yaml workflow:review
```

**Pipeline:**
```
🔍 Workflow: Revisão de Código

1. Formatação
   → pnpm format:check

2. Linter
   → pnpm lint

3. Tipos (any)
   → .claude/commands.yaml check:types

4. Segurança
   → .claude/commands.yaml check:security

5. IDs (hashId vs id)
   → .claude/commands.yaml validate:ids

✅ Revisão automática concluída

⚠️  Revisor deve também validar:
  - Duplicação de código
  - Complexidade ciclomática
  - SRP (Single Responsibility)
  - Nomes descritivos
```

**Quando usar:**
- Code review de PR
- Validação de padrões
- Auditoria de qualidade
- Antes de merge

**Responsável:** Agente Revisor

**Checklist manual adicional:**
- [ ] Sem duplicação óbvia
- [ ] Complexidade razoável (< 10 por função)
- [ ] SRP respeitado
- [ ] Nomes descritivos
- [ ] Imports organizados
- [ ] Sem código morto
- [ ] Testes adequados

---

## workflow:test

**Descrição:** Pipeline completo de testes

**Uso:**
```bash
.claude/commands.yaml workflow:test
```

**Pipeline:**
```
1. Testes unitários
   → pnpm test

2. Cobertura
   → pnpm test:coverage

3. Validar baseline
   → .claude/commands.yaml test:regression

✅ Testes OK!

Cobertura: 100%
Testes: 223/223 passando
Baseline: mantido
```

**Quando usar:**
- Validar suíte completa de testes
- Após refatoração grande
- Verificar saúde dos testes
- CI/CD pipeline

---

## Comparação de Workflows

| Workflow | Tempo | Quando Usar | Gates |
|----------|-------|-------------|-------|
| **feature** | 1-2 min | Nova feature, PR | Todos (6 etapas) |
| **hotfix** | 5-10 min | Emergência | Mínimos (4 etapas) |
| **review** | 30s | Code review | Padrões (5 checks) |
| **test** | 30s | Validar testes | Testes (3 checks) |

---

## Exemplos de Uso

### Desenvolver nova feature
```bash
# 1. Implementar código
vim src/components/NewFeature.tsx

# 2. Validar localmente
.claude/commands.yaml workflow:feature

# 3. Se passar, commit
git add .
git commit -m "feat: nova feature X"

# 4. Push e PR
git push
```

### Hotfix urgente
```bash
# 1. Criar branch de hotfix
git checkout -b hotfix/critical-bug

# 2. Corrigir bug
vim src/services/problematic.service.ts

# 3. Testar correção
.claude/commands.yaml workflow:hotfix

# 4. Deploy rápido
git commit -m "fix: correção crítica"
git push
# Criar release imediata
```

### Code review
```bash
# 1. Checkout do PR
git fetch origin pull/42/head:pr-42
git checkout pr-42

# 2. Revisar código
.claude/commands.yaml workflow:review

# 3. Se passar, revisar manualmente
# 4. Aprovar ou solicitar mudanças
```

---

## Personalizar Workflows

Você pode criar seus próprios workflows combinando comandos:

```bash
# Workflow customizado: quick-check
pnpm lint && pnpm test && echo "✅ Quick check OK!"

# Workflow customizado: pre-commit
pnpm format && pnpm lint && pnpm test

# Workflow customizado: full-validation
.claude/commands.yaml workflow:feature && \
.claude/commands.yaml workflow:review && \
pnpm tauri:build
```

**Adicione ao package.json:**
```json
{
  "scripts": {
    "pre-commit": "pnpm format && pnpm lint && pnpm test",
    "quick-check": "pnpm lint && pnpm test",
    "full-validation": "pnpm format:check && pnpm lint && pnpm test && pnpm build"
  }
}
```




