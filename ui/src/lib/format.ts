const timeFormat = new Intl.DateTimeFormat(undefined, {
  hour: '2-digit',
  minute: '2-digit'
});
const weekdayFormat = new Intl.DateTimeFormat(undefined, { weekday: 'short' });
const dayFormat = new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short' });
const yearFormat = new Intl.DateTimeFormat(undefined, {
  day: 'numeric',
  month: 'short',
  year: 'numeric'
});
const fullFormat = new Intl.DateTimeFormat(undefined, {
  weekday: 'short',
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit'
});

function startOfDay(value: number): number {
  const date = new Date(value);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

export function formatTime(ms: number): string {
  return timeFormat.format(new Date(ms));
}

export function formatListDate(ms: number, now = Date.now()): string {
  const today = startOfDay(now);
  const day = startOfDay(ms);
  if (day === today) return formatTime(ms);
  if (day === today - 86_400_000) return 'Yesterday';
  if (today - day < 6 * 86_400_000) return weekdayFormat.format(new Date(ms));
  if (new Date(ms).getFullYear() === new Date(now).getFullYear()) {
    return dayFormat.format(new Date(ms));
  }
  return yearFormat.format(new Date(ms));
}

export function formatDay(ms: number, now = Date.now()): string {
  const today = startOfDay(now);
  const day = startOfDay(ms);
  if (day === today) return 'Today';
  if (day === today - 86_400_000) return 'Yesterday';
  if (today - day < 6 * 86_400_000) return weekdayFormat.format(new Date(ms));
  if (new Date(ms).getFullYear() === new Date(now).getFullYear()) {
    return dayFormat.format(new Date(ms));
  }
  return yearFormat.format(new Date(ms));
}

export function formatFullDate(ms: number): string {
  if (!ms) return '';
  return fullFormat.format(new Date(ms));
}

export function isSameDay(a: number, b: number): boolean {
  return startOfDay(a) === startOfDay(b);
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

export function initials(name: string, address: string): string {
  const source = (name || address.split('@')[0] || '?').trim();
  const parts = source.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  }
  return source.slice(0, 2).toUpperCase();
}

export function senderLabel(name: string, address: string): string {
  return name && name !== address ? name : address;
}

export function plural(count: number, singular: string, pluralForm = `${singular}s`): string {
  return `${count} ${count === 1 ? singular : pluralForm}`;
}
