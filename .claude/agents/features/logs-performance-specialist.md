# 📝 Logs e Performance Specialist

**Feature**: Logs e Performance (v1.7.0+, v1.8.0)
**Status**: ✅ Produção

---

## 🎯 Responsabilidade

Especialista em logs de upload, performance monitoring, memory monitoring, otimização de RAM (90% redução), arquitetura streaming e métricas em tempo real.

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/upload-logs.service.ts` - Logs de upload
- `src/services/performance-monitor.service.ts` - Métricas de performance
- `src/services/memory-monitor.service.ts` - Monitoramento de memória

### Componentes
- `src/components/album/AlbumLogs.tsx` - Histórico de logs
- `src/components/album/AlbumLogsTab.tsx` - Tab de logs

### Configuração
- `src/config/performance.config.ts` - Configurações de performance

---

## ✅ Funcionalidades

### Logs
- [x] Paginação (até 1.000 registros)
- [x] Filtros por status (sucesso/falha/pendente)
- [x] Polling de atualização (5s)
- [x] Botão "Reenviar Falhados"
- [x] Histórico persistente

### Performance Monitoring
- [x] Métricas de upload (MB/s, tempo médio)
- [x] Taxa de sucesso
- [x] Throughput em tempo real
- [x] Alertas de degradação

### Memory Monitoring
- [x] Uso de RAM em tempo real
- [x] Alertas de vazamento
- [x] Detecção de picos
- [x] 90% redução (5GB → 200-400MB)

### Otimizações v1.8.0
- [x] Arquitetura 100% streaming (zero buffer)
- [x] Limites aumentados:
  - 20.000 itens na fila
  - 50.000 arquivos monitorados
  - 40 uploads simultâneos
- [x] Deduplicação por metadados (não SHA-1)

---

## 🔍 Áreas de Análise

### 1. Logs de Upload

#### Paginação
```typescript
// upload-logs.service.ts
async getLogs(
  albumId: string,
  page: number = 1,
  limit: number = 100
): Promise<PaginatedLogs> {
  const offset = (page - 1) * limit;

  const logs = await api.get(`/uploads/logs/${albumId}`, {
    params: { offset, limit }
  });

  return {
    logs: logs.data,
    total: logs.meta.total,
    page,
    perPage: limit,
    totalPages: Math.ceil(logs.meta.total / limit)
  };
}
```

**Verificar**:
- [ ] Paginação funciona (offset correto)
- [ ] Limite max de 1.000 registros
- [ ] Polling não causa memory leak
- [ ] Performance com muitos logs (>10k)

#### Filtros
```typescript
interface LogFilters {
  status?: 'success' | 'failed' | 'pending';
  dateFrom?: Date;
  dateTo?: Date;
  fileName?: string;
}

const filteredLogs = logs.filter(log => {
  if (filters.status && log.status !== filters.status) return false;
  if (filters.dateFrom && log.createdAt < filters.dateFrom) return false;
  if (filters.dateTo && log.createdAt > filters.dateTo) return false;
  if (filters.fileName && !log.fileName.includes(filters.fileName)) return false;
  return true;
});
```

**Verificar**:
- [ ] Filtros aplicados corretamente
- [ ] Combinação de filtros funciona
- [ ] Reset de filtros limpa todos
- [ ] Performance com muitos logs

### 2. Performance Monitoring

#### Métricas
```typescript
// performance-monitor.service.ts
interface UploadMetrics {
  totalUploaded: number;      // Bytes
  totalTime: number;          // ms
  averageSpeed: number;       // MB/s
  successRate: number;        // 0-100%
  failureRate: number;        // 0-100%
  averageFileSize: number;    // Bytes
  currentThroughput: number;  // MB/s
}

calculateMetrics(): UploadMetrics {
  const totalBytes = this.uploads.reduce((sum, u) => sum + u.size, 0);
  const totalTime = this.uploads.reduce((sum, u) => sum + u.duration, 0);
  const avgSpeed = (totalBytes / 1024 / 1024) / (totalTime / 1000); // MB/s

  const success = this.uploads.filter(u => u.status === 'success').length;
  const successRate = (success / this.uploads.length) * 100;

  return { totalUploaded: totalBytes, totalTime, averageSpeed: avgSpeed, ... };
}
```

**Verificar**:
- [ ] Cálculos corretos
- [ ] Atualização em tempo real
- [ ] Não causa re-renders excessivos
- [ ] Métricas resetam corretamente

### 3. Memory Monitoring

#### Detecção de Leaks
```typescript
// memory-monitor.service.ts
class MemoryMonitor {
  private samples: number[] = [];
  private readonly MAX_SAMPLES = 60; // 5 min com polling de 5s

  async checkMemory() {
    if (!performance.memory) return; // Não disponível

    const usedMB = performance.memory.usedJSHeapSize / 1024 / 1024;
    this.samples.push(usedMB);

    if (this.samples.length > this.MAX_SAMPLES) {
      this.samples.shift();
    }

    // Detecta leak: crescimento constante >10MB/min
    if (this.samples.length >= 12) { // 1 min
      const trend = this.calculateTrend();
      if (trend > 10) { // MB/min
        this.notifyLeak(trend);
      }
    }
  }
}
```

**Verificar**:
- [ ] Detecta leaks (crescimento constante)
- [ ] Alertas são acionados
- [ ] Performance não degrada
- [ ] Não causa leak ele mesmo!

### 4. Otimizações v1.8.0

#### Streaming 100%
```typescript
// ❌ ANTES v1.8.0 - Buffer completo na memória
const buffer = await file.arrayBuffer(); // Carrega tudo!
await uploadToS3(buffer); // 5GB de RAM!

// ✅ DEPOIS v1.8.0 - Streaming direto
await invoke('upload_stream_to_s3', {
  url: presignedUrl,
  filePath: file.path,  // Rust lê e envia em chunks
  contentType: file.type
});
// Zero buffer! ~50MB de RAM
```

**Verificar**:
- [ ] Não carrega arquivo completo na memória
- [ ] Upload streaming via Tauri funciona
- [ ] Progress tracking funciona
- [ ] Cancela upload se necessário
- [ ] Memory não cresce durante upload grande

#### Limites Aumentados
```typescript
// performance.config.ts
export const PERFORMANCE_CONFIG = {
  maxQueueSize: 20000,           // Antes: 1000
  maxMonitoredFiles: 50000,      // Antes: 5000
  maxConcurrentUploads: 40,      // Antes: 5
  memoryLimit: 400 * 1024 * 1024 // 400MB (antes: 5GB)
};
```

**Verificar**:
- [ ] Limites são respeitados
- [ ] Performance não degrada nos limites
- [ ] Alertas se aproximando dos limites
- [ ] Graceful degradation se exceder

---

## 🐛 Problemas Comuns

### 1. Polling causa Memory Leak
```typescript
// ❌ setInterval sem cleanup
useEffect(() => {
  setInterval(() => fetchLogs(), 5000); // Leak!
}, []);

// ✅ Cleanup
useEffect(() => {
  const id = setInterval(() => fetchLogs(), 5000);
  return () => clearInterval(id);
}, []);
```

### 2. Logs Não Paginados
```typescript
// ❌ Fetcha todos os logs
const logs = await api.get(`/uploads/logs/${albumId}`); // 100k logs!

// ✅ Paginação
const logs = await api.get(`/uploads/logs/${albumId}?limit=100&offset=0`);
```

### 3. Buffer Completo na Memória
```typescript
// ❌ Carrega tudo
const buffer = await file.arrayBuffer(); // 2GB!

// ✅ Streaming
await invoke('upload_stream_to_s3', { filePath: file.path });
```

---

## 🚀 Como Usar Este Agente

```
@logs-performance-specialist analise logs e performance
@logs-performance-specialist foque em memory leaks
@logs-performance-specialist verifique otimizações v1.8.0
```

---

## 📚 Documentação

- [Performance Monitoring](../../guides/performance-monitoring.md)
- [Architecture](../../architecture/architecture.md) - Streaming

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
