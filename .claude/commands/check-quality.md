# /check-quality

Ativa os 3 agentes especializados globais em paralelo para análise completa de qualidade do código.

## Uso

```
/check-quality [path]
```

**Argumentos**:
- `path` (opcional): Caminho específico para analisar (ex: `src/services/`, `src/components/`)
- Se omitido, analisa o projeto completo

---

## O Que Faz

Executa **3 agentes especializados em paralelo**:

### 1. 🧹 Cleanup Specialist
**Foco**: Código e documentação desnecessários

Verifica:
- [ ] Imports não utilizados
- [ ] Variáveis/funções declaradas e não usadas
- [ ] Código comentado (>5 linhas)
- [ ] Duplicação de lógica
- [ ] Componentes similares que podem ser unificados
- [ ] Arquivos muito grandes (>500 linhas)
- [ ] TODOs/FIXMEs obsoletos
- [ ] Comentários redundantes
- [ ] Documentação desatualizada
- [ ] Arquivos .md duplicados
- [ ] Dependências não utilizadas
- [ ] Assets não referenciados

### 2. ⚛️ React Best Practices Specialist
**Foco**: Boas práticas React e TypeScript

Verifica:
- [ ] useEffect com dependências corretas
- [ ] useMemo/useCallback apropriados
- [ ] Estado derivado vs. armazenado
- [ ] React.memo em componentes custosos
- [ ] Props drilling (max 3 níveis)
- [ ] Keys em listas (únicas e estáveis)
- [ ] Acessibilidade (a11y)
- [ ] Error Boundaries
- [ ] Ternários não muito aninhados
- [ ] Estado não é mutado diretamente
- [ ] Props têm tipos TypeScript
- [ ] Event handlers tipados
- [ ] Custom hooks seguem convenção 'use*'
- [ ] Cleanup em useEffect

### 3. ⚡ Performance Optimizer Specialist
**Foco**: Performance e otimizações

Verifica:
- [ ] Re-renders desnecessários
- [ ] Objetos/arrays recriados a cada render
- [ ] Context que causa re-renders massivos
- [ ] Event listeners não removidos
- [ ] Timers/intervals não cancelados
- [ ] setState após unmount
- [ ] Bundle size (imports completos de libs)
- [ ] Imagens não otimizadas
- [ ] Code splitting por rota
- [ ] Lazy loading para componentes pesados
- [ ] N+1 queries (waterfalls)
- [ ] useEffect loops infinitos
- [ ] Listas grandes sem virtualização
- [ ] Computações pesadas não memoizadas
- [ ] Polling desnecessário

---

## Saída Esperada

### Relatório Consolidado

```markdown
# 🔍 Relatório de Qualidade - Banlek Uploader

## 📊 Resumo Executivo

| Categoria              | Issues | Prioridade Alta | Prioridade Média | Prioridade Baixa |
| ---------------------- | ------ | --------------- | ---------------- | ----------------|
| **Cleanup**            | 23     | 3               | 8                | 12              |
| **React Best Practices** | 17     | 5               | 7                | 5               |
| **Performance**        | 12     | 2               | 6                | 4               |
| **TOTAL**              | **52** | **10**          | **21**           | **21**          |

---

## 🔴 Prioridade Alta (10 issues)

### 1. [Cleanup] Código Comentado em upload.service.ts
**Linhas**: 245-380 (135 linhas)
**Problema**: Código antigo de retry já reimplementado
**Ação**: Remover (confiar no Git)

### 2. [React] useEffect com dependências incorretas
**Arquivo**: `AlbumList.tsx:45`
**Problema**: `albumId` usado mas não nas deps
**Ação**: Adicionar `albumId` às dependências

### 3. [Performance] PhotoGallery re-renders excessivos
**Impacto**: 2000+ re-renders/min
**Problema**: Lista de 500 fotos re-renderiza toda vez
**Ação**: Usar React.memo em PhotoItem

---

## 🟡 Prioridade Média (21 issues)

### 4. [Cleanup] Imports não usados (15 arquivos)
**Ação**: Executar `pnpm lint --fix`

### 5. [React] Props drilling excessivo
**Arquivo**: `Upload.tsx`
**Problema**: `user` props passado por 5 níveis
**Ação**: Criar UserContext

---

## 🟢 Prioridade Baixa (21 issues)

### 6. [Performance] Images não otimizadas
**Impacto**: Primeira carga lenta
**Ação**: Converter para WebP + lazy loading

---

## 📝 Recomendações Gerais

1. Configurar ts-prune no CI
2. Adicionar pre-commit hook para bloquear console.log
3. Revisar TODOs mensalmente
4. Limitar tamanho de arquivos (ESLint rule)

---

## 🎯 Próximos Passos

1. Corrigir 10 issues de prioridade alta
2. Agendar refatoração de código duplicado
3. Otimizar PhotoGallery (maior impacto)
```

---

## Exemplos

### Análise Completa do Projeto
```
/check-quality
```

### Análise de Pasta Específica
```
/check-quality src/services/
```

### Análise de Componentes
```
/check-quality src/components/upload/
```

---

## Opções Avançadas

### Foco em um Aspecto
```
/check-quality --focus=cleanup
/check-quality --focus=react
/check-quality --focus=performance
```

### Exportar Relatório
```
/check-quality --export=markdown > quality-report.md
/check-quality --export=json > quality-report.json
```

### Integração com PR
```
# No PR review
@claude /check-quality src/
```

---

## Frequência Recomendada

- **Antes de release**: Obrigatório
- **Após features grandes**: Recomendado
- **Weekly**: Opcional mas útil
- **Em PRs**: Se mudanças >500 linhas

---

## Métricas de Sucesso

Um código de qualidade deve ter:

| Métrica                | Meta    |
| ---------------------- | ------- |
| **Imports não usados** | 0       |
| **Variáveis não usadas** | 0       |
| **Código comentado**   | <10 linhas |
| **Arquivos >500 linhas** | <5      |
| **TODOs obsoletos**    | 0       |
| **Duplicação**         | <5%     |
| **Dependências não usadas** | 0       |
| **Re-renders desnecessários** | <1% |
| **Memory leaks**       | 0       |
| **Bundle size**        | <500KB  |

---

## Notas

- Análise é **read-only** (não modifica código)
- Relatório inclui exemplos de código
- Sugestões são acionáveis (não vagas)
- Priorização baseada em impacto vs. esforço

---

**Última Atualização**: 2025-11-12
**Versão**: 1.0.0
