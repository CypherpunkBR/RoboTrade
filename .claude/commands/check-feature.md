# /check-feature

Ativa um agente especializado para analisar uma funcionalidade específica do Banlek Uploader.

## Uso

```
/check-feature <nome-da-feature>
```

## Features Disponíveis

### Funcionalidades Principais

1. **upload-fotos** - Upload de Fotos (v1.0.0+)
   - Validação, preview, pastas, S3, progress tracking
   - Especialista: `@upload-fotos-specialist`

2. **upload-videos** - Upload de Vídeos + Multipart (v1.5.0+, v1.6.0+)
   - Validação, multipart (>100MB), chunks 5MB, extração de duração
   - Especialista: `@upload-videos-specialist`

3. **monitoramento-pastas** - Monitoramento de Pastas (v1.4.0+)
   - Detecção automática, Fibonacci adaptativo, retry, deduplicação
   - Especialista: `@monitoramento-pastas-specialist`

4. **sistema-filas** - Sistema de Filas (v1.9.12+) ⚠️ Breaking Changes
   - Queue Manager centralizado, múltiplas filas, processamento automático
   - Especialista: `@sistema-filas-specialist`

5. **galeria** - Galeria Unificada (v1.6.0+)
   - MediaGallery com tabs, seleção múltipla, lazy loading
   - Especialista: `@galeria-specialist`

6. **integracao-macos** - Integração macOS (v1.3.0+)
   - Deep links, menu de contexto, Keychain, notificações
   - Especialista: `@integracao-macos-specialist`

7. **logs-performance** - Logs e Performance (v1.7.0+, v1.8.0)
   - Logs paginados, performance monitoring, 90% redução de RAM
   - Especialista: `@logs-performance-specialist`

8. **autenticacao** - Autenticação e Segurança (v1.0.0+)
   - Sanctum, Keychain, auto-refresh, SSO
   - Especialista: `@autenticacao-specialist`

9. **interface-ui** - Interface e UX (v1.0.0+, v1.9.0)
   - Design system, UX patterns, acessibilidade
   - Especialista: `@interface-ui-specialist`

10. **integracao-backend** - Integração com Backend (v1.0.0+)
    - API REST (30+ endpoints), streaming S3, Tauri commands
    - Especialista: `@integracao-backend-specialist`

11. **sentry** - Sentry Integration (v1.9.1+)
    - Error tracking, performance monitoring, telemetria
    - Especialista: `@sentry-specialist`

---

## Exemplos

### Analisar Upload de Fotos
```
/check-feature upload-fotos
```

**Resultado**: Ativa `@upload-fotos-specialist` que irá:
- Verificar validação de fotos
- Analisar upload streaming para S3
- Verificar hierarquia de pastas (tipo: 1)
- Checar progress tracking
- Identificar problemas (albumId como número, etc)

### Analisar Sistema de Filas
```
/check-feature sistema-filas
```

**Resultado**: Ativa `@sistema-filas-specialist` que irá:
- Verificar migração para v1.9.12 (breaking changes)
- Analisar Queue Manager centralizado
- Verificar que `uploadStore.queue` foi removido
- Checar processamento automático
- Validar uso de queueManager diretamente

### Analisar Integração macOS
```
/check-feature integracao-macos
```

**Resultado**: Ativa `@integracao-macos-specialist` que irá:
- Verificar deep links (banlek://)
- Analisar armazenamento no Keychain
- Checar menu de contexto no Finder
- Verificar notificações nativas
- Confirmar que tokens NUNCA estão em localStorage

---

## Sem Argumentos

Se chamado sem argumentos, lista todas as features disponíveis:

```
/check-feature
```

**Saída**:
```
📋 Features Disponíveis para Análise:

1. upload-fotos - Upload de Fotos
2. upload-videos - Upload de Vídeos + Multipart
3. monitoramento-pastas - Monitoramento de Pastas
4. sistema-filas - Sistema de Filas (v1.9.12 Breaking Changes)
5. galeria - Galeria Unificada
6. integracao-macos - Integração macOS
7. logs-performance - Logs e Performance
8. autenticacao - Autenticação e Segurança
9. interface-ui - Interface e UX
10. integracao-backend - Integração com Backend
11. sentry - Sentry Integration

Uso: /check-feature <nome-da-feature>
Exemplo: /check-feature upload-videos
```

---

## Análise Focada

Você pode especificar foco na análise:

```
/check-feature upload-videos --focus=multipart
/check-feature sistema-filas --focus=migration
/check-feature autenticacao --focus=security
```

---

## Notas

- Cada agente tem conhecimento profundo da funcionalidade
- Agentes verificam conformidade com padrões do projeto
- Agentes identificam problemas comuns e anti-patterns
- Agentes sugerem melhorias e otimizações

---

**Última Atualização**: 2025-11-12
**Versão**: 1.0.0
