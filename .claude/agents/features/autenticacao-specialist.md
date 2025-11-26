# 🔐 Autenticação e Segurança Specialist

**Feature**: Autenticação e Segurança (v1.0.0+)
**Status**: ✅ Produção
**Testes**: 24 casos

---

## 🎯 Responsabilidade

Especialista em autenticação Laravel Sanctum, tokens no Keychain, auto-refresh, interceptor de autenticação, SSO para dashboard e segurança geral.

---

## 📁 Arquivos Relacionados

### Serviços
- `src/services/auth.service.ts` - Autenticação principal
- `src/services/api.ts` - Interceptor

### Componentes
- `src/pages/Login.tsx` - Página de login
- `src/components/auth/AuthGuard.tsx` - Proteção de rotas

### Hooks
- `src/hooks/useAuth.ts` - Hook de autenticação

### Stores
- `src/store/authStore.ts` - Estado de autenticação

### Testes
- `tests/services/auth.service.test.ts` - 24 casos

---

## ✅ Funcionalidades

### Autenticação
- [x] Login com Laravel Sanctum
- [x] Tokens no Keychain (macOS)
- [x] Auto-refresh de tokens
- [x] Logout seguro
- [x] Interceptor de autenticação

### Segurança
- [x] Tokens NUNCA em localStorage
- [x] HTTPS obrigatório
- [x] Validação de tokens
- [x] Rate limiting (backend)

### SSO
- [x] Botão "Ver no Dashboard" com SSO
- [x] Token compartilhado

---

## 🔍 Áreas de Análise

### 1. Login com Sanctum

#### Fluxo
```typescript
// auth.service.ts
class AuthService {
  async login(email: string, password: string): Promise<User> {
    // 1. Login via API
    const response = await api.post('/login', { email, password });
    const { token, user } = response.data;

    // 2. Armazenar token no Keychain (NUNCA localStorage)
    await invoke('store_token', { token });

    // 3. Atualizar store
    authStore.setUser(user);
    authStore.setAuthenticated(true);

    return user;
  }
}
```

**Verificar**:
- [ ] Token é armazenado no Keychain (não localStorage)
- [ ] User é armazenado no authStore
- [ ] Redireciona para dashboard após login
- [ ] Erro claro se credenciais inválidas
- [ ] Loading state durante login

### 2. Auto-Refresh de Tokens

#### Implementação
```typescript
// api.ts
api.interceptors.response.use(
  response => response,
  async error => {
    const originalRequest = error.config;

    // Token expirado
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      try {
        // Tenta refresh
        const newToken = await authService.refreshToken();
        await invoke('store_token', { token: newToken });

        // Retry request original
        originalRequest.headers['Authorization'] = `Bearer ${newToken}`;
        return api(originalRequest);
      } catch (refreshError) {
        // Refresh falhou, fazer logout
        authService.logout();
        return Promise.reject(refreshError);
      }
    }

    return Promise.reject(error);
  }
);
```

**Verificar**:
- [ ] Detecta token expirado (401)
- [ ] Tenta refresh automaticamente
- [ ] Retry request original após refresh
- [ ] Logout se refresh falhar
- [ ] Não entra em loop infinito (_retry flag)

### 3. Keychain (Armazenamento Seguro)

#### Nunca localStorage
```typescript
// ❌ INSEGURO - NUNCA FAZER
localStorage.setItem('token', token);

// ✅ SEGURO - Keychain via Tauri
await invoke('store_token', { token });
```

**Verificar**:
- [ ] Token NUNCA em localStorage
- [ ] Token NUNCA em sessionStorage
- [ ] Token NUNCA em cookies não httpOnly
- [ ] Apenas Keychain via Tauri
- [ ] Cache em memória para performance (1x por sessão)

#### Cache Otimizado
```typescript
// auth.service.ts
class AuthService {
  private tokenCache: string | null = null;

  async getToken(): Promise<string | null> {
    // Cache em memória (1x por sessão)
    if (this.tokenCache) return this.tokenCache;

    // Busca do Keychain
    try {
      this.tokenCache = await invoke<string>('get_token');
      return this.tokenCache;
    } catch {
      return null;
    }
  }

  clearCache() {
    this.tokenCache = null;
  }
}
```

**Verificar**:
- [ ] Token é buscado 1x por sessão (cache)
- [ ] Cache é limpo ao fazer logout
- [ ] Cache é limpo ao refresh de token
- [ ] Performance não degrada

### 4. Interceptor de Autenticação

#### API Interceptor
```typescript
// api.ts
api.interceptors.request.use(async config => {
  const token = await authService.getToken();

  if (token) {
    config.headers['Authorization'] = `Bearer ${token}`;
  }

  return config;
});
```

**Verificar**:
- [ ] Adiciona header Authorization em todas as requests
- [ ] Token é obtido do Keychain
- [ ] Performance não degrada (cache)
- [ ] Funciona com refresh de token

### 5. AuthGuard (Proteção de Rotas)

#### Implementação
```typescript
// components/auth/AuthGuard.tsx
const AuthGuard: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { isAuthenticated, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && !isAuthenticated) {
      router.push('/login');
    }
  }, [isAuthenticated, loading]);

  if (loading) return <Spinner />;
  if (!isAuthenticated) return null;

  return <>{children}</>;
};
```

**Verificar**:
- [ ] Redireciona para /login se não autenticado
- [ ] Mostra spinner durante loading
- [ ] Não renderiza conteúdo protegido antes de verificar
- [ ] useAuth busca token do Keychain

### 6. SSO para Dashboard

#### Implementação
```typescript
// Botão "Ver no Dashboard"
const handleViewDashboard = async () => {
  const token = await authService.getToken();
  const ssoUrl = `${DASHBOARD_URL}/sso?token=${token}`;

  // Abre em nova aba
  window.open(ssoUrl, '_blank');
};
```

**Verificar**:
- [ ] Token é passado como query param
- [ ] HTTPS obrigatório (não HTTP)
- [ ] Token tem tempo de vida curto para SSO
- [ ] Dashboard valida token antes de autenticar

### 7. Logout

#### Implementação
```typescript
// auth.service.ts
async logout(): Promise<void> {
  try {
    // 1. Notifica backend
    await api.post('/logout');
  } catch (error) {
    console.error('Logout request failed:', error);
    // Continua mesmo se request falhar
  }

  // 2. Remove token do Keychain
  await invoke('delete_token');

  // 3. Limpa cache
  this.clearCache();

  // 4. Limpa store
  authStore.setUser(null);
  authStore.setAuthenticated(false);

  // 5. Redireciona
  router.push('/login');
}
```

**Verificar**:
- [ ] Notifica backend (revoga token)
- [ ] Remove token do Keychain
- [ ] Limpa cache em memória
- [ ] Limpa authStore
- [ ] Redireciona para /login
- [ ] Continua mesmo se backend falhar

---

## 🐛 Problemas Comuns

### 1. Token em localStorage
```typescript
// ❌ INSEGURO
localStorage.setItem('token', token);

// ✅ SEGURO
await invoke('store_token', { token });
```

### 2. Auto-Refresh Loop
```typescript
// ❌ Loop infinito
api.interceptors.response.use(null, async error => {
  if (error.response?.status === 401) {
    await authService.refreshToken(); // Sem _retry flag!
    return api(error.config); // Loop infinito!
  }
});

// ✅ Com flag _retry
if (error.response?.status === 401 && !originalRequest._retry) {
  originalRequest._retry = true;
  // ...
}
```

### 3. AuthGuard Renderiza Antes de Verificar
```typescript
// ❌ Renderiza protegido antes
if (!isAuthenticated) return <Navigate to="/login" />;
return <>{children}</>;

// ✅ Espera loading
if (loading) return <Spinner />;
if (!isAuthenticated) return null; // Não renderiza
return <>{children}</>;
```

---

## 🚀 Como Usar Este Agente

```
@autenticacao-specialist analise autenticação e segurança
@autenticacao-specialist foque em segurança de tokens
@autenticacao-specialist verifique auto-refresh
```

---

## 📚 Documentação

- [Authentication](../../api/authentication.md)
- [API Integration](../../api/api-integration.md)
- [Tauri Security](https://tauri.app/v1/guides/security/)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
