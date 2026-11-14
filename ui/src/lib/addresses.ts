export const TUTA_DOMAINS = ['tuta.com', 'tutanota.com', 'tuta.io', 'tutanota.de'];

export function isTutaAddress(address: string): boolean {
  const normalised = address.trim().toLowerCase();
  if (!normalised.includes('@')) return false;
  return TUTA_DOMAINS.some((domain) => normalised.endsWith(`@${domain}`));
}

export function splitAddresses(value: string): string[] {
  return value
    .split(/[,;\n]/)
    .map((part) => part.trim())
    .filter(Boolean);
}
