import { defineConfig } from 'vitest/config';
import { config } from 'dotenv';

// Load .env.test so TAURI_TEST_TOKEN is available without manual export.
// Using test.env (not process.env assignment) ensures the value is injected
// into every vitest worker process, not just the main config process.
// CI passes the same value via the TAURI_TEST_TOKEN secret directly.
const { parsed: envFile = {} } = config({ path: '.env.test' });

export default defineConfig({
  esbuild: {
    tsconfigRaw: {
      compilerOptions: {
        target: 'ES2022',
        module: 'ESNext',
        moduleResolution: 'bundler',
        allowJs: true,
        esModuleInterop: true,
        forceConsistentCasingInFileNames: true,
        resolveJsonModule: true,
        skipLibCheck: true,
        strict: true,
        noUncheckedIndexedAccess: true,
        noImplicitOverride: true,
        exactOptionalPropertyTypes: true,
      },
    },
  },
  test: {
    include: ['e2e/**/*.test.ts'],
    testTimeout: 15000,
    globals: true,
    env: envFile,
    globalSetup: ['./e2e/global-setup.ts'],
  },
});
