# 🔌 Integração com Backend Specialist

**Feature**: Integração com Backend (v1.0.0+)
**Status**: ✅ Produção

---

## 🎯 Responsabilidade

Especialista em integração com API REST Laravel (30+ endpoints), upload streaming para S3, URLs pré-assinadas, comandos Tauri e cache.

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/api.ts` - Cliente API REST
- `src/services/album.service.ts` - 6 endpoints de álbuns
- `src/services/evento.service.ts` - 6 endpoints de eventos
- `src/services/upload.service.ts` - Endpoints de upload

### Tauri (Rust)
- `src-tauri/src/main.rs` - 16 comandos

### Documentação
- `docs/api/backend-api.md` - 30+ endpoints
- `docs/api/tauri-commands.md` - 16 comandos

---

## ✅ Endpoints Implementados

### Autenticação
- POST `/login` - Login
- POST `/logout` - Logout
- GET `/me` - User atual

### Álbuns
- GET `/albuns` - Listar álbuns
- GET `/albuns/:id` - Detalhar álbum
- GET `/albuns/compartilhados` - Álbuns compartilhados
- GET `/albuns/:id/fotos` - Fotos do álbum
- GET `/albuns/:id/videos` - Vídeos do álbum
- GET `/albuns/:id/pastas` - Pastas do álbum
- POST `/pastas/criar` - Criar pasta

### Eventos
- GET `/events/meus-eventos` - Listar eventos
- GET `/events/eventos-participantes` - Eventos como participante
- GET `/events/:id/fotos` - Fotos do evento
- GET `/events/:id/videos` - Vídeos do evento

### Uploads
- GET `/uploads/url-pre-assinada` - Obter URL S3
- POST `/uploads/salvar-foto` - Registrar upload
- POST `/uploads/salvar-video` - Registrar vídeo
- GET `/uploads/logs/:albumId` - Logs de upload
- POST `/uploads/multipart/init` - Iniciar multipart
- POST `/uploads/multipart/part` - Upload de chunk
- POST `/uploads/multipart/complete` - Finalizar multipart

---

## 🔍 Áreas de Análise

### 1. Cliente API

#### Configuração Base
```typescript
// api.ts
import axios from 'axios';

export const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json'
  }
});
```

**Verificar**:
- [ ] baseURL configurado corretamente
- [ ] Timeout adequado (30s)
- [ ] Headers corretos
- [ ] HTTPS obrigatório (não HTTP)

#### Interceptors
```typescript
// Request interceptor (auth)
api.interceptors.request.use(async config => {
  const token = await authService.getToken();
  if (token) {
    config.headers['Authorization'] = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor (refresh)
api.interceptors.response.use(
  response => response,
  async error => {
    if (error.response?.status === 401) {
      // Auto-refresh
    }
    return Promise.reject(error);
  }
);
```

**Verificar**:
- [ ] Authorization header em requests
- [ ] Auto-refresh em 401
- [ ] Error handling global
- [ ] Retry logic

### 2. Upload Streaming para S3

#### URLs Pré-assinadas
```typescript
// upload.service.ts
async getPresignedUrl(
  fileName: string,
  contentType: string,
  albumId: string,
  folderId?: number
): Promise<{ url: string; key: string }> {
  const response = await api.get('/uploads/url-pre-assinada', {
    params: { fileName, contentType, albumId, folderId }
  });

  return {
    url: response.data.url,
    key: response.data.key
  };
}
```

**Verificar**:
- [ ] `albumId` é sempre string ⚠️ CRÍTICO
- [ ] `contentType` correto (image/*, video/*)
- [ ] URL expira em tempo adequado (5-15 min)
- [ ] Error handling se URL expirar

#### Upload Streaming via Tauri
```typescript
// Não carrega arquivo na memória
await invoke('upload_stream_to_s3', {
  url: presignedUrl,
  filePath: file.path,
  contentType: file.type
});
```

**Verificar**:
- [ ] Não usa `file.arrayBuffer()` (carrega tudo)
- [ ] Streaming via Tauri (Rust)
- [ ] Progress tracking funciona
- [ ] Cancela upload se necessário

### 3. Cache e Otimizações

#### Cache de Álbuns
```typescript
// album.service.ts
class AlbumService {
  private cache: Map<string, { data: Album[]; timestamp: number }> = new Map();
  private readonly CACHE_TTL = 5 * 60 * 1000; // 5 min

  async getAlbums(forceRefresh = false): Promise<Album[]> {
    if (!forceRefresh) {
      const cached = this.cache.get('albums');
      if (cached && Date.now() - cached.timestamp < this.CACHE_TTL) {
        return cached.data;
      }
    }

    const response = await api.get('/albuns');
    this.cache.set('albums', {
      data: response.data,
      timestamp: Date.now()
    });

    return response.data;
  }
}
```

**Verificar**:
- [ ] Cache tem TTL (não infinito)
- [ ] ForceRefresh permite bypass
- [ ] Limpeza de cache antigo
- [ ] Performance não degrada

### 4. Comandos Tauri

#### Filesystem
```rust
// src-tauri/src/main.rs
#[tauri::command]
async fn select_folder() -> Result<String, String> {
    // Abre dialog de seleção
}

#[tauri::command]
async fn list_files(folder_path: String, recursive: bool) -> Result<Vec<FileInfo>, String> {
    // Lista arquivos
}
```

**Verificar**:
- [ ] Comandos retornam Result (error handling)
- [ ] Async onde necessário
- [ ] Performance não bloqueia UI

#### Keychain
```rust
#[tauri::command]
async fn store_token(token: String) -> Result<(), String> {
    // Keychain
}
```

**Verificar**:
- [ ] Token no Keychain (não arquivo)
- [ ] Error handling adequado

### 5. Error Handling

#### Tratamento Global
```typescript
// api.ts
api.interceptors.response.use(
  response => response,
  error => {
    if (error.response) {
      // Erro do servidor (4xx, 5xx)
      const message = error.response.data?.message || 'Erro no servidor';
      toast.error(message);
    } else if (error.request) {
      // Sem resposta (timeout, network)
      toast.error('Erro de conexão. Verifique sua internet.');
    } else {
      // Erro na configuração
      toast.error('Erro inesperado');
    }

    return Promise.reject(error);
  }
);
```

**Verificar**:
- [ ] Distingue tipos de erro (network, server, config)
- [ ] Mensagens user-friendly
- [ ] Toast notifications
- [ ] Erro não crasha app

---

## 🐛 Problemas Comuns

### 1. albumId como Número
```typescript
// ❌ ERRO CRÍTICO
await api.get('/uploads/url-pre-assinada', {
  params: { albumId: 123 } // Número!
});

// ✅ CORRETO
await api.get('/uploads/url-pre-assinada', {
  params: { albumId: "G27ec858..." } // String
});
```

### 2. Arquivo Carregado na Memória
```typescript
// ❌ Carrega tudo (2GB!)
const buffer = await file.arrayBuffer();
await uploadToS3(url, buffer);

// ✅ Streaming
await invoke('upload_stream_to_s3', { url, filePath: file.path });
```

### 3. Sem Retry em Falhas de Rede
```typescript
// ❌ Sem retry
await api.get('/albums'); // Falha 1x = erro

// ✅ Com retry (exponential backoff)
await retryRequest(() => api.get('/albums'), { maxRetries: 3 });
```

---

## 🚀 Como Usar Este Agente

```
@integracao-backend-specialist analise integração com backend
@integracao-backend-specialist foque em upload streaming
@integracao-backend-specialist verifique error handling
```

---

## 📚 Documentação

- [Backend API](../../api/backend-api.md) - 30+ endpoints
- [Tauri Commands](../../api/tauri-commands.md) - 16 comandos
- [Upload Flow](../../architecture/upload-flow.md)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
