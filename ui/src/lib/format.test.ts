import { describe, expect, it } from 'vitest';
import { formatBytes, initials, isSameDay } from './format';
import { parseMailto } from './mailto';

describe('format', () => {
  it('formats sizes with sensible units', () => {
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(2048)).toBe('2.0 KB');
    expect(formatBytes(2 * 1024 * 1024)).toBe('2.0 MB');
  });

  it('derives initials from names and addresses', () => {
    expect(initials('Ada Lovelace', 'ada@example.com')).toBe('AL');
    expect(initials('', 'grace@example.com')).toBe('GR');
    expect(initials('', '@example.com')).toBe('?');
  });

  it('compares calendar days, not instants', () => {
    const morning = new Date(2026, 10, 14, 8, 0).getTime();
    const evening = new Date(2026, 10, 14, 22, 0).getTime();
    expect(isSameDay(morning, evening)).toBe(true);
    expect(isSameDay(morning, morning + 86_400_000)).toBe(false);
  });
});

describe('mailto', () => {
  it('parses addresses, cc and subject', () => {
    const request = parseMailto(
      'mailto:ada@example.com,grace@example.com?cc=team@example.com&subject=Hello%20there&body=Line%201'
    );
    expect(request?.to).toEqual(['ada@example.com', 'grace@example.com']);
    expect(request?.cc).toEqual(['team@example.com']);
    expect(request?.subject).toBe('Hello there');
    expect(request?.body).toBe('Line 1');
  });

  it('rejects non-mailto links', () => {
    expect(parseMailto('https://example.com')).toBeNull();
  });
});
