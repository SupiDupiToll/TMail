import type { MailtoRequest } from './types';

export function parseMailto(href: string): MailtoRequest | null {
  if (!href.toLowerCase().startsWith('mailto:')) return null;
  const rest = href.slice('mailto:'.length);
  const [addressPart, queryPart = ''] = rest.split('?', 2);
  const decode = (value: string) => {
    try {
      return decodeURIComponent(value.replace(/\+/g, ' '));
    } catch {
      return value;
    }
  };
  const split = (value: string) =>
    value
      .split(',')
      .map((part) => part.trim())
      .filter(Boolean);
  const params = new URLSearchParams(queryPart);
  return {
    to: split(decode(addressPart)),
    cc: split(params.get('cc') ?? ''),
    bcc: split(params.get('bcc') ?? ''),
    subject: params.get('subject') ?? '',
    body: params.get('body') ?? ''
  };
}
