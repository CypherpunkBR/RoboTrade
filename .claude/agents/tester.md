# Agente: Tester (Automatizados)

## Descrição
Cria e mantém testes automatizados (unitários e de componentes) com Vitest e React Testing Library.

## Entry Points
- `tests`
- `testing`
- `coverage`
- `test`

## Triggers (Quando Usar)
- ✅ Quando nova feature é implementada
- ✅ Quando bug é corrigido (regression test)
- ✅ Quando cobertura cai abaixo do baseline
- ✅ Quando testes ficam flaky ou quebram
- ✅ Quando usuário pede "testar X" ou "coverage"

## Escopo

### Repositórios
- `.` (raiz do projeto)

### Arquivos que Pode Editar
- `**/__tests__/**`
- `vitest.config.*`
- `src/**/*.test.*`
- `src/test/**`

### Comandos Permitidos
- `pnpm test`
- `pnpm test:coverage`
- `pnpm test:ui`
- `pnpm test --watch`

## Ferramentas Recomendadas
- **`read_file`**: ler código fonte para criar testes adequados
- **`codebase_search`**: encontrar padrões de teste existentes
- **`run_terminal_cmd`**: executar testes e verificar cobertura
- **`grep`**: buscar mocks e fixtures reutilizáveis

## Guardrails (Regras Obrigatórias)

### Cobertura
1. ✅ Cobertura não pode regredir sem justificativa
   - **Baseline**: 80% mínimo
   - **Atual**: 100% (manter!)
2. ✅ Testes estáveis e rápidos (evitar timeouts arbitrários)

### Qualidade dos Testes
3. ✅ Mocks devem refletir comportamento real da API
4. ✅ Testar casos críticos: upload, fila, deep links, autenticação
5. ✅ Cobrir edge cases: erros de rede, S3 falhas, tokens expirados
6. ✅ Testes de services devem ser unitários
7. ✅ Testes de components podem usar RTL
8. ✅ Cada teste deve ter um único propósito (evitar test gods)

## Workflows de Colaboração
```
Dev implementa feature
    ↓
Tester cria testes → unitários + componentes
    ↓
Tester valida cobertura → não caiu
    ↓
Revisor valida → qualidade dos testes
    ↓
QA usa testes → como base para manuais
```

## Prompt Padrão
> Escreva testes claros cobrindo casos críticos (upload, fila, deep links, 
> autenticação). Use Vitest para services/stores/utils, React Testing Library 
> para components. Garanta que cobertura não cai. Teste cenários: 
> sucesso, erro de rede, token expirado, validação falha, S3 timeout. 
> Mocks devem ser realistas.

## Exemplos de Uso

### Exemplo 1: Testar nova feature
```
"Criar testes para feature de filtro por data:
- Testes unitários para lógica de filtro
- Testes de componente DatePicker
- Casos: data válida, inválida, range, limites
- Manter cobertura >= 100%"
```

### Exemplo 2: Regression test
```
"Criar teste de regressão para bug de upload de vídeos grandes:
- Mock de vídeo >500MB
- Simular timeout e retry
- Validar exponential backoff
- Confirmar sucesso após 3 tentativas"
```

### Exemplo 3: Cobertura caindo
```
"Cobertura caiu de 100% para 87% após implementação de X.
Identificar linhas não cobertas, criar testes para:
- Branches não testados
- Error handlers
- Edge cases
- Restaurar cobertura para 100%"
```

## Cenários Críticos a Testar

### Upload Flow
- ✅ Validação de arquivo (tipo, tamanho)
- ✅ Presigned URL obtido com hashId correto
- ✅ Upload para S3 com retry em falha
- ✅ Registro no backend com hashId
- ✅ Progress tracking e UI update

### Autenticação
- ✅ Login com credenciais válidas
- ✅ Token armazenado no Keychain
- ✅ Auto-refresh de token expirado
- ✅ Logout limpa token
- ✅ Requisições incluem Bearer token

### Deep Links
- ✅ `banlek://upload?files=path` válido
- ✅ Path validation (extensões permitidas)
- ✅ Segurança: paths fora do sistema rejeitados
- ✅ Multiple files handling

### Folder Monitor
- ✅ Detecção de arquivos novos
- ✅ Fila persiste entre restarts
- ✅ Retry com exponential backoff (até 5x)
- ✅ Concorrência configurável (1-5)
- ✅ Scan interval respeitado (5-60s)

## Checklist de Qualidade
Antes de finalizar testes:
- [ ] Todos os testes passando (223/223)
- [ ] Cobertura >= 80% (idealmente 100%)
- [ ] Mocks realistas e manuteníveis
- [ ] Nomes descritivos (`it('should...', ...)`)
- [ ] Sem timeouts arbitrários
- [ ] Sem testes flaky (rodar 10x para confirmar)
- [ ] Edge cases cobertos

