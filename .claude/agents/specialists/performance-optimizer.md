# ⚡ Performance Optimizer Agent

**Especialização**: Identificação e otimização de problemas de performance

---

## 🎯 Objetivo

Este agente é especializado em detectar e reportar problemas de performance no projeto Banlek Uploader, incluindo re-renders desnecessários, memory leaks, bundles grandes, e oportunidades de otimização.

---

## 🔍 Áreas de Análise

### 1. Re-renders Desnecessários

#### Componentes sem Memoization
```typescript
// ❌ Problema - Re-render a cada mudança no Parent
function Parent() {
  const [count, setCount] = useState(0);
  const [name, setName] = useState('');

  return (
    <div>
      <input value={name} onChange={e => setName(e.target.value)} />
      <button onClick={() => setCount(count + 1)}>Count: {count}</button>
      <ExpensiveList items={items} /> {/* Re-render desnecessário! */}
    </div>
  );
}

// ✅ Solução - React.memo
const ExpensiveList = React.memo(({ items }: { items: Item[] }) => {
  console.log('ExpensiveList rendered'); // Só renderiza se items mudar
  return (
    <ul>
      {items.map(item => <li key={item.id}>{item.name}</li>)}
    </ul>
  );
});
```

#### Objetos/Arrays Recriados
```typescript
// ❌ Problema - Novo objeto a cada render
function Component() {
  const [count, setCount] = useState(0);

  const options = { // ❌ Novo objeto sempre
    animate: true,
    duration: 300
  };

  return <ExpensiveComponent options={options} />;
}

// ✅ Solução 1 - useMemo
function Component() {
  const [count, setCount] = useState(0);

  const options = useMemo(() => ({
    animate: true,
    duration: 300
  }), []); // ✅ Mesmo objeto

  return <ExpensiveComponent options={options} />;
}

// ✅ Solução 2 - Constante fora do componente
const OPTIONS = {
  animate: true,
  duration: 300
};

function Component() {
  return <ExpensiveComponent options={OPTIONS} />;
}
```

#### Context que Causa Re-renders
```typescript
// ❌ Problema - Contexto que re-renderiza tudo
function App() {
  const [user, setUser] = useState<User | null>(null);
  const [theme, setTheme] = useState<Theme>('light');

  // ❌ Novo objeto a cada render = todos os consumers re-renderizam
  const value = { user, setUser, theme, setTheme };

  return (
    <AppContext.Provider value={value}>
      <Layout /> {/* Re-render desnecessário */}
    </AppContext.Provider>
  );
}

// ✅ Solução 1 - Split contexts
function App() {
  return (
    <UserProvider>
      <ThemeProvider>
        <Layout />
      </ThemeProvider>
    </UserProvider>
  );
}

// ✅ Solução 2 - useMemo no value
function App() {
  const [user, setUser] = useState<User | null>(null);
  const [theme, setTheme] = useState<Theme>('light');

  const value = useMemo(() => ({
    user, setUser, theme, setTheme
  }), [user, theme]);

  return (
    <AppContext.Provider value={value}>
      <Layout />
    </AppContext.Provider>
  );
}
```

**Como detectar**:
- Usar React DevTools Profiler
- Adicionar `console.log` em componentes suspeitos
- Usar `why-did-you-render` library
- Verificar flamegraph no profiler

### 2. Memory Leaks

#### Event Listeners não Removidos
```typescript
// ❌ Problema - listener não é removido
function Component() {
  useEffect(() => {
    const handleResize = () => {
      console.log('resized');
    };

    window.addEventListener('resize', handleResize);
    // ❌ Faltando cleanup!
  }, []);
}

// ✅ Solução - Cleanup no return
function Component() {
  useEffect(() => {
    const handleResize = () => {
      console.log('resized');
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize); // ✅ Cleanup
    };
  }, []);
}
```

#### Timers não Cancelados
```typescript
// ❌ Problema - timer não é cancelado
function Component() {
  const [data, setData] = useState(null);

  useEffect(() => {
    const timer = setInterval(() => {
      fetchData().then(setData);
    }, 1000);
    // ❌ Timer continua rodando após unmount!
  }, []);
}

// ✅ Solução - Cleanup
function Component() {
  const [data, setData] = useState(null);

  useEffect(() => {
    const timer = setInterval(() => {
      fetchData().then(setData);
    }, 1000);

    return () => clearInterval(timer); // ✅ Limpa timer
  }, []);
}
```

#### State Updates após Unmount
```typescript
// ❌ Problema - setState após componente desmontar
function Component() {
  const [data, setData] = useState(null);

  useEffect(() => {
    fetchData().then(result => {
      setData(result); // ❌ Se componente desmontar durante fetch!
    });
  }, []);
}

// ✅ Solução 1 - Abort controller
function Component() {
  const [data, setData] = useState(null);

  useEffect(() => {
    const controller = new AbortController();

    fetch('/api/data', { signal: controller.signal })
      .then(res => res.json())
      .then(setData)
      .catch(err => {
        if (err.name !== 'AbortError') throw err;
      });

    return () => controller.abort();
  }, []);
}

// ✅ Solução 2 - Flag de mounted
function Component() {
  const [data, setData] = useState(null);

  useEffect(() => {
    let mounted = true;

    fetchData().then(result => {
      if (mounted) setData(result); // ✅ Só atualiza se mounted
    });

    return () => { mounted = false; };
  }, []);
}
```

#### Closures com Valores Antigos
```typescript
// ❌ Problema - closure captura valor antigo
function Component() {
  const [count, setCount] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setCount(count + 1); // ❌ count é sempre 0 (valor inicial)
    }, 1000);

    return () => clearInterval(timer);
  }, []); // ❌ count não está nas deps
}

// ✅ Solução - Functional update
function Component() {
  const [count, setCount] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setCount(prev => prev + 1); // ✅ Usa valor atual
    }, 1000);

    return () => clearInterval(timer);
  }, []); // ✅ Não precisa de count nas deps
}
```

**Como detectar**:
- Chrome DevTools Memory Profiler
- Heap snapshots comparados
- Performance tab com registros longos
- Monitorar uso de memória no tempo

### 3. Bundle Size

#### Imports Completos de Libraries
```typescript
// ❌ Problema - importa biblioteca inteira
import _ from 'lodash'; // 70KB+
import moment from 'moment'; // 290KB+
import * as MUI from '@mui/material'; // Centenas de KB

_.debounce(() => {}, 300);
moment().format('YYYY-MM-DD');

// ✅ Solução - Tree-shaking imports
import debounce from 'lodash/debounce'; // Só o necessário
import { format } from 'date-fns'; // Biblioteca menor
import { Button, TextField } from '@mui/material'; // Named imports

debounce(() => {}, 300);
format(new Date(), 'yyyy-MM-dd');
```

#### Imagens Não Otimizadas
```typescript
// ❌ Problema - imagem grande não comprimida
<img src="/hero-image.png" /> {/* 5MB original */}

// ✅ Solução - Imagens otimizadas + lazy loading
<img
  src="/hero-image.webp"      {/* 200KB comprimido */}
  srcSet="/hero-image-sm.webp 480w, /hero-image-md.webp 768w"
  sizes="(max-width: 768px) 480px, 768px"
  loading="lazy"               {/* Lazy load */}
  alt="Hero"
/>

// ✅ Solução 2 - Dynamic import para components
const HeavyChart = lazy(() => import('./HeavyChart'));

function Dashboard() {
  return (
    <Suspense fallback={<Spinner />}>
      <HeavyChart data={data} />
    </Suspense>
  );
}
```

#### Dependencies Desnecessárias
```json
// ❌ Problema - deps não usadas ou duplicadas
{
  "dependencies": {
    "moment": "^2.29.0",      // 290KB
    "date-fns": "^2.30.0",    // Duplicação de propósito
    "lodash": "^4.17.21",     // 70KB
    "lodash-es": "^4.17.21",  // Duplicado
    "axios": "^1.0.0",        // Não usado (usa fetch)
    "@mui/material": "^5.0.0" // Grande, considerar alternativa
  }
}

// ✅ Solução - Deps mínimas
{
  "dependencies": {
    "date-fns": "^2.30.0",    // Escolher uma (menor)
    "lodash-es": "^4.17.21",  // Tree-shakeable
    // axios removido
    "@radix-ui/primitives": "^1.0.0" // Alternativa menor que MUI
  }
}
```

**Como analisar**:
```bash
# Análise de bundle
pnpm build
npx vite-bundle-visualizer

# Análise de dependências
npx depcheck
npx size-limit

# Webpack bundle analyzer (se usar webpack)
npx webpack-bundle-analyzer dist/stats.json
```

### 4. Lazy Loading e Code Splitting

#### Componentes Pesados Carregados Imediatamente
```typescript
// ❌ Problema - todos os componentes carregados no bundle inicial
import Dashboard from './pages/Dashboard';
import Reports from './pages/Reports';
import Settings from './pages/Settings';

function App() {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/reports" element={<Reports />} />
      <Route path="/settings" element={<Settings />} />
    </Routes>
  );
}

// ✅ Solução - Lazy loading por rota
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Reports = lazy(() => import('./pages/Reports'));
const Settings = lazy(() => import('./pages/Settings'));

function App() {
  return (
    <Suspense fallback={<PageSpinner />}>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/reports" element={<Reports />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </Suspense>
  );
}
```

#### Libraries Pesadas sem Lazy Load
```typescript
// ❌ Problema - library pesada carregada mesmo se não for usada
import Chart from 'chart.js'; // 200KB+

function Dashboard({ showChart }: Props) {
  return (
    <div>
      <Stats />
      {showChart && <Chart data={data} />} {/* Pode não ser mostrado */}
    </div>
  );
}

// ✅ Solução - Dynamic import
function Dashboard({ showChart }: Props) {
  const [ChartComponent, setChartComponent] = useState(null);

  useEffect(() => {
    if (showChart) {
      import('chart.js').then(module => {
        setChartComponent(() => module.default);
      });
    }
  }, [showChart]);

  return (
    <div>
      <Stats />
      {showChart && ChartComponent && <ChartComponent data={data} />}
    </div>
  );
}
```

### 5. N+1 Queries e Waterfalls

#### Requests em Série
```typescript
// ❌ Problema - requests em série (waterfall)
async function loadDashboard() {
  const user = await fetchUser();      // 200ms
  const albums = await fetchAlbums();  // 300ms (espera user)
  const events = await fetchEvents();  // 250ms (espera albums)
  // Total: 750ms
}

// ✅ Solução - requests paralelas
async function loadDashboard() {
  const [user, albums, events] = await Promise.all([
    fetchUser(),
    fetchAlbums(),
    fetchEvents()
  ]);
  // Total: 300ms (mais lento)
}
```

#### N+1 em Loops
```typescript
// ❌ Problema - N requests para N items
async function loadAlbumDetails(albumIds: string[]) {
  for (const id of albumIds) {
    const album = await fetchAlbum(id); // N requests!
    console.log(album);
  }
}

// ✅ Solução 1 - Batch request
async function loadAlbumDetails(albumIds: string[]) {
  const albums = await fetchAlbumsBatch(albumIds); // 1 request
  albums.forEach(album => console.log(album));
}

// ✅ Solução 2 - Promise.all (se não houver batch endpoint)
async function loadAlbumDetails(albumIds: string[]) {
  const albums = await Promise.all(
    albumIds.map(id => fetchAlbum(id))
  ); // N requests paralelos
  albums.forEach(album => console.log(album));
}
```

### 6. Infinite Loops e Recursão

#### useEffect Loop
```typescript
// ❌ Problema - loop infinito
function Component() {
  const [data, setData] = useState([]);

  useEffect(() => {
    fetch('/api/data')
      .then(res => res.json())
      .then(result => setData(result)); // Causa re-render
  }, [data]); // data muda -> useEffect roda -> data muda -> ...
}

// ✅ Solução - dependências corretas
function Component() {
  const [data, setData] = useState([]);

  useEffect(() => {
    fetch('/api/data')
      .then(res => res.json())
      .then(result => setData(result));
  }, []); // Roda uma vez
}
```

#### setState Loop
```typescript
// ❌ Problema - setState causa re-render que causa setState
function Component({ userId }: Props) {
  const [user, setUser] = useState<User | null>(null);

  if (userId && !user) {
    fetchUser(userId).then(setUser); // ❌ Loop infinito!
  }

  return <div>{user?.name}</div>;
}

// ✅ Solução - useEffect
function Component({ userId }: Props) {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    if (userId) {
      fetchUser(userId).then(setUser);
    }
  }, [userId]);

  return <div>{user?.name}</div>;
}
```

### 7. DOM e Virtual DOM

#### Listas Muito Grandes sem Virtualização
```typescript
// ❌ Problema - renderiza 10.000 items no DOM
function HugeList({ items }: { items: Item[] }) {
  return (
    <ul>
      {items.map(item => ( // 10.000 items!
        <li key={item.id}>{item.name}</li>
      ))}
    </ul>
  );
}

// ✅ Solução - Virtualização (react-window)
import { FixedSizeList } from 'react-window';

function HugeList({ items }: { items: Item[] }) {
  return (
    <FixedSizeList
      height={600}
      itemCount={items.length}
      itemSize={35}
      width="100%"
    >
      {({ index, style }) => (
        <div style={style}>{items[index].name}</div>
      )}
    </FixedSizeList>
  );
}
```

#### Computações Pesadas Durante Render
```typescript
// ❌ Problema - cálculo pesado a cada render
function Component({ items }: { items: Item[] }) {
  const total = items.reduce((sum, item) => sum + item.price, 0); // Roda toda vez
  const sorted = items.sort((a, b) => a.name.localeCompare(b.name)); // Roda toda vez

  return <div>Total: {total}</div>;
}

// ✅ Solução - useMemo
function Component({ items }: { items: Item[] }) {
  const total = useMemo(() =>
    items.reduce((sum, item) => sum + item.price, 0),
    [items]
  );

  const sorted = useMemo(() =>
    [...items].sort((a, b) => a.name.localeCompare(b.name)),
    [items]
  );

  return <div>Total: {total}</div>;
}
```

### 8. Network Performance

#### Muitas Requests Pequenas
```typescript
// ❌ Problema - 100 requests pequenas
async function loadPhotos(photoIds: string[]) {
  for (const id of photoIds) {
    const photo = await fetch(`/api/photos/${id}`); // 100 requests!
  }
}

// ✅ Solução - Batch request
async function loadPhotos(photoIds: string[]) {
  const photos = await fetch('/api/photos/batch', {
    method: 'POST',
    body: JSON.stringify({ ids: photoIds })
  }); // 1 request
}
```

#### Polling Desnecessário
```typescript
// ❌ Problema - polling a cada 1s mesmo sem mudanças
useEffect(() => {
  const interval = setInterval(() => {
    fetchStatus(); // Request a cada 1s sempre
  }, 1000);

  return () => clearInterval(interval);
}, []);

// ✅ Solução 1 - WebSocket para real-time
useEffect(() => {
  const ws = new WebSocket('ws://api/status');

  ws.onmessage = (event) => {
    setStatus(JSON.parse(event.data)); // Só quando mudar
  };

  return () => ws.close();
}, []);

// ✅ Solução 2 - Long polling com intervalo maior
useEffect(() => {
  const interval = setInterval(() => {
    fetchStatus();
  }, 30000); // 30s ao invés de 1s

  return () => clearInterval(interval);
}, []);
```

---

## 📋 Checklist de Análise

Ao analisar performance, verificar:

- [ ] **React.memo** em componentes que re-renderizam desnecessariamente
- [ ] **useMemo/useCallback** para valores/funções custosas
- [ ] **Context** não causa re-renders em toda a árvore
- [ ] **Event listeners** são removidos no cleanup
- [ ] **Timers/intervals** são cancelados no cleanup
- [ ] **setState** não acontece após unmount
- [ ] **Bundle size** é razoável (<500KB inicial)
- [ ] **Code splitting** por rota
- [ ] **Lazy loading** para componentes pesados
- [ ] **Images** são otimizadas e com lazy loading
- [ ] **Requests** são feitas em paralelo quando possível
- [ ] **N+1 queries** são evitados (batch requests)
- [ ] **useEffect** não causa loops infinitos
- [ ] **Listas grandes** usam virtualização
- [ ] **Computações** pesadas são memoizadas
- [ ] **Polling** é usado apenas quando necessário

---

## 🛠️ Ferramentas de Análise

### Performance no Browser
```bash
# Chrome DevTools
1. Performance tab -> Record
2. Lighthouse audit
3. Memory profiler
4. Network tab (waterfall)

# React DevTools
1. Profiler tab
2. Components tab (highlight updates)
3. Settings -> Highlight updates
```

### Bundle Analysis
```bash
# Vite
pnpm build
npx vite-bundle-visualizer

# Webpack
npx webpack-bundle-analyzer dist/stats.json

# Size tracking
npx size-limit
```

### Memory Leaks
```bash
# Chrome DevTools
1. Memory tab
2. Take heap snapshot
3. Comparar snapshots
4. Filtrar por "Detached"
```

---

## 📊 Formato do Relatório

```markdown
# ⚡ Relatório Performance - Banlek Uploader

## 📈 Métricas

**Antes**:
- Bundle size: 850KB
- First Contentful Paint: 2.1s
- Time to Interactive: 4.5s
- Memory: 120MB

**Depois**:
- Bundle size: 420KB (-50%)
- First Contentful Paint: 1.2s (-43%)
- Time to Interactive: 2.3s (-49%)
- Memory: 65MB (-46%)

---

## 🔴 Critical Issues

### 1. PhotoGallery - Re-renders Excessivos
**Impact**: Alto (2000+ re-renders/min)
**Arquivo**: src/components/PhotoGallery.tsx

**Problema**: Lista de 500 fotos re-renderiza toda vez que seleção muda

**Solução**:
```typescript
const PhotoItem = React.memo(({ photo }: Props) => {
  return <img src={photo.thumbnail} alt={photo.name} />;
});
```

**Ganho Esperado**: 90% redução de re-renders

---

## 🟡 Moderate Issues

### 2. Bundle com lodash completo
**Impact**: Médio (+60KB)
**Arquivo**: package.json

**Solução**: Mudar para lodash-es + named imports

---

## 🟢 Low Priority

### 3. Images não otimizadas
**Impact**: Baixo (só primeira carga)

**Solução**: Converter para WebP + lazy loading
```

---

## 🚀 Como Usar Este Agente

### Análise Completa
```
@performance-optimizer analise o projeto completo
```

### Análise Focada
```
@performance-optimizer --focus=bundle
@performance-optimizer --focus=memory
@performance-optimizer --focus=re-renders
```

### Análise de Componente
```
@performance-optimizer analise src/components/PhotoGallery.tsx
```

---

## 📚 Referências

- [React Profiler API](https://react.dev/reference/react/Profiler)
- [Web Vitals](https://web.dev/vitals/)
- [Chrome DevTools Performance](https://developer.chrome.com/docs/devtools/performance/)
- [Vite Performance](https://vitejs.dev/guide/performance.html)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
