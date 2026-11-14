import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

async function audit(page: import('@playwright/test').Page, label: string) {
  const results = await new AxeBuilder({ page })
    // Mail bodies are untrusted third-party HTML; axe must judge our chrome,
    // not a sender's markup (which we deliberately do not restyle).
    .exclude('iframe')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
    .analyze();
  const serious = results.violations.filter(
    (violation) => violation.impact === 'serious' || violation.impact === 'critical'
  );
  if (serious.length) {
    throw new Error(
      `${label}: ${serious
        .map(
          (violation) =>
            `${violation.id}: ${violation.nodes
              .map((node) => node.target.join(' ') + ' ' + (node.failureSummary ?? '').replace(/\n/g, ' '))
              .join(' | ')}`
        )
        .join(' ;; ')}`
    );
  }
  return results;
}

test('login screen has no serious accessibility violations', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Sign in', level: 1 })).toBeVisible();
  await audit(page, 'login');
});

test('mail workspace has no serious accessibility violations', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await audit(page, 'inbox');
  await page.getByRole('listbox', { name: /Inbox messages/ }).focus();
  await page.keyboard.press('j');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await expect(page.getByRole('region', { name: 'Message', exact: true })).toBeVisible();
  await audit(page, 'reader');
});

test('composer and settings have no serious accessibility violations', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('c');
  await expect(page.getByRole('dialog', { name: /New message/ })).toBeVisible();
  await audit(page, 'composer');
  await page.getByRole('button', { name: 'Close composer' }).click();
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  await audit(page, 'settings');
});

test('landmarks and heading order are coherent', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await expect(page.getByRole('complementary', { name: 'Mailbox' })).toBeVisible();
  const headings = await page
    .locator('h1, h2, h3, h4, h5, h6')
    .evaluateAll((nodes) => nodes.map((node) => Number(node.tagName.slice(1))));
  let previous = 0;
  for (const level of headings) {
    expect(level).toBeLessThanOrEqual(previous + 1);
    previous = level;
  }
});
