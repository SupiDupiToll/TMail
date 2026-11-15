<script lang="ts">
  import { app } from '../state.svelte';
  import { formatBytes, formatFullDate, senderLabel } from '../format';
  import { sanitizeMailHtml } from '../mail/sanitize';
  import { parseMailto } from '../mailto';
  import { openExternal } from '../api';
  import Avatar from './Avatar.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    onShortcut?: (key: string, ctrl: boolean, shift: boolean) => void;
    onBack?: () => void;
  }
  let { onShortcut, onBack }: Props = $props();

  let iframe = $state<HTMLIFrameElement | null>(null);
  let bodyHeight = $state(320);
  let hoveredLink = $state<string | null>(null);

  const detail = $derived(app.detail);
  const allowRemote = $derived(
    detail ? app.isRemoteAllowed(detail.summary.sender.address) : false
  );
  const visibleAttachments = $derived(
    (detail?.attachments ?? []).filter((attachment) => !attachment.is_inline)
  );
  const sanitized = $derived(
    detail
      ? sanitizeMailHtml(detail.body_html, {
          allowRemote,
          sender: detail.summary.sender.address
        })
      : null
  );

  $effect(() => {
    detail?.summary.id;
    bodyHeight = 320;
    hoveredLink = null;
  });

  let frameCleanup: (() => void) | null = null;
  let moreMenu = $state<HTMLDetailsElement | null>(null);

  $effect(() => {
    if (!moreMenu) return;
    const onDocumentClick = (event: MouseEvent) => {
      if (moreMenu?.open && !moreMenu.contains(event.target as Node)) {
        moreMenu.open = false;
      }
    };
    document.addEventListener('click', onDocumentClick);
    return () => document.removeEventListener('click', onDocumentClick);
  });

  function cleanupFrame(): void {
    frameCleanup?.();
    frameCleanup = null;
  }

  function handleLink(href: string): void {
    if (/^mailto:/i.test(href)) {
      const request = parseMailto(href);
      if (request) app.openMailtoRequest(request);
    } else if (/^https?:/i.test(href)) {
      void openExternal(href).catch((error) =>
        app.toast(`Could not open link: ${app.message(error)}`, 'error')
      );
    }
  }

  function attachFrame(attempt = 0): void {
    if (!iframe) return;
    cleanupFrame();
    const doc = iframe.contentDocument;
    if (!doc?.body) {
      if (attempt < 10) setTimeout(() => attachFrame(attempt + 1), 40);
      return;
    }

    const measure = () => {
      const height = Math.max(
        doc.body.scrollHeight,
        doc.documentElement.scrollHeight
      );
      const next = Math.min(20000, Math.max(120, height));
      if (Math.abs(next - bodyHeight) > 1) bodyHeight = next;
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(doc.body);
    window.addEventListener('resize', measure);

    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest<HTMLElement>('a[data-href]');
      if (!anchor) return;
      event.preventDefault();
      handleLink(anchor.getAttribute('data-href') ?? '');
    };
    const onOver = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest<HTMLElement>('a[data-href]');
      hoveredLink = anchor?.getAttribute('data-href') ?? null;
    };
    const onLeave = () => {
      hoveredLink = null;
    };
    const onKey = (event: KeyboardEvent) => {
      const key = event.key;
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest<HTMLElement>('a[data-href]');
      if (anchor && (key === 'Enter' || key === ' ')) {
        event.preventDefault();
        handleLink(anchor.getAttribute('data-href') ?? '');
        return;
      }
      const action =
        ['j', 'k', 'u', 'e', '#', 'r', 'a', 'f', 'c', 'x', 'g', 'R', 'Delete', 'Escape', '/', '?'].includes(key) ||
        (event.ctrlKey && key.toLowerCase() === 'k');
      if (!action) return;
      onShortcut?.(key, event.ctrlKey, event.shiftKey);
      if (key === 'Escape') event.preventDefault();
    };

    doc.addEventListener('click', onClick);
    doc.addEventListener('mouseover', onOver);
    doc.documentElement.addEventListener('mouseleave', onLeave);
    doc.addEventListener('keydown', onKey);

    frameCleanup = () => {
      observer.disconnect();
      window.removeEventListener('resize', measure);
      doc.removeEventListener('click', onClick);
      doc.removeEventListener('mouseover', onOver);
      doc.documentElement.removeEventListener('mouseleave', onLeave);
      doc.removeEventListener('keydown', onKey);
    };
  }

  $effect(() => {
    detail?.summary.id;
    allowRemote;
    cleanupFrame();
    bodyHeight = 320;
    hoveredLink = null;
    const timer = setTimeout(() => attachFrame(), 0);
    return () => {
      clearTimeout(timer);
      cleanupFrame();
    };
  });

  function mailtoLabel(): string {
    return detail ? senderLabel(detail.summary.sender.name, detail.summary.sender.address) : '';
  }

  function recipientsLine(): string {
    if (!detail) return '';
    const to = detail.recipients.to.map((a) => a.address).join(', ');
    const cc = detail.recipients.cc.map((a) => a.address);
    return cc.length ? `to ${to} · cc ${cc.join(', ')}` : `to ${to || 'undisclosed recipients'}`;
  }
</script>

<section class="reader" aria-label="Message">
  {#if app.loadingMail}
    <div class="reader-loading" aria-busy="true" aria-label="Opening message">
      <div class="skeleton-line w40"></div>
      <div class="skeleton-line w70"></div>
      <div class="skeleton-line w90"></div>
      <div class="skeleton-line w60"></div>
    </div>
  {:else if app.error && !detail}
    <div class="reader-state">
      <div class="state-icon"><Icon name="bolt" size={24} /></div>
      <h2>Could not open this message</h2>
      <p>{app.error}</p>
      <button class="btn" onclick={() => app.selectedMailId && app.openMail(app.selectedMailId)}>
        Try again
      </button>
    </div>
  {:else if !detail}
    <div class="reader-state quiet">
      <div class="state-icon"><Icon name="mailbox" size={26} /></div>
      <h2>No message selected</h2>
      <p>Pick a message with <kbd class="kbd">j</kbd> / <kbd class="kbd">k</kbd>, or press <kbd class="kbd">Ctrl</kbd><kbd class="kbd">K</kbd> for every action.</p>
    </div>
  {:else}
    <header class="reader-head">
      <div class="subject-row">
        <button class="icon-btn back-btn" aria-label="Back to message list" onclick={() => onBack?.()}>
          <Icon name="arrow-left" size={17} />
        </button>
        <h2 title={detail.summary.subject}>{detail.summary.subject}</h2>
        <div class="actions" role="toolbar" aria-label="Message actions">
          <button class="icon-btn" aria-label="Reply" title="Reply (r)" onclick={() => app.compose('reply')}>
            <Icon name="reply" size={17} />
          </button>
          <button class="icon-btn" aria-label="Reply all" title="Reply all (a)" onclick={() => app.compose('reply_all')}>
            <Icon name="reply-all" size={17} />
          </button>
          <button class="icon-btn" aria-label="Forward" title="Forward (f)" onclick={() => app.compose('forward')}>
            <Icon name="forward" size={17} />
          </button>
          <span class="sep" aria-hidden="true"></span>
          <button
            class="icon-btn"
            aria-label={detail.summary.unread ? 'Mark read' : 'Mark unread'}
            title="Mark unread (u)"
            onclick={() => app.toggleUnread([detail.summary.id])}
          >
            <Icon name={detail.summary.unread ? 'mail-open' : 'unread'} size={17} />
          </button>
          <button class="icon-btn" aria-label="Archive" title="Archive (e)" onclick={() => app.archive([detail.summary.id])}>
            <Icon name="archive" size={17} />
          </button>
          <button class="icon-btn" aria-label="Move to trash" title="Trash (#)" onclick={() => app.trash([detail.summary.id])}>
            <Icon name="trash" size={17} />
          </button>
          <details class="menu" bind:this={moreMenu}>
            <summary class="icon-btn" aria-label="More actions" title="More actions">
              <Icon name="more" size={17} />
            </summary>
            <div class="menu-pop" role="menu" aria-label="More actions">
              {#each app.snapshot?.folders ?? [] as folder (folder.id)}
                {#if folder.id !== app.selectedFolderId}
                  <button role="menuitem" onclick={() => { moreMenu && (moreMenu.open = false); app.moveTo(folder.id, [detail.summary.id]); }}>
                    <Icon name="folder" size={14} />
                    Move to {folder.name || folder.kind}
                  </button>
                {/if}
              {/each}
              <hr />
              <button role="menuitem" class="danger" onclick={() => { moreMenu && (moreMenu.open = false); app.deleteForever([detail.summary.id]); }}>
                <Icon name="trash" size={14} />
                Delete permanently
              </button>
            </div>
          </details>
        </div>
      </div>

      <div class="meta-row">
        <Avatar name={detail.summary.sender.name} address={detail.summary.sender.address} size={34} />
        <div class="meta">
          <div class="meta-line">
            <span class="sender">{mailtoLabel()}</span>
            <span class="address">&lt;{detail.summary.sender.address}&gt;</span>
            {#if detail.summary.confidential}
              <span class="chip chip-accent"><Icon name="shield" size={11} /> Confidential</span>
            {/if}
          </div>
          <div class="meta-line faint tiny">
            <span>{recipientsLine()}</span>
            <span aria-hidden="true">·</span>
            <time datetime={new Date(detail.sent_ms || detail.summary.received_ms).toISOString()}>
              {formatFullDate(detail.sent_ms || detail.summary.received_ms)}
            </time>
          </div>
        </div>
      </div>

      {#if detail.summary.phishing_status > 0}
        <div class="notice danger" role="alert">
          <Icon name="shield" size={16} />
          <div>
            <strong>This message looks suspicious.</strong>
            <p>The sender or links do not match the display name. Verify before replying or opening links.</p>
          </div>
        </div>
      {/if}

      {#if !allowRemote}
        <div class="notice">
          <Icon name="image-off" size={15} />
          <span class="notice-copy"><strong>Remote images blocked</strong></span>
          <div class="notice-actions">
            <button class="btn btn-sm" onclick={() => app.allowRemote(detail.summary.sender.address, false)}>
              Load
            </button>
            <button class="btn btn-sm btn-ghost" onclick={() => { app.allowRemote(detail.summary.sender.address, true); app.toast(`Images allowed for ${detail.summary.sender.address}`, 'info'); }}>
              Always allow
            </button>
          </div>
        </div>
      {/if}
    </header>

    <div class="body-wrap" role="group" aria-label="Message content">
      {#if sanitized}
        <iframe
          bind:this={iframe}
          title={`Message body from ${detail.summary.sender.address}`}
          srcdoc={sanitized.srcdoc}
          sandbox={sanitized.sandbox}
          referrerpolicy="no-referrer"
          style:height={`${bodyHeight}px`}
          onload={() => attachFrame()}
        ></iframe>
      {/if}
    </div>

    {#if sanitized?.links.length}
      <div class="link-strip" aria-label="Links in this message">
        <span class="link-strip-label">Links</span>
        {#each sanitized.links.slice(0, 10) as link (link.href)}
          <button class="link-chip" title={link.href} onclick={() => handleLink(link.href)}>
            {link.label}
          </button>
        {/each}
      </div>
    {/if}

    {#if visibleAttachments.length > 0}
      <section class="attachments" aria-label="Attachments">
        <h3>{visibleAttachments.length === 1 ? '1 attachment' : `${visibleAttachments.length} attachments`}</h3>
        {#each visibleAttachments as attachment (attachment.id)}
          <div class="attachment">
            <span class="file-icon" aria-hidden="true"><Icon name="attachment" size={15} /></span>
            <div class="file-meta">
              <button class="file-name" title={`Preview ${attachment.name}`} onclick={() => app.previewAttachment(attachment)}>
                {attachment.name}
              </button>
              <span class="faint tiny">{attachment.mime_type} · {formatBytes(attachment.size)}</span>
            </div>
            <button
              class="icon-btn"
              aria-label={`Preview ${attachment.name}`}
              title="Preview"
              onclick={() => app.previewAttachment(attachment)}
            >
              <Icon name="eye" size={16} />
            </button>
            <button class="btn btn-sm" onclick={() => app.saveAttachment(attachment.id, attachment.name)}>
              <Icon name="download" size={14} />
              Save
            </button>
          </div>
        {/each}
      </section>
    {/if}

    {#if hoveredLink}
      <div class="status-line" aria-live="polite">
        <span class="link-preview">{hoveredLink}</span>
      </div>
    {/if}
  {/if}
</section>

<style>
  .reader {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--bg);
    overflow: hidden;
  }
  .reader-head {
    flex: none;
    padding: 20px 28px 14px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
    overflow-y: auto;
    max-height: 58vh;
  }
  .subject-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .subject-row h2 {
    flex: 1;
    min-width: 0;
    font-size: 1.55rem;
    line-height: 1.25;
    font-weight: 720;
    letter-spacing: -0.012em;
    overflow-wrap: anywhere;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 1px;
    flex: none;
    margin: -4px -6px 0 0;
  }
  .back-btn {
    display: none;
    margin: -3px 0 0 -6px;
  }
  @media (max-width: 1120px) {
    .back-btn {
      display: inline-flex;
    }
  }
  .sep {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 5px;
  }
  .meta-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 14px;
  }
  .meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .meta-line {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex-wrap: wrap;
  }
  .sender {
    font-weight: 720;
    font-size: var(--text-sm);
  }
  .address {
    color: var(--faint);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .notice {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin-top: 12px;
    padding: 9px 11px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--panel-2);
    color: var(--muted);
    font-size: var(--text-xs);
  }
  .notice strong {
    color: var(--text);
    font-weight: 680;
  }
  .notice p {
    margin-top: 1px;
    line-height: 1.45;
  }
  .notice-copy {
    flex: 1;
    min-width: 0;
  }
  .notice-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: none;
  }
  .notice.danger {
    border-color: var(--danger);
    background: var(--danger-soft);
    color: var(--danger);
  }
  .notice.danger strong {
    color: var(--danger);
  }
  .body-wrap {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 24px 28px 14px;
  }
  iframe {
    display: block;
    width: 100%;
    max-width: 100%;
    margin: 0 auto;
    border: 0;
    border-radius: var(--radius-md);
    background: #ffffff;
    color-scheme: light;
    overflow: hidden;
  }
  .link-strip {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
    flex: none;
    padding: 8px 28px;
    border-top: 1px solid var(--border);
    background: var(--panel);
  }
  .link-strip-label {
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--faint);
    margin-right: 2px;
  }
  .link-chip {
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .link-chip:hover {
    background: color-mix(in srgb, var(--accent) 18%, var(--panel));
  }
  .attachments {
    flex: none;
    padding: 12px 28px 8px;
    border-top: 1px solid var(--border);
    background: var(--panel);
  }
  .attachments h3 {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--muted);
    margin-bottom: 7px;
  }
  .attachment {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 6px 0;
    border-top: 1px solid var(--border);
  }
  .attachment:first-of-type {
    border-top: 0;
  }
  .file-icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    background: var(--panel-2);
    color: var(--faint);
    flex: none;
  }
  .file-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }
  .file-name {
    font-size: var(--text-sm);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: left;
    color: var(--text);
  }
  .file-name:hover {
    color: var(--accent);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .status-line {
    flex: none;
    min-height: 28px;
    display: flex;
    align-items: center;
    padding: 0 28px;
    border-top: 1px solid var(--border);
    background: var(--panel);
    overflow: hidden;
  }
  .link-preview {
    font-size: var(--text-xs);
    color: var(--accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .reader-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100%;
    padding: 40px;
    text-align: center;
    color: var(--muted);
  }
  .reader-state h2 {
    font-size: var(--text-lg);
    color: var(--text);
  }
  .reader-state p {
    max-width: 46ch;
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .reader-state.quiet .state-icon {
    color: var(--faint);
  }
  .state-icon {
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--panel-2);
    color: var(--faint);
    margin-bottom: 4px;
  }
  .reader-loading {
    padding: 28px 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .skeleton-line {
    height: 10px;
    border-radius: 3px;
    background: var(--panel-3);
  }
  .w40 { width: 40%; }
  .w60 { width: 60%; }
  .w70 { width: 70%; }
  .w90 { width: 90%; }

  .menu {
    position: relative;
  }
  .menu summary {
    list-style: none;
    cursor: pointer;
  }
  .menu summary::-webkit-details-marker {
    display: none;
  }
  .menu-pop {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 40;
    min-width: 210px;
    max-height: 340px;
    overflow-y: auto;
    padding: 5px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--panel);
    box-shadow: var(--shadow);
  }
  .menu-pop button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    text-align: left;
    color: var(--text);
  }
  .menu-pop button:hover,
  .menu-pop button:focus-visible {
    background: var(--hover-bg);
  }
  .menu-pop button.danger {
    color: var(--danger);
  }
  .menu-pop hr {
    border: 0;
    border-top: 1px solid var(--border);
    margin: 5px 0;
  }
</style>
