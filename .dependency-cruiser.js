export default {
  forbidden: [
    // no circular dependencies
    { name: 'no-circular', severity: 'warn', from: {}, to: { circular: true } },
    // no orphan modules
    { name: 'no-orphans', severity: 'warn', from: { orphan: true }, to: {} },
    // forbid direct fs in frontend code
    {
      name: 'no-fs-in-frontend',
      severity: 'warn',
      from: { path: '^src/' },
      to: { path: '^(node:)?fs', dependencyTypes: [ 'core' ] }
    },
    // keep services isolated from components
    {
      name: 'no-component-to-service-cycles',
      severity: 'warn',
      from: { path: '^src/components/' },
      to: { path: '^src/components/', circular: true }
    }
  ],
  options: {
    doNotFollow: { path: 'node_modules' },
    tsPreCompilationDeps: true,
    combinedDependencies: true,
    reporterOptions: { dot: { collapsePattern: 'node_modules/[^/]+|src/(components|services|store|utils|hooks)/' } }
  }
};
