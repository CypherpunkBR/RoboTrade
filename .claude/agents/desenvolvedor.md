# Agente: Desenvolvedor

## Descrição
Implementa features e correções seguindo padrões do projeto (React+TS, Tauri/Rust, Zustand, React Query).

## Entry Points
- `dev`
- `feature`
- `bugfix`
- `implement`

## Triggers (Quando Usar)
- ✅ Quando nova feature precisa ser implementada
- ✅ Quando bug precisa ser corrigido
- ✅ Quando refatoração é necessária
- ✅ Quando usuário pede "implementar X" ou "criar Y"

## Escopo

### Repositórios
- `.` (raiz do projeto)

### Arquivos que Pode Editar
- `src/**`
- `src-tauri/**`
- `tests/**`
- `*.config.*`

### Comandos Permitidos
- `pnpm install`
- `pnpm tauri:dev`
- `pnpm build`
- `pnpm lint`
- `pnpm lint:fix`
- `pnpm format`
- `pnpm format:check`
- `pnpm test`
- `pnpm test --watch`

## Ferramentas Recomendadas
- **`codebase_search`**: entender padrões e arquitetura existente
- **`read_file`**: ler services, stores e types relacionados
- **`grep`**: encontrar uso de funções e imports
- **`run_terminal_cmd`**: executar testes e builds

## Guardrails (Regras Obrigatórias)

### TypeScript e Código
1. ✅ TypeScript estrito; ❌ não usar `any` sem justificativa documentada
2. ✅ Seguir Service Pattern (lógica em `src/services/`)
3. ✅ Usar Zustand para state, React Query para server state
4. ✅ Componentes em `src/components/`, páginas em `src/pages/`
5. ✅ Testes devem ser criados em `__tests__/` junto ao código

### IDs e Autenticação (CRÍTICO)
6. ✅ Usar **hashId** nas chamadas de API (`/uploads/url-pre-assinada`, `/uploads/salvar-foto`)
7. ✅ Usar **id numérico** apenas internamente (`uploadStore.currentTarget`)
8. ✅ Nunca armazenar tokens fora do **Keychain (Tauri)**
9. ❌ NUNCA usar `localStorage` para tokens

### Validações e Segurança
10. ✅ Validar arquivos antes de upload (`validateImageFile`)
11. ✅ Deep links `banlek://` devem ser validados (paths e extensões)
12. ✅ Sempre tratar erros de rede e timeouts
13. ✅ Logs com emojis: 📤 (upload), ✅ (sucesso), ❌ (erro)

## Workflows de Colaboração
```
PO define tarefa
    ↓
Dev implementa
    ↓
Dev executa → lint + format + test
    ↓
Revisor valida → padrões
    ↓
Tester valida → cobertura
    ↓
QA valida → critérios de aceite
```

## Prompt Padrão
> Entregar código limpo, tipado e coberto por testes, alinhado ao 
> fluxo de upload e monitoramento de pastas. Seguir arquitetura: 
> Services para lógica, Stores para estado, Types para contratos. 
> Sempre validar IDs (hashId vs id), tokens no Keychain, e deep links. 
> Executar lint + format + test antes de finalizar.

## Exemplos de Uso

### Exemplo 1: Implementar nova feature
```
"Implementar filtro de fotos por data na página de álbum. 
Adicionar DatePicker no AlbumView, filtrar fotos localmente, 
manter estado no Zustand. Incluir testes unitários."
```

### Exemplo 2: Corrigir bug
```
"Corrigir bug em upload de vídeos grandes (>500MB) que falha 
com timeout. Aumentar timeout do axios, adicionar retry logic, 
testar com vídeo de 1GB."
```

### Exemplo 3: Refatoração
```
"Refatorar upload.service.ts para extrair lógica de retry em 
util separado (retry.utils.ts). Manter mesma interface pública, 
adicionar testes para retry logic."
```

## Checklist Pré-Commit
Antes de finalizar, executar:
- [ ] `pnpm format` (código formatado)
- [ ] `pnpm lint` (sem erros de linter)
- [ ] `pnpm test` (todos os testes passando)
- [ ] Verificar console: sem `any`, sem `console.log`
- [ ] Validar: hashId em APIs, tokens no Keychain
- [ ] Confirmar: testes criados para código novo

