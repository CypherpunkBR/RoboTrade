# Contributing Guide

Obrigado pelo interesse em contribuir com o RoboTrade! Este guia explica como participar do desenvolvimento.

## Code of Conduct

Esperamos que todos os contribuidores:

- Sejam respeitosos e inclusivos
- Aceitem críticas construtivas
- Foquem no que é melhor para a comunidade
- Mostrem empatia com outros membros

## Como Contribuir

### Reportando Bugs

1. **Verifique se já existe**: Busque nas [Issues](https://github.com/CypherpunkBR/RoboTrade/issues)
2. **Crie uma issue** com:
   - Título claro e descritivo
   - Passos para reproduzir
   - Comportamento esperado vs atual
   - Versão do RoboTrade e SO
   - Logs relevantes (sem dados sensíveis!)

Template:

```markdown
## Descrição do Bug

[Descrição clara do problema]

## Passos para Reproduzir

1. Vá para '...'
2. Clique em '...'
3. Veja o erro

## Comportamento Esperado

[O que deveria acontecer]

## Comportamento Atual

[O que realmente acontece]

## Ambiente

- RoboTrade: v1.0.0
- OS: macOS 14.0
- Rust: 1.75.0

## Logs

```
[Cole logs relevantes aqui]
```

## Screenshots

[Se aplicável]
```

### Sugerindo Features

1. **Verifique se já existe**: Busque nas Issues com label `enhancement`
2. **Crie uma issue** explicando:
   - O problema que a feature resolve
   - Proposta de solução
   - Alternativas consideradas
   - Impacto na arquitetura

### Contribuindo com Código

#### 1. Fork e Clone

```bash
# Fork via GitHub UI, depois:
git clone https://github.com/SEU_USERNAME/RoboTrade.git
cd RoboTrade
git remote add upstream https://github.com/CypherpunkBR/RoboTrade.git
```

#### 2. Crie uma Branch

```bash
# Atualize sua main
git checkout main
git pull upstream main

# Crie branch para sua feature/fix
git checkout -b feature/nome-da-feature
# ou
git checkout -b fix/descricao-do-bug
```

#### 3. Desenvolva

- Siga os [Coding Standards](./coding-standards.md)
- Escreva testes para código novo
- Atualize documentação se necessário

#### 4. Commit

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Formato: <type>(<scope>): <description>

# Types:
# - feat: Nova feature
# - fix: Bug fix
# - docs: Documentação
# - style: Formatação (não afeta código)
# - refactor: Refatoração
# - test: Testes
# - chore: Tarefas de manutenção

# Exemplos:
git commit -m "feat(analytics): add RSI indicator"
git commit -m "fix(order): handle rate limit error correctly"
git commit -m "docs(readme): update installation instructions"
git commit -m "test(strategy): add unit tests for SMA crossover"
```

#### 5. Push e Pull Request

```bash
git push origin feature/nome-da-feature
```

Depois, abra um Pull Request via GitHub com:

- Título seguindo Conventional Commits
- Descrição do que foi feito
- Link para issue relacionada (se houver)
- Screenshots (se mudança visual)

Template de PR:

```markdown
## Descrição

[Descrição clara das mudanças]

## Tipo de Mudança

- [ ] Bug fix
- [ ] Nova feature
- [ ] Breaking change
- [ ] Documentação

## Como Testar

1. [Passo 1]
2. [Passo 2]

## Checklist

- [ ] Código segue os coding standards
- [ ] Testes adicionados/atualizados
- [ ] Documentação atualizada
- [ ] CI passando

## Issues Relacionadas

Closes #123
```

## Processo de Review

1. **Automated Checks**: CI roda testes, linting, formatação
2. **Code Review**: Maintainers revisam o código
3. **Feedback**: Pode haver pedidos de mudanças
4. **Merge**: Após aprovação, o PR é mergeado

### Critérios de Review

- Código segue os padrões do projeto
- Testes cobrem casos importantes
- Não introduz vulnerabilidades de segurança
- Performance não é degradada
- Documentação está atualizada

## Áreas que Precisam de Ajuda

### Boas Primeiras Contribuições

Issues marcadas com `good first issue` são ideais para começar:

- Melhorias de documentação
- Testes adicionais
- Pequenos bug fixes
- Refatorações simples

### Áreas Prioritárias

- **Indicadores técnicos**: Implementar novos indicadores
- **Integrações**: Suporte a outras exchanges
- **UI/UX**: Melhorias na interface
- **Performance**: Otimizações de cálculo
- **Testes**: Aumentar cobertura

## Desenvolvimento Local

### Setup Completo

```bash
# Clone
git clone https://github.com/SEU_USERNAME/RoboTrade.git
cd RoboTrade

# Instale Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instale dependências do frontend
cd src-tauri && npm install && cd ..

# Rode em desenvolvimento
cargo tauri dev
```

### Workflow Recomendado

```bash
# Terminal 1: Watch de compilação
cargo watch -x check

# Terminal 2: Rode a aplicação
cargo tauri dev

# Antes de commit
cargo fmt --all
cargo clippy -- -D warnings
cargo test
```

### Debugging

```bash
# Com logs detalhados
RUST_LOG=debug cargo tauri dev

# Backtrace em erros
RUST_BACKTRACE=1 cargo tauri dev
```

## Comunicação

### Canais

- **GitHub Issues**: Bugs, features, discussões técnicas
- **GitHub Discussions**: Perguntas gerais, ideias
- **Pull Requests**: Revisão de código

### Dicas

- Seja claro e conciso
- Forneça contexto suficiente
- Responda feedback prontamente
- Agradeça pelos reviews

## Reconhecimento

Contribuidores são reconhecidos:

- No arquivo CONTRIBUTORS.md
- Nas release notes
- No README (contribuidores significativos)

## Licença

Ao contribuir, você concorda que suas contribuições serão licenciadas sob a mesma licença do projeto (MIT).

## Dúvidas?

- Abra uma Discussion no GitHub
- Pergunte na issue/PR relevante
- Consulte a documentação existente

Obrigado por contribuir!
