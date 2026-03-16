/**
 * Vitest global setup — starts the test_server binary before all tests and
 * stops it afterwards.
 *
 * If the server is already running (e.g. CI started it separately), this
 * module detects that and skips the start/stop lifecycle.
 *
 * If TAURI_TEST_TOKEN is not set, setup is a no-op and individual tests will
 * skip themselves via their `describe.skipIf(!isLive)` guard.
 */

import { spawn, type ChildProcess } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

// vitest.config.ts calls dotenv.config() which populates process.env in the main process,
// so these values are available here even though globalSetup runs outside worker threads.
const BASE_URL = process.env.TAURI_TEST_BASE_URL ?? 'http://localhost:3738';
const HEALTH_URL = `${BASE_URL}/health`;
const BINARY = join(
  process.cwd(),
  'src-tauri',
  'target',
  'debug',
  process.platform === 'win32' ? 'test_server.exe' : 'test_server',
);

let serverProcess: ChildProcess | undefined;

async function isServerReady(): Promise<boolean> {
  try {
    const res = await fetch(HEALTH_URL);
    return res.ok;
  } catch {
    return false;
  }
}

async function waitForServer(): Promise<void> {
  const retries = 30;
  const delayMs = 1000;
  for (let i = 0; i < retries; i++) {
    if (await isServerReady()) return;
    await new Promise<void>((resolve) => setTimeout(resolve, delayMs));
  }
  throw new Error(`Test server at ${HEALTH_URL} did not become ready after ${retries}s.`);
}

export async function setup(): Promise<void> {
  const token = process.env.TAURI_TEST_TOKEN;
  if (!token) return;

  // If the server is already up (CI manages it externally), do nothing.
  if (await isServerReady()) {
    console.log('[global-setup] Server already running — skipping spawn.');
    return;
  }

  if (!existsSync(BINARY)) {
    throw new Error(
      `Test server binary not found at:\n  ${BINARY}\n` +
        `Build it first with:\n  pnpm build:test-server`,
    );
  }

  console.log('[global-setup] Spawning test server…');
  const port = process.env.TEST_SERVER_PORT ?? '3738';
  serverProcess = spawn(BINARY, [], {
    env: { ...process.env, TAURI_AUTH_TOKEN: token, TEST_SERVER_PORT: port },
    stdio: 'inherit',
  });

  serverProcess.on('error', (err) => {
    console.error('[global-setup] Failed to spawn test server:', err.message);
  });

  await waitForServer();
  console.log('[global-setup] Server ready.');
}

export async function teardown(): Promise<void> {
  if (!serverProcess) return;
  console.log('[global-setup] Shutting down test server…');
  serverProcess.kill('SIGTERM');
  await new Promise<void>((resolve) => {
    serverProcess!.once('exit', resolve);
    // Safety net: resolve after 5 s even if the process doesn't exit cleanly.
    setTimeout(resolve, 5000);
  });
  serverProcess = undefined;
}
