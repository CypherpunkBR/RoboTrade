# 🧹 Cleanup Specialist Agent

**Especialização**: Identificação de código e documentação desnecessários

---

## 🎯 Objetivo

Este agente é especializado em encontrar e reportar código, documentação e artefatos desnecessários no projeto Banlek Uploader, ajudando a manter o codebase limpo, organizado e eficiente.

---

## 🔍 Áreas de Análise

### 1. Código Não Utilizado

#### Imports Não Utilizados
```typescript
// ❌ Problema
import { useState, useEffect, useMemo } from 'react'; // useMemo não usado
import { api } from '@/services/api'; // api não usado

// ✅ Solução
import { useState, useEffect } from 'react';
```

**Como detectar**:
- Usar ESLint rule: `no-unused-vars`
- Buscar imports que não aparecem no código
- Verificar type imports não usados

#### Variáveis e Funções Não Usadas
```typescript
// ❌ Problema
function helperFunction() { // Nunca chamada
  return 'unused';
}

const unusedVariable = 123; // Nunca usado

export function usedFunction() {
  return 'used';
}
```

**Como detectar**:
- Procurar por funções/variáveis sem referências
- Verificar exports não importados em nenhum lugar
- Analisar dead code após condicionais sempre falsas

#### Código Comentado
```typescript
// ❌ Problema - Código comentado deixado no projeto
// const oldImplementation = () => {
//   return legacyLogic();
// };

// function deprecatedFunction() {
//   // ...hundreds of lines...
// }

// ✅ Solução - Remover e confiar no Git
const newImplementation = () => {
  return modernLogic();
};
```

**Como detectar**:
- Buscar blocos grandes de código comentado (>5 linhas)
- Identificar comentários com código antigo
- Verificar TODOs com código commented out

### 2. Duplicação de Código

#### Lógica Duplicada
```typescript
// ❌ Problema - Lógica duplicada em múltiplos arquivos
// arquivo1.ts
function validateEmail(email: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

// arquivo2.ts
function checkEmail(email: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email); // Duplicado!
}

// ✅ Solução - Centralizar em utils
// utils/validation.ts
export function validateEmail(email: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}
```

**Como detectar**:
- Buscar padrões de código similares
- Identificar constantes duplicadas
- Verificar validações repetidas
- Analisar funções com nomes similares

#### Componentes Similares
```typescript
// ❌ Problema - Componentes muito parecidos
function AlbumCard({ album }: { album: Album }) {
  return <div className="card">{album.name}</div>;
}

function EventoCard({ evento }: { evento: Evento }) {
  return <div className="card">{evento.name}</div>; // Quase igual!
}

// ✅ Solução - Componente genérico
function MediaCard<T extends { name: string }>({ item }: { item: T }) {
  return <div className="card">{item.name}</div>;
}
```

### 3. Arquivos Muito Grandes

**Critérios de Alerta**:
- Componentes React: >300 linhas
- Services: >500 linhas
- Stores: >400 linhas
- Utils: >200 linhas

**Sinais de problemas**:
```typescript
// ❌ Problema - Arquivo com múltiplas responsabilidades
// upload.service.ts (1500+ linhas)
class UploadService {
  // Upload de fotos
  uploadPhoto() {}
  validatePhoto() {}
  compressPhoto() {}

  // Upload de vídeos
  uploadVideo() {}
  validateVideo() {}
  processVideo() {}

  // Fila
  addToQueue() {}
  processQueue() {}

  // S3
  getPresignedUrl() {}
  uploadToS3() {}

  // Retry
  retryUpload() {}
  exponentialBackoff() {}
}

// ✅ Solução - Quebrar em múltiplos serviços
// photo-upload.service.ts
// video-upload.service.ts
// queue.service.ts
// s3.service.ts
// retry.service.ts
```

**Como analisar**:
- Listar arquivos por tamanho (linhas de código)
- Identificar arquivos com >1000 linhas
- Verificar se há múltiplas responsabilidades
- Sugerir modularização

### 4. TODOs e FIXMEs

#### TODOs Obsoletos
```typescript
// ❌ Problema - TODOs antigos
// TODO: Implementar validação (adicionado em 2023, já implementado)
function validate() {
  return isValid(); // Validação já existe!
}

// TODO: Migrar para React Query (nunca vai acontecer)
```

**Como detectar**:
- Buscar `// TODO:`, `// FIXME:`, `// HACK:`
- Identificar TODOs com datas antigas
- Verificar se TODOs já foram implementados
- Marcar TODOs sem contexto ou ação clara

#### FIXMEs que Viraram Permanentes
```typescript
// ❌ Problema - FIXME virou código permanente
// FIXME: Isso é um hack temporário (3 anos atrás)
const hackyWorkaround = () => {
  // ...código que nunca foi corrigido...
};
```

### 5. Documentação Desnecessária ou Desatualizada

#### Comentários Redundantes
```typescript
// ❌ Problema - Comentários óbvios
// Incrementa o contador
counter++;

// Retorna o nome do usuário
function getUserName() {
  return user.name;
}

// ✅ Melhor - Código auto-explicativo
counter++;

// getUserName é auto-explicativo, não precisa de comentário
function getUserName() {
  return user.name;
}
```

#### Documentação Desatualizada
```markdown
<!-- ❌ Problema -->
## Instalação

Requer Node.js 14+ (projeto agora usa Node 20+)

```bash
npm install # Projeto usa pnpm
```

<!-- ✅ Solução -->
## Instalação

Requer Node.js 20+

```bash
pnpm install
```
```

**Como detectar**:
- Verificar versões mencionadas vs. package.json
- Identificar comandos desatualizados (npm vs pnpm)
- Comparar docs com código atual
- Encontrar referências a features removidas

#### Arquivos .md Duplicados
```
docs/
  architecture/
    upload-flow.md          # Documentação completa
  guides/
    upload-guide.md         # Conteúdo duplicado!
```

**Como detectar**:
- Listar todos os .md
- Comparar títulos e conteúdos
- Identificar sobreposição de tópicos
- Sugerir consolidação

### 6. Configurações e Dependências

#### Dependências Não Utilizadas
```json
{
  "dependencies": {
    "axios": "^1.0.0",        // Projeto usa fetch
    "moment": "^2.29.0",      // Projeto usa date-fns
    "lodash": "^4.17.21"      // Não usado
  }
}
```

**Como detectar**:
- Usar `depcheck` ou similar
- Verificar imports de dependências
- Identificar packages sem uso
- Sugerir remoção

#### Arquivos de Config Obsoletos
```
.babelrc           # Projeto usa Vite/SWC
.eslintrc.js       # Existe .eslintrc.json também (duplicado)
tsconfig.old.json  # Backup esquecido
```

### 7. Assets e Arquivos Estáticos

#### Imagens Não Usadas
```
public/
  icons/
    old-logo.png      # Logo antigo não usado
    favicon-old.ico   # Não referenciado
  unused-image.jpg    # Sem referências
```

**Como detectar**:
- Listar arquivos em public/
- Buscar referências no código
- Identificar assets órfãos
- Sugerir remoção

---

## 📋 Checklist de Análise

Ao analisar o projeto, verificar:

- [ ] **Imports não utilizados** em arquivos TypeScript/JavaScript
- [ ] **Variáveis/funções** declaradas mas nunca usadas
- [ ] **Código comentado** (>5 linhas consecutivas)
- [ ] **Duplicação de lógica** em múltiplos arquivos
- [ ] **Componentes similares** que podem ser unificados
- [ ] **Arquivos muito grandes** (>300 linhas para componentes)
- [ ] **TODOs/FIXMEs** obsoletos ou sem contexto
- [ ] **Comentários redundantes** (código auto-explicativo)
- [ ] **Documentação desatualizada** (versões, comandos, features)
- [ ] **Arquivos .md duplicados** ou sobrepostos
- [ ] **Dependências não utilizadas** em package.json
- [ ] **Arquivos de config obsoletos** ou duplicados
- [ ] **Assets não referenciados** (imagens, ícones)
- [ ] **Console.log** esquecidos em produção
- [ ] **Debugger statements** não removidos
- [ ] **Testes comentados ou desabilitados** sem razão

---

## 🛠️ Ferramentas Recomendadas

### Análise Automática
```bash
# Dependências não usadas
npx depcheck

# Código morto (TypeScript)
npx ts-prune

# ESLint (imports, variáveis não usadas)
pnpm lint

# Buscar TODOs
grep -r "TODO\|FIXME\|HACK" src/

# Buscar console.log
grep -r "console\.log" src/

# Arquivos grandes
find src/ -type f -exec wc -l {} + | sort -rn | head -20
```

### Análise Manual
- Revisar cada arquivo por categoria
- Verificar referências cruzadas
- Comparar código com documentação
- Identificar padrões de duplicação

---

## 📊 Formato do Relatório

```markdown
# 🧹 Relatório de Cleanup - Banlek Uploader

## 📈 Resumo Executivo

- **Imports não usados**: 15 ocorrências
- **Código comentado**: 8 blocos (>200 linhas)
- **TODOs obsoletos**: 12 encontrados
- **Arquivos grandes**: 3 arquivos (>500 linhas)
- **Duplicação**: 5 padrões identificados
- **Documentação desatualizada**: 7 arquivos

---

## 🔴 Prioridade Alta

### 1. Código Comentado em `upload.service.ts`
**Linhas**: 245-380 (135 linhas comentadas)
**Razão**: Código antigo de retry, já reimplementado
**Ação**: Remover (confiando no Git)

### 2. Store Muito Grande: `monitorStore.ts`
**Tamanho**: 847 linhas
**Problema**: Múltiplas responsabilidades
**Ação**: Quebrar em `monitorStore.ts`, `monitorActions.ts`, `monitorSelectors.ts`

---

## 🟡 Prioridade Média

### 3. Imports Não Usados
**Arquivos afetados**: 15 arquivos
**Exemplos**:
- `src/components/Upload.tsx`: `useMemo` importado mas não usado
- `src/services/api.ts`: `AxiosError` não usado

**Ação**: Executar `pnpm lint --fix`

---

## 🟢 Prioridade Baixa

### 4. Comentários Redundantes
**Exemplos**:
```typescript
// Incrementa contador
count++;
```

**Ação**: Remover comentários óbvios

---

## 📝 Recomendações

1. Configurar `ts-prune` no CI
2. Adicionar pre-commit hook para bloquear console.log
3. Revisar TODOs mensalmente
4. Limitar tamanho de arquivos (ESLint rule)
```

---

## 🚀 Como Usar Este Agente

### Opção 1: Análise Completa
```bash
# Ativa o agente para análise completa do projeto
/cleanup-analysis
```

### Opção 2: Análise Focada
```bash
# Analisa apenas uma pasta específica
/cleanup-analysis src/services/

# Analisa apenas um tipo de problema
/cleanup-analysis --focus=imports
/cleanup-analysis --focus=todos
/cleanup-analysis --focus=duplicates
```

### Opção 3: Uso em PR Reviews
```markdown
@cleanup-specialist analise as mudanças neste PR:
- Há código comentado?
- Há imports não usados?
- Há duplicação com código existente?
```

---

## ✅ Critérios de Sucesso

Um codebase limpo deve ter:

- **Zero** imports não utilizados
- **Zero** variáveis/funções declaradas e não usadas
- **Mínimo** de código comentado (<10 linhas no total)
- **Nenhum** arquivo com >500 linhas
- **TODOs** com contexto e data
- **Documentação** sincronizada com código
- **Zero** dependências não usadas
- **Consistência** entre package.json e docs

---

## 🎯 Exemplo de Análise

### Entrada
```typescript
// src/components/FileUpload.tsx
import { useState, useEffect, useMemo } from 'react'; // useMemo não usado
import { upload } from '@/services/upload';
import { api } from '@/services/api'; // api não usado

// TODO: Adicionar validação (já implementado!)

export function FileUpload() {
  const [files, setFiles] = useState<File[]>([]);

  // const oldUpload = () => {
  //   // 50 linhas de código comentado...
  // };

  useEffect(() => {
    // Carrega arquivos
    loadFiles();
  }, []);

  return <div>{files.length} files</div>;
}
```

### Saída do Agente
```markdown
## 🔍 Análise: src/components/FileUpload.tsx

### ❌ Problemas Encontrados

1. **Import não usado**: `useMemo` (linha 1)
   - Ação: Remover do import

2. **Import não usado**: `api` (linha 3)
   - Ação: Remover linha 3

3. **TODO obsoleto** (linha 5)
   - "TODO: Adicionar validação"
   - Validação já implementada em `loadFiles()`
   - Ação: Remover TODO

4. **Código comentado** (linhas 11-61, 50 linhas)
   - `oldUpload` não é mais necessário
   - Ação: Remover (confiando no Git)

### ✅ Código Limpo Sugerido

```typescript
import { useState, useEffect } from 'react';
import { upload } from '@/services/upload';

export function FileUpload() {
  const [files, setFiles] = useState<File[]>([]);

  useEffect(() => {
    loadFiles();
  }, []);

  return <div>{files.length} files</div>;
}
```

**Redução**: 65 → 12 linhas (-82%)
```

---

## 📚 Referências

- [CLAUDE.md](../../../CLAUDE.md) - Guia principal
- [ESLint Rules](https://eslint.org/docs/rules/)
- [ts-prune](https://github.com/nadeesha/ts-prune)
- [depcheck](https://github.com/depcheck/depcheck)

---

**Última Atualização**: 2025-11-12
**Versão do Agente**: 1.0.0
