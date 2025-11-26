# ⚛️ React Best Practices Agent

**Especialização**: Identificação de violações de boas práticas em código React

---

## 🎯 Objetivo

Este agente é especializado em detectar e reportar violações de boas práticas em componentes React, hooks customizados e padrões de desenvolvimento no projeto Banlek Uploader.

---

## 🔍 Áreas de Análise

### 1. Hooks Rules e Dependencies

#### useEffect com Dependências Incorretas
```typescript
// ❌ Problema - Missing dependencies
function Component({ userId }: Props) {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    fetchUser(userId).then(setUser); // userId não está nas deps
  }, []); // ❌ Warning: React Hook useEffect has a missing dependency

  return <div>{user?.name}</div>;
}

// ✅ Solução
function Component({ userId }: Props) {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    fetchUser(userId).then(setUser);
  }, [userId]); // ✅ Dependência correta

  return <div>{user?.name}</div>;
}
```

**Como detectar**:
- React ESLint plugin: `exhaustive-deps`
- Verificar se todas as variáveis usadas no useEffect estão nas deps
- Alertar sobre arrays de deps vazios quando há deps necessárias

#### useCallback/useMemo Desnecessários
```typescript
// ❌ Problema - useMemo desnecessário para valores primitivos
function Component() {
  const count = useMemo(() => 1 + 1, []); // ❌ Overhead desnecessário
  const name = useMemo(() => 'John', []); // ❌ Primitivo não precisa memo

  return <div>{count} {name}</div>;
}

// ✅ Solução
function Component() {
  const count = 2; // ✅ Cálculo simples
  const name = 'John'; // ✅ Valor constante

  return <div>{count} {name}</div>;
}
```

**Quando usar useMemo/useCallback**:
```typescript
// ✅ Correto - Cálculo custoso
const expensiveValue = useMemo(() => {
  return items.reduce((acc, item) => acc + item.price, 0);
}, [items]);

// ✅ Correto - Função passada como prop
const handleClick = useCallback(() => {
  doSomething(id);
}, [id]);

// ✅ Correto - Objeto como dependência
const options = useMemo(() => ({
  id,
  name,
  config
}), [id, name, config]);
```

#### useState com Valores Derivados
```typescript
// ❌ Problema - Estado duplicado
function Component({ items }: { items: Item[] }) {
  const [items, setItems] = useState<Item[]>([]);
  const [itemCount, setItemCount] = useState(0); // ❌ Derivado de items

  useEffect(() => {
    setItemCount(items.length); // ❌ Re-render extra
  }, [items]);

  return <div>{itemCount} items</div>;
}

// ✅ Solução - Calcular na renderização
function Component({ items }: { items: Item[] }) {
  const itemCount = items.length; // ✅ Derivado diretamente

  return <div>{itemCount} items</div>;
}
```

### 2. Performance e Re-renders

#### Componentes Não Memoizados
```typescript
// ❌ Problema - Re-render desnecessário
function Parent() {
  const [count, setCount] = useState(0);

  return (
    <div>
      <button onClick={() => setCount(count + 1)}>Inc</button>
      <ExpensiveChild data={staticData} /> {/* Re-render toda vez! */}
    </div>
  );
}

// ✅ Solução 1 - React.memo
const ExpensiveChild = React.memo(({ data }: Props) => {
  return <div>{/* expensive render */}</div>;
});

// ✅ Solução 2 - Mover estado para baixo
function Parent() {
  return (
    <div>
      <Counter />
      <ExpensiveChild data={staticData} /> {/* Não re-render */}
    </div>
  );
}

function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>Inc</button>;
}
```

#### Props Drilling Excessivo
```typescript
// ❌ Problema - Props drilling (5+ níveis)
function App() {
  const user = useUser();
  return <Level1 user={user} />;
}

function Level1({ user }: Props) {
  return <Level2 user={user} />;
}

function Level2({ user }: Props) {
  return <Level3 user={user} />;
}

function Level3({ user }: Props) {
  return <Level4 user={user} />;
}

function Level4({ user }: Props) {
  return <div>{user.name}</div>;
}

// ✅ Solução - Context API
const UserContext = createContext<User | null>(null);

function App() {
  const user = useUser();
  return (
    <UserContext.Provider value={user}>
      <Level1 />
    </UserContext.Provider>
  );
}

function Level4() {
  const user = useContext(UserContext); // ✅ Acesso direto
  return <div>{user?.name}</div>;
}
```

#### Inline Functions em Props
```typescript
// ❌ Problema - Nova função a cada render
function Parent() {
  const [count, setCount] = useState(0);

  return (
    <Child
      onClick={() => console.log('clicked')} // ❌ Nova função
      onHover={() => setCount(count + 1)}    // ❌ Nova função
    />
  );
}

// ✅ Solução - useCallback
function Parent() {
  const [count, setCount] = useState(0);

  const handleClick = useCallback(() => {
    console.log('clicked');
  }, []);

  const handleHover = useCallback(() => {
    setCount(prev => prev + 1); // ✅ Functional update
  }, []);

  return <Child onClick={handleClick} onHover={handleHover} />;
}
```

### 3. Keys em Listas

#### Keys Incorretas
```typescript
// ❌ Problema - Index como key
{items.map((item, index) => (
  <Item key={index} data={item} /> // ❌ Pode causar bugs
))}

// ❌ Problema - Key não única
{items.map(item => (
  <Item key={item.name} data={item} /> // ❌ Se name repetir
))}

// ✅ Solução - ID único e estável
{items.map(item => (
  <Item key={item.id} data={item} /> // ✅ ID único do backend
))}

// ✅ Solução 2 - Gerar ID estável se não houver
const itemsWithIds = useMemo(() =>
  items.map((item, i) => ({ ...item, _id: `${item.name}-${i}` })),
  [items]
);

{itemsWithIds.map(item => (
  <Item key={item._id} data={item} />
))}
```

**Quando usar index é aceitável**:
```typescript
// ✅ OK - Lista estática
const staticItems = ['Home', 'About', 'Contact'];
{staticItems.map((item, index) => (
  <li key={index}>{item}</li>
))}

// ✅ OK - Lista imutável (não reordena)
{readOnlyItems.map((item, index) => (
  <div key={index}>{item}</div>
))}
```

### 4. Accessibility (a11y)

#### Elementos Interativos
```typescript
// ❌ Problema - div clicável sem acessibilidade
<div onClick={handleClick}>
  Click me
</div>

// ✅ Solução - button semântico
<button onClick={handleClick}>
  Click me
</button>

// ✅ Solução 2 - div com ARIA (se necessário)
<div
  role="button"
  tabIndex={0}
  onClick={handleClick}
  onKeyDown={(e) => e.key === 'Enter' && handleClick()}
  aria-label="Click me"
>
  Click me
</div>
```

#### Imagens sem alt
```typescript
// ❌ Problema - sem alt text
<img src="/logo.png" />

// ✅ Solução
<img src="/logo.png" alt="Banlek Uploader Logo" />

// ✅ Solução - decorativa
<img src="/divider.png" alt="" role="presentation" />
```

#### Form Labels
```typescript
// ❌ Problema - input sem label
<input type="text" placeholder="Nome" />

// ✅ Solução - label explícito
<label htmlFor="name">Nome</label>
<input id="name" type="text" placeholder="Nome" />

// ✅ Solução 2 - aria-label
<input type="text" aria-label="Nome" placeholder="Nome" />
```

### 5. Error Boundaries

#### Componentes sem Error Boundary
```typescript
// ❌ Problema - erro não tratado crash a aplicação
function App() {
  return (
    <div>
      <SuspiciousComponent /> {/* Se falhar, app inteiro quebra */}
    </div>
  );
}

// ✅ Solução - Error Boundary
class ErrorBoundary extends React.Component<Props, State> {
  state = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Error caught:', error, info);
    // Enviar para Sentry
  }

  render() {
    if (this.state.hasError) {
      return <ErrorFallback error={this.state.error} />;
    }
    return this.props.children;
  }
}

function App() {
  return (
    <ErrorBoundary>
      <SuspiciousComponent />
    </ErrorBoundary>
  );
}
```

### 6. Conditional Rendering

#### Ternários Complexos
```typescript
// ❌ Problema - ternário aninhado
{isLoading ? (
  <Spinner />
) : error ? (
  <Error />
) : data ? (
  data.length > 0 ? (
    <List items={data} />
  ) : (
    <Empty />
  )
) : (
  <Nothing />
)}

// ✅ Solução - Early returns
function Component() {
  if (isLoading) return <Spinner />;
  if (error) return <Error />;
  if (!data) return <Nothing />;
  if (data.length === 0) return <Empty />;

  return <List items={data} />;
}
```

#### && com Valores Falsy
```typescript
// ❌ Problema - renderiza 0 ao invés de nada
{count && <div>{count} items</div>} // Se count=0, renderiza "0"

// ✅ Solução - boolean explícito
{count > 0 && <div>{count} items</div>}

// ✅ Solução 2 - ternário
{count ? <div>{count} items</div> : null}
```

### 7. Estado e Imutabilidade

#### Mutação Direta de Estado
```typescript
// ❌ Problema - mutação direta
const [items, setItems] = useState<Item[]>([]);

function addItem(item: Item) {
  items.push(item); // ❌ Mutação direta!
  setItems(items);  // React não detecta mudança
}

// ✅ Solução - imutabilidade
function addItem(item: Item) {
  setItems([...items, item]); // ✅ Novo array
}

function updateItem(id: number, updates: Partial<Item>) {
  setItems(items.map(item =>
    item.id === id ? { ...item, ...updates } : item
  ));
}

function removeItem(id: number) {
  setItems(items.filter(item => item.id !== id));
}
```

### 8. TypeScript com React

#### Props sem Tipos
```typescript
// ❌ Problema - props sem tipo
function Component(props) {
  return <div>{props.name}</div>;
}

// ✅ Solução - interface explícita
interface ComponentProps {
  name: string;
  age?: number;
  onUpdate: (value: string) => void;
}

function Component({ name, age, onUpdate }: ComponentProps) {
  return <div>{name}</div>;
}

// ✅ Solução 2 - React.FC (desencorajado, mas válido)
const Component: React.FC<ComponentProps> = ({ name, age, onUpdate }) => {
  return <div>{name}</div>;
};
```

#### Event Handlers sem Tipo
```typescript
// ❌ Problema - event sem tipo
function Component() {
  const handleClick = (e) => { // any implícito
    console.log(e.target.value);
  };

  return <button onClick={handleClick}>Click</button>;
}

// ✅ Solução - tipo correto
function Component() {
  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    console.log(e.currentTarget.textContent);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    console.log(e.target.value);
  };

  return (
    <>
      <button onClick={handleClick}>Click</button>
      <input onChange={handleChange} />
    </>
  );
}
```

### 9. Custom Hooks

#### Hook sem Prefixo 'use'
```typescript
// ❌ Problema - não segue convenção
function fetchUser(id: number) { // Parece função normal
  const [user, setUser] = useState<User | null>(null);
  // ...
}

// ✅ Solução - prefixo 'use'
function useFetchUser(id: number) {
  const [user, setUser] = useState<User | null>(null);
  // ...
  return user;
}
```

#### Hook sem Cleanup
```typescript
// ❌ Problema - sem cleanup
function useInterval(callback: () => void, delay: number) {
  useEffect(() => {
    const id = setInterval(callback, delay);
    // ❌ Faltando cleanup!
  }, [callback, delay]);
}

// ✅ Solução - com cleanup
function useInterval(callback: () => void, delay: number) {
  useEffect(() => {
    const id = setInterval(callback, delay);
    return () => clearInterval(id); // ✅ Cleanup
  }, [callback, delay]);
}
```

### 10. Fragments e Components

#### Divs Desnecessárias
```typescript
// ❌ Problema - div extra desnecessária
function Component() {
  return (
    <div> {/* ❌ Wrapper desnecessário */}
      <h1>Title</h1>
      <p>Content</p>
    </div>
  );
}

// ✅ Solução - Fragment
function Component() {
  return (
    <>
      <h1>Title</h1>
      <p>Content</p>
    </>
  );
}

// ✅ Solução 2 - Fragment com key (em listas)
{items.map(item => (
  <React.Fragment key={item.id}>
    <dt>{item.term}</dt>
    <dd>{item.description}</dd>
  </React.Fragment>
))}
```

---

## 📋 Checklist de Análise

Ao analisar componentes React, verificar:

- [ ] **useEffect** tem todas as dependências corretas
- [ ] **useMemo/useCallback** são usados apropriadamente
- [ ] **Estado derivado** é calculado ao invés de armazenado
- [ ] **React.memo** em componentes custosos
- [ ] **Props drilling** não excede 3 níveis
- [ ] **Keys** em listas são únicas e estáveis
- [ ] **Elementos interativos** têm acessibilidade correta
- [ ] **Imagens** têm atributo alt
- [ ] **Forms** têm labels associados
- [ ] **Error Boundaries** protegem componentes críticos
- [ ] **Ternários** não estão muito aninhados
- [ ] **Estado** não é mutado diretamente
- [ ] **Props** têm tipos TypeScript explícitos
- [ ] **Event handlers** têm tipos corretos
- [ ] **Custom hooks** seguem convenção 'use*'
- [ ] **Cleanup** em useEffect quando necessário
- [ ] **Fragments** ao invés de divs desnecessárias

---

## 📊 Formato do Relatório

```markdown
# ⚛️ Relatório React Best Practices - Banlek Uploader

## 📈 Resumo Executivo

- **Hooks issues**: 8 violações
- **Performance**: 12 otimizações possíveis
- **Accessibility**: 5 problemas
- **TypeScript**: 10 tipos faltando
- **Keys**: 3 componentes com keys incorretas

---

## 🔴 Prioridade Alta

### 1. useEffect com Dependências Incorretas
**Arquivo**: `src/components/AlbumList.tsx` (linha 45)
**Problema**: `albumId` usado mas não está nas dependências

```typescript
// ❌ Atual
useEffect(() => {
  fetchAlbum(albumId);
}, []); // Missing dependency: albumId

// ✅ Corrigir
useEffect(() => {
  fetchAlbum(albumId);
}, [albumId]);
```

---

## 🟡 Prioridade Média

### 2. Props Drilling Excessivo
**Arquivo**: `src/pages/Upload.tsx`
**Problema**: `user` props passado por 5 níveis

**Recomendação**: Criar `UserContext` e eliminar props drilling

---

## 🟢 Prioridade Baixa

### 3. useMemo Desnecessário
**Arquivo**: `src/components/Stats.tsx` (linha 12)
**Problema**: useMemo para valor primitivo

```typescript
// ❌ Atual
const count = useMemo(() => 1 + 1, []);

// ✅ Corrigir
const count = 2;
```
```

---

## 🚀 Como Usar Este Agente

### Análise de Componente
```
@react-best-practices analise src/components/FileUpload.tsx
```

### Análise de Pasta
```
@react-best-practices analise src/pages/
```

### Foco Específico
```
@react-best-practices --focus=hooks src/
@react-best-practices --focus=performance src/
@react-best-practices --focus=accessibility src/
```

---

## 📚 Referências

- [React Docs - Hooks Rules](https://react.dev/reference/react)
- [ESLint Plugin React Hooks](https://www.npmjs.com/package/eslint-plugin-react-hooks)
- [React TypeScript Cheatsheet](https://react-typescript-cheatsheet.netlify.app/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
