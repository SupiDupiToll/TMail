import {
  api,
  isMock,
  isTauri,
  notify,
  subscribeEvents,
  subscribeMailto,
  subscribeOpenMail
} from './api';
import { splitAddresses } from './addresses';
import { senderLabel } from './format';
import type { AppInfo } from './api';
import type {
  AttachmentMeta,
  DraftInput,
  MailDetail,
  MailSummary,
  MailtoRequest,
  OutgoingAttachment,
  SearchHit,
  Snapshot,
  SyncEvent
} from './types';
import { CONVERSATION_FORWARD, CONVERSATION_NEW, CONVERSATION_REPLY } from './types';

export type AuthState = 'booting' | 'signed_out' | 'totp' | 'signed_in';
export type ThemePreference = 'system' | 'light' | 'dark';
export type Density = 'compact' | 'comfortable' | 'spacious';

export interface Toast {
  id: number;
  message: string;
  kind: 'info' | 'error' | 'success';
  action?: { label: string; run: () => void };
}

export interface PreviewState {
  open: boolean;
  loading: boolean;
  name: string;
  mimeType: string;
  kind: 'image' | 'text' | 'other';
  dataUrl: string;
  text: string;
  error: string | null;
  attachment: AttachmentMeta | null;
}

export interface ComposerState {
  open: boolean;
  mode: 'new' | 'reply' | 'reply_all' | 'forward' | 'draft';
  from: string;
  to: string;
  cc: string;
  bcc: string;
  showCc: boolean;
  subject: string;
  body: string;
  plaintext: boolean;
  confidential: boolean;
  confidentialPassword: string;
  sendAt: number | null;
  previousMessageId: string | null;
  conversationType: number;
  attachments: OutgoingAttachment[];
  replyToId: string | null;
  busy: boolean;
  error: string | null;
  dirty: boolean;
}

function emptyComposer(): ComposerState {
  return {
    open: false,
    mode: 'new',
    from: '',
    to: '',
    cc: '',
    bcc: '',
    showCc: false,
    subject: '',
    body: '',
    plaintext: false,
    confidential: false,
    confidentialPassword: '',
    sendAt: null,
    previousMessageId: null,
    conversationType: CONVERSATION_NEW,
    attachments: [],
    replyToId: null,
    busy: false,
    error: null,
    dirty: false
  };
}

function base64ToText(data: string): string {
  const binary = atob(data);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return new TextDecoder('utf-8', { fatal: false }).decode(bytes).slice(0, 200_000);
}

function clampNumber(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, Math.round(value)));
}

function getString(key: string, fallback = ''): string {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    return fallback;
  }
}

function setString(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* ignore quota/disabled storage */
  }
}

export class MailApp {
  auth = $state<AuthState>('booting');
  info = $state<AppInfo | null>(null);
  snapshot = $state<Snapshot | null>(null);
  error = $state<string | null>(null);
  bootError = $state<string | null>(null);

  selectedFolderId = $state<string>('');
  mails = $state<MailSummary[]>([]);
  loadingFolder = $state(false);
  refreshing = $state(false);
  selectedMailId = $state<string | null>(null);
  detail = $state<MailDetail | null>(null);
  loadingMail = $state(false);

  searchQuery = $state('');
  searchHits = $state<SearchHit[]>([]);
  searching = $state(false);
  searchOpen = $state(false);

  commandOpen = $state(false);
  settingsOpen = $state(false);
  helpOpen = $state(false);
  composer = $state<ComposerState>(emptyComposer());

  connection = $state<'online' | 'offline' | 'connecting'>('connecting');
  synthetic = $state(isMock);

  theme = $state<ThemePreference>(
    (getString('tmail.theme', 'system') as ThemePreference) || 'system'
  );
  density = $state<Density>((getString('tmail.density', 'compact') as Density) || 'compact');
  sidebarWidth = $state(248);
  listWidth = $state(420);
  sidebarCollapsed = $state(false);
  defaultSender = $state('');
  signature = $state('');
  notificationsEnabled = $state(true);
  plaintextDefault = $state(false);

  remoteSenders = $state<string[]>([]);
  remoteOnce = $state<string[]>([]);
  selectedIds = $state<string[]>([]);
  toasts = $state<Toast[]>([]);
  unreadTotal = $state(0);
  preview = $state<PreviewState>({
    open: false,
    loading: false,
    name: '',
    mimeType: '',
    kind: 'other',
    dataUrl: '',
    text: '',
    error: null,
    attachment: null
  });

  private toastSeq = 0;
  private media: MediaQueryList | null = null;
  private inboxKnownIds = new Set<string>();
  private inboxTimer: ReturnType<typeof setInterval> | null = null;
  private inboxCheckInFlight = false;
  private readTimer: ReturnType<typeof setTimeout> | null = null;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;
  private unlisten: Array<() => void> = [];
  private initialised = false;

  async init(): Promise<void> {
    if (this.initialised) return;
    this.initialised = true;
    try {
      this.info = await api.appInfo();
    } catch (error) {
      this.info = {
        version: 'unknown',
        product: 'TMail',
        api_url: '',
        mock: isMock
      };
    }
    this.applyTheme(this.theme);
    this.applyDensity(this.density);
    this.remoteSenders = this.loadRemoteSenders();
    this.sidebarWidth = clampNumber(Number(getString('tmail.sidebar-width', '248')), 190, 380);
    this.listWidth = clampNumber(Number(getString('tmail.list-width', '420')), 300, 680);
    this.sidebarCollapsed = getString('tmail.sidebar-collapsed', '0') === '1';
    this.defaultSender = getString('tmail.default-sender', '');
    this.signature = getString('tmail.signature', '');
    this.notificationsEnabled = getString('tmail.notifications', '1') !== '0';
    this.plaintextDefault = getString('tmail.plaintext-default', '0') === '1';
    if (typeof window !== 'undefined') {
      this.media = window.matchMedia('(prefers-color-scheme: dark)');
      this.media.addEventListener('change', this.onSystemTheme);
    }

    this.unlisten.push(await subscribeEvents((event) => this.onSyncEvent(event)));
    this.unlisten.push(await subscribeMailto((request) => this.openMailto(request)));
    this.unlisten.push(await subscribeOpenMail((mailId) => void this.openMailById(mailId)));

    if (isTauri) {
      try {
        const pending = await api.takePendingMailto();
        if (pending) this.openMailto(pending);
      } catch {
        /* the command is optional during startup */
      }
    }

    this.startInboxWatch();
    let lastEmail = getString('tmail.last-email') || (isMock ? 'you@tuta.com' : '');
    if (!lastEmail && isTauri) {
      try {
        const last = await api.lastAccount();
        if (last) {
          lastEmail = last.email;
          setString('tmail.last-email', last.email.toLowerCase());
        }
      } catch {
        /* first run or no metadata */
      }
    }
    if (lastEmail) {
      try {
        const snapshot = await api.resume(lastEmail);
        if (snapshot) {
          this.applySnapshot(snapshot);
          return;
        }
      } catch (error) {
        this.bootError = this.message(error);
      }
    }
    this.auth = 'signed_out';
  }

  private async onSyncEvent(event: SyncEvent): Promise<void> {
    switch (event.type) {
      case 'connected':
        this.connection = 'online';
        break;
      case 'disconnected':
        this.connection = 'offline';
        break;
      case 'session_expired':
        this.auth = 'signed_out';
        this.error = 'Your session expired. Sign in again to continue.';
        break;
      case 'cache_warning':
        this.toast(event.message, 'info');
        break;
      case 'mails_changed':
        if (this.refreshTimer) clearTimeout(this.refreshTimer);
        this.refreshTimer = setTimeout(() => {
          if (this.auth === 'signed_in' && this.selectedFolderId) {
            void this.refreshCurrent(true);
          }
          void this.checkInboxForNewMail();
        }, 350);
        break;
    }
  }

  private openMailto(request: MailtoRequest): void {
    if (!request.to.length && !request.subject && !request.body) return;
    this.composer = {
      ...emptyComposer(),
      open: true,
      from: this.preferredSender(),
      plaintext: this.plaintextDefault,
      to: request.to.join(', '),
      cc: request.cc.join(', '),
      bcc: request.bcc.join(', '),
      showCc: request.cc.length > 0 || request.bcc.length > 0,
      subject: request.subject,
      body: this.signature.trim()
        ? `${request.body.trimEnd()}\n\n-- \n${this.signature.trim()}`
        : request.body
    };
  }

  openMailtoRequest(request: MailtoRequest): void {
    this.openMailto(request);
  }

  async login(email: string, password: string): Promise<boolean> {
    this.error = null;
    try {
      const result = await api.login(email, password);
      if (result.status === 'totp_required') {
        this.auth = 'totp';
        return false;
      }
      setString('tmail.last-email', email.trim().toLowerCase());
      this.applySnapshot(result.snapshot);
      return true;
    } catch (error) {
      this.error = this.message(error);
      return false;
    }
  }

  async submitTotp(code: number): Promise<boolean> {
    this.error = null;
    try {
      const snapshot = await api.submitTotp(code);
      this.applySnapshot(snapshot);
      return true;
    } catch (error) {
      this.error = this.message(error);
      return false;
    }
  }

  async cancelTotp(): Promise<void> {
    await api.cancelLogin();
    this.auth = 'signed_out';
  }

  private onSystemTheme = (): void => {
    if (this.theme === 'system') this.applyTheme('system');
  };

  applySnapshot(snapshot: Snapshot): void {
    // Remember the account for the next start. This also covers the TOTP path,
    // where login() returns early and previously never wrote the marker.
    if (snapshot.email) setString('tmail.last-email', snapshot.email.toLowerCase());
    this.snapshot = snapshot;
    this.unreadTotal = snapshot.unread.reduce((sum, entry) => sum + entry.unread, 0);
    this.auth = 'signed_in';
    this.connection = snapshot.connected ? 'online' : 'connecting';
    const preferred = this.selectedFolderId
      ? snapshot.folders.find((folder) => folder.id === this.selectedFolderId)
      : null;
    const target =
      preferred ??
      snapshot.folders.find((folder) => folder.kind === 'inbox') ??
      snapshot.folders[0];
    if (this.composer.open && !this.composer.from) {
      this.composer = {
        ...this.composer,
        from: snapshot.sender_addresses[0] ?? snapshot.email
      };
    }
    if (target) void this.selectFolder(target.id);
    void this.checkInboxForNewMail();
  }

  async logout(): Promise<void> {
    try {
      await api.logout();
    } finally {
      this.snapshot = null;
      this.mails = [];
      this.detail = null;
      this.selectedMailId = null;
      this.selectedFolderId = '';
      this.inboxKnownIds.clear();
      this.auth = 'signed_out';
      this.composer = emptyComposer();
    }
  }

  async selectFolder(folderId: string): Promise<void> {
    if (!folderId) return;
    this.selectedFolderId = folderId;
    this.selectedMailId = null;
    this.detail = null;
    this.selectedIds = [];
    this.searchOpen = false;
    this.searchQuery = '';
    this.error = null;
    this.loadingFolder = true;
    void api.setCurrentFolder(folderId).catch(() => {});
    try {
      const cached = await api.openFolder(folderId, 300);
      if (this.selectedFolderId !== folderId) return;
      this.mails = cached;
      this.loadingFolder = false;
      void this.refreshCurrent(true);
    } catch (error) {
      if (this.selectedFolderId !== folderId) return;
      this.loadingFolder = false;
      this.error = this.message(error);
    }
  }

  async refreshCurrent(silent = false): Promise<void> {
    if (!this.selectedFolderId) return;
    if (!silent) this.refreshing = true;
    try {
      const folderId = this.selectedFolderId;
      const mails = await api.refreshFolder(folderId, 300);
      if (this.selectedFolderId !== folderId) return;
      this.mails = mails;
      const counts = await api.unreadCounts();
      if (this.snapshot) this.snapshot = { ...this.snapshot, unread: counts };
      const total = counts.reduce((sum, entry) => sum + entry.unread, 0);
      this.unreadTotal = total;
      this.connection = 'online';
    } catch (error) {
      this.connection = 'offline';
      if (!silent) this.error = this.message(error);
    } finally {
      this.refreshing = false;
    }
  }

  async openMail(id: string): Promise<void> {
    this.selectedMailId = id;
    this.detail = null;
    this.loadingMail = true;
    this.error = null;
    try {
      const detail = await api.openMail(id);
      if (this.selectedMailId !== id) return;
      this.detail = detail;
      this.loadingMail = false;
      const summary = this.mails.find((mail) => mail.id === id);
      if (summary?.unread) {
        if (this.readTimer) clearTimeout(this.readTimer);
        this.readTimer = setTimeout(() => {
          void this.markRead(id, true);
          this.toast('Marked read', 'info', {
            label: 'Undo',
            run: () => void this.markRead(id, false)
          });
        }, 800);
      }
    } catch (error) {
      this.loadingMail = false;
      this.error = this.message(error);
    }
  }

  private async markRead(id: string, read: boolean): Promise<void> {
    try {
      await api.setUnread([id], !read);
      const mail = this.mails.find((m) => m.id === id);
      if (mail) mail.unread = !read;
      if (this.snapshot) {
        const counts = await api.unreadCounts();
        this.snapshot = { ...this.snapshot, unread: counts };
      }
    } catch {
      /* read state will be corrected on the next refresh */
    }
  }

  async toggleUnread(ids: string[] = this.targetIds()): Promise<void> {
    if (!ids.length) return;
    const first = this.mails.find((mail) => mail.id === ids[0]);
    const unread = !(first?.unread ?? false);
    try {
      await api.setUnread(ids, unread);
      for (const mail of this.mails) if (ids.includes(mail.id)) mail.unread = unread;
      const counts = await api.unreadCounts();
      if (this.snapshot) this.snapshot = { ...this.snapshot, unread: counts };
      this.toast(unread ? 'Marked unread' : 'Marked read');
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  async archive(ids: string[] = this.targetIds()): Promise<void> {
    if (!ids.length) return;
    const source = this.selectedFolderId;
    try {
      await api.archive(ids);
      this.mails = this.mails.filter((mail) => !ids.includes(mail.id));
      this.detail = null;
      this.selectedMailId = null;
      this.selectedIds = [];
      const count = ids.length === 1 ? '1 message' : `${ids.length} messages`;
      this.toast(`Archived ${count}`, 'success', {
        label: 'Undo',
        run: () => {
          void (async () => {
            try {
              await api.moveMails(ids, source);
              if (this.selectedFolderId === source) await this.selectFolder(source);
            } catch (error) {
              this.toast(this.message(error), 'error');
            }
          })();
        }
      });
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  async trash(ids: string[] = this.targetIds()): Promise<void> {
    if (!ids.length) return;
    const source = this.selectedFolderId;
    try {
      await api.trash(ids);
      this.mails = this.mails.filter((mail) => !ids.includes(mail.id));
      this.detail = null;
      this.selectedMailId = null;
      this.selectedIds = [];
      const count = ids.length === 1 ? '1 message' : `${ids.length} messages`;
      this.toast(`Moved ${count} to trash`, 'success', {
        label: 'Undo',
        run: () => {
          void (async () => {
            try {
              await api.moveMails(ids, source);
              if (this.selectedFolderId === source) await this.selectFolder(source);
            } catch (error) {
              this.toast(this.message(error), 'error');
            }
          })();
        }
      });
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  async deleteForever(ids: string[] = this.targetIds()): Promise<void> {
    if (!ids.length || !this.selectedFolderId) return;
    try {
      await api.deleteForever(ids, this.selectedFolderId);
      this.mails = this.mails.filter((mail) => !ids.includes(mail.id));
      this.detail = null;
      this.selectedMailId = null;
      this.selectedIds = [];
      this.toast('Deleted permanently', 'success');
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  async moveTo(folderId: string, ids: string[] = this.targetIds()): Promise<void> {
    if (!ids.length) return;
    try {
      await api.moveMails(ids, folderId);
      this.mails = this.mails.filter((mail) => !ids.includes(mail.id));
      this.detail = null;
      this.selectedMailId = null;
      this.selectedIds = [];
      const target = this.snapshot?.folders.find((folder) => folder.id === folderId);
      this.toast(`Moved to ${target?.name ?? 'folder'}`);
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  targetIds(): string[] {
    if (this.selectedIds.length) return this.selectedIds;
    return this.selectedMailId ? [this.selectedMailId] : [];
  }

  toggleSelected(id: string): void {
    this.selectedIds = this.selectedIds.includes(id)
      ? this.selectedIds.filter((value) => value !== id)
      : [...this.selectedIds, id];
  }

  clearSelection(): void {
    this.selectedIds = [];
  }

  async search(query: string): Promise<void> {
    this.searchQuery = query;
    if (!query.trim()) {
      this.searchHits = [];
      this.searchOpen = false;
      return;
    }
    this.searchOpen = true;
    this.searching = true;
    try {
      this.searchHits = await api.search(query, 60);
    } catch (error) {
      this.error = this.message(error);
    } finally {
      this.searching = false;
    }
  }

  closeSearch(): void {
    this.searchOpen = false;
    this.searchQuery = '';
    this.searchHits = [];
  }

  // ---------------------------------------------------------------- composer

  compose(mode: 'new' | 'reply' | 'reply_all' | 'forward' = 'new'): void {
    const detail = this.detail;
    if (mode !== 'new' && !detail) return;
    const base = emptyComposer();
    base.open = true;
    base.mode = mode;
    base.from = this.preferredSender();
    base.plaintext = this.plaintextDefault;
    base.replyToId = detail?.summary.id ?? null;

    if (detail && mode !== 'new') {
      const sender = detail.summary.sender;
      const to = (address: string) => address;
      base.previousMessageId = detail.conversation_message_id || null;
      base.conversationType =
        mode === 'forward' ? CONVERSATION_FORWARD : CONVERSATION_REPLY;
      if (mode === 'forward') {
        base.subject = prefixSubject(detail.summary.subject, 'Fwd:');
        base.body = quoteBody(detail);
      } else {
        base.to = mode === 'reply' ? to(sender.address) : toRecipients(detail, sender.address);
        base.subject = prefixSubject(detail.summary.subject, 'Re:');
        base.body = quoteBody(detail);
      }
    } else {
      base.conversationType = CONVERSATION_NEW;
    }
    if (this.signature.trim()) {
      base.body = `${base.body.trimEnd()}\n\n-- \n${this.signature.trim()}`;
    }
    this.composer = base;
  }

  closeComposer(): void {
    if (this.composer.busy) return;
    this.composer = { ...this.composer, open: false };
  }

  updateComposer(patch: Partial<ComposerState>): void {
    this.composer = { ...this.composer, ...patch, dirty: true };
  }

  async addAttachment(files: FileList | File[]): Promise<void> {
    for (const file of Array.from(files)) {
      if (file.size > 20 * 1024 * 1024) {
        this.toast(`${file.name} is larger than the 20 MB server limit`, 'error');
        continue;
      }
      const bytes = new Uint8Array(await file.arrayBuffer());
      const { bytesToBase64 } = await import('./api');
      this.composer = {
        ...this.composer,
        attachments: [
          ...this.composer.attachments,
          {
            filename: file.name,
            mime_type: file.type || 'application/octet-stream',
            data: bytesToBase64(bytes),
            size: file.size
          }
        ],
        dirty: true
      };
    }
  }

  removeAttachment(index: number): void {
    this.composer = {
      ...this.composer,
      attachments: this.composer.attachments.filter((_, i) => i !== index),
      dirty: true
    };
  }

  private draftInput(): DraftInput {
    const c = this.composer;
    return {
      to: splitAddresses(c.to),
      cc: splitAddresses(c.cc),
      bcc: splitAddresses(c.bcc),
      subject: c.subject.trim(),
      body_html: textToHtml(c.body),
      plaintext: c.plaintext,
      confidential: c.confidential,
      confidential_password: c.confidentialPassword || null,
      sender_address: c.from || this.snapshot?.email || null,
      send_at_ms: c.sendAt,
      previous_message_id: c.previousMessageId,
      conversation_type: c.conversationType
    };
  }

  async send(): Promise<boolean> {
    if (this.composer.busy) return false;
    const scheduled = this.composer.sendAt !== null;
    this.composer = { ...this.composer, busy: true, error: null };
    try {
      await api.sendMail(this.draftInput(), this.composer.attachments);
      this.composer = emptyComposer();
      this.toast(scheduled ? 'Message scheduled' : 'Message sent', 'success');
      if (this.snapshot) {
        const sent = this.snapshot.folders.find((folder) => folder.kind === 'sent');
        if (sent && this.selectedFolderId === sent.id) void this.refreshCurrent(true);
      }
      return true;
    } catch (error) {
      this.composer = {
        ...this.composer,
        busy: false,
        error: this.message(error)
      };
      return false;
    }
  }

  async saveDraft(): Promise<void> {
    if (this.composer.busy) return;
    this.composer = { ...this.composer, busy: true, error: null };
    try {
      await api.saveDraft(this.draftInput(), this.composer.attachments);
      this.composer = emptyComposer();
      this.toast('Draft saved', 'success');
    } catch (error) {
      this.composer = { ...this.composer, busy: false, error: this.message(error) };
    }
  }

  async previewAttachment(attachment: AttachmentMeta): Promise<void> {
    if (!this.detail) return;
    if (attachment.size > 12 * 1024 * 1024) {
      this.toast('This attachment is too large to preview; save it instead.', 'error');
      return;
    }
    this.preview = {
      open: true,
      loading: true,
      name: attachment.name,
      mimeType: attachment.mime_type,
      kind: 'other',
      dataUrl: '',
      text: '',
      error: null,
      attachment
    };
    try {
      const data = await api.readAttachment(
        this.detail.summary.id,
        attachment.id,
        attachment.name,
        attachment.mime_type
      );
      const mime = data.mime_type.toLowerCase();
      if (/^image\/(png|jpe?g|gif|webp|bmp)$/.test(mime)) {
        this.preview = {
          ...this.preview,
          loading: false,
          kind: 'image',
          dataUrl: `data:${mime};base64,${data.data}`
        };
      } else if (
        mime.startsWith('text/') ||
        mime.includes('json') ||
        mime.includes('csv') ||
        mime.includes('xml')
      ) {
        this.preview = {
          ...this.preview,
          loading: false,
          kind: 'text',
          text: base64ToText(data.data)
        };
      } else {
        this.preview = { ...this.preview, loading: false, kind: 'other' };
      }
    } catch (error) {
      this.preview = { ...this.preview, loading: false, error: this.message(error) };
    }
  }

  closePreview(): void {
    this.preview = { ...this.preview, open: false };
  }

  async saveAttachment(attachmentId: string, name: string): Promise<void> {
    if (!this.detail) return;
    try {
      const result = await api.saveAttachment(this.detail.summary.id, attachmentId, name);
      if (result.saved) this.toast(`Saved ${name}`, 'success');
    } catch (error) {
      this.toast(this.message(error), 'error');
    }
  }

  // ------------------------------------------------------------------ settings

  setTheme(theme: ThemePreference): void {
    this.theme = theme;
    setString('tmail.theme', theme);
    this.applyTheme(theme);
  }

  applyTheme(theme: ThemePreference): void {
    const system = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    document.documentElement.dataset.theme = theme === 'system' ? system : theme;
    document.documentElement.dataset.themePreference = theme;
  }

  setDensity(density: Density): void {
    this.density = density;
    setString('tmail.density', density);
    this.applyDensity(density);
  }

  setSidebarWidth(width: number): void {
    this.sidebarWidth = clampNumber(width, 190, 380);
    setString('tmail.sidebar-width', String(Math.round(this.sidebarWidth)));
  }

  setListWidth(width: number): void {
    this.listWidth = clampNumber(width, 300, 680);
    setString('tmail.list-width', String(Math.round(this.listWidth)));
  }

  toggleSidebar(): void {
    this.sidebarCollapsed = !this.sidebarCollapsed;
    setString('tmail.sidebar-collapsed', this.sidebarCollapsed ? '1' : '0');
  }

  preferredSender(): string {
    const addresses = this.snapshot?.sender_addresses ?? [];
    if (this.defaultSender && addresses.some((address) => address === this.defaultSender)) {
      return this.defaultSender;
    }
    return addresses[0] ?? this.snapshot?.email ?? '';
  }

  setDefaultSender(address: string): void {
    this.defaultSender = address;
    setString('tmail.default-sender', address);
  }

  setSignature(signature: string): void {
    this.signature = signature;
    setString('tmail.signature', signature);
  }

  setNotifications(enabled: boolean): void {
    this.notificationsEnabled = enabled;
    setString('tmail.notifications', enabled ? '1' : '0');
  }

  setPlaintextDefault(enabled: boolean): void {
    this.plaintextDefault = enabled;
    setString('tmail.plaintext-default', enabled ? '1' : '0');
  }

  resetLayout(): void {
    this.setSidebarWidth(248);
    this.setListWidth(420);
    if (this.sidebarCollapsed) this.toggleSidebar();
  }

  applyDensity(density: Density): void {
    document.documentElement.dataset.density = density;
  }

  private loadRemoteSenders(): string[] {
    try {
      return JSON.parse(getString('tmail.remote-senders', '[]')) as string[];
    } catch {
      return [];
    }
  }

  isRemoteAllowed(sender: string): boolean {
    const key = sender.toLowerCase();
    return this.remoteSenders.includes(key) || this.remoteOnce.includes(key);
  }

  allowRemote(sender: string, persist = true): void {
    const key = sender.toLowerCase();
    if (persist) {
      if (!this.remoteSenders.includes(key)) {
        this.remoteSenders = [...this.remoteSenders, key];
        setString('tmail.remote-senders', JSON.stringify(this.remoteSenders));
      }
      this.remoteOnce = this.remoteOnce.filter((value) => value !== key);
    } else if (!this.remoteOnce.includes(key)) {
      this.remoteOnce = [...this.remoteOnce, key];
    }
  }

  revokeRemote(sender: string): void {
    const key = sender.toLowerCase();
    this.remoteSenders = this.remoteSenders.filter((value) => value !== key);
    this.remoteOnce = this.remoteOnce.filter((value) => value !== key);
    setString('tmail.remote-senders', JSON.stringify(this.remoteSenders));
  }

  private startInboxWatch(): void {
    if (this.inboxTimer) clearInterval(this.inboxTimer);
    // Event-bus delivery is the fast path; this is the safety net for missed
    // counter batches and for the first run after a reconnect.
    this.inboxTimer = setInterval(() => {
      if (this.auth === 'signed_in') void this.checkInboxForNewMail();
    }, 60_000);
  }

  private async checkInboxForNewMail(): Promise<void> {
    if (this.inboxCheckInFlight || this.auth !== 'signed_in') return;
    const inbox = this.snapshot?.folders.find((folder) => folder.kind === 'inbox');
    if (!inbox) return;
    this.inboxCheckInFlight = true;
    const firstRun = this.inboxKnownIds.size === 0;
    try {
      const mails = await api.refreshFolder(inbox.id, 80);
      const fresh = mails.filter(
        (mail) => mail.unread && !this.inboxKnownIds.has(mail.id)
      );
      for (const mail of mails) this.inboxKnownIds.add(mail.id);
      if (!firstRun && fresh.length > 0 && this.notificationsEnabled && (await this.isWindowHidden())) {
        const newest = fresh[0];
        void notify(
          senderLabel(newest.sender.name, newest.sender.address),
          newest.subject,
          newest.id
        );
      }
    } catch {
      // Offline or session expired; the next event or poll retries.
    } finally {
      this.inboxCheckInFlight = false;
    }
  }

  private async isWindowHidden(): Promise<boolean> {
    if (!isTauri) {
      return document.visibilityState !== 'visible' || !document.hasFocus();
    }
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      // A visible window on another workspace is still "visible" to the OS;
      // focus is the signal users expect for desktop notifications.
      return !(await window.isVisible()) || !(await window.isFocused());
    } catch {
      return document.visibilityState !== 'visible' || !document.hasFocus();
    }
  }

  async openMailById(mailId: string): Promise<void> {
    if (this.auth !== 'signed_in') return;
    const inbox = this.snapshot?.folders.find((folder) => folder.kind === 'inbox');
    if (inbox && this.selectedFolderId !== inbox.id) {
      await this.selectFolder(inbox.id);
    }
    await this.openMail(mailId);
  }

  async openFromSearch(hit: SearchHit): Promise<void> {
    const folder = hit.mail.folder_ids[0];
    if (folder && folder !== this.selectedFolderId) {
      await this.selectFolder(folder);
    }
    this.searchOpen = false;
    await this.openMail(hit.mail.id);
  }

  // --------------------------------------------------------------------- misc

  toast(
    message: string,
    kind: Toast['kind'] = 'info',
    action?: Toast['action']
  ): void {
    const id = ++this.toastSeq;
    this.toasts = [...this.toasts, { id, message, kind, action }];
    const ttl = action ? 8000 : kind === 'error' ? 6500 : 3500;
    setTimeout(() => this.dismissToast(id), ttl);
  }

  dismissToast(id: number): void {
    this.toasts = this.toasts.filter((toast) => toast.id !== id);
  }

  notificationFor(mail: MailSummary): void {
    void notify(mail.sender.name || mail.sender.address, mail.subject);
  }

  message(error: unknown): string {
    if (error && typeof error === 'object' && 'code' in error) {
      const code = String((error as { code: unknown }).code);
      if (code === 'session_invalid' || code === 'not_signed_in') {
        this.auth = 'signed_out';
        this.composer = emptyComposer();
      }
    }
    if (error instanceof Error) return error.message;
    if (typeof error === 'string') return error;
    if (error && typeof error === 'object' && 'message' in error) {
      return String((error as { message: unknown }).message);
    }
    return 'Something went wrong. Try again.';
  }

  destroy(): void {
    for (const off of this.unlisten) off();
    this.unlisten = [];
    this.media?.removeEventListener('change', this.onSystemTheme);
    this.media = null;
    if (this.inboxTimer) clearInterval(this.inboxTimer);
    this.inboxTimer = null;
  }
}

function toRecipients(detail: MailDetail, sender: string): string {
  const addresses = new Set<string>();
  for (const address of detail.recipients.to) addresses.add(address.address);
  if (detail.recipients.cc.length) {
    for (const address of detail.recipients.cc) addresses.add(address.address);
  }
  addresses.delete('you@tuta.com');
  addresses.delete(sender);
  return [sender, ...addresses].join(', ');
}

function prefixSubject(subject: string, prefix: string): string {
  return subject.toLowerCase().startsWith(prefix.toLowerCase())
    ? subject
    : `${prefix} ${subject}`;
}

function quoteBody(detail: MailDetail): string {
  const text = detail.body_text
    .split('\n')
    .map((line) => `> ${line}`)
    .join('\n');
  return `\n\nOn ${new Date(detail.sent_ms || detail.summary.received_ms).toLocaleString()}, ${detail.summary.sender.name} wrote:\n${text}`;
}

export function textToHtml(text: string): string {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
  return escaped
    .split(/\n{2,}/)
    .map((paragraph) => `<p>${paragraph.replace(/\n/g, '<br>')}</p>`)
    .join('');
}

export const app = new MailApp();

if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __tmail: MailApp }).__tmail = app;
}
