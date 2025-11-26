# Comandos de Debug

Comandos para auxiliar no debug e troubleshooting.

## debug:upload

**Descrição:** Exibe logs detalhados do fluxo de upload

**Uso:**
```bash
.claude/commands.yaml debug:upload
```

**Output:**
```
📤 Fluxo de Upload - Pontos de Log:

1. validateImageFile() - src/services/upload.service.ts
2. getPresignedUrl() - Usa hashId
3. uploadToS3() - Usa presigned URL
4. registerPhoto() - POST /uploads/salvar-foto com hashId
5. uploadStore.updateProgress() - Atualiza UI

Logs no console (DevTools):
  📤 Iniciando upload
  ✅ Upload concluído
  ❌ Erro no upload

Para debug, abra DevTools: Cmd+Opt+I
```

**Quando usar:**
- Upload não funciona
- Debug de fluxo completo
- Entender sequência de chamadas
- Onboarding de novos devs

**Pontos de debug:**
1. **Validação**: `validateImageFile()` em `upload.service.ts`
2. **Presigned URL**: Verificar `hashId` sendo usado
3. **S3 Upload**: Timeout, CORS, presigned URL válida
4. **Registro**: Backend recebe hashId correto
5. **UI**: Progress updates e estados

---

## debug:deep-links

**Descrição:** Informações sobre debug de deep links (banlek://)

**Uso:**
```bash
.claude/commands.yaml debug:deep-links
```

**Output:**
```
🔗 Deep Links - Sistema de Menu de Contexto

URL Scheme: banlek://upload

Exemplos:
  banlek://upload?files=/path/to/file.jpg
  banlek://upload?album=456&files=/path/to/folder

Implementação:
  - Frontend: src/hooks/useDeepLink.ts
  - Backend: src-tauri/src/main.rs

Validações necessárias:
  ✅ Path existe
  ✅ Extensão permitida (.jpg, .jpeg, .mp4, .mov)
  ✅ Path não é malicioso (sanitização)

Para testar:
  open 'banlek://upload?files=/Users/user/foto.jpg'
```

**Quando usar:**
- Deep links não funcionam
- Menu de contexto não aparece
- Teste de integração macOS
- Debug de validação de paths

**Como testar manualmente:**
```bash
# Teste simples
open 'banlek://upload?files=/Users/$(whoami)/Desktop/foto.jpg'

# Com álbum específico
open 'banlek://upload?album=123&files=/path/to/foto.jpg'

# Múltiplos arquivos
open 'banlek://upload?files=/path/foto1.jpg,/path/foto2.jpg'
```

**Troubleshooting:**
- URL scheme não registrado? → Reinstalar app
- Path não validado? → Ver `useDeepLink.ts` linha de validação
- Arquivo não carregado? → Verificar extensão permitida

---

## debug:monitor

**Descrição:** Informações sobre debug do monitor de pastas

**Uso:**
```bash
.claude/commands.yaml debug:monitor
```

**Output:**
```
📁 Monitor de Pastas - Sincronização Automática

Componentes:
  - FolderMonitorService: src/services/folder-monitor.service.ts
  - UploadQueueService: src/services/upload-queue.service.ts
  - MonitorStore: src/store/monitorStore.ts

Configurações:
  - Intervalo: 5-60 segundos
  - Concorrência: 1-5 uploads simultâneos
  - Retry: até 5 tentativas (exponential backoff)
  - Persistência: fila salva em localStorage

Para debug:
  1. Verificar monitorStore.monitors
  2. Verificar uploadStore.queue
  3. Console logs: 📁 (monitor), 📤 (upload)
```

**Quando usar:**
- Monitor não detecta arquivos novos
- Upload automático não funciona
- Fila não persiste
- Configurações não salvam

**Debug checklist:**
1. **Monitor ativo?**
   ```typescript
   // No DevTools console
   monitorStore.getState().monitors
   // Deve ter item com isActive: true
   ```

2. **Intervalo correto?**
   ```typescript
   preferencesService.get('scanInterval')
   // Deve retornar 5-60 (segundos)
   ```

3. **Permissões de leitura?**
   - Tentar ler pasta manualmente
   - Verificar permissões macOS

4. **Fila de upload?**
   ```typescript
   uploadStore.getState().queue
   // Deve mostrar arquivos enfileirados
   ```

---

## repo:map

**Descrição:** Exibe estrutura do repositório

**Uso:**
```bash
.claude/commands.yaml repo:map
```

**Output:**
```
📂 Estrutura do Repositório - Banlek Uploader

=== Frontend (src/) ===
src/
├── components/
│   ├── album/
│   ├── upload/
│   └── ui/
├── services/
├── store/
└── types/

=== Backend Tauri (src-tauri/) ===
src-tauri/
├── src/
│   └── main.rs
└── tauri.conf.json

=== Documentação (docs/) ===
docs/
├── architecture/
├── api/
└── guides/

=== Configuração (.claude/) ===
.claude/
├── agents/
└── commands/
```

**Quando usar:**
- Entender estrutura do projeto
- Onboarding de novos devs
- Documentação
- Explorar codebase

---

## check:coverage-delta

**Descrição:** Compara cobertura de testes antes e depois de mudanças

**Uso:**
```bash
.claude/commands.yaml check:coverage-delta
```

**Workflow recomendado:**
```bash
# 1. Antes das mudanças - salvar baseline
pnpm test:coverage --reporter=json > /tmp/coverage-baseline.json

# 2. Fazer mudanças no código

# 3. Depois das mudanças - comparar
.claude/commands.yaml check:coverage-delta
```

**Output:**
```
📊 Verificando delta de cobertura...

Cobertura baseline: 100%
Cobertura atual: 98%
Delta: -2% ⚠️

Arquivos com cobertura reduzida:
  - src/services/upload.service.ts: 100% → 95%
  - src/store/uploadStore.ts: 100% → 98%
```

**Quando usar:**
- Após refatoração grande
- Validar que cobertura não caiu
- CI/CD para bloquear PRs com perda de cobertura

---

## check:dependencies

**Descrição:** Verifica dependências desatualizadas e vulnerabilidades

**Uso:**
```bash
.claude/commands.yaml check:dependencies
```

**Output:**
```
📦 Verificando dependências...

=== Dependências desatualizadas ===
Package          Current  Wanted  Latest
@tauri-apps/api  1.4.0    1.4.2   1.5.0
react            18.2.0   18.2.0  18.3.0

=== Vulnerabilidades conhecidas ===
found 0 vulnerabilities
```

**Quando usar:**
- Mensalmente (manutenção)
- Antes de release
- Auditoria de segurança
- Após report de vulnerabilidade

**Comandos relacionados:**
```bash
# Ver outdated
pnpm outdated

# Atualizar minor/patch
pnpm update

# Atualizar major (cuidado!)
pnpm update --latest

# Audit de segurança
pnpm audit

# Fix vulnerabilidades
pnpm audit --fix
```

---

## Dicas de Debug

### DevTools (Frontend)
```bash
# Abrir app em dev mode
pnpm tauri:dev

# Dentro do app: Cmd+Opt+I (macOS)
# Ver console logs, network, storage
```

### Logs Rust (Backend)
```bash
# Logs aparecem no terminal onde rodou tauri:dev
# Adicionar logs no código Rust:
println!("Debug: {:?}", value);
```

### Debug de Estado (Zustand)
```javascript
// No DevTools console
uploadStore.getState()
authStore.getState()
monitorStore.getState()
```

### Debug de API
```javascript
// Ver interceptor de axios
// src/services/api.ts

// Network tab no DevTools:
// - Verificar headers (Authorization)
// - Verificar payload (hashId vs id)
// - Verificar response
```

