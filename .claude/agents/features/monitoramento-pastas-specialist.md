# 📁 Monitoramento de Pastas Specialist

**Feature**: Monitoramento de Pastas (v1.4.0+)
**Status**: ✅ Produção
**Testes**: 16 casos de teste

---

## 🎯 Responsabilidade

Especialista em monitoramento automático de pastas, detecção de novos arquivos em tempo real, scan recursivo, fila persistente, retry automático e otimizações (Fibonacci adaptativo, deduplicação).

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/folder-monitor.service.ts` - Serviço principal
- `src/services/upload-queue.service.ts` - Gestão de fila
- `src/services/indexed-db.service.ts` - Persistência
- `src/services/sent-files-registry.service.ts` - Registro de enviados

### Componentes
- `src/components/monitor/MonitorControls.tsx` - Controles (start/stop/pause)
- `src/components/monitor/MonitorStats.tsx` - Estatísticas em tempo real
- `src/components/monitor/AlbumPickerModal.tsx` - Seleção de álbum/evento
- `src/components/monitor/FolderPickerModal.tsx` - Seleção e criação de pastas

### Stores
- `src/store/monitorStore.ts` - Estado de monitoramento (847 linhas)

### Tipos
- `src/types/monitor.types.ts` - MonitorConfig, MonitorStats

### Config
- `src/config/performance.config.ts` - Limites e otimizações

### Testes
- `tests/services/folder-monitor.service.test.ts` - 16 casos

---

## ✅ Funcionalidades Implementadas

### Monitoramento
- [x] Detecção em tempo real (intervalo 5-60s)
- [x] Scan recursivo em subdiretórios
- [x] Fibonacci adaptativo (otimiza scan)
- [x] Suporta até 50.000 arquivos monitorados
- [x] Pausa/Resume sem perder estado

### Fila e Upload
- [x] Fila persistente (IndexedDB)
- [x] Retry automático (até 5 tentativas)
- [x] Backoff exponencial Fibonacci
- [x] Concorrência configurável (1-40, padrão: 20)
- [x] Prioridade (novos arquivos primeiro)

### Deduplicação
- [x] Baseada em nome + tamanho (não SHA-1)
- [x] Registro de arquivos já enviados
- [x] Evita reenvio em rescans
- [x] Limpeza periódica de registros antigos

### Performance
- [x] Não bloqueia UI durante scan
- [x] Batch processing de novos arquivos
- [x] Memory limit (50.000 arquivos)
- [x] Throttling de eventos

---

## 🔍 Áreas de Análise

### 1. Configuração de Monitoramento

#### MonitorConfig
```typescript
interface MonitorConfig {
  folderPath: string;       // Pasta a monitorar
  albumId: string;          // ⚠️ STRING (crítico)
  albumName: string;
  folderId?: number;        // Pasta de destino
  interval: number;         // 5000-60000ms
  recursive: boolean;       // Scan subdiretórios
  mediaType: 'photo' | 'video' | 'both';
}
```

**Verificar**:
- [ ] `albumId` é sempre string
- [ ] Intervalo está entre 5-60s
- [ ] `folderPath` existe e é acessível
- [ ] Permissões de leitura na pasta
- [ ] mediaType filtra corretamente

### 2. Detecção de Arquivos

#### Scan com Fibonacci Adaptativo
```typescript
// folder-monitor.service.ts
async scanFolder() {
  const files = await this.listFilesRecursive(this.config.folderPath);

  // Fibonacci adaptativo:
  // - Poucos arquivos: scan rápido
  // - Muitos arquivos: scan mais espaçado
  const delay = this.calculateFibonacciDelay(files.length);
  this.scheduleNextScan(delay);
}
```

**Verificar**:
- [ ] Scan recursivo funciona
- [ ] Detecta novos arquivos
- [ ] Não re-adiciona arquivos já enviados
- [ ] Fibonacci adaptativo otimiza performance
- [ ] Não causa lag na UI

#### Filtros
```typescript
// Filtra por tipo de mídia
const filteredFiles = files.filter(file => {
  if (mediaType === 'photo') return isPhoto(file);
  if (mediaType === 'video') return isVideo(file);
  return isPhoto(file) || isVideo(file); // both
});
```

**Verificar**:
- [ ] Filtra corretamente por tipo
- [ ] Ignora arquivos não suportados
- [ ] Ignora arquivos do sistema (.DS_Store, Thumbs.db)
- [ ] Valida antes de adicionar à fila

### 3. Fila Persistente

#### IndexedDB Persistence
```typescript
// indexed-db.service.ts
async saveMonitorQueue(albumId: string, items: QueueItem[]) {
  await db.monitorQueues.put({
    albumId,
    items,
    updatedAt: Date.now()
  });
}
```

**Verificar**:
- [ ] Fila persiste após fechar app
- [ ] Restaura fila ao reabrir
- [ ] Limpeza de filas antigas (>30 dias)
- [ ] Não estoura limites de storage

### 4. Retry Automático

#### Backoff Fibonacci
```typescript
// Tentativas: 1s, 1s, 2s, 3s, 5s
const delays = [1, 1, 2, 3, 5]; // Segundos

async retryUpload(item: QueueItem) {
  if (item.attempts >= 5) {
    this.markAsFailed(item);
    return;
  }

  const delay = delays[item.attempts] * 1000;
  await sleep(delay);

  await this.uploadFile(item);
}
```

**Verificar**:
- [ ] Retry até 5 tentativas
- [ ] Delays corretos (Fibonacci)
- [ ] Para em erros irrecuperáveis (401, 403, 404)
- [ ] Marca como failed após 5 tentativas
- [ ] Notifica usuário de falhas

### 5. Deduplicação

#### Registro de Arquivos Enviados
```typescript
// sent-files-registry.service.ts
interface SentFile {
  name: string;
  size: number;
  albumId: string;
  sentAt: number;
}

async isDuplicate(file: File, albumId: string): Promise<boolean> {
  const sent = await this.getSentFiles(albumId);
  return sent.some(f => f.name === file.name && f.size === file.size);
}
```

**Verificar**:
- [ ] Deduplica por nome + tamanho (não SHA-1)
- [ ] Registro persiste entre sessões
- [ ] Limpeza de registros antigos (>90 dias)
- [ ] Performance não degrada com muitos registros
- [ ] Permite forçar reenvio se necessário

### 6. Concorrência

#### Upload Simultâneo
```typescript
// performance.config.ts
export const QUEUE_CONFIG = {
  maxConcurrentUploads: 20,    // Padrão
  minConcurrentUploads: 1,
  maxConcurrentUploads: 40,
  maxQueueSize: 20000
};
```

**Verificar**:
- [ ] Concorrência entre 1-40
- [ ] Configurável pelo usuário
- [ ] Não sobrecarrega rede/CPU
- [ ] Degrada gracefully se muitos arquivos

### 7. UI e Estatísticas

#### MonitorStats
```typescript
// monitor/MonitorStats.tsx
interface Stats {
  total: number;
  pending: number;
  uploading: number;
  done: number;
  failed: number;
  progress: number;  // 0-100
}
```

**Verificar**:
- [ ] Estatísticas em tempo real
- [ ] Progress 0-100 correto
- [ ] Contadores precisos
- [ ] Atualização não causa re-renders excessivos
- [ ] UI não trava durante uploads

---

## 🐛 Problemas Comuns

### 1. albumId é Número
```typescript
// ❌ ERRO CRÍTICO
const config = {
  albumId: 123  // Número!
};

// ✅ CORRETO
const config = {
  albumId: "G27ec858..."  // String
};
```

### 2. Scan Bloqueia UI
```typescript
// ❌ Scan síncrono bloqueia
const files = fs.readdirSync(folder); // Bloqueia!

// ✅ Scan assíncrono
const files = await fs.promises.readdir(folder);
```

### 3. Memory Leak em Long Polling
```typescript
// ❌ setInterval sem cleanup
setInterval(() => scan(), 5000);

// ✅ clearInterval no cleanup
useEffect(() => {
  const id = setInterval(() => scan(), 5000);
  return () => clearInterval(id);
}, []);
```

### 4. Re-adiciona Arquivos Já Enviados
```typescript
// ❌ Sem deduplicação
allFiles.forEach(f => addToQueue(f));

// ✅ Deduplica primeiro
const newFiles = allFiles.filter(f => !isDuplicate(f));
newFiles.forEach(f => addToQueue(f));
```

---

## 📊 Métricas de Qualidade

### Performance
- **Scan time**: <2s para 1.000 arquivos
- **Max monitored files**: 50.000
- **Memory**: <200MB para 20.000 itens na fila
- **CPU**: <5% durante scan
- **UI responsiveness**: Sem lag perceptível

### Confiabilidade
- **Detection rate**: 100% dos novos arquivos
- **False positives**: <0.1% (arquivos já enviados)
- **Retry success**: >95% após 5 tentativas

---

## 🚀 Como Usar Este Agente

### Análise Completa
```
@monitoramento-pastas-specialist analise a feature de monitoramento
```

### Foco em Performance
```
@monitoramento-pastas-specialist analise performance:
- Fibonacci adaptativo funciona?
- Scan não bloqueia UI?
- Memory está controlada?
```

### Foco em Deduplicação
```
@monitoramento-pastas-specialist verifique deduplicação:
- Registro de enviados funciona?
- Limpeza automática acontece?
- Performance com muitos registros?
```

---

## 📚 Documentação Relacionada

- [Folder Monitor](../../features/folder-monitor.md) - Guia completo
- [Performance Config](../../config/performance.config.ts) - Configurações
- [Queue Manager](../../api/queue-manager-api.md) - Sistema de filas

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
