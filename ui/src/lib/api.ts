import type {
  DraftInput,
  FolderNode,
  LoginResult,
  MailDetail,
  MailSummary,
  MailtoRequest,
  OutgoingAttachment,
  SearchHit,
  SendReceipt,
  Snapshot,
  SyncEvent
} from './types';
import { mock } from './mock';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const isMock =
  !isTauri &&
  typeof window !== 'undefined' &&
  (import.meta.env.DEV || new URLSearchParams(window.location.search).has('mock'));

if (isMock) {
  const params = new URLSearchParams(window.location.search);
  if (params.get('mock') === 'inbox' || sessionStorage.getItem('tmail.mock-signed-in') === '1') {
    mock.signedIn = true;
  }
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(command, args);
  }
  return mockInvoke(command, args) as T;
}

function mockInvoke(command: string, args?: Record<string, unknown>): unknown {
  switch (command) {
    case 'app_info':
      return {
        version: '0.1.0-mock',
        product: 'TMail (unofficial, preview)',
        api_url: 'https://app.tuta.com',
        mock: true
      };
    case 'resume':
      return mock.signedIn ? mock.snapshot() : null;
    case 'login':
      return { status: 'logged_in', snapshot: mock.login() } satisfies LoginResult;
    case 'submit_totp':
      return mock.login();
    case 'snapshot':
      return mock.snapshot();
    case 'list_folders':
      return mock.snapshot().folders;
    case 'open_folder':
      return mock.folder(String(args?.folderId), Number(args?.limit ?? 200));
    case 'refresh_folder':
      return mock.folder(String(args?.folderId), Number(args?.limit ?? 200));
    case 'open_mail':
      return mock.mail(String(args?.id));
    case 'search':
      return mock.search(String(args?.query), Number(args?.limit ?? 50));
    case 'set_unread':
      mock.setUnread(args?.ids as string[], Boolean(args?.unread));
      return null;
    case 'move_mails':
      mock.move(args?.ids as string[], String(args?.targetFolder));
      return null;
    case 'trash':
      mock.remove(args?.ids as string[]);
      return null;
    case 'archive':
      mock.move(args?.ids as string[], 'archive');
      return null;
    case 'delete_forever':
      mock.remove(args?.ids as string[]);
      return null;
    case 'send_mail':
      return mock.send(args?.draft as DraftInput, args?.attachments as OutgoingAttachment[]);
    case 'save_draft':
      return mock.saveDraft(args?.draft as DraftInput);
    case 'save_attachment': {
      mock.saveAttachment(String(args?.name));
      return { saved: true };
    }
    case 'read_attachment': {
      const mime = String(args?.mimeType ?? 'application/octet-stream');
      const name = String(args?.name ?? 'attachment');
      if (mime.startsWith('image/')) {
        return {
          name,
          mime_type: mime,
          data: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAFgwJ/lv8AAAAASUVORK5CYII='
        };
      }
      const text = `Synthetic preview for ${name}.\n\nThis is an attachment from the TMail preview.\nIt is not real data.`;
      const bytes = new TextEncoder().encode(text);
      return { name, mime_type: mime, data: bytesToBase64(bytes) };
    }
    case 'logout':
    case 'cancel_login':
    case 'clear_cache':
      return null;
    case 'unread_counts':
      return mock.snapshot().unread;
    default:
      throw new Error(`Unknown mock command: ${command}`);
  }
}

function base64ToBytes(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

export interface PendingMailto {
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  body: string;
}

export interface AppInfo {
  version: string;
  product: string;
  api_url: string;
  mock: boolean;
}

export interface SaveAttachmentResult {
  saved: boolean;
  path?: string;
}

export interface AttachmentData {
  name: string;
  mime_type: string;
  data: string;
}

export const api = {
  appInfo: () => invoke<AppInfo>('app_info'),
  resume: (email: string) => invoke<Snapshot | null>('resume', { email }),
  login: (email: string, password: string) =>
    invoke<LoginResult>('login', { email, password }),
  submitTotp: (code: number) => invoke<Snapshot>('submit_totp', { code }),
  cancelLogin: () => invoke<void>('cancel_login'),
  snapshot: () => invoke<Snapshot>('snapshot'),
  logout: () => invoke<void>('logout'),
  listFolders: () => invoke<FolderNode[]>('list_folders'),
  openFolder: (folderId: string, limit = 200) =>
    invoke<MailSummary[]>('open_folder', { folderId, limit }),
  refreshFolder: (folderId: string, limit = 200) =>
    invoke<MailSummary[]>('refresh_folder', { folderId, limit }),
  openMail: (id: string) => invoke<MailDetail>('open_mail', { id }),
  search: (query: string, limit = 50) => invoke<SearchHit[]>('search', { query, limit }),
  setUnread: (ids: string[], unread: boolean) =>
    invoke<void>('set_unread', { ids, unread }),
  moveMails: (ids: string[], targetFolder: string) =>
    invoke<void>('move_mails', { ids, targetFolder }),
  trash: (ids: string[]) => invoke<void>('trash', { ids }),
  archive: (ids: string[]) => invoke<void>('archive', { ids }),
  deleteForever: (ids: string[], folderId: string) =>
    invoke<void>('delete_forever', { ids, folderId }),
  setCurrentFolder: (folderId: string) =>
    invoke<void>('set_current_folder', { folderId }),
  sendMail: (draft: DraftInput, attachments: OutgoingAttachment[]) =>
    invoke<SendReceipt>('send_mail', { draft, attachments }),
  saveDraft: (draft: DraftInput, attachments: OutgoingAttachment[]) =>
    invoke<string>('save_draft', { draft, attachments }),
  saveAttachment: (mailId: string, attachmentId: string, name: string) =>
    invoke<SaveAttachmentResult>('save_attachment', { mailId, attachmentId, name }),
  readAttachment: (mailId: string, attachmentId: string, name: string, mimeType: string) =>
    invoke<AttachmentData>('read_attachment', { mailId, attachmentId, name, mimeType }),
  unreadCounts: () => invoke<{ folder_id: string; unread: number }[]>('unread_counts'),
  clearCache: () => invoke<void>('clear_cache'),
  quit: () => invoke<void>('quit_app'),
  takePendingMailto: () => invoke<PendingMailto | null>('take_pending_mailto')
};

export async function subscribeEvents(handler: (event: SyncEvent) => void): Promise<() => void> {
  if (!isTauri) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<SyncEvent>('sync-event', (event) => handler(event.payload));
  return unlisten;
}

export async function subscribeMailto(handler: (request: MailtoRequest) => void): Promise<() => void> {
  if (!isTauri) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<MailtoRequest>('mailto', (event) => handler(event.payload));
  return unlisten;
}

export async function openExternal(url: string): Promise<void> {
  const scheme = url.split(':')[0]?.toLowerCase();
  if (!['https', 'http', 'mailto'].includes(scheme)) {
    throw new Error('Blocked link scheme');
  }
  if (isTauri) {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
  } else {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

export async function notify(title: string, body: string): Promise<void> {
  if (!isTauri) return;
  try {
    const plugin = await import('@tauri-apps/plugin-notification');
    let granted = await plugin.isPermissionGranted();
    if (!granted) granted = (await plugin.requestPermission()) === 'granted';
    if (granted) plugin.sendNotification({ title, body });
  } catch {
    /* notifications are optional */
  }
}
