# 📊 Sistema de Filas Specialist

**Feature**: Sistema de Filas (v1.9.12+)
**Status**: ✅ Produção (Refatoração Crítica)
**Testes**: 45+ testes

---

## 🎯 Responsabilidade

Especialista no sistema de filas centralizado (Queue Manager), múltiplas filas isoladas, processamento automático, validação e deduplicação centralizadas. **BREAKING CHANGES em v1.9.12**.

---

## 📁 Arquivos Relacionados

### Serviços (Refatorados v1.9.12)
- `src/services/queue-manager.service.ts` - ⭐ FONTE ÚNICA DA VERDADE
- `src/services/upload-queue.service.ts` - Processamento de fila
- `src/services/indexed-db.service.ts` - Persistência

### Stores
- `src/store/uploadStore.ts` - Wrapper fino para queueManager

### Tipos
- `src/types/queue.types.ts` - QueueItem, QueueStats, QueueTarget

### Configuração
- `src/config/performance.config.ts` - Limites de fila

### Testes
- `tests/services/queue-manager.service.test.ts` - 30+ testes
- `tests/integration/queue-flow.test.ts` - 15 casos

---

## ⚠️ BREAKING CHANGES v1.9.12

### Antes (v1.9.11)
```typescript
// ❌ REMOVIDO - uploadStore tinha estado próprio
const { queue, addFiles, startUpload } = useUploadStore();

// Items na store
const items = queue;  // ❌ Removido

// Adicionar e iniciar manualmente
addFiles(files);
startUpload();  // ❌ Removido
```

### Depois (v1.9.12+)
```typescript
// ✅ CORRETO - queueManager é fonte única
import { queueManager } from '@/services/queue-manager.service';

// Obter fila de um álbum
const queue = queueManager.getQueue({ type: 'album', id: albumId });
const items = queue.getQueue();

// Adicionar arquivos (processamento automático)
await queueManager.addFiles(
  { type: 'album', id: albumId },
  files,
  albumName,
  folderId,
  'manual'
);
// Não precisa de startUpload()! Processamento é automático.
```

---

## ✅ Funcionalidades Implementadas

### Queue Manager Centralizado (v1.9.12)
- [x] Fonte única da verdade (Map isolado por álbum/evento)
- [x] Múltiplas filas simultâneas
- [x] Processamento automático (sem `startUpload` manual)
- [x] Validação centralizada (antes de adicionar)
- [x] Deduplicação centralizada (nome + tamanho)
- [x] Redução de código: 54% (1241 → 566 linhas)

### Persistência
- [x] Fila persiste em IndexedDB
- [x] Restaura fila ao reabrir app
- [x] Limpeza automática (filas antigas >30 dias)
- [x] Suporta até 20.000 itens por fila

### Operações
- [x] `addFiles()` - Adiciona com validação
- [x] `remove()` - Remove item
- [x] `retry()` - Marca para retry
- [x] `clearDone()` - Limpa concluídos
- [x] `clearFailed()` - Limpa falhados
- [x] `clearPending()` - Limpa pendentes
- [x] `getStats()` - Estatísticas da fila

---

## 🔍 Áreas de Análise

### 1. Queue Manager API

#### addFiles - Validação Centralizada
```typescript
// src/services/queue-manager.service.ts
async addFiles(
  target: QueueTarget,
  files: File[],
  albumName: string,
  folderId?: number,
  source: 'manual' | 'monitor' = 'manual'
): Promise<AddFilesResult> {
  // 1. Validação
  const validFiles = await this.validateFiles(files);

  // 2. Deduplicação
  const newFiles = await this.deduplicateFiles(validFiles, target);

  // 3. Adicionar à fila
  const queue = this.getOrCreateQueue(target);
  queue.addItems(newFiles);

  // 4. Persistir
  await this.persistQueue(target);

  // 5. Processar automaticamente
  this.processQueue(target);

  return { added: newFiles.length, skipped: files.length - newFiles.length };
}
```

**Verificar**:
- [ ] Validação antes de adicionar (não depois)
- [ ] Deduplicação funciona
- [ ] Processamento automático inicia
- [ ] Persistência acontece
- [ ] Retorna resultado correto

#### getQueue - Obter Fila Específica
```typescript
// Obter fila de um álbum/evento específico
const queue = queueManager.getQueue({ type: 'album', id: albumId });

// Métodos da fila:
queue.getQueue();        // QueueItem[]
queue.getStats();        // QueueStats
queue.remove(itemId);    // Remove item
queue.retry(itemId);     // Marca para retry
queue.clearDone();       // Limpa concluídos
queue.clearFailed();     // Limpa falhados
queue.clearPending();    // Limpa pendentes
```

**Verificar**:
- [ ] `getQueue()` retorna itens corretos
- [ ] `getStats()` calcula corretamente
- [ ] `remove()` remove item e persiste
- [ ] `retry()` marca e reinicia
- [ ] `clear*()` limpam corretamente

### 2. Múltiplas Filas Isoladas

#### Map Isolado
```typescript
// Cada álbum/evento tem fila separada
private queues: Map<string, UploadQueue> = new Map();

// Key: `album:${albumId}` ou `event:${eventId}`
private getQueueKey(target: QueueTarget): string {
  return `${target.type}:${target.id}`;
}
```

**Verificar**:
- [ ] Filas são isoladas (não compartilham itens)
- [ ] Key é única por álbum/evento
- [ ] Múltiplas filas podem processar simultaneamente
- [ ] Não há race condition entre filas

### 3. Processamento Automático

#### Auto-start após addFiles
```typescript
// queue-manager.service.ts
async addFiles(...) {
  // ... validação e adição

  // Processamento automático (sem necessidade de startUpload)
  this.processQueue(target);
}

private async processQueue(target: QueueTarget) {
  const queue = this.getQueue(target);

  while (queue.hasPending() && !queue.isPaused()) {
    const item = queue.getNextPending();
    await this.uploadItem(item, target);
  }
}
```

**Verificar**:
- [ ] Processamento inicia automaticamente
- [ ] Não precisa chamar `startUpload()` manualmente
- [ ] Respeita concorrência configurada
- [ ] Para se fila for pausada
- [ ] Retry automático em falhas

### 4. Validação Centralizada

#### Antes de Adicionar
```typescript
// Valida ANTES de adicionar à fila
private async validateFiles(files: File[]): Promise<File[]> {
  const validFiles: File[] = [];

  for (const file of files) {
    const validation = await this.validateFile(file);
    if (validation.valid) {
      validFiles.push(file);
    } else {
      // Notifica usuário do erro
      this.notifyValidationError(file, validation.error);
    }
  }

  return validFiles;
}
```

**Verificar**:
- [ ] Validação acontece ANTES de adicionar
- [ ] Arquivos inválidos não entram na fila
- [ ] Notifica usuário de erros
- [ ] Não bloqueia UI durante validação

### 5. Deduplicação Centralizada

#### Por Nome + Tamanho
```typescript
// Deduplica baseado em metadados (não SHA-1)
private async deduplicateFiles(
  files: File[],
  target: QueueTarget
): Promise<File[]> {
  const queue = this.getQueue(target);
  const existing = queue.getQueue();

  return files.filter(file => {
    return !existing.some(item =>
      item.name === file.name && item.size === file.size
    );
  });
}
```

**Verificar**:
- [ ] Deduplica por nome + tamanho (não SHA-1)
- [ ] Performance não degrada com muitos itens
- [ ] Permite forçar reenvio se necessário
- [ ] Funciona para fotos e vídeos

### 6. uploadStore como Wrapper

#### Wrapper Fino
```typescript
// src/store/uploadStore.ts
// uploadStore agora é apenas wrapper para queueManager

export const useUploadStore = create<UploadStore>((set, get) => ({
  // Estado do target atual
  currentTarget: null,
  currentId: null,
  currentType: null,

  // Wrappers para queueManager
  addFiles: async (files, folderId, source) => {
    const { currentTarget, currentId, currentType } = get();
    if (!currentTarget || !currentId) {
      throw new Error('Álbum/evento não configurado');
    }

    return await queueManager.addFiles(
      { type: currentType, id: currentId },
      files,
      currentTarget.nome || currentTarget.titulo,
      folderId,
      source
    );
  },

  removeFile: (itemId) => {
    const { currentId, currentType } = get();
    const queue = queueManager.getQueue({ type: currentType, id: currentId });
    queue.remove(itemId);
  },

  // ... outros wrappers
}));
```

**Verificar**:
- [ ] uploadStore NÃO mantém estado da fila
- [ ] Apenas delega para queueManager
- [ ] currentTarget é mantido (compatibilidade)
- [ ] Wrappers cobrem operações principais

---

## 🐛 Problemas Comuns

### 1. Usar uploadStore.queue
```typescript
// ❌ ERRO - queue não existe mais
const { queue } = useUploadStore();
const items = queue;  // undefined!

// ✅ CORRETO - usar queueManager
const items = queueManager.getQueueItems({ type: 'album', id: albumId });
```

### 2. Chamar startUpload Manualmente
```typescript
// ❌ ERRO - startUpload removido
await addFiles(files);
startUpload();  // Não existe mais!

// ✅ CORRETO - processamento automático
await addFiles(files);
// Processamento já começou automaticamente
```

### 3. Validação Após Adicionar
```typescript
// ❌ Errado - valida depois
addToQueue(file);
if (!isValid(file)) removeFromQueue(file);

// ✅ Correto - queueManager valida antes
await queueManager.addFiles(...);  // Validação já aconteceu
```

### 4. Múltiplas Chamadas addFiles
```typescript
// ❌ Pode causar duplicados
await queueManager.addFiles(target, files1);
await queueManager.addFiles(target, files2);  // Reprocessa fila!

// ✅ Batch files antes
await queueManager.addFiles(target, [...files1, ...files2]);
```

---

## 📊 Métricas de Qualidade

### Performance
- **Queue size**: Até 20.000 itens
- **Add files**: <50ms para 100 arquivos
- **Get stats**: <10ms
- **Memory**: ~10MB para 10.000 itens

### Confiabilidade
- **Persistence**: 100% (IndexedDB)
- **Deduplication accuracy**: >99.9%
- **Processing start**: <100ms após addFiles

### Code Quality
- **Redução de código**: 54% (1241 → 566 linhas)
- **Centralização**: 1 fonte da verdade (queueManager)
- **Breaking changes**: Documentados completamente

---

## 🚀 Como Usar Este Agente

### Análise Completa
```
@sistema-filas-specialist analise o sistema de filas
```

### Migração para v1.9.12
```
@sistema-filas-specialist verifique migração:
- uploadStore.queue removido?
- startUpload removido?
- Usando queueManager diretamente?
```

### Code Review de PR
```
@sistema-filas-specialist revise mudanças no sistema de filas
```

---

## 📚 Documentação Relacionada

- [Queue Manager API](../../api/queue-manager-api.md) - API completa
- [Refactoring v1.9.12](../../architecture/refactoring-v1.9.12.md) - Breaking changes
- [Migration Guide](../../guides/MIGRATION-GUIDE-v1.9.12.md) - Guia de migração

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
