# Comandos de Qualidade

Comandos para manter padrões de código e qualidade.

## lint

**Descrição:** Executa linter (ESLint)

**Uso:**
```bash
pnpm lint
```

**O que verifica:**
- Erros de sintaxe
- Padrões de código
- Regras ESLint configuradas
- TypeScript errors básicos

**Quando usar:**
- Antes de commit
- Durante desenvolvimento
- Pipeline de CI/CD

---

## lint:fix

**Descrição:** Executa linter com correções automáticas

**Uso:**
```bash
pnpm lint:fix
```

**O que corrige:**
- Formatação básica
- Imports não usados
- Espaçamentos
- Alguns problemas de estilo

**Quando usar:**
- Correção rápida de erros de lint
- Após refatoração grande
- Limpeza de código

**⚠️ Aviso:** Sempre revise as mudanças antes de commitar

---

## format

**Descrição:** Formata código com Prettier

**Uso:**
```bash
pnpm format
```

**O que formata:**
- Indentação
- Aspas
- Ponto-e-vírgula
- Line breaks
- Todos os arquivos: .ts, .tsx, .js, .json, .md

**Quando usar:**
- Antes de commit (obrigatório)
- Após escrever código novo
- Trabalho em equipe (consistência)

---

## format:check

**Descrição:** Verifica formatação sem modificar arquivos

**Uso:**
```bash
pnpm format:check
```

**Exit code:**
- `0` - Tudo formatado corretamente
- `1` - Alguns arquivos precisam formatação

**Quando usar:**
- CI/CD pipeline (gate)
- Pre-commit hook
- Validação antes de PR

**Exemplo de uso em workflow:**
```bash
# Validar antes de commit
pnpm format:check || (echo "Execute: pnpm format" && exit 1)
```

