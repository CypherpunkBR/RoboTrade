# 📸 Upload de Fotos Specialist

**Feature**: Upload de Fotos (v1.0.0+)
**Status**: ✅ Produção
**Testes**: 81 testes (100% cobertura)

---

## 🎯 Responsabilidade

Especialista em upload de fotos para álbuns e eventos Banlek, incluindo validação, preview, organização em pastas, progress tracking e integração com S3.

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/upload.service.ts` - Serviço principal de upload
- `src/services/album.service.ts` - Gestão de álbuns
- `src/services/evento.service.ts` - Gestão de eventos
- `src/services/indexed-db.service.ts` - Persistência local
- `src/services/api.ts` - Cliente API REST

### Componentes
- `src/components/upload/DropZone.tsx` - Drag and drop de fotos
- `src/components/upload/FileItem.tsx` - Item individual de foto
- `src/components/upload/FileList.tsx` - Lista de fotos
- `src/components/upload/UploadStats.tsx` - Estatísticas de upload
- `src/components/album/PhotoGallery.tsx` - Galeria de fotos

### Tipos
- `src/types/photo.types.ts` - Tipos de foto (PhotoEntity, etc)
- `src/types/upload.types.ts` - Tipos de upload
- `src/types/api.types.ts` - Tipos da API

### Configuração
- `src/config/validation.config.ts` - Regras de validação
- `src/config/performance.config.ts` - Configurações de performance

### Testes
- `tests/services/upload.service.test.ts` - 45+ testes
- `tests/components/upload/DropZone.test.tsx` - 12 testes
- `tests/components/upload/FileItem.test.tsx` - 8 testes
- `tests/components/upload/FileList.test.tsx` - 10 testes
- `tests/integration/upload-flow.test.ts` - 6 testes

---

## ✅ Funcionalidades Implementadas

### Upload Básico
- [x] Suporte a JPG, JPEG
- [x] Validação de formato (MIME type)
- [x] Validação de tamanho (1KB - 50MB)
- [x] Upload streaming para S3
- [x] URLs pré-assinadas
- [x] Progress tracking em tempo real

### Organização
- [x] Upload para álbuns
- [x] Upload para eventos
- [x] Hierarquia de pastas (tipo: 1)
- [x] Criar pastas/subpastas
- [x] Seleção de pasta de destino

### UI/UX
- [x] Drag and drop de arquivos
- [x] Preview de thumbnails
- [x] Progress bar individual
- [x] Progress bar geral
- [x] Seleção múltipla (Ctrl/Cmd+Click)
- [x] Exclusão em lote

### Performance
- [x] Upload simultâneo (configurável 1-40)
- [x] Streaming (sem carregar na memória)
- [x] Retry com backoff exponencial
- [x] Deduplicação por nome+size

---

## 🔍 Áreas de Análise

### 1. Validação de Fotos

#### Verificar Regras de Validação
```typescript
// src/config/validation.config.ts
export const PHOTO_CONFIG = {
  minSize: 1024,           // 1KB
  maxSize: 52428800,       // 50MB
  allowedExtensions: [
    '.jpg', '.jpeg'
  ],
  allowedMimeTypes: [
    'image/jpeg'
  ]
};
```

**Checklist**:
- [ ] Validação de extensão está funcionando
- [ ] Validação de MIME type está funcionando
- [ ] Validação de tamanho (min e max)
- [ ] Mensagens de erro são claras
- [ ] Validação é feita antes de adicionar à fila

#### Validação no Service
```typescript
// src/services/upload.service.ts
async validateImageFile(file: File): Promise<ValidationResult> {
  // Verifica:
  // 1. Extensão
  // 2. MIME type
  // 3. Tamanho
  // 4. Arquivo não é corrompido
}
```

**Verificar**:
- [ ] Valida ANTES de adicionar à fila
- [ ] Mensagens de erro claras
- [ ] Bloqueia arquivos inválidos
- [ ] Não permite bypass de validação

### 2. Upload para S3

#### URLs Pré-assinadas
```typescript
// Verificar integração com backend
const { url, key } = await api.get('/uploads/url-pre-assinada', {
  params: {
    fileName,
    contentType,
    albumId,   // ⚠️ DEVE SER STRING
    folderId
  }
});
```

**Checklist**:
- [ ] `albumId` é sempre string (CRÍTICO)
- [ ] Timeout de URL pré-assinada é adequado
- [ ] Retry em caso de falha de obtenção de URL
- [ ] Erro claro se URL expirar

#### Streaming Upload
```typescript
// Upload streaming via Tauri
await invoke('upload_stream_to_s3', {
  url: presignedUrl,
  filePath: file.path,
  contentType: file.type
});
```

**Verificar**:
- [ ] Não carrega arquivo completo na memória
- [ ] Progress tracking funciona
- [ ] Cancela upload se necessário
- [ ] Limpa recursos após upload/erro

### 3. Progress Tracking

#### Progress Individual
```typescript
// FileItem.tsx deve mostrar progress 0-100
<Progress value={item.progress} max={100} />
```

**Verificar**:
- [ ] Progress atualiza em tempo real
- [ ] Progress 0-100 (não >100)
- [ ] Cor muda baseado em status (pending/uploading/done/failed)
- [ ] Mensagem de status clara

#### Progress Geral
```typescript
// UploadStats.tsx
const totalProgress = useMemo(() => {
  return items.reduce((sum, item) => sum + (item.progress || 0), 0) / items.length;
}, [items]);
```

**Verificar**:
- [ ] Cálculo correto da média
- [ ] Atualização não causa re-renders excessivos
- [ ] Mostra estatísticas (total, done, failed, pending)

### 4. Organização em Pastas

#### Hierarquia
```typescript
// Tipo de pasta para fotos: 1
interface Pasta {
  id: number;
  nome: string;
  albumId: string;  // ⚠️ STRING
  tipo: 1;          // Fotos
  pai_id?: number;  // Subpasta
}
```

**Verificar**:
- [ ] Cria pastas com `tipo: 1`
- [ ] Suporta subpastas (pai_id)
- [ ] Valida nome de pasta
- [ ] Não permite nomes duplicados no mesmo nível

#### Seleção de Pasta
```typescript
// FolderPickerModal deve filtrar tipo: 1
const photoFolders = folders.filter(f => f.tipo === 1);
```

**Verificar**:
- [ ] Modal mostra apenas pastas de fotos
- [ ] Permite criar nova pasta
- [ ] Permite criar subpasta
- [ ] Valida permissões do usuário

### 5. Performance e Otimização

#### Deduplicação
```typescript
// Antes de adicionar à fila, verificar duplicados
const isDuplicate = existingFiles.some(f =>
  f.name === file.name && f.size === file.size
);
```

**Verificar**:
- [ ] Deduplica por nome + tamanho (não SHA-1)
- [ ] Não bloqueia UI durante deduplicação
- [ ] Avisa usuário de duplicados
- [ ] Permite forçar reenvio se necessário

#### Upload Simultâneo
```typescript
// Configuração de concorrência
const MAX_CONCURRENT_UPLOADS = 20; // Padrão
```

**Verificar**:
- [ ] Concorrência é respeitada (1-40)
- [ ] Não sobrecarga rede/CPU
- [ ] Permite ajustar em settings
- [ ] Performance degrada gracefully

### 6. Retry e Error Handling

#### Retry com Backoff
```typescript
// Fibonacci backoff: 1s, 1s, 2s, 3s, 5s
async retryUpload(item: QueueItem, attempt: number) {
  const delay = fibonacci(attempt) * 1000;
  await sleep(delay);
  return this.uploadPhoto(item);
}
```

**Verificar**:
- [ ] Retry até 5 tentativas
- [ ] Backoff exponencial Fibonacci
- [ ] Para em caso de erro irrecuperável (401, 403)
- [ ] Mensagem de erro clara após falhas

#### Error Messages
```typescript
// Mensagens de erro devem ser claras
if (error.code === 'FILE_TOO_LARGE') {
  return 'Arquivo muito grande. Máximo: 50MB';
}
```

**Verificar**:
- [ ] Erros são user-friendly
- [ ] Mostra causa do erro
- [ ] Sugere ação corretiva
- [ ] Não expõe detalhes técnicos sensíveis

---

## 🐛 Problemas Comuns

### 1. albumId é Número
```typescript
// ❌ ERRO CRÍTICO
const albumId = 123; // Número!
await uploadService.uploadPhoto(file, albumId, folderId);

// ✅ CORRETO
const albumId = "G27ec858..."; // String criptografada
await uploadService.uploadPhoto(file, albumId, folderId);
```

### 2. Validação Após Adicionar à Fila
```typescript
// ❌ Errado - valida depois
addToQueue(file);
if (!isValid(file)) {
  removeFromQueue(file);
}

// ✅ Correto - valida antes
if (!isValid(file)) {
  showError('Arquivo inválido');
  return;
}
addToQueue(file);
```

### 3. Progress >100%
```typescript
// ❌ Possível bug
const progress = (uploadedBytes / totalBytes) * 100; // Pode ser >100

// ✅ Correto
const progress = Math.min((uploadedBytes / totalBytes) * 100, 100);
```

### 4. Memory Leak em Streaming
```typescript
// ❌ Não limpa recursos
await uploadToS3(file);

// ✅ Limpa recursos
try {
  await uploadToS3(file);
} finally {
  cleanupResources(file);
}
```

---

## 📊 Métricas de Qualidade

### Performance
- **Upload speed**: Média de 5-10MB/s (depende de rede)
- **Concorrência**: 20 uploads simultâneos (padrão)
- **Memory**: <50MB para 1000 fotos na fila
- **CPU**: <10% durante uploads

### Confiabilidade
- **Success rate**: >99% (com retry)
- **Retry rate**: <5% dos uploads
- **Error rate**: <1%

### UX
- **Time to first upload**: <500ms após seleção
- **Progress updates**: A cada 100ms
- **Feedback visual**: Imediato (<100ms)

---

## 🚀 Como Usar Este Agente

### Análise Completa da Feature
```
@upload-fotos-specialist analise a feature completa de upload de fotos
```

### Análise de Validação
```
@upload-fotos-specialist foque em validação:
- Verificar regras de validação
- Testar casos edge (arquivos corrompidos, MIME incorreto)
- Mensagens de erro claras
```

### Análise de Performance
```
@upload-fotos-specialist analise performance:
- Deduplicação é eficiente?
- Concorrência está otimizada?
- Memory leaks?
```

### Code Review de PR
```
@upload-fotos-specialist revise as mudanças em upload de fotos neste PR
```

---

## 📚 Documentação Relacionada

- [Upload Flow](../../architecture/upload-flow.md) - Fluxo completo
- [Backend API](../../api/backend-api.md) - Endpoints de upload
- [Tauri Commands](../../api/tauri-commands.md) - Comandos Rust
- [Validation Config](../../config/validation.config.ts) - Regras de validação
- [ADR-002](../../architecture/adr/ADR-002-id-criptografado-unico.md) - IDs criptografados ⚠️ CRÍTICO

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
