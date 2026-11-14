import { test, expect } from '@playwright/test';

test('inbox renders and the list is keyboard reachable', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });

  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await expect(list.getByRole('option').first()).toBeVisible();
  await expect(page.getByText('preview', { exact: true })).toBeVisible();

  await list.focus();
  await page.keyboard.press('j');
  await expect(page.getByRole('heading', { level: 2 }).first()).toBeVisible();

  expect(errors).toEqual([]);
});

test('command palette opens with Ctrl+K and searches', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('Control+k');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await expect(palette).toBeVisible();
  const input = palette.getByRole('combobox', { name: 'Search mail or run a command' });
  await expect(input).toBeFocused();
  await input.fill('quarterly');
  await expect(palette.getByText(/Quarterly report/i).first()).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(palette).toBeHidden();
});

test('composer opens, validates recipients, and closes', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('c');
  const dialog = page.getByRole('dialog', { name: /New message/ });
  await expect(dialog).toBeVisible();
  const send = dialog.getByRole('button', { name: /^Send/ });
  await expect(send).toBeDisabled();
  await dialog.getByLabel('To').fill('ada@example.com');
  await dialog.getByLabel('Subject').fill('Hello');
  await dialog.getByLabel('Message body').fill('A short message.');
  await expect(send).toBeEnabled();
  await page.keyboard.press('Escape');
  await expect(page.getByText('Discard this message?')).toBeVisible();
  await page.getByRole('button', { name: 'Keep editing' }).click();
  await page.getByRole('button', { name: 'Close composer' }).click();
  await page.getByRole('button', { name: 'Discard' }).click();
  await expect(dialog).toBeHidden();
});

test('sign-in form reaches the mailbox with valid input', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Sign in', level: 1 })).toBeVisible();
  await page.getByLabel('Mail address').fill('you@tuta.com');
  await page.getByLabel('Password', { exact: true }).fill('correct horse battery staple');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await expect(page.getByRole('listbox', { name: /Inbox messages/ })).toBeVisible();
});

test('composer selects an alias and explains encryption state', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('c');
  const dialog = page.getByRole('dialog', { name: /New message/ });
  await expect(dialog).toBeVisible();
  const from = dialog.getByLabel('From');
  await expect(from).toHaveValue('you@tuta.com');
  await from.selectOption('hello@aliases.example');
  await expect(from).toHaveValue('hello@aliases.example');

  await dialog.getByLabel('To').fill('ada@tuta.com');
  await expect(dialog.getByText('Encrypted', { exact: true })).toBeVisible();

  await dialog.getByLabel('To').fill('ada@example.com');
  await expect(dialog.getByText('Not encrypted', { exact: true })).toBeVisible();
  await expect(dialog.getByText(/Password-protected external mail is not available yet/)).toBeVisible();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Discard' }).click();
});
