# 🎨 Interface e UX Specialist

**Feature**: Interface do Usuário (v1.0.0+, v1.9.0)
**Status**: ✅ Produção

---

## 🎯 Responsabilidade

Especialista em componentes UI, UX patterns, acessibilidade, design system, responsividade e experiência do usuário.

---

## 📁 Arquivos Relacionados

### Páginas (10 páginas principais)
- `src/pages/Login.tsx` - Autenticação
- `src/pages/Dashboard.tsx` - Resumo geral
- `src/pages/Albums.tsx` - Listagem de álbuns
- `src/pages/Eventos.tsx` - Listagem de eventos
- `src/pages/SelectAlbum.tsx` - Seleção
- `src/pages/Upload.tsx` - Upload principal
- `src/pages/Monitor.tsx` - Monitoramento
- `src/pages/Logs.tsx` - Histórico
- `src/pages/Settings.tsx` - Configurações

### Componentes UI (18 componentes base)
- `src/components/ui/Button.tsx`
- `src/components/ui/Input.tsx`
- `src/components/ui/DataTable.tsx`
- `src/components/ui/Tabs.tsx`
- `src/components/ui/Progress.tsx`
- `src/components/ui/Dialog.tsx`
- E mais...

### Layout
- `src/components/layout/AppHeader.tsx`
- `src/components/layout/BackButton.tsx`

### Styles
- `src/styles/globals.css` - TailwindCSS

---

## ✅ Funcionalidades UX (v1.9.0)

### Navegação
- [x] Menu de navegação (Álbuns/Eventos)
- [x] Botão "Voltar" consistente
- [x] Breadcrumbs

### Interação
- [x] Drag and drop funcional
- [x] Seleção múltipla
- [x] Filtros e ordenação
- [x] Paginação
- [x] Loading states
- [x] Error states
- [x] Empty states

### Feedback
- [x] Toast notifications
- [x] Progress bars
- [x] Confirmações
- [x] Validação inline

### Acessibilidade
- [x] Tab navigation
- [x] Keyboard shortcuts
- [x] ARIA labels
- [x] Focus management

---

## 🔍 Áreas de Análise

### 1. Design System

#### Componentes Base
```typescript
// Button.tsx
interface ButtonProps {
  variant?: 'default' | 'primary' | 'secondary' | 'outline' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}

const Button: React.FC<ButtonProps> = ({
  variant = 'default',
  size = 'md',
  disabled,
  loading,
  children,
  onClick
}) => {
  return (
    <button
      className={cn(
        'button',
        `button-${variant}`,
        `button-${size}`,
        { 'button-disabled': disabled || loading }
      )}
      disabled={disabled || loading}
      onClick={onClick}
    >
      {loading && <Spinner />}
      {children}
    </button>
  );
};
```

**Verificar**:
- [ ] Componentes seguem padrões consistentes
- [ ] Props são tipadas (TypeScript)
- [ ] Variants são limitadas (não aceita string qualquer)
- [ ] Estados (disabled, loading) são visuais
- [ ] Acessibilidade (ARIA, keyboard)

#### Consistência Visual
```typescript
// Tokens de design (TailwindCSS)
const colors = {
  primary: '#3B82F6',
  secondary: '#10B981',
  danger: '#EF4444',
  warning: '#F59E0B',
  info: '#6366F1'
};

const spacing = {
  xs: '0.25rem',
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
  xl: '2rem'
};
```

**Verificar**:
- [ ] Cores são consistentes em todo o app
- [ ] Spacing segue tokens definidos
- [ ] Typography é consistente
- [ ] Não há valores mágicos (hardcoded)

### 2. UX Patterns

#### Loading States
```typescript
// ✅ Bom - Loading state claro
{isLoading ? (
  <Skeleton count={5} />
) : (
  <List items={items} />
)}

// ❌ Ruim - Sem loading
{items && <List items={items} />}
```

**Verificar**:
- [ ] Todas as async operations têm loading state
- [ ] Skeleton screens onde apropriado
- [ ] Spinners são usados consistentemente
- [ ] Loading não bloqueia UI inteira

#### Error States
```typescript
// ✅ Bom - Error state com ação
{error && (
  <ErrorMessage>
    <p>{error.message}</p>
    <Button onClick={retry}>Tentar Novamente</Button>
  </ErrorMessage>
)}

// ❌ Ruim - Sem feedback
{error && <p>Erro</p>}
```

**Verificar**:
- [ ] Erros são mostrados claramente
- [ ] Mensagens são user-friendly
- [ ] Ações de recuperação (retry, cancel)
- [ ] Erros não crasham o app

#### Empty States
```typescript
// ✅ Bom - Empty state com CTA
{items.length === 0 && (
  <EmptyState
    icon={<FolderIcon />}
    title="Nenhum álbum encontrado"
    description="Crie seu primeiro álbum para começar"
    action={<Button onClick={createAlbum}>Criar Álbum</Button>}
  />
)}

// ❌ Ruim - Sem feedback
{items.length === 0 && <p>Sem itens</p>}
```

**Verificar**:
- [ ] Empty states são informativos
- [ ] Incluem ícone/ilustração
- [ ] Sugerem ação (CTA)
- [ ] Consistentes em todo o app

### 3. Interação

#### Drag and Drop
```typescript
// DropZone.tsx
const onDrop = useCallback((acceptedFiles: File[]) => {
  // Validação
  const validFiles = acceptedFiles.filter(isValid);

  // Feedback visual
  setIsDragging(false);
  setFiles(validFiles);

  // Toast de feedback
  if (validFiles.length < acceptedFiles.length) {
    toast.warning(`${acceptedFiles.length - validFiles.length} arquivo(s) inválido(s)`);
  }
}, []);

const { getRootProps, getInputProps, isDragActive } = useDropzone({
  onDrop,
  accept: ACCEPTED_MIME_TYPES,
  multiple: true
});
```

**Verificar**:
- [ ] Drag and drop funciona
- [ ] Visual feedback durante drag
- [ ] Validação de arquivos
- [ ] Feedback de erros
- [ ] Acessível via input file

#### Seleção Múltipla
```typescript
// DataTable.tsx
const [selected, setSelected] = useState<Set<string>>(new Set());

const handleSelectAll = () => {
  if (selected.size === items.length) {
    setSelected(new Set());
  } else {
    setSelected(new Set(items.map(i => i.id)));
  }
};

const handleSelectOne = (id: string) => {
  setSelected(prev => {
    const next = new Set(prev);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    return next;
  });
};
```

**Verificar**:
- [ ] Checkbox "Selecionar Todos" funciona
- [ ] Seleção individual funciona
- [ ] Visual feedback de seleção
- [ ] Performance com muitos itens
- [ ] Limpa seleção após ação (delete, etc)

### 4. Acessibilidade

#### Keyboard Navigation
```typescript
// Modal.tsx
const handleKeyDown = (e: React.KeyboardEvent) => {
  if (e.key === 'Escape') {
    onClose();
  }

  if (e.key === 'Tab') {
    // Trap focus dentro do modal
    trapFocus(e);
  }
};

useEffect(() => {
  if (isOpen) {
    // Focus no primeiro elemento focusável
    firstFocusableElement?.focus();
  }
}, [isOpen]);
```

**Verificar**:
- [ ] Tab navigation funciona
- [ ] Escape fecha modais/dialogs
- [ ] Enter submete forms
- [ ] Focus trap em modais
- [ ] Focus management correto

#### ARIA Labels
```typescript
// Button.tsx
<button
  aria-label="Fechar modal"
  aria-disabled={disabled}
  aria-busy={loading}
>
  <CloseIcon />
</button>

// Input.tsx
<div>
  <label htmlFor="email">Email</label>
  <input
    id="email"
    type="email"
    aria-required="true"
    aria-invalid={!!error}
    aria-describedby={error ? 'email-error' : undefined}
  />
  {error && <span id="email-error">{error}</span>}
</div>
```

**Verificar**:
- [ ] Elementos interativos têm aria-label
- [ ] Inputs têm labels associados
- [ ] Erros têm aria-describedby
- [ ] Estados têm ARIA adequado

### 5. Responsividade

#### Breakpoints
```typescript
// TailwindCSS breakpoints
const breakpoints = {
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px'
};

// Componente responsivo
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
  {items.map(item => <Card key={item.id} item={item} />)}
</div>
```

**Verificar**:
- [ ] Layout adapta em mobile
- [ ] Imagens são responsivas
- [ ] Tabelas scrollam em mobile
- [ ] Touch targets são grandes (44px min)

---

## 🐛 Problemas Comuns

### 1. Sem Loading State
```typescript
// ❌ Sem feedback
const items = useQuery('/items');
return <List items={items} />;

// ✅ Com loading
const { data, isLoading } = useQuery('/items');
if (isLoading) return <Skeleton />;
return <List items={data} />;
```

### 2. Erro Sem Feedback
```typescript
// ❌ Engole erro
try {
  await upload();
} catch {}

// ✅ Mostra erro
try {
  await upload();
} catch (error) {
  toast.error(error.message);
}
```

### 3. Acessibilidade Ignorada
```typescript
// ❌ Sem acessibilidade
<div onClick={handleClick}>Click me</div>

// ✅ Acessível
<button onClick={handleClick}>Click me</button>
```

---

## 🚀 Como Usar Este Agente

```
@interface-ui-specialist analise interface e UX
@interface-ui-specialist foque em acessibilidade
@interface-ui-specialist verifique design system
```

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
