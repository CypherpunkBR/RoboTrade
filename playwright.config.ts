import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for E2E tests
 *
 * IMPORTANT: Tauri apps require special handling:
 * - Only 1 worker (cannot run multiple app instances)
 * - Uses webServer to start app in dev mode
 * - Tests run against localhost:1420
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  // Test directory
  testDir: './e2e/tests',

  // Global setup/teardown
  globalSetup: './e2e/setup/global-setup.ts',
  globalTeardown: './e2e/setup/global-teardown.ts',

  // Run tests in files in parallel (but only 1 worker total for Tauri)
  fullyParallel: false,

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry on CI only
  retries: process.env.CI ? 2 : 0,

  // CRITICAL: Only 1 worker for Tauri apps (cannot run multiple instances)
  workers: 1,

  // Reporter to use (centralized in reports/)
  reporter: [
    ['html', { outputFolder: 'reports/e2e/html' }],
    ['json', { outputFile: 'reports/e2e/results.json' }],
    ['list'],
  ],

  // Shared settings for all projects
  use: {
    // Base URL for navigation
    baseURL: 'http://localhost:1421',

    // Collect trace when retrying failed test
    trace: 'on-first-retry',

    // Screenshots
    screenshot: 'only-on-failure',

    // Videos
    video: 'retain-on-failure',

    // Action timeout (click, fill, etc)
    actionTimeout: 10000,

    // Navigation timeout
    navigationTimeout: 30000,
  },

  // Configure projects for desktop browsers
  projects: [
    {
      name: 'setup',
      testMatch: /.*\.setup\.ts/,
      testDir: './e2e/setup',
    },
    {
      name: 'tauri-app',
      use: {
        ...devices['Desktop Chrome'],
        // Viewport size (Tauri default window size)
        viewport: { width: 1280, height: 800 },
        // Don't use storageState - each test configures its own auth
        // storageState: './e2e/.auth/user.json',
      },
      // Don't depend on setup - each test is self-contained
      // dependencies: ['setup'],
    },
  ],

  // Run local dev server before starting tests
  // Uses Vite in e2e mode with Tauri mocks for faster, more reliable e2e tests
  webServer: {
    command: 'pnpm dev:e2e',
    url: 'http://localhost:1421',
    reuseExistingServer: !process.env.CI,
    timeout: 30 * 1000, // 30 seconds for Vite to start
    stdout: 'pipe',
    stderr: 'pipe',
  },

  // Timeout for each test
  timeout: 60 * 1000, // 60 seconds

  // Expect timeout
  expect: {
    timeout: 10 * 1000, // 10 seconds
  },

  // Output folder for test artifacts (centralized in reports/)
  outputDir: 'reports/e2e/test-results',
});
