import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('dark theme', () => {
  test.use({ colorScheme: 'dark' });

  test('renders dark, keeps contrast and does not overflow', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('tmail.theme', 'dark');
    });
    await page.goto('/?mock=inbox');
    await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
    const theme = await page.evaluate(() => document.documentElement.dataset.theme);
    expect(theme).toBe('dark');

    const results = await new AxeBuilder({ page })
      .exclude('iframe')
      .withTags(['wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze();
    const serious = results.violations.filter(
      (violation) => violation.impact === 'serious' || violation.impact === 'critical'
    );
    expect(serious, serious.map((v) => v.id).join(', ')).toEqual([]);

    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth
    );
    expect(overflow).toBeLessThanOrEqual(1);
  });
});

test.describe('HiDPI and text scaling', () => {
  test.use({ deviceScaleFactor: 1.5, viewport: { width: 1280, height: 800 } });

  test('1.5x scale does not overflow or clip rows', async ({ page }) => {
    await page.goto('/?mock=inbox');
    const list = page.getByRole('listbox', { name: /Inbox messages/ });
    await expect(list).toBeVisible();
    const box = await list.getByRole('option').first().boundingBox();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(43);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth
    );
    expect(overflow).toBeLessThanOrEqual(1);
  });

  test('200% text keeps every primary action reachable', async ({ page }) => {
    await page.goto('/?mock=inbox');
    await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
    await page.evaluate(() => {
      document.documentElement.style.fontSize = '32px';
    });
    await expect(page.getByRole('button', { name: /Compose/ }).first()).toBeVisible();
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth
    );
    expect(overflow).toBeLessThanOrEqual(1);
  });
});

test('archive is undoable through the real move service', async ({ page }) => {
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  const before = Number(await page.locator('.count').innerText());
  await page.keyboard.press('e');
  await expect(page.locator('.count')).toHaveText(String(before - 1));
  await page.getByRole('button', { name: 'Undo' }).click();
  await expect(page.locator('.count')).toHaveText(String(before));
});
