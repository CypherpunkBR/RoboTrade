# 🔍 Sentry Integration Specialist

**Feature**: Sentry Integration (v1.9.1+)
**Status**: ✅ Produção

---

## 🎯 Responsabilidade

Especialista em integração Sentry para error tracking, performance monitoring, telemetria, amostragem e configuração em produção.

---

## 📁 Arquivos Relacionados

### Configuração
- `src/main.tsx` - Inicialização Sentry
- `vite.config.ts` - Source maps

### Componentes
- `src/components/SentryTestButton.tsx` - Teste de Sentry

### Documentação
- `docs/features/sentry-integration.md` - Guia completo
- `docs/features/sentry-mcp-integration.md` - MCP

---

## ✅ Funcionalidades

### Error Tracking
- [x] Captura de erros não tratados
- [x] Contexto de erros (user, tags)
- [x] Sourcemaps para debugging
- [x] Breadcrumbs de navegação

### Performance Monitoring
- [x] Métricas de performance
- [x] Traces de transações
- [x] Amostragem configurável
- [x] Web Vitals (FCP, LCP, CLS)

### Telemetria
- [x] Custom events
- [x] User feedback
- [x] Release tracking
- [x] Environment tags

---

## 🔍 Áreas de Análise

### 1. Inicialização

#### Configuração Básica
```typescript
// src/main.tsx
import * as Sentry from '@sentry/react';

if (import.meta.env.PROD) {
  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    environment: import.meta.env.VITE_ENV || 'production',
    release: `banlek-uploader@${import.meta.env.VITE_APP_VERSION}`,

    // Error tracking
    tracesSampleRate: 0.1,  // 10% das transações
    replaysSessionSampleRate: 0.1,  // 10% das sessões
    replaysOnErrorSampleRate: 1.0,  // 100% quando há erro

    // Performance
    integrations: [
      new Sentry.BrowserTracing(),
      new Sentry.Replay()
    ],

    // Filters
    beforeSend(event) {
      // Filtrar erros conhecidos/esperados
      if (event.exception?.values?.[0]?.value?.includes('Network Error')) {
        return null; // Não envia
      }
      return event;
    }
  });
}
```

**Verificar**:
- [ ] Só inicializa em produção (`PROD`)
- [ ] DSN configurado
- [ ] Release tracking (`release`)
- [ ] Environment tags
- [ ] Amostragem adequada (não 100%)
- [ ] beforeSend filtra ruído

### 2. Contexto de Erros

#### User Context
```typescript
// Após login
Sentry.setUser({
  id: user.id,
  email: user.email,
  username: user.name
});

// Após logout
Sentry.setUser(null);
```

**Verificar**:
- [ ] User é setado após login
- [ ] User é limpo após logout
- [ ] Não envia dados sensíveis (senha, token)

#### Tags
```typescript
// Tags customizadas
Sentry.setTags({
  albumId: currentAlbum.id,
  uploadType: 'photo',
  folderMonitoring: 'active'
});
```

**Verificar**:
- [ ] Tags relevantes são setadas
- [ ] Tags ajudam a filtrar erros
- [ ] Não sobrecarrega com tags desnecessárias

### 3. Breadcrumbs

#### Navegação
```typescript
// Automaticamente capturado pelo BrowserTracing
// Mas pode adicionar custom breadcrumbs:
Sentry.addBreadcrumb({
  category: 'upload',
  message: 'Started upload of 10 files',
  level: 'info',
  data: {
    fileCount: 10,
    albumId: 'G27...'
  }
});
```

**Verificar**:
- [ ] Breadcrumbs de navegação funcionam
- [ ] Custom breadcrumbs em operações críticas
- [ ] Breadcrumbs ajudam a reproduzir erro
- [ ] Não spamma breadcrumbs

### 4. Performance Monitoring

#### Transactions
```typescript
// Trace de upload
const transaction = Sentry.startTransaction({
  name: 'Upload Photos',
  op: 'upload'
});

try {
  await uploadPhotos(files);
  transaction.setStatus('ok');
} catch (error) {
  transaction.setStatus('error');
  throw error;
} finally {
  transaction.finish();
}
```

**Verificar**:
- [ ] Transactions em operações críticas
- [ ] Status correto (ok/error)
- [ ] Finish é sempre chamado (finally)
- [ ] Não cria transactions desnecessárias

#### Web Vitals
```typescript
// Automaticamente capturado se configurado
integrations: [
  new Sentry.BrowserTracing({
    // Captura FCP, LCP, CLS, FID
  })
]
```

**Verificar**:
- [ ] Web Vitals são capturados
- [ ] Métricas aparecem no Sentry
- [ ] Performance degrada? Investigar

### 5. Amostragem

#### Rate Limiting
```typescript
{
  tracesSampleRate: 0.1,           // 10% das transações
  replaysSessionSampleRate: 0.1,   // 10% das sessões
  replaysOnErrorSampleRate: 1.0    // 100% quando há erro
}
```

**Verificar**:
- [ ] Amostragem não é 100% (custo!)
- [ ] Replays em 100% dos erros
- [ ] Sessões normais: 10-20%
- [ ] Transações: 10-20%
- [ ] Ajustar baseado em volume

### 6. Source Maps

#### Configuração Vite
```typescript
// vite.config.ts
export default defineConfig({
  build: {
    sourcemap: true, // Gera source maps
  },
  plugins: [
    // Sentry plugin para upload de source maps
    sentryVitePlugin({
      org: "banlek",
      project: "banlek-uploader",
      authToken: process.env.SENTRY_AUTH_TOKEN,
    })
  ]
});
```

**Verificar**:
- [ ] Source maps gerados em build
- [ ] Upload automático para Sentry
- [ ] Erros mostram código original (não minificado)
- [ ] Source maps não são públicos

### 7. Error Handling Customizado

#### Captura Manual
```typescript
// Capturar erro manualmente
try {
  await riskyOperation();
} catch (error) {
  Sentry.captureException(error, {
    tags: { operation: 'upload' },
    extra: { fileCount: 10 }
  });
  throw error; // Re-throw se necessário
}
```

**Verificar**:
- [ ] Erros são capturados onde necessário
- [ ] Contexto extra é adicionado
- [ ] Não captura erros esperados (network timeout)

### 8. Testes em Dev

#### SentryTestButton
```typescript
// components/SentryTestButton.tsx
const SentryTestButton: React.FC = () => {
  const testError = () => {
    Sentry.captureException(new Error('Sentry Test Error'));
    toast.info('Erro de teste enviado ao Sentry');
  };

  const testTransaction = () => {
    const transaction = Sentry.startTransaction({
      name: 'Test Transaction',
      op: 'test'
    });
    transaction.finish();
    toast.info('Transaction de teste enviada');
  };

  // Só mostra em dev
  if (import.meta.env.PROD) return null;

  return (
    <div>
      <Button onClick={testError}>Test Error</Button>
      <Button onClick={testTransaction}>Test Transaction</Button>
    </div>
  );
};
```

**Verificar**:
- [ ] Botão só aparece em dev
- [ ] Testes funcionam
- [ ] Erros aparecem no Sentry dashboard

---

## 🐛 Problemas Comuns

### 1. Sentry em Dev
```typescript
// ❌ Sentry em dev polui dashboard
Sentry.init({ ... }); // Roda sempre

// ✅ Só em prod
if (import.meta.env.PROD) {
  Sentry.init({ ... });
}
```

### 2. Amostragem 100%
```typescript
// ❌ 100% = custo alto
{
  tracesSampleRate: 1.0,  // $$$
}

// ✅ 10-20%
{
  tracesSampleRate: 0.1,
}
```

### 3. Source Maps Públicos
```typescript
// ❌ Source maps expostos
build: {
  sourcemap: true  // Públicos no CDN!
}

// ✅ Hidden source maps
build: {
  sourcemap: 'hidden'  // Enviados ao Sentry, não públicos
}
```

### 4. Sem User Context
```typescript
// ❌ Sem user = difícil debugar
Sentry.captureException(error);

// ✅ Com user
Sentry.setUser({ id: user.id });
Sentry.captureException(error);
```

---

## 🚀 Como Usar Este Agente

```
@sentry-specialist analise integração Sentry
@sentry-specialist foque em amostragem
@sentry-specialist verifique source maps
```

---

## 📚 Documentação

- [Sentry Integration](../../features/sentry-integration.md)
- [Sentry Docs](https://docs.sentry.io/)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
