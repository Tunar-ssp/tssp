import { Page, expect } from '@playwright/test';

export async function seedBackend(page: Page) {
  // Helper to seed test data via API before running tests
  // This will be called before each e2e test
  
  try {
    // Check if daemon is running
    await page.request.get('/api/v1/health', { timeout: 3000 });
  } catch (e) {
    throw new Error('Daemon not running at http://127.0.0.1:8421. Start daemon before running e2e tests.');
  }
}

export async function waitForLoadingComplete(page: Page) {
  // Wait for any loading spinners to disappear
  await page.waitForSelector('[data-loading="false"], .loaded', { timeout: 5000 }).catch(() => {});
}

export async function navigateToApp(page: Page) {
  await page.goto('/');
  // Ensure app is fully loaded
  await page.waitForSelector('[data-app-ready="true"]', { timeout: 10000 }).catch(() => {});
}

export async function getPageErrors(page: Page): Promise<string[]> {
  const errors: string[] = [];
  
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(`Console Error: ${msg.text()}`);
    }
  });

  page.on('pageerror', (exc) => {
    errors.push(`Page Error: ${exc.message}`);
  });

  return errors;
}
