import { test, expect, type Page } from '@playwright/test';

const viewports = [
  { width: 800, height: 600, name: 'small' },
  { width: 1280, height: 800, name: 'laptop' },
  { width: 1440, height: 900, name: 'desktop' },
  { width: 1920, height: 1080, name: 'large' }
];

async function shellMetrics(page: Page) {
  return page.evaluate(() => {
    const doc = document.documentElement;
    const body = document.body;
    const shell = document.querySelector<HTMLElement>('.shell');
    const list = document.querySelector<HTMLElement>('.list-pane');
    const sidebar = document.querySelector<HTMLElement>('.sidebar');
    return {
      docScrollWidth: doc.scrollWidth,
      docClientWidth: doc.clientWidth,
      bodyScrollWidth: body.scrollWidth,
      shell: shell ? { w: shell.clientWidth, sw: shell.scrollWidth } : null,
      list: list ? { w: list.clientWidth, sw: list.scrollWidth } : null,
      sidebarVisible: sidebar ? getComputedStyle(sidebar.parentElement!).transform === 'none' || sidebar.getBoundingClientRect().left >= 0 : false
    };
  });
}

for (const viewport of viewports) {
  test(`no horizontal overflow at ${viewport.name} (${viewport.width})`, async ({ page }) => {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.goto('/?mock=inbox');
    await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
    const metrics = await shellMetrics(page);
    expect(metrics.docScrollWidth).toBeLessThanOrEqual(metrics.docClientWidth + 1);
    expect(metrics.bodyScrollWidth).toBeLessThanOrEqual(metrics.docClientWidth + 1);
    if (metrics.shell) expect(metrics.shell.sw).toBeLessThanOrEqual(metrics.shell.w + 1);
    if (metrics.list) expect(metrics.list.sw).toBeLessThanOrEqual(metrics.list.w + 1);
  });
}

test('rows are dense, truncate long subjects and do not overlap', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  const options = list.getByRole('option');
  const count = await options.count();
  expect(count).toBeGreaterThan(5);

  const first = await options.first().boundingBox();
  expect(first?.height).toBeGreaterThanOrEqual(50);
  expect(await page.locator('.time').count()).toBe(0);
  const weight = await list
    .getByRole('option')
    .filter({ has: page.locator('.unread') })
    .first()
    .locator('.sender')
    .evaluate((element) => Number(getComputedStyle(element).fontWeight));
  expect(weight).toBeGreaterThanOrEqual(700);

  const overflow = await list.evaluate((element) => {
    const rows = Array.from(element.querySelectorAll<HTMLElement>('[role="option"]'));
    return rows.filter((row) => row.scrollWidth > row.clientWidth + 1).length;
  });
  expect(overflow).toBe(0);

  const longRow = list.getByRole('option').filter({ hasText: 'A very long subject' });
  if ((await longRow.count()) > 0) {
    const box = await longRow.first().boundingBox();
    expect(box?.width ?? 0).toBeGreaterThan(100);
  }
});

test('comfortable density increases row height', async ({ page }) => {
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  const compact = (await list.getByRole('option').first().boundingBox())?.height ?? 0;
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Comfortable' }).click();
  await page.getByRole('button', { name: 'Close settings' }).click();
  const comfortable = (await list.getByRole('option').first().boundingBox())?.height ?? 0;
  expect(comfortable).toBeGreaterThan(compact + 8);
});

test('focus rings are visible and every primary action is keyboard reachable', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  const seen: string[] = [];
  for (let i = 0; i < 18; i++) {
    await page.keyboard.press('Tab');
    const data = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      if (!el || el === document.body || el.tagName === 'SCRIPT') return null;
      const style = getComputedStyle(el);
      const rect = el.getBoundingClientRect();
      return {
        label:
          el.getAttribute('aria-label') ||
          el.getAttribute('title') ||
          el.textContent?.trim().slice(0, 40) ||
          el.tagName,
        outline: style.outlineStyle,
        width: parseFloat(style.outlineWidth || '0'),
        visible: rect.width > 0 && rect.height > 0
      };
    });
    if (!data) continue;
    seen.push(data.label);
    expect(data.visible, `focused element should be visible: ${data.label}`).toBe(true);
    expect(data.outline, `focus ring missing on: ${data.label}`).not.toBe('none');
    expect(data.width).toBeGreaterThanOrEqual(1.5);
  }
  expect(seen.length).toBeGreaterThan(6);
});

test('mail HTML is sandboxed, scripts stripped and remote pixels blocked', async ({ page }) => {
  const requests: string[] = [];
  await page.route('**/*', async (route) => {
    const url = route.request().url();
    if (url.includes('tracker.example')) requests.push(url);
    await route.continue();
  });

  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();

  const iframe = page.locator('iframe');
  await expect(iframe).toHaveCount(1);
  const sandbox = await iframe.getAttribute('sandbox');
  expect(sandbox).toContain('allow-scripts');
  expect(sandbox).not.toContain('allow-same-origin');
  expect(await iframe.getAttribute('referrerpolicy')).toBe('no-referrer');

  const srcdoc = (await iframe.getAttribute('srcdoc')) ?? '';
  expect(srcdoc).toContain("default-src 'none'");
  expect(srcdoc).toContain('script-src data:');
  expect(srcdoc).toContain('style-src data:');
  expect(srcdoc).toContain('img-src data:');
  expect(srcdoc).not.toContain('tracker.example');
  expect(srcdoc).not.toContain('window.__pwned');
  expect(srcdoc).toContain('[remote image blocked');
  const cssMatch = srcdoc.match(/href="data:text\/css;charset=utf-8,([^"]+)"/);
  const frameCss = cssMatch ? decodeURIComponent(cssMatch[1]) : '';
  expect(frameCss).toContain('Atkinson Hyperlegible Next');
  expect(frameCss).toContain('border-collapse');
  expect(srcdoc).toContain('data-href="https://example.com/verify"');
  const frameBody = page.frameLocator('iframe').locator('body');
  const frameMetrics = await frameBody.evaluate((body) => {
    const style = getComputedStyle(body);
    const link = body.querySelector('a');
    return {
      font: style.fontFamily,
      linkColor: link ? getComputedStyle(link).color : null,
      scrollHeight: document.documentElement.scrollHeight,
      clientHeight: document.documentElement.clientHeight
    };
  });
  expect(frameMetrics.font).toContain('Source Serif');
  if (frameMetrics.linkColor) expect(frameMetrics.linkColor).not.toBe('rgb(0, 0, 238)');
  await expect
    .poll(() => frameBody.evaluate(() => document.documentElement.scrollHeight - document.documentElement.clientHeight))
    .toBeLessThanOrEqual(1);
  expect(requests).toEqual([]);
});

test('reader link preview and remote-content opt-in are explicit', async ({ page }) => {
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByText('Remote images blocked', { exact: true })).toBeVisible();
  await expect(page.frameLocator('iframe').getByText(/remote image blocked/)).toBeVisible();
  await page.getByRole('button', { name: 'Always allow' }).click();
  await expect(page.getByText('Remote images blocked', { exact: true })).toHaveCount(0);
  const srcdoc = (await page.locator('iframe').getAttribute('srcdoc')) ?? '';
  expect(srcdoc).toContain('img-src data: https:');
  await expect(page.frameLocator('iframe').getByText(/remote image blocked/)).toHaveCount(0);
});

test('text contrast meets AA for rendered UI', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();

  const failures = await page.evaluate(() => {
    function parse(color: string): [number, number, number, number] {
      const match = color.match(/rgba?\(([^)]+)\)/);
      if (!match) return [0, 0, 0, 0];
      const parts = match[1].split(',').map((part) => parseFloat(part.trim()));
      return [parts[0], parts[1], parts[2], parts[3] ?? 1];
    }
    function lum([r, g, b]: number[]): number {
      const channel = (value: number) => {
        const v = value / 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
      };
      return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    }
    function background(element: Element): [number, number, number, number] {
      let current: Element | null = element;
      while (current) {
        const [r, g, b, a] = parse(getComputedStyle(current).backgroundColor);
        if (a > 0.95) return [r, g, b, a];
        current = current.parentElement;
      }
      return [255, 255, 255, 1];
    }
    const nodes = Array.from(
      document.querySelectorAll<HTMLElement>(
        '.sidebar .folder-name, .account-email, .list-pane h1, .list-pane .sender, .list-pane .subject, .list-pane .time, .reader h2, .reader .sender, .reader .address, .btn, .chip, .label'
      )
    );
    const out: { text: string; ratio: number; size: number; cls: string }[] = [];
    for (const node of nodes) {
      if (!node.offsetParent) continue;
      const style = getComputedStyle(node);
      const color = parse(style.color);
      const bg = background(node);
      const ratio =
        (Math.max(lum(color), lum(bg)) + 0.05) / (Math.min(lum(color), lum(bg)) + 0.05);
      const size = parseFloat(style.fontSize);
      const required = size >= 24 || (size >= 18.66 && Number(style.fontWeight) >= 700) ? 3 : 4.5;
      if (ratio + 0.01 < required) {
        out.push({ text: node.textContent?.trim().slice(0, 30) ?? '', ratio, size, cls: node.className });
      }
    }
    return out;
  });
  expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
});

test('links are intercepted by the trusted parent, not navigated in-frame', async ({ page }) => {
  await page.addInitScript(() => {
    (window as unknown as { __opened: string | null }).__opened = null;
    window.open = ((url?: string | URL) => {
      (window as unknown as { __opened: string | null }).__opened = String(url ?? '');
      return null;
    }) as typeof window.open;
  });
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  const frame = page.frameLocator('iframe');
  const link = frame.getByRole('link', { name: 'Manage this booking' });
  await expect(link).toBeVisible();
  await link.click();
  await expect
    .poll(() => page.evaluate(() => (window as unknown as { __opened: string | null }).__opened))
    .toBe('https://example.com/verify');
  expect(page.url()).toContain('/?mock=inbox');


});

test('attachments preview in-app and close on the backdrop', async ({ page }) => {
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();

  await page.getByRole('button', { name: 'Preview revenue-by-region.csv' }).click();
  const dialog = page.getByRole('dialog', { name: 'revenue-by-region.csv' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText(/Synthetic preview for revenue-by-region.csv/)).toBeVisible();
  await page.mouse.click(5, 5);
  await expect(dialog).toBeHidden();

  await page.getByRole('button', { name: 'Preview quarterly-report.pdf' }).click();
  const pdf = page.getByRole('dialog', { name: 'quarterly-report.pdf' });
  await expect(pdf.getByText('Preview not available')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(pdf).toBeHidden();
});

test('sidebar collapses and columns can be resized from the keyboard', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();

  const sidebar = page.getByRole('complementary', { name: 'Mailbox' });
  const expanded = (await sidebar.boundingBox())?.width ?? 0;
  await page.getByRole('button', { name: 'Collapse sidebar' }).click();
  await expect(page.getByRole('button', { name: 'Expand sidebar' })).toBeVisible();
  const collapsed = (await sidebar.boundingBox())?.width ?? 0;
  expect(collapsed).toBeLessThan(expanded - 100);
  await page.getByRole('button', { name: 'Expand sidebar' }).click();
  const restored = (await sidebar.boundingBox())?.width ?? 0;
  expect(restored).toBeGreaterThan(collapsed + 100);

  const list = page.getByRole('region', { name: 'Inbox message list' });
  const listBefore = (await list.boundingBox())?.width ?? 0;
  const resizer = page.getByRole('button', { name: 'Resize message list' });
  await resizer.focus();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  const listAfter = (await list.boundingBox())?.width ?? 0;
  expect(listAfter).toBeGreaterThan(listBefore + 10);
});

test('composer expands and schedules a send', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('c');
  const dialog = page.getByRole('dialog', { name: /New message/ });
  await expect(dialog).toBeVisible();
  const before = (await dialog.boundingBox())?.width ?? 0;
  await dialog.getByRole('button', { name: 'Expand composer' }).click();
  const expanded = (await dialog.boundingBox())?.width ?? 0;
  expect(expanded).toBeGreaterThan(before + 20);
  await dialog.getByRole('button', { name: 'Restore composer size' }).click();

  await dialog.getByLabel('To').fill('ada@tuta.com');
  await dialog.getByLabel('Subject').fill('Scheduled hello');
  await dialog.getByLabel('Message body').fill('This goes out later.');
  await dialog.getByRole('button', { name: 'Schedule send' }).click();
  const when = dialog.getByLabel('Send at');
  await when.fill('2030-01-02T09:30');
  await dialog.getByRole('button', { name: 'Set' }).click();
  await expect(dialog.getByRole('button', { name: /^Schedule/ })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Discard' }).click();
});

test('mail body uses the available reader width', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  const ratio = await page.evaluate(() => {
    const frame = document.querySelector('iframe');
    const reader = document.querySelector('.reader');
    if (!frame || !reader) return 0;
    return frame.clientWidth / reader.clientWidth;
  });
  expect(ratio).toBeGreaterThan(0.8);
});

test('contacts autocomplete, thread strip and encryption badge are visible', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await expect(page.locator('.thread-strip').getByRole('button')).toHaveCount(3);
  await expect(page.getByText('TutaCrypt', { exact: true })).toBeVisible();

  await page.keyboard.press('Escape');
  await page.keyboard.press('c');
  const composer = page.getByRole('dialog', { name: /New message/ });
  await expect(composer).toBeVisible();
  const to = composer.getByLabel('To');
  await to.fill('Ada');
  await expect(composer.getByRole('option', { name: /Ada Lovelace/ })).toBeVisible();
  await composer.getByRole('option', { name: /Ada Lovelace/ }).click();
  await expect(to).toHaveValue('ada@analytical.example');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Keep editing' }).click();
  await page.getByRole('button', { name: 'Close composer' }).click();
  await page.getByRole('button', { name: 'Discard' }).click();
});

test('full search view filters results and can clear', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
  await page.keyboard.press('Control+k');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await palette.getByRole('combobox').fill('quarterly');
  await expect(palette.getByText(/Search all mail for/)).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: /Search: quarterly/ })).toBeVisible();
  const filters = page.locator('.search-filters');
  await expect(filters.getByRole('button', { name: 'Unread', exact: true })).toBeVisible();
  await expect(filters.getByRole('button', { name: 'Attachments', exact: true })).toBeVisible();
  await filters.getByRole('button', { name: 'Unread', exact: true }).click();
  await page.getByRole('button', { name: 'Clear search' }).click();
  await expect(page.getByRole('heading', { name: 'Inbox', level: 1 })).toBeVisible();
});

test('labels can be applied from the reader menu', async ({ page }) => {
  await page.goto('/?mock=inbox');
  const list = page.getByRole('listbox', { name: /Inbox messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await page.locator('summary[aria-label="More actions"]').click();
  await page.getByRole('menuitem', { name: 'Urgent', exact: true }).click();
  await expect(page.getByText('Label added')).toBeVisible();
});

test('continue editing opens an existing draft', async ({ page }) => {
  await page.goto('/?mock=inbox');
  await page.getByRole('button', { name: 'Drafts', exact: true }).click();
  const list = page.getByRole('listbox', { name: /Drafts messages/ });
  await expect(list).toBeVisible();
  await list.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 2 })).toBeVisible();
  await page.getByRole('button', { name: 'Continue editing' }).click();
  const composer = page.getByRole('dialog', { name: /Draft/ });
  await expect(composer).toBeVisible();
  await expect(composer.getByLabel('Subject')).toHaveValue('Draft: reply to Ada');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Keep editing' }).click();
  await page.getByRole('button', { name: 'Close composer' }).click();
  await page.getByRole('button', { name: 'Discard' }).click();
});
