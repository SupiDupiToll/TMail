import { test, expect } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

test.skip(!process.env.SHOTS, 'set SHOTS=1 to write review screenshots');

const out = resolve(process.cwd(), '../docs/review-shots');
mkdirSync(out, { recursive: true });

async function shot(page: import('@playwright/test').Page, name: string) {
  await page.screenshot({ path: resolve(out, `${name}.png`), fullPage: false });
}

test('capture review screens', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });

  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Sign in', level: 1 })).toBeVisible();
  await shot(page, 'login-light');

  await page.addInitScript(() => localStorage.setItem('tmail.theme', 'dark'));
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Sign in', level: 1 })).toBeVisible();
  await shot(page, 'login-dark');

  await page.addInitScript(() => localStorage.setItem('tmail.theme', 'light'));
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await shot(page, 'inbox-light');

  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await shot(page, 'reader-light');

  await page.keyboard.press('Escape');
  await page.keyboard.press('c');
  const composer = page.getByRole('dialog', { name: /New message/ });
  await expect(composer).toBeVisible();
  await composer.getByLabel('To').fill('ada@analytical.example');
  await composer.getByLabel('Subject').fill('Re: Analytical engines and the next release');
  await composer.getByLabel('Message body').fill('I read the proposal twice. The shape is right, but the replay needs an explicit cursor.\n\nLet us keep the cursor store and make the replay a separate step.');
  await shot(page, 'composer-light');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Discard' }).click();
  await expect(composer).toBeHidden();
  await page.locator('body').click({ position: { x: 5, y: 5 } });

  await page.keyboard.press('Control+k');
  await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
  const paletteInput = page.getByRole('combobox', { name: 'Search mail or run a command' });
  await expect(paletteInput).toBeVisible();
  await paletteInput.fill('quarterly');
  await expect(page.getByText(/Quarterly report/i).first()).toBeVisible();
  await shot(page, 'palette-light');
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  await shot(page, 'settings-light');
  await page.getByRole('button', { name: 'Close settings' }).click();

  await page.evaluate(() => (window as unknown as { __tmail: { setTheme: (t: string) => void } }).__tmail.setTheme('dark'));
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe('dark');
  await shot(page, 'inbox-dark');
  await page.getByRole('listbox', { name: /Inbox messages/ }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await shot(page, 'reader-dark');

  await page.keyboard.press('c');
  const darkComposer = page.getByRole('dialog', { name: /New message/ });
  await expect(darkComposer).toBeVisible();
  await darkComposer.getByLabel('To').fill('ada@analytical.example');
  await darkComposer.getByLabel('Subject').fill('Re: Analytical engines and the next release');
  await darkComposer.getByLabel('Message body').fill('Dark-mode compose preview.\n\nThe ledger stays calm at night.');
  await shot(page, 'composer-dark');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Discard' }).click();
  await expect(darkComposer).toBeHidden();
  await page.locator('body').click({ position: { x: 5, y: 5 } });

  await page.keyboard.press('Control+k');
  await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
  await page.getByRole('combobox', { name: 'Search mail or run a command' }).fill('quarterly');
  await expect(page.getByText(/Quarterly report/i).first()).toBeVisible();
  await shot(page, 'palette-dark');
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  await shot(page, 'settings-dark');
  await page.getByRole('button', { name: 'Close settings' }).click();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.evaluate(() => (window as unknown as { __tmail: { setTheme: (t: string) => void } }).__tmail.setTheme('light'));
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await shot(page, 'mobile-inbox');
  await page.getByRole('listbox', { name: /Inbox messages/ }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await shot(page, 'mobile-reader');
});
