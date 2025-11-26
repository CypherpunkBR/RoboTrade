# Comandos de Testes

Comandos para executar e gerenciar testes automatizados.

## test

**Descrição:** Executa todos os testes (Vitest)

**Uso:**
```bash
pnpm test
```

**O que executa:**
- Todos os arquivos `**/__tests__/**/*.test.ts`
- Testes unitários (services, stores, utils)
- Testes de componentes (React Testing Library)

**Output:**
- Total de testes: 223/223
- Tempo de execução
- Falhas (se houver)

**Quando usar:**
- Antes de commit (obrigatório)
- Após implementar feature
- Após correção de bug
- CI/CD pipeline

---

## test:watch

**Descrição:** Executa testes em modo watch

**Uso:**
```bash
pnpm test --watch
```

**Comportamento:**
- Re-executa testes ao salvar arquivos
- Modo interativo
- Filtros disponíveis

**Quando usar:**
- Desenvolvimento de testes
- TDD (Test-Driven Development)
- Debug de testes

**Atalhos no modo watch:**
- `a` - rodar todos os testes
- `f` - rodar apenas testes que falharam
- `p` - filtrar por nome do arquivo
- `t` - filtrar por nome do teste
- `q` - sair

---

## test:coverage

**Descrição:** Executa testes com relatório de cobertura

**Uso:**
```bash
pnpm test:coverage
```

**Output:**
- Relatório no terminal
- Relatório HTML em `coverage/`
- Métricas: statements, branches, functions, lines

**Métricas atuais:**
- **Cobertura total**: 100%
- **Baseline mínimo**: 80%

**Quando usar:**
- Validar cobertura após nova feature
- Verificar se cobertura caiu
- Identificar código não testado

**Ver relatório HTML:**
```bash
pnpm test:coverage
open coverage/index.html
```

---

## test:ui

**Descrição:** Abre interface visual de testes (Vitest UI)

**Uso:**
```bash
pnpm test:ui
```

**Features:**
- Interface gráfica no browser
- Visualização de testes
- Filtros e busca
- Console logs
- Cobertura visual

**Quando usar:**
- Explorar suíte de testes
- Debug visual
- Apresentações/demos

**URL:** `http://localhost:51204/__vitest__/`

---

## test:regression

**Descrição:** Verifica se cobertura de testes não regrediu (baseline 80%)

**Uso:**
```bash
# Via commands.yaml
.claude/commands.yaml test:regression

# Implementação manual
COVERAGE=$(pnpm test:coverage --reporter=json | jq -r '.total.lines.pct')
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
  echo "❌ Cobertura caiu para $COVERAGE% (mínimo: 80%)"
  exit 1
else
  echo "✅ Cobertura OK: $COVERAGE%"
fi
```

**Quando usar:**
- Antes de merge/PR
- CI/CD pipeline (gate)
- Após refatoração grande

**Baseline:**
- **Mínimo aceitável**: 80%
- **Atual**: 100%
- **Meta**: manter 100%

---

## Exemplos de Uso

### Desenvolvimento de feature com TDD
```bash
# 1. Iniciar watch mode
pnpm test --watch

# 2. Escrever teste (RED)
# 3. Implementar código (GREEN)
# 4. Refatorar (REFACTOR)
# 5. Repetir
```

### Validar antes de commit
```bash
# Pipeline completo
pnpm format && pnpm lint && pnpm test

# Se passar, está pronto para commit
```

### Verificar cobertura após feature
```bash
# Executar com cobertura
pnpm test:coverage

# Ver relatório
open coverage/index.html

# Validar que não caiu
.claude/commands.yaml test:regression
```

### Debug de teste específico
```bash
# Rodar apenas um arquivo
pnpm test src/services/__tests__/upload.service.test.ts

# Ou usar watch mode com filtro
pnpm test --watch
# Depois pressionar 'p' e filtrar por 'upload'
```

