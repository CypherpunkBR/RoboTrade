# 🖼️ Galeria Unificada Specialist

**Feature**: MediaGallery (v1.6.0+)
**Status**: ✅ Produção
**Testes**: 15 testes

---

## 🎯 Responsabilidade

Especialista na galeria unificada MediaGallery com tabs (Todos/Fotos/Vídeos), seleção múltipla, exclusão em lote, ordenação, badges, lazy loading e cache.

---

## 📁 Arquivos Relacionados

### Componentes
- `src/components/album/MediaGallery.tsx` - Galeria principal
- `src/components/album/MediaItem.tsx` - Item individual
- `src/components/album/PhotoGallery.tsx` - Galeria legada (fotos)

### Serviços
- `src/services/album.service.ts` - getPhotos, getVideos, deletePhoto, deleteVideo
- `src/services/evento.service.ts` - getPhotos, getVideos, deletePhoto, deleteVideo
- `src/services/indexed-db.service.ts` - Cache de thumbnails

### Hooks
- `src/hooks/useThumbnailCache.ts` - Cache otimizado

### Tipos
- `src/types/photo.types.ts` - PhotoEntity
- `src/types/video.types.ts` - VideoEntity
- `src/types/media.types.ts` - MediaEntity (união)

### Testes
- `tests/components/MediaGallery.test.tsx` - 15 testes

---

## ✅ Funcionalidades

### Tabs
- [x] Tab "Todas" (fotos + vídeos)
- [x] Tab "Fotos" (só fotos)
- [x] Tab "Vídeos" (só vídeos)
- [x] Contagem por tab

### Seleção
- [x] Seleção simples (click)
- [x] Seleção múltipla (Ctrl/Cmd+Click)
- [x] Seleção por range (Shift+Click)
- [x] Selecionar todos/nenhum

### Ações
- [x] Exclusão em lote
- [x] Download em lote
- [x] Compartilhar (futuro)

### UI
- [x] Badge de tipo (Foto/Vídeo)
- [x] Ícone de play (vídeos)
- [x] Duração formatada (MM:SS)
- [x] Indicador de processamento
- [x] Lazy loading com virtualização
- [x] Ordenação (mais recentes primeiro)

---

## 🔍 Áreas de Análise

### 1. Tabs e Filtros

#### Implementação
```typescript
<Tabs defaultValue="todos">
  <TabsList>
    <TabsTrigger value="todos">Todas ({totalCount})</TabsTrigger>
    <TabsTrigger value="fotos">Fotos ({photoCount})</TabsTrigger>
    <TabsTrigger value="videos">Vídeos ({videoCount})</TabsTrigger>
  </TabsList>

  <TabsContent value="todos">
    {allMedia.map(item => <MediaItem key={item.id} item={item} />)}
  </TabsContent>

  <TabsContent value="fotos">
    {photos.map(item => <MediaItem key={item.id} item={item} />)}
  </TabsContent>

  <TabsContent value="videos">
    {videos.map(item => <MediaItem key={item.id} item={item} />)}
  </TabsContent>
</Tabs>
```

**Verificar**:
- [ ] Tabs funcionam corretamente
- [ ] Contagens são precisas
- [ ] Mudança de tab não re-fetcha dados desnecessariamente
- [ ] Filtros são aplicados corretamente

### 2. Seleção Múltipla

#### Lógica de Seleção
```typescript
const [selected, setSelected] = useState<Set<string>>(new Set());

const handleClick = (itemId: string, e: React.MouseEvent) => {
  if (e.metaKey || e.ctrlKey) {
    // Ctrl/Cmd+Click: toggle seleção
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(itemId)) next.delete(itemId);
      else next.add(itemId);
      return next;
    });
  } else if (e.shiftKey) {
    // Shift+Click: seleciona range
    selectRange(lastClicked, itemId);
  } else {
    // Click simples: seleciona apenas este
    setSelected(new Set([itemId]));
  }
};
```

**Verificar**:
- [ ] Ctrl/Cmd+Click adiciona/remove da seleção
- [ ] Shift+Click seleciona range correto
- [ ] Click simples limpa seleção anterior
- [ ] Visual feedback de seleção
- [ ] Performance com muitos itens selecionados

### 3. Exclusão em Lote

#### Implementação
```typescript
const handleDeleteSelected = async () => {
  const confirmed = await confirm(
    `Excluir ${selected.size} ${selected.size === 1 ? 'item' : 'itens'}?`
  );

  if (!confirmed) return;

  const results = await Promise.allSettled(
    Array.from(selected).map(id => deleteMedia(id))
  );

  const failed = results.filter(r => r.status === 'rejected');
  if (failed.length > 0) {
    toast.error(`${failed.length} item(s) não puderam ser excluídos`);
  } else {
    toast.success(`${selected.size} item(s) excluídos`);
  }

  setSelected(new Set());
  refetch();
};
```

**Verificar**:
- [ ] Confirmação antes de excluir
- [ ] Exclusão em paralelo (Promise.allSettled)
- [ ] Feedback de erros parciais
- [ ] Refetch após exclusão
- [ ] Limpa seleção após exclusão

### 4. Badges e Indicadores

#### MediaItem
```typescript
// MediaItem.tsx
<div className="media-item">
  <img src={item.thumbnail} alt={item.name} />

  {/* Badge de tipo */}
  {item.type === 'video' && (
    <Badge variant="secondary">Vídeo</Badge>
  )}

  {/* Ícone de play */}
  {item.type === 'video' && (
    <PlayIcon className="play-overlay" />
  )}

  {/* Duração */}
  {item.type === 'video' && item.duration && (
    <span className="duration">{formatDuration(item.duration)}</span>
  )}

  {/* Processamento */}
  {item.processing && (
    <Spinner className="processing-indicator" />
  )}
</div>
```

**Verificar**:
- [ ] Badge aparece apenas em vídeos
- [ ] Ícone de play overlay posicionado corretamente
- [ ] Duração formatada (MM:SS)
- [ ] Indicador de processamento visível
- [ ] CSS não causa layout shift

### 5. Lazy Loading e Virtualização

#### react-window
```typescript
import { FixedSizeGrid } from 'react-window';

<FixedSizeGrid
  columnCount={columnCount}
  columnWidth={200}
  height={600}
  rowCount={Math.ceil(items.length / columnCount)}
  rowHeight={200}
  width={width}
>
  {({ columnIndex, rowIndex, style }) => {
    const index = rowIndex * columnCount + columnIndex;
    const item = items[index];
    if (!item) return null;

    return (
      <div style={style}>
        <MediaItem item={item} />
      </div>
    );
  }}
</FixedSizeGrid>
```

**Verificar**:
- [ ] Virtualização ativada para listas >100 itens
- [ ] Scroll performance é smooth
- [ ] Itens renderizam ao entrar no viewport
- [ ] Memory não estoura com 1000+ itens

### 6. Cache de Thumbnails

#### useThumbnailCache
```typescript
// hooks/useThumbnailCache.ts
const useThumbnailCache = (items: MediaEntity[]) => {
  const [thumbnails, setThumbnails] = useState<Map<string, string>>(new Map());

  useEffect(() => {
    items.forEach(async (item) => {
      if (!thumbnails.has(item.id)) {
        const cached = await indexedDB.getThumbnail(item.id);
        if (cached) {
          setThumbnails(prev => new Map(prev).set(item.id, cached));
        } else {
          // Fetch e cache
          const url = await fetchThumbnail(item.id);
          await indexedDB.cacheThumbnail(item.id, url);
          setThumbnails(prev => new Map(prev).set(item.id, url));
        }
      }
    });
  }, [items]);

  return thumbnails;
};
```

**Verificar**:
- [ ] Thumbnails são cacheados em IndexedDB
- [ ] Não refetcha thumbnails já cacheados
- [ ] Limpeza de cache antigo (>30 dias)
- [ ] Performance não degrada com muitos thumbnails

---

## 🐛 Problemas Comuns

### 1. Re-render Excessivo
```typescript
// ❌ Re-render toda lista ao selecionar
<MediaItem isSelected={selected.has(item.id)} />

// ✅ Memo do MediaItem
const MediaItem = React.memo(({ item, isSelected }) => {
  return <div className={isSelected ? 'selected' : ''}>{item.name}</div>;
});
```

### 2. Exclusão Sem Confirmação
```typescript
// ❌ Exclui sem confirmar
const handleDelete = () => {
  deleteMedia(id);
};

// ✅ Confirma antes
const handleDelete = async () => {
  const confirmed = await confirm('Excluir este item?');
  if (confirmed) deleteMedia(id);
};
```

### 3. Shift+Click Não Funciona
```typescript
// ❌ lastClicked não é atualizado
const handleClick = (id, e) => {
  if (e.shiftKey) selectRange(lastClicked, id);
};

// ✅ Atualiza lastClicked
const [lastClicked, setLastClicked] = useState(null);
const handleClick = (id, e) => {
  if (e.shiftKey) selectRange(lastClicked, id);
  setLastClicked(id);
};
```

---

## 🚀 Como Usar Este Agente

```
@galeria-specialist analise a galeria unificada
@galeria-specialist foque em seleção múltipla
@galeria-specialist verifique performance com 1000+ itens
```

---

## 📚 Documentação

- [MediaGallery Component](../../components/album/MediaGallery.tsx)
- [Lazy Loading Guide](../../guides/lazy-loading.md)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
