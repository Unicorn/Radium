import { test, expect, _electron as electron } from '@playwright/test';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const TAURI_BIN = resolve(
  __dirname,
  '../../../dist/target/release/radium-desktop',
);

test('main window loads and displays content', async () => {
  const app = await electron.launch({
    executablePath: TAURI_BIN,
    args: [],
  });

  const window = await app.firstWindow();
  await window.waitForLoadState('domcontentloaded');

  // Expect a title to be visible on the dashboard
  await expect(window.locator('h1', { hasText: 'Radium Desktop' })).toBeVisible();

  // Check if the connection status is displayed (assuming it starts as disconnected or connects automatically)
  await expect(window.locator('div', { hasText: 'Connection Status:' })).toBeVisible();

  await app.close();
});
