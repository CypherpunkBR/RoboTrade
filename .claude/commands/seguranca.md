# Comandos de Segurança

Comandos para validar regras de segurança específicas do projeto.

## check:security

**Descrição:** Valida regras de segurança do projeto (tokens, IDs, deep links)

**Uso:**
```bash
.claude/commands.yaml check:security
```

**O que verifica:**
1. ❌ Tokens em `localStorage` (proibido)
2. ❌ Tokens em `sessionStorage` (proibido)
3. ✅ Tokens apenas via Keychain (Tauri)

**Implementação:**
```bash
echo "🔒 Verificando segurança..."

# Verifica localStorage para tokens
if grep -r "localStorage.setItem.*token" src/ --include="*.ts" --include="*.tsx" 2>/dev/null; then
  echo "❌ Token sendo armazenado em localStorage (usar Keychain via Tauri)"
  exit 1
fi

# Verifica sessionStorage para tokens
if grep -r "sessionStorage.setItem.*token" src/ --include="*.ts" --include="*.tsx" 2>/dev/null; then
  echo "❌ Token sendo armazenado em sessionStorage (usar Keychain via Tauri)"
  exit 1
fi

echo "✅ Segurança OK: tokens não em localStorage/sessionStorage"
```

**Quando usar:**
- Antes de commit de código de autenticação
- CI/CD pipeline (gate)
- Revisão de código
- Auditoria de segurança

**Regras validadas:**
- ✅ Tokens devem ser armazenados via Tauri Keychain: `invoke('store_token', { token })`
- ❌ NUNCA em `localStorage.setItem('token', ...)`
- ❌ NUNCA em `sessionStorage.setItem('token', ...)`

---

## validate:ids

**Descrição:** Verifica uso correto de hashId vs id numérico

**Uso:**
```bash
.claude/commands.yaml validate:ids
```

**O que verifica:**
1. Endpoints de upload usam `hashId`
2. Stores internos podem usar `id` numérico
3. Identificar usos suspeitos

**Implementação:**
```bash
echo "🔍 Validando uso de IDs..."

# Busca potenciais usos incorretos em endpoints de upload
echo "Verificando endpoints de upload..."
grep -n "url-pre-assinada\|salvar-foto\|salvar-video" src/**/*.ts 2>/dev/null | grep -v "hashId" && \
  echo "⚠️  Possível uso de ID incorreto em endpoint de upload (use hashId)" || \
  echo "✅ Endpoints de upload OK"

echo "✅ Validação de IDs concluída"
```

**Quando usar:**
- Antes de commit de código de upload
- Após implementar nova feature de upload
- Review de código
- Debug de erros de API

**Regras validadas:**
- ✅ `/uploads/url-pre-assinada` → usar `hashId`
- ✅ `/uploads/salvar-foto` → usar `hashId`
- ✅ `/uploads/salvar-video` → usar `hashId`
- ✅ `uploadStore.currentTarget` → pode ter ambos `{ id, hashId }`

**Exemplo correto:**
```typescript
// ✅ Correto - usando hashId
const presignedUrl = await api.post('/uploads/url-pre-assinada', {
  targetHashId: album.id  // hashId, não id!
});

// ✅ Correto - store pode ter ambos
uploadStore.setTarget({
  id: album.id,          // id numérico para uso interno
  hashId: album.id,  // hashId para APIs
  type: 'album'
});
```

---

## check:types

**Descrição:** Verifica se há uso de 'any' no código TypeScript

**Uso:**
```bash
.claude/commands.yaml check:types
```

**O que verifica:**
1. `: any` em variáveis
2. `<any>` em generics
3. `as any` em type assertions

**Implementação:**
```bash
echo "🔍 Buscando tipos 'any'..."

# Busca 'any' excluindo node_modules e comentários
RESULT=$(grep -rn ": any\|<any>\|as any" src/ --include="*.ts" --include="*.tsx" | grep -v "node_modules" | grep -v "//" || true)

if [ -n "$RESULT" ]; then
  echo "⚠️  Encontrado uso de 'any':"
  echo "$RESULT"
  echo ""
  echo "TypeScript strict: evite 'any', use tipos específicos"
  exit 1
else
  echo "✅ Nenhum 'any' encontrado - tipos OK!"
fi
```

**Quando usar:**
- Antes de commit
- Code review
- Refatoração de tipos
- Auditoria de qualidade

**Regras:**
- ❌ `any` é proibido sem justificativa documentada
- ✅ TypeScript strict mode ativo
- ✅ Tipos explícitos obrigatórios

**Alternativas ao `any`:**
```typescript
// ❌ Evitar
const data: any = response.data;

// ✅ Usar tipos específicos
interface UploadResponse {
  url: string;
  expires: number;
}
const data: UploadResponse = response.data;

// ✅ Ou unknown + type guard
const data: unknown = response.data;
if (isUploadResponse(data)) {
  // data é UploadResponse aqui
}

// ✅ Ou generic
function handleData<T>(data: T): void { ... }
```

---

## Workflow de Segurança

Execute todos os checks de segurança antes de commit:

```bash
# 1. Verificar tokens
.claude/commands.yaml check:security

# 2. Verificar IDs
.claude/commands.yaml validate:ids

# 3. Verificar tipos
.claude/commands.yaml check:types

# Ou usar workflow completo que inclui tudo:
.claude/commands.yaml workflow:review
```

---

## Problemas Comuns

### Problema: Token em localStorage
```typescript
// ❌ ERRADO
localStorage.setItem('auth_token', token);

// ✅ CORRETO
import { invoke } from '@tauri-apps/api';
await invoke('store_token', { token });
```

### Problema: ID numérico em API
```typescript
// ❌ ERRADO
await api.post('/uploads/salvar-foto', {
  targetHashId: album.id  // id numérico!
});

// ✅ CORRETO
await api.post('/uploads/salvar-foto', {
  targetHashId: album.id  // hashId!
});
```

### Problema: Tipo 'any'
```typescript
// ❌ ERRADO
const handleUpload = (file: any) => { ... }

// ✅ CORRETO
interface UploadFile {
  name: string;
  size: number;
  type: string;
}
const handleUpload = (file: UploadFile) => { ... }
```

