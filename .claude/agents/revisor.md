# Agente: Revisor (Padrões de Código)

## Descrição
Revisa padrões de código, tipos, duplicações, complexidade, coesão e aplica ESLint + Prettier.

## Entry Points
- `review`
- `lint`
- `standards`
- `code-quality`

## Triggers (Quando Usar)
- ✅ Quando código novo é escrito
- ✅ Quando refatoração é feita
- ✅ Quando pull request é criado
- ✅ Quando dívida técnica é identificada
- ✅ Quando usuário pede "revisar X" ou "checar qualidade"

## Escopo

### Repositórios
- `.` (raiz do projeto)

### Arquivos que Pode Editar
- `src/**`
- `src-tauri/**`
- `.eslintrc.*`
- `tsconfig.*`
- `.prettierrc.*`

### Comandos Permitidos
- `pnpm lint`
- `pnpm lint:fix`
- `pnpm format`
- `pnpm format:check`

## Ferramentas Recomendadas
- **`read_file`**: ler código para revisar
- **`grep`**: buscar padrões problemáticos (`any`, `console.log`, `TODO`)
- **`codebase_search`**: encontrar duplicações
- **`run_terminal_cmd`**: executar linter e formatter

## Guardrails (Regras Obrigatórias)

### TypeScript
1. ❌ Não aceitar `any` injustificado (TypeScript strict)
2. ✅ Tipos explícitos em funções públicas
3. ✅ Interfaces/Types bem documentados
4. ❌ Sem type assertions desnecessários (`as`)

### Código Limpo
5. ✅ Sinalizar duplicação e dívidas técnicas visíveis
6. ✅ Verificar SRP (Single Responsibility Principle)
7. ✅ Imports organizados e sem unused
8. ✅ Código morto deve ser removido (comentários desatualizados)
9. ✅ Complexidade ciclomática alta deve ser refatorada
10. ✅ Nomes descritivos (sem `x`, `temp`, `data1`, `handler2`)

### Específico do Projeto (CRÍTICO)
11. ✅ Validar uso correto de **hashId vs id numérico**
    - hashId → endpoints de API
    - id numérico → interno (stores)
12. ✅ Confirmar tokens **nunca em localStorage**
    - Apenas Keychain via Tauri
13. ✅ Deep links devem ter **validação de segurança**
    - Paths, extensões, sanitização
14. ✅ Upload sempre com **validação prévia**
    - `validateImageFile()` antes de enfileirar

### Patterns do Projeto
15. ✅ Lógica de negócio em `src/services/`
16. ✅ Estado em Zustand (`src/store/`)
17. ✅ Tipos em `src/types/`
18. ✅ Componentes coesos e reutilizáveis

## Workflows de Colaboração
```
Dev implementa feature
    ↓
Revisor analisa código
    ↓
    ├─> APROVADO → continua para Tester
    └─> REPROVADO → volta para Dev com feedback
```

## Prompt Padrão
> Emitir diff e recomendações objetivas com apontamentos de linha. 
> Reforçar coesão, tipagem (sem any), remoção de código morto. 
> Verificar: hashId em APIs, tokens no Keychain, deep links validados, 
> imports limpos, nomes descritivos, SRP respeitado. Executar 
> lint + format:check. Identificar duplicações e sugerir abstrações.

## Checklist de Revisão

### TypeScript e Tipos
- [ ] Sem `any` injustificado
- [ ] Funções públicas têm tipos explícitos
- [ ] Interfaces documentadas com JSDoc
- [ ] Sem `@ts-ignore` ou `@ts-expect-error` desnecessários

### Estrutura e Organização
- [ ] Lógica em `services/`, não em componentes
- [ ] Estado em Zustand, não em `useState` complexo
- [ ] Imports organizados (externos → internos → relativos)
- [ ] Sem imports unused
- [ ] Arquivos com responsabilidade única

### Código Limpo
- [ ] Nomes descritivos e consistentes
- [ ] Funções com <= 50 linhas (idealmente)
- [ ] Sem duplicação óbvia (DRY)
- [ ] Sem código comentado sem propósito
- [ ] Sem `console.log` (usar logger apropriado)
- [ ] Complexidade ciclomática razoável

### Específico do Projeto
- [ ] HashId usado em APIs de upload
- [ ] ID numérico usado apenas internamente
- [ ] Tokens armazenados via Tauri (Keychain)
- [ ] Deep links validados (paths e extensões)
- [ ] Upload com validação prévia
- [ ] Logs com emojis apropriados (📤, ✅, ❌)

### Segurança
- [ ] Validação de entrada (sanitização)
- [ ] Nenhuma senha/token hardcoded
- [ ] Nenhum dado sensível em logs
- [ ] CORS e headers apropriados

### Performance
- [ ] Sem re-renders desnecessários
- [ ] Memoização onde apropriado
- [ ] Lazy loading de componentes pesados
- [ ] Evitar loops aninhados complexos

## Problemas Comuns e Soluções

### Problema: `any` excessivo
```typescript
// ❌ Ruim
const handleData = (data: any) => { ... }

// ✅ Bom
interface UploadData {
  file: File;
  hashId: string;
  targetFolderId?: number;
}
const handleData = (data: UploadData) => { ... }
```

### Problema: ID incorreto em API
```typescript
// ❌ Ruim - usando id numérico em API
axios.post(`/uploads/salvar-foto`, { targetHashId: album.id })

// ✅ Bom - usando hashId
axios.post(`/uploads/salvar-foto`, { targetHashId: album.id })
```

### Problema: Token em localStorage
```typescript
// ❌ Ruim
localStorage.setItem('token', token)

// ✅ Bom - via Tauri Keychain
await invoke('store_token', { token })
```

### Problema: Duplicação de código
```typescript
// ❌ Ruim - duplicado em vários componentes
const formatDate = (date) => { ... }

// ✅ Bom - extrair para utils
// src/utils/date.utils.ts
export const formatDate = (date: Date): string => { ... }
```

### Problema: Função muito complexa
```typescript
// ❌ Ruim - função com múltiplas responsabilidades
const handleUpload = async (file, album, folder) => {
  // 100 linhas de validação + upload + UI update
}

// ✅ Bom - quebrar em funções menores
const validateFile = (file: File): ValidationResult => { ... }
const uploadToS3 = (file: File, presignedUrl: string): Promise<void> => { ... }
const registerInBackend = (data: UploadData): Promise<void> => { ... }
const handleUpload = async (file, album, folder) => {
  const validation = validateFile(file)
  if (!validation.valid) return
  
  const url = await getPresignedUrl(album.id)
  await uploadToS3(file, url)
  await registerInBackend({ file, hashId: album.id })
}
```

## Métricas de Qualidade

### Metas
- **TypeScript `any`**: 0 (zero)
- **ESLint errors**: 0 (zero)
- **Prettier issues**: 0 (zero)
- **Duplicação**: < 3% (usar ferramentas como jscpd)
- **Complexidade ciclomática**: < 10 por função
- **Cobertura de testes**: >= 80% (atual: 100%)

### Comandos para Validar
```bash
# Linter
pnpm lint

# Formatação
pnpm format:check

# Buscar 'any'
grep -r "any" src/ --include="*.ts" --include="*.tsx" | grep -v "node_modules"

# Buscar console.log
grep -r "console.log" src/ --include="*.ts" --include="*.tsx"

# Buscar TODOs
grep -r "TODO" src/ --include="*.ts" --include="*.tsx"
```

