# Agentes de Desenvolvimento com IA

Esta pasta contém as definições de agentes especializados para desenvolvimento assistido por IA no Banlek Uploader.

## Estrutura

### Agentes de Workflow (7 agentes)

Agentes gerais para workflows de desenvolvimento:

- **[documentador.md](./documentador.md)** - Documentação e manutenção de docs
- **[product-owner.md](./product-owner.md)** - Planejamento e backlog
- **[desenvolvedor.md](./desenvolvedor.md)** - Implementação de features
- **[tester.md](./tester.md)** - Testes automatizados
- **[qa.md](./qa.md)** - Validação manual e exploratória
- **[revisor.md](./revisor.md)** - Padrões de código e qualidade
- **[release-manager.md](./release-manager.md)** - Releases e versionamento

### Agentes Especializados Globais (3 agentes)

Agentes para análise de qualidade de código:

- **[cleanup-specialist.md](./specialists/cleanup-specialist.md)** - Código e documentação desnecessários
- **[react-best-practices.md](./specialists/react-best-practices.md)** - Boas práticas React e TypeScript
- **[performance-optimizer.md](./specialists/performance-optimizer.md)** - Performance e otimizações

### Agentes por Funcionalidade (11 agentes)

Agentes especializados em funcionalidades específicas:

- **[upload-fotos-specialist.md](./features/upload-fotos-specialist.md)** - Upload de Fotos
- **[upload-videos-specialist.md](./features/upload-videos-specialist.md)** - Upload de Vídeos + Multipart
- **[monitoramento-pastas-specialist.md](./features/monitoramento-pastas-specialist.md)** - Monitoramento de Pastas
- **[sistema-filas-specialist.md](./features/sistema-filas-specialist.md)** - Sistema de Filas (v1.9.12)
- **[galeria-specialist.md](./features/galeria-specialist.md)** - Galeria Unificada
- **[integracao-macos-specialist.md](./features/integracao-macos-specialist.md)** - Integração macOS
- **[logs-performance-specialist.md](./features/logs-performance-specialist.md)** - Logs e Performance
- **[autenticacao-specialist.md](./features/autenticacao-specialist.md)** - Autenticação e Segurança
- **[interface-ui-specialist.md](./features/interface-ui-specialist.md)** - Interface e UX
- **[integracao-backend-specialist.md](./features/integracao-backend-specialist.md)** - Integração com Backend
- **[sentry-specialist.md](./features/sentry-specialist.md)** - Sentry Integration

## Como Usar

### Seleção de Agente

Ao trabalhar em uma tarefa, selecione o agente apropriado baseado nos **Entry Points** ou **Triggers** descritos em cada arquivo.

#### Agentes de Workflow
```
"@documentador atualizar docs da API de upload"
"@dev implementar filtro por data"
"@tester criar testes para monitor de pastas"
"@qa validar fluxo de upload completo"
"@revisor checar qualidade do código novo"
"@release preparar versão 1.5.0"
```

#### Agentes Especializados Globais
```
"@cleanup-specialist analise código desnecessário"
"@react-best-practices verifique hooks e componentes"
"@performance-optimizer analise re-renders"
```

#### Agentes por Funcionalidade
```
"@upload-fotos-specialist analise validação de fotos"
"@upload-videos-specialist verifique multipart upload"
"@sistema-filas-specialist analise migração v1.9.12"
"@autenticacao-specialist verifique segurança de tokens"
```

### Comandos Slash

Atalhos para ativar agentes:

- **`/check-quality [path]`** - Ativa os 3 agentes globais em paralelo
- **`/check-feature <nome>`** - Ativa agente específico de funcionalidade

**Exemplos:**
```
/check-quality src/services/
/check-feature upload-videos
/check-feature sistema-filas --focus=migration
```

### Workflows

Os agentes colaboram entre si seguindo workflows específicos (ver seção "Workflows de Colaboração" em cada arquivo).

#### Workflow de Desenvolvimento
```
PO → Documentador → Dev → Revisor → Tester → QA → Release Manager
```

#### Workflow de Análise de Qualidade
```
/check-quality → Cleanup + React Best Practices + Performance → Relatório Consolidado
```

#### Workflow de Análise de Feature
```
/check-feature <nome> → Specialist da Feature → Relatório Específico
```

#### Workflow de Code Review
```
PR aberto → Revisor → Specialists relevantes → Aprovação/Correções
```

## Princípios

1. **Especialização**: Cada agente tem responsabilidade bem definida
2. **Guardrails**: Regras específicas do projeto são enforçadas
3. **Colaboração**: Agentes trabalham em sequência, passando contexto
4. **Qualidade**: Gates de qualidade obrigatórios em cada etapa
5. **Paralelização**: Agentes globais executam em paralelo para eficiência
6. **Foco**: Agentes de funcionalidade analisam aspectos específicos do sistema

## Regras Críticas do Projeto

Todos os agentes devem seguir:
- ✅ hashId em APIs de upload
- ✅ id numérico apenas internamente
- ✅ Tokens no Keychain (Tauri), nunca localStorage
- ✅ Deep links validados
- ✅ TypeScript strict (sem any)
- ✅ Cobertura >= 80% (atual: 100%)

## Casos de Uso

### 1. Antes de Release
```bash
/check-quality                          # Análise completa
/check-feature upload-videos            # Verifica features críticas
/check-feature sistema-filas
/check-feature autenticacao
```

### 2. Code Review de PR
```bash
@revisor analise este PR
/check-quality src/path/modified/       # Foca no que mudou
```

### 3. Debugging de Performance
```bash
@performance-optimizer analise re-renders em PhotoGallery
/check-feature galeria --focus=performance
```

### 4. Refatoração Grande
```bash
/check-quality src/services/            # Antes
# ... faz refatoração ...
/check-quality src/services/            # Depois (compara)
```

### 5. Análise de Segurança
```bash
@autenticacao-specialist verifique tokens
@integracao-macos-specialist confirme Keychain
/check-feature autenticacao --focus=security
```

---

## Estatísticas

- **Total de agentes**: 21
  - Workflow: 7
  - Globais: 3
  - Por funcionalidade: 11
- **Comandos slash**: 2
- **Cobertura de funcionalidades**: 11 funcionalidades principais
- **Versão**: v1.10.0

---

## Referências

- [CLAUDE.md](../../CLAUDE.md) - Guia completo do projeto
- [commands/](../commands/) - Comandos slash disponíveis
- [features/README.md](../../docs/features/README.md) - Documentação de funcionalidades
- Seção "Operational Playbook" no CLAUDE.md para RACI, gates e checklists

---

**Última Atualização**: 2025-11-12
**Versão**: 1.0.0

