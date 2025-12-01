# Pull Request

## 📝 Descrição

<!-- Descreva as mudanças implementadas neste PR -->

## 🎯 Tipo de Mudança

<!-- Marque os itens aplicáveis -->

- [ ] 🐛 Bug fix (mudança que corrige um problema)
- [ ] ✨ Nova feature (mudança que adiciona funcionalidade)
- [ ] 🔨 Refatoração (mudança que não adiciona feature nem corrige bug)
- [ ] 📚 Documentação (mudanças apenas em documentação)
- [ ] 🧪 Testes (adiciona ou melhora testes)
- [ ] ⚡ Performance (melhoria de performance)
- [ ] 🔒 Segurança (correção de vulnerabilidade ou melhoria de segurança)

## 🔗 Issues Relacionadas

Closes #
Relates to #

## 🧪 Checklist de Testes

### Testes Unitários
- [ ] Testes unitários adicionados/atualizados
- [ ] Todos os testes unitários passando (`cargo test --lib`)
- [ ] Cobertura ≥80% nas áreas modificadas

### Testes de Integração
- [ ] Testes de integração adicionados (se aplicável)
- [ ] Todos os testes de integração passando

## ✅ Checklist de Qualidade

### Código
- [ ] Código segue convenções do projeto
- [ ] Sem `println!` ou `print!` (usar `tracing`)
- [ ] Sem `unwrap()` em código de produção
- [ ] Variáveis com nomes descritivos

### Formatação & Linting
- [ ] `cargo fmt` executado
- [ ] `cargo clippy` sem warnings
- [ ] Sem warnings de compilação

### Segurança
- [ ] Sem API keys hardcoded
- [ ] Secrets usam `secrecy::Secret<T>`
- [ ] Validação de entrada apropriada

### Documentação
- [ ] Docstrings em funções públicas
- [ ] README atualizado (se necessário)
- [ ] CHANGELOG.md atualizado

## 📊 Cobertura

```
TOTAL Coverage: XX.XX%
```

## 🧠 Notas para Revisores

<!-- Informações adicionais para facilitar a revisão -->
