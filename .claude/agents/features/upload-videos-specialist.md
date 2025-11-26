# 🎥 Upload de Vídeos Specialist

**Feature**: Upload de Vídeos (v1.5.0+) + Multipart (v1.6.0+)
**Status**: ✅ Produção
**Testes**: 235+ testes (100% cobertura)

---

## 🎯 Responsabilidade

Especialista em upload de vídeos incluindo validação, multipart upload para arquivos >100MB, extração de duração, processamento assíncrono e hierarquia de pastas separada (tipo: 2).

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/upload.service.ts` - Upload de vídeos
- `src/services/multipart-upload.service.ts` - Upload multipart (chunks de 5MB)
- `src/services/album.service.ts` - getVideos, deleteVideo
- `src/services/evento.service.ts` - getVideos, deleteVideo

### Componentes
- `src/components/album/MediaGallery.tsx` - Galeria unificada
- `src/components/album/MediaItem.tsx` - Item com badge de vídeo
- `src/components/monitor/FolderPickerModal.tsx` - Suporte a tipo: 2

### Tipos
- `src/types/video.types.ts` - VideoEntity, VideoMetadata
- `src/types/upload.types.ts` - MultipartUploadConfig

### Configuração
- `src/config/validation.config.ts` - Regras para vídeos

### Testes
- `tests/services/multipart-upload.service.test.ts` - 12 testes
- `tests/components/MediaGallery.test.tsx` - 15 testes
- `tests/integration/video-upload-flow.test.ts` - 20 casos

---

## ✅ Funcionalidades Implementadas

### Upload Básico (<100MB)
- [x] Suporte a MP4, MOV
- [x] Validação (formato, tamanho 100KB-2GB, duração)
- [x] Extração automática de duração (Tauri `get_video_duration`)
- [x] Upload streaming para S3
- [x] Hierarquia de pastas (tipo: 2)

### Upload Multipart (>100MB)
- [x] Chunks de 5MB
- [x] 3 uploads de chunks simultâneos
- [x] Retry granular por chunk (Fibonacci backoff)
- [x] Progress unificado (0-100)
- [x] Finalização automática (CompleteMultipartUpload)

### Galeria
- [x] Tab "Vídeos" em MediaGallery
- [x] Badge de tipo (Vídeo)
- [x] Indicador de processamento
- [x] Duração formatada (MM:SS)
- [x] Exclusão de vídeos

---

## 🔍 Áreas de Análise

### 1. Validação de Vídeos

#### Regras de Validação
```typescript
// src/config/validation.config.ts
export const VIDEO_CONFIG = {
  minSize: 102400,          // 100KB
  maxSize: 2147483648,      // 2GB
  maxDuration: 7200,        // 2 horas
  allowedExtensions: [
    '.mp4', '.mov'
  ],
  allowedMimeTypes: [
    'video/mp4', 'video/quicktime'
  ]
};
```

**Checklist**:
- [ ] Validação de formato
- [ ] Validação de tamanho (100KB - 2GB)
- [ ] Validação de duração (<2h)
- [ ] Extração de duração funciona
- [ ] Mensagens de erro claras

### 2. Multipart Upload

#### Detecção Automática
```typescript
// Multipart se >100MB
const MULTIPART_THRESHOLD = 100 * 1024 * 1024; // 100MB

if (file.size > MULTIPART_THRESHOLD) {
  return await this.uploadVideoMultipart(file, albumId, folderId);
} else {
  return await this.uploadVideoSimple(file, albumId, folderId);
}
```

**Verificar**:
- [ ] Threshold correto (100MB)
- [ ] Decisão automática (sem input do usuário)
- [ ] Fallback para simple se multipart falhar

#### Chunks de 5MB
```typescript
// src/services/multipart-upload.service.ts
const CHUNK_SIZE = 5 * 1024 * 1024; // 5MB
const MAX_CONCURRENT_CHUNKS = 3;

async uploadChunk(chunkData: ArrayBuffer, partNumber: number) {
  // Upload com retry
  const uploadUrl = await this.getChunkUploadUrl(uploadId, partNumber);
  await fetch(uploadUrl, {
    method: 'PUT',
    body: chunkData
  });
}
```

**Verificar**:
- [ ] Chunks de exatamente 5MB (exceto último)
- [ ] 3 chunks simultâneos (não mais)
- [ ] Retry granular por chunk (até 5 tentativas)
- [ ] ETags são coletados para finalização
- [ ] Finalização automática após todos os chunks

#### Progress Unificado
```typescript
// Progress 0-100 unificado
const totalProgress = uploadedChunks / totalChunks * 100;
```

**Verificar**:
- [ ] Progress 0-100 (não >100)
- [ ] Atualiza durante upload de chunks
- [ ] Progress não "pula" (smooth)
- [ ] Mostra velocidade (MB/s) se possível

### 3. Extração de Duração

#### Comando Tauri
```typescript
// src-tauri/src/main.rs
#[tauri::command]
async fn get_video_duration(file_path: String) -> Result<f64, String> {
  // Usa ffprobe ou similar
  // Retorna duração em segundos
}
```

**Verificar**:
- [ ] Extrai duração antes de upload
- [ ] Funciona para todos os formatos suportados
- [ ] Erro claro se vídeo está corrompido
- [ ] Não bloqueia UI (async)

### 4. Hierarquia de Pastas (tipo: 2)

#### Separação de Pastas
```typescript
// Pastas de vídeos têm tipo: 2
interface Pasta {
  id: number;
  nome: string;
  albumId: string;
  tipo: 2;  // Vídeos
}
```

**Verificar**:
- [ ] Cria pastas com `tipo: 2`
- [ ] FolderPickerModal filtra por tipo
- [ ] Não mistura fotos e vídeos
- [ ] Backend registra corretamente

### 5. Galeria Unificada

#### MediaGallery com Tabs
```typescript
// src/components/album/MediaGallery.tsx
<Tabs defaultValue="todos">
  <TabsList>
    <TabsTrigger value="todos">Todas ({total})</TabsTrigger>
    <TabsTrigger value="fotos">Fotos ({photoCount})</TabsTrigger>
    <TabsTrigger value="videos">Vídeos ({videoCount})</TabsTrigger>
  </TabsList>
</Tabs>
```

**Verificar**:
- [ ] Tab "Vídeos" funciona
- [ ] Contagem correta de vídeos
- [ ] Badge "Vídeo" visível
- [ ] Ícone de play overlay
- [ ] Duração formatada (MM:SS)
- [ ] Exclusão de vídeos funciona

---

## 🐛 Problemas Comuns

### 1. Multipart Não Ativado
```typescript
// ❌ Upload simples para arquivo >100MB
await uploadVideoSimple(largeFile); // Vai falhar ou demorar muito

// ✅ Verifica tamanho e usa multipart
if (file.size > MULTIPART_THRESHOLD) {
  await uploadVideoMultipart(largeFile);
}
```

### 2. Progress >100%
```typescript
// ❌ Chunks podem fazer progress ultrapassar 100
const progress = (uploadedChunks / totalChunks) * 100; // >100?

// ✅ Limita a 100
const progress = Math.min((uploadedChunks / totalChunks) * 100, 100);
```

### 3. Retry de Chunk Não Funciona
```typescript
// ❌ Retry do upload completo ao invés de chunk
if (chunkFails) {
  retryEntireUpload(); // Reinicia tudo!
}

// ✅ Retry apenas do chunk falhado
if (chunkFails) {
  retryChunk(partNumber); // Só reupload deste chunk
}
```

### 4. ETags Não Coletados
```typescript
// ❌ Finalização sem ETags
await completeMultipartUpload(uploadId); // Falha

// ✅ Coleta ETags de cada chunk
const etags = [];
for (const chunk of chunks) {
  const response = await uploadChunk(chunk);
  etags.push({ ETag: response.headers['etag'], PartNumber: chunk.partNumber });
}
await completeMultipartUpload(uploadId, etags);
```

---

## 📊 Métricas de Qualidade

### Performance
- **Upload speed**: 3-8MB/s (multipart)
- **Chunk upload time**: 1-3s por chunk (5MB)
- **Concurrent chunks**: 3 (balanceado)
- **Memory**: <100MB para 1 vídeo de 2GB

### Confiabilidade
- **Success rate (multipart)**: >98%
- **Chunk retry rate**: <3%
- **Fallback to simple**: Raro (<1%)

---

## 🚀 Como Usar Este Agente

### Análise Completa
```
@upload-videos-specialist analise a feature de upload de vídeos
```

### Foco em Multipart
```
@upload-videos-specialist foque em multipart:
- Chunks de 5MB?
- Retry granular?
- Finalização automática?
```

### Foco em Validação
```
@upload-videos-specialist verifique validação de vídeos:
- Duração é extraída?
- Formatos suportados?
```

---

## 📚 Documentação Relacionada

- [Video Upload](../../features/video-upload.md) - Guia completo
- [Multipart Upload](../../architecture/multipart-upload.md) - Arquitetura técnica
- [Backend API](../../api/backend-api.md) - Endpoints de multipart

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
