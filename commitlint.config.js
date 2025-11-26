/**
 * CommitLint Configuration
 * Valida mensagens de commit seguindo Conventional Commits
 *
 * Formato: <type>(<scope>): <subject>
 *
 * Types permitidos:
 * - feat: Nova funcionalidade
 * - fix: Correção de bug
 * - docs: Documentação
 * - style: Formatação, sem mudança de código
 * - refactor: Refatoração
 * - perf: Melhoria de performance
 * - test: Adição/correção de testes
 * - build: Build system ou dependências
 * - ci: Configuração de CI
 * - chore: Outras mudanças que não modificam src ou test
 *
 * Exemplos válidos:
 * - feat(exchange): add binance websocket support
 * - fix(core): resolve memory leak in order processing
 * - perf(market-data): optimize tick aggregation
 * - test(trading): add benchmark for order execution
 */

export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Type é obrigatório e deve ser lowercase
    'type-enum': [
      2,
      'always',
      [
        'feat',      // Nova funcionalidade
        'fix',       // Correção de bug
        'docs',      // Documentação
        'style',     // Formatação, sem mudança de código
        'refactor',  // Refatoração
        'perf',      // Melhoria de performance
        'test',      // Adição/correção de testes
        'build',     // Build system ou dependências
        'ci',        // Configuração de CI
        'chore',     // Outras mudanças
        'revert',    // Reverte commit anterior
      ],
    ],
    // Subject deve começar com minúscula
    'subject-case': [2, 'always', 'lower-case'],
    // Subject não pode ser vazio
    'subject-empty': [2, 'never'],
    // Subject não pode terminar com ponto
    'subject-full-stop': [2, 'never', '.'],
    // Header (primeira linha) não pode ter mais de 100 caracteres
    'header-max-length': [2, 'always', 100],
    // Body deve ter linha em branco antes
    'body-leading-blank': [1, 'always'],
    // Footer deve ter linha em branco antes
    'footer-leading-blank': [1, 'always'],
  },
};
