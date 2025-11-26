# 🍎 Integração macOS Specialist

**Feature**: Integração macOS (v1.3.0+)
**Status**: ✅ Produção
**Testes**: 18 casos

---

## 🎯 Responsabilidade

Especialista em integração com macOS incluindo Deep Links (banlek://), menu de contexto no Finder, Quick Actions, Keychain (armazenamento seguro de tokens) e notificações nativas.

---

## 📁 Arquivos Relacionados

### Tauri (Rust)
- `src-tauri/src/main.rs` - Comandos Keychain
- `src-tauri/tauri.conf.json` - Configuração de Deep Links

### Serviços
- `src/services/auth.service.ts` - Integração com Keychain

### Hooks
- `src/hooks/useDeepLink.ts` - Handler de Deep Links
- `src/hooks/useCliArgs.ts` - Argumentos CLI

### Scripts
- `scripts/install-context-menu.sh` - Instalação de menu de contexto

### Documentação
- `docs/features/context-menu.md` - Guia completo

### Testes
- `tests/integration/deep-links.test.ts` - 18 casos

---

## ✅ Funcionalidades

### Deep Links
- [x] Esquema `banlek://upload?files=...&album=...`
- [x] Integração com Finder (Quick Action)
- [x] Auto-abrir app se fechado
- [x] Parsing de argumentos
- [x] Validação de parâmetros

### Keychain
- [x] Armazenamento seguro de tokens
- [x] Comandos Rust: `store_token`, `get_token`, `delete_token`
- [x] Cache otimizado (pedido 1x por sessão)
- [x] Migração de localStorage

### Menu de Contexto
- [x] "Enviar para Banlek" no Finder
- [x] Seleção múltipla de arquivos
- [x] Quick Actions integradas

### Notificações
- [x] Notificações nativas do sistema
- [x] Progress notifications
- [x] Completion notifications

---

## 🔍 Áreas de Análise

### 1. Deep Links

#### Configuração (tauri.conf.json)
```json
{
  "tauri": {
    "bundle": {
      "macOS": {
        "urlSchemes": ["banlek"]
      }
    }
  }
}
```

**Verificar**:
- [ ] Esquema `banlek://` registrado no sistema
- [ ] App abre automaticamente ao clicar em deep link
- [ ] Deep link funciona com app fechado

#### Parsing de URL
```typescript
// hooks/useDeepLink.ts
const parseDeepLink = (url: string): DeepLinkData | null => {
  // banlek://upload?files=/path/to/file.jpg&album=G27...
  const match = url.match(/^banlek:\/\/upload\?(.+)$/);
  if (!match) return null;

  const params = new URLSearchParams(match[1]);
  return {
    files: params.get('files')?.split(',') || [],
    album: params.get('album'),
    folder: params.get('folder') ? parseInt(params.get('folder')) : undefined
  };
};
```

**Verificar**:
- [ ] Parsing de múltiplos arquivos (separados por vírgula)
- [ ] Parsing de albumId (string criptografada)
- [ ] Parsing de folderId (opcional)
- [ ] Validação de parâmetros
- [ ] Mensagem de erro clara se inválido

#### Handler
```typescript
useEffect(() => {
  const unlisten = listen('deep-link', (event: DeepLinkEvent) => {
    const data = parseDeepLink(event.payload);
    if (data) {
      handleDeepLinkUpload(data);
    }
  });

  return () => { unlisten(); };
}, []);
```

**Verificar**:
- [ ] Listener é registrado uma vez
- [ ] Cleanup ao desmontar
- [ ] Redireciona para página correta
- [ ] Pre-seleciona álbum/pasta

### 2. Keychain (Armazenamento Seguro)

#### Comandos Rust
```rust
// src-tauri/src/main.rs
use keyring::Entry;

#[tauri::command]
async fn store_token(token: String) -> Result<(), String> {
    let entry = Entry::new("banlek-uploader", "auth_token")
        .map_err(|e| e.to_string())?;

    entry.set_password(&token)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_token() -> Result<String, String> {
    let entry = Entry::new("banlek-uploader", "auth_token")
        .map_err(|e| e.to_string())?;

    let token = entry.get_password()
        .map_err(|e| e.to_string())?;

    Ok(token)
}

#[tauri::command]
async fn delete_token() -> Result<(), String> {
    let entry = Entry::new("banlek-uploader", "auth_token")
        .map_err(|e| e.to_string())?;

    entry.delete_password()
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**Verificar**:
- [ ] `store_token` salva no Keychain
- [ ] `get_token` recupera do Keychain
- [ ] `delete_token` remove do Keychain
- [ ] Erros são tratados adequadamente
- [ ] Service name: `banlek-uploader`
- [ ] Account name: `auth_token`

#### Integração no Frontend
```typescript
// services/auth.service.ts
import { invoke } from '@tauri-apps/api/tauri';

class AuthService {
  async storeToken(token: string): Promise<void> {
    try {
      await invoke('store_token', { token });
    } catch (error) {
      console.error('Failed to store token:', error);
      throw new Error('Não foi possível armazenar o token com segurança');
    }
  }

  async getToken(): Promise<string | null> {
    try {
      return await invoke<string>('get_token');
    } catch (error) {
      // Token não encontrado (primeira vez)
      return null;
    }
  }

  async deleteToken(): Promise<void> {
    try {
      await invoke('delete_token');
    } catch (error) {
      console.error('Failed to delete token:', error);
    }
  }
}
```

**Verificar**:
- [ ] Nunca usa `localStorage` para tokens
- [ ] Só usa Keychain (via Tauri)
- [ ] Cache em memória para performance (1x por sessão)
- [ ] Limpa cache ao fazer logout
- [ ] Erro claro se Keychain falhar

### 3. Menu de Contexto

#### Instalação
```bash
# scripts/install-context-menu.sh
#!/bin/bash

# Cria Quick Action para "Enviar para Banlek"
AUTOMATOR_PATH="$HOME/Library/Services/Enviar para Banlek.workflow"

# Cria workflow Automator
# ... (código de criação do workflow)

# Registra no Finder
# ... (código de registro)

echo "✅ Menu de contexto instalado com sucesso!"
```

**Verificar**:
- [ ] Script de instalação funciona
- [ ] Menu aparece no Finder ao clicar com botão direito
- [ ] Funciona com seleção múltipla de arquivos
- [ ] Abre app com deep link correto

#### Quick Actions
```applescript
-- Quick Action script
on run {input, parameters}
    set fileList to {}
    repeat with aFile in input
        set end of fileList to POSIX path of aFile
    end repeat

    set filesParam to my join(fileList, ",")
    set deepLink to "banlek://upload?files=" & filesParam

    do shell script "open " & quoted form of deepLink

    return input
end run
```

**Verificar**:
- [ ] Quick Action criada corretamente
- [ ] Múltiplos arquivos são concatenados com vírgula
- [ ] Deep link é aberto corretamente
- [ ] App é iniciado se fechado

### 4. Notificações Nativas

#### macOS Notifications
```typescript
// services/notification.service.ts
import { sendNotification } from '@tauri-apps/api/notification';

class NotificationService {
  async notifyUploadComplete(count: number) {
    await sendNotification({
      title: 'Upload Concluído',
      body: `${count} arquivo(s) enviado(s) com sucesso!`,
      icon: 'success'
    });
  }

  async notifyUploadFailed(count: number) {
    await sendNotification({
      title: 'Erro no Upload',
      body: `${count} arquivo(s) falharam. Clique para ver detalhes.`,
      icon: 'error'
    });
  }
}
```

**Verificar**:
- [ ] Notificações aparecem no Notification Center
- [ ] Ícones corretos (success, error, info)
- [ ] Click em notificação abre app
- [ ] Não spamma notificações (batch de 5s)

---

## 🐛 Problemas Comuns

### 1. Deep Link Não Abre App
```json
// ❌ Esquema não registrado
{
  "tauri": {
    "bundle": {
      // Faltando urlSchemes
    }
  }
}

// ✅ Esquema registrado
{
  "tauri": {
    "bundle": {
      "macOS": {
        "urlSchemes": ["banlek"]
      }
    }
  }
}
```

### 2. Token em localStorage
```typescript
// ❌ INSEGURO - localStorage
localStorage.setItem('token', token);

// ✅ SEGURO - Keychain
await invoke('store_token', { token });
```

### 3. Notificações Repetidas
```typescript
// ❌ Notifica a cada arquivo
files.forEach(f => notifyUploadComplete(f));

// ✅ Batch notification
await uploadAll(files);
notifyUploadComplete(files.length);
```

---

## 🚀 Como Usar Este Agente

```
@integracao-macos-specialist analise integração macOS
@integracao-macos-specialist foque em deep links
@integracao-macos-specialist verifique segurança do Keychain
```

---

## 📚 Documentação

- [Context Menu Guide](../../features/context-menu.md)
- [Tauri Security](https://tauri.app/v1/guides/security/)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
