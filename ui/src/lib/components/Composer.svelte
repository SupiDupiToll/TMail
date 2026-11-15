<script lang="ts">
  import { app } from '../state.svelte';
  import { formatBytes } from '../format';
  import { isTutaAddress, splitAddresses } from '../addresses';
  import Icon from './Icon.svelte';

  let dialog = $state<HTMLDialogElement | null>(null);
  let toInput = $state<HTMLInputElement | null>(null);
  let subjectInput = $state<HTMLInputElement | null>(null);
  let bodyInput = $state<HTMLTextAreaElement | null>(null);
  let fileInput = $state<HTMLInputElement | null>(null);
  let discardOpen = $state(false);
  let maximized = $state(false);
  let scheduleOpen = $state(false);
  let scheduleInput = $state('');

  const composer = $derived(app.composer);
  const senders = $derived(app.snapshot?.sender_addresses ?? []);

  const recipients = $derived([
    ...splitAddresses(composer.to),
    ...splitAddresses(composer.cc),
    ...splitAddresses(composer.bcc)
  ]);
  const externalRecipients = $derived(recipients.filter((address) => !isTutaAddress(address)));
  const allTuta = $derived(recipients.length > 0 && externalRecipients.length === 0);

  $effect(() => {
    if (composer.open && dialog && !dialog.open) {
      dialog.showModal();
      requestAnimationFrame(() => {
        if (composer.to) subjectInput?.focus();
        else toInput?.focus();
      });
    } else if (!composer.open && dialog?.open) {
      dialog.close();
    }
  });

  function close(): void {
    if (composer.busy) return;
    const hasContent = Boolean(
      composer.body.trim() ||
        composer.subject.trim() ||
        composer.to.trim() ||
        composer.cc.trim() ||
        composer.bcc.trim() ||
        composer.attachments.length
    );
    if (hasContent) {
      discardOpen = true;
      return;
    }
    app.closeComposer();
  }

  function discard(): void {
    discardOpen = false;
    app.closeComposer();
  }

  function onCancel(event: Event): void {
    event.preventDefault();
    close();
  }

  async function send(): Promise<void> {
    if (composer.busy) return;
    const ok = await app.send();
    if (ok) discardOpen = false;
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      void send();
    }
  }

  async function pickFiles(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    if (input.files) await app.addAttachment(input.files);
    input.value = '';
  }

  const title = $derived(
    composer.mode === 'reply'
      ? 'Reply'
      : composer.mode === 'reply_all'
        ? 'Reply all'
        : composer.mode === 'forward'
          ? 'Forward'
          : composer.mode === 'draft'
            ? 'Draft'
            : 'New message'
  );

  const canSend = $derived(
    !composer.busy &&
      (composer.to.trim() || composer.cc.trim() || composer.bcc.trim()) &&
      !composer.error
  );

  function openSchedule(): void {
    if (!scheduleInput) {
      const next = new Date(Date.now() + 60 * 60 * 1000);
      next.setSeconds(0, 0);
      const local = new Date(next.getTime() - next.getTimezoneOffset() * 60_000);
      scheduleInput = local.toISOString().slice(0, 16);
    }
    scheduleOpen = true;
  }

  function applySchedule(): void {
    const timestamp = new Date(scheduleInput).getTime();
    if (!Number.isFinite(timestamp)) return;
    if (timestamp <= Date.now()) {
      app.toast('Choose a time in the future.', 'error');
      return;
    }
    app.updateComposer({ sendAt: timestamp });
    scheduleOpen = false;
  }

  function clearSchedule(): void {
    app.updateComposer({ sendAt: null });
    scheduleOpen = false;
  }
</script>

<dialog
  bind:this={dialog}
  class="composer-dialog"
  class:maximized
  oncancel={onCancel}
  onkeydown={onKeydown}
  aria-labelledby="composer-title"
  onclick={(event) => {
    if (event.target === dialog) close();
  }}
>
  <section class="composer" aria-label={title}>
    <header class="compose-head">
      <h2 id="composer-title">{title}</h2>
      {#if allTuta}
        <span class="crypto-chip" title="All recipients are Tuta users; end-to-end encryption is automatic.">
          <Icon name="shield" size={13} />
          Encrypted
        </span>
      {:else if externalRecipients.length > 0}
        <span class="crypto-chip off" title="External recipients receive this message without end-to-end encryption.">
          <Icon name="eye" size={13} />
          Not encrypted
        </span>
      {/if}
      <div class="head-actions">
        <button class="btn btn-sm btn-ghost" onclick={() => app.saveDraft()} disabled={composer.busy}>
          Save draft
        </button>
        <button
          class="icon-btn"
          aria-label={maximized ? 'Restore composer size' : 'Expand composer'}
          title={maximized ? 'Restore' : 'Expand'}
          onclick={() => (maximized = !maximized)}
        >
          <Icon name={maximized ? 'minimize' : 'maximize'} size={16} />
        </button>
        <button class="icon-btn" aria-label="Close composer" title="Close (Esc)" onclick={close}>
          <Icon name="close" size={17} />
        </button>
      </div>
    </header>

    {#if discardOpen}
      <div class="discard" role="alert">
        <span>Discard this message?</span>
        <button class="btn btn-sm" onclick={() => (discardOpen = false)}>Keep editing</button>
        <button class="btn btn-sm btn-danger" onclick={discard}>Discard</button>
      </div>
    {/if}

    <div class="fields">
      <div class="field-row">
        <label class="field-label" for="compose-from">From</label>
        <div class="from-wrap">
          <select
            id="compose-from"
            class="from-select"
            bind:value={composer.from}
            onchange={() => app.updateComposer({ from: composer.from })}
          >
            {#each senders as sender (sender)}
              <option value={sender}>{sender}</option>
            {/each}
          </select>
          <Icon name="chevron-down" size={14} />
        </div>
      </div>
      <div class="field-row">
        <label class="field-label" for="compose-to">To</label>
        <input id="compose-to" bind:this={toInput} class="bare-input" bind:value={composer.to} placeholder="name@example.com" autocomplete="off" />
        <button class="mini-link" type="button" onclick={() => app.updateComposer({ showCc: !composer.showCc })}>
          {composer.showCc ? 'Hide' : 'Cc/Bcc'}
        </button>
      </div>
      {#if composer.showCc}
        <div class="field-row">
          <label class="field-label" for="compose-cc">Cc</label>
          <input id="compose-cc" class="bare-input" bind:value={composer.cc} placeholder="Optional" autocomplete="off" />
        </div>
        <div class="field-row">
          <label class="field-label" for="compose-bcc">Bcc</label>
          <input id="compose-bcc" class="bare-input" bind:value={composer.bcc} placeholder="Optional" autocomplete="off" />
        </div>
      {/if}
      <div class="field-row">
        <label class="field-label" for="compose-subject">Subject</label>
        <input id="compose-subject" bind:this={subjectInput} class="bare-input subject-input" bind:value={composer.subject} placeholder="Subject" autocomplete="off" />
      </div>
    </div>

    {#if externalRecipients.length > 0}
      <p class="crypto-note">
        <Icon name="eye" size={13} />
        External recipients receive this without end-to-end encryption. Password-protected external mail is not available yet.
      </p>
    {/if}

    <textarea
      bind:this={bodyInput}
      class="body"
      bind:value={composer.body}
      placeholder="Write your message…"
      aria-label="Message body"
      spellcheck="true"
    ></textarea>

    {#if composer.attachments.length > 0}
      <div class="attachments" aria-label="Attached files">
        {#each composer.attachments as attachment, index (attachment.filename + index)}
          <span class="attach-chip">
            <Icon name="attachment" size={13} />
            <span class="attach-name">{attachment.filename}</span>
            <span class="faint tiny">{formatBytes(attachment.size)}</span>
            <button class="remove" aria-label={`Remove ${attachment.filename}`} onclick={() => app.removeAttachment(index)}>
              <Icon name="close" size={11} />
            </button>
          </span>
        {/each}
      </div>
    {/if}

    {#if composer.error}
      <div class="send-error" role="alert">
        <Icon name="bolt" size={15} />
        <div>
          <strong>Message not sent.</strong>
          <p>{composer.error}</p>
          <p class="tiny">Your text is still here.</p>
        </div>
        <button
          class="btn btn-sm"
          onclick={() => navigator.clipboard?.writeText(composer.error ?? '')}
        >
          Copy details
        </button>
      </div>
    {/if}

    {#if scheduleOpen}
      <div class="schedule-row" role="group" aria-label="Schedule send">
        <Icon name="clock" size={15} />
        <input
          class="schedule-input"
          type="datetime-local"
          bind:value={scheduleInput}
          aria-label="Send at"
        />
        <button class="btn btn-sm" onclick={applySchedule}>Set</button>
        <button class="btn btn-sm btn-ghost" onclick={clearSchedule}>Now</button>
      </div>
    {/if}

    <footer class="compose-foot">
      <button class="btn btn-primary send" onclick={send} disabled={!canSend}>
        {#if composer.busy}<span class="spinner"></span>{/if}
        {composer.busy ? 'Sending…' : composer.sendAt ? 'Schedule' : 'Send'}
        <span class="send-kbd"><kbd class="kbd">Ctrl</kbd><kbd class="kbd">↵</kbd></span>
      </button>
      <button class="icon-btn" aria-label="Attach file" title="Attach file" onclick={() => fileInput?.click()}>
        <Icon name="attachment" size={17} />
      </button>
      <button
        class="icon-btn"
        aria-label={composer.sendAt ? 'Change scheduled send' : 'Schedule send'}
        title={composer.sendAt ? 'Change schedule' : 'Schedule send'}
        class:scheduled={composer.sendAt !== null}
        onclick={openSchedule}
      >
        <Icon name="clock" size={17} />
      </button>
      <input bind:this={fileInput} type="file" multiple hidden onchange={pickFiles} />
      <div class="tolong"></div>
      {#if composer.sendAt}
        <span class="schedule-chip">
          <Icon name="clock" size={12} />
          {new Date(composer.sendAt).toLocaleString()}
          <button aria-label="Clear schedule" onclick={() => app.updateComposer({ sendAt: null })}>
            <Icon name="close" size={10} />
          </button>
        </span>
      {/if}
      <label class="toggle" title="Send as text/plain">
        <input type="checkbox" checked={composer.plaintext} onchange={(e) => app.updateComposer({ plaintext: (e.target as HTMLInputElement).checked })} />
        Plain text
      </label>
    </footer>
  </section>
</dialog>

<style>
  .composer-dialog {
    width: min(820px, calc(100vw - 48px));
    max-width: none;
    height: min(720px, calc(100vh - 64px));
    max-height: none;
    margin: auto 24px 24px auto;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    overflow: hidden;
    resize: both;
    min-width: 520px;
    min-height: 420px;
  }
  .composer-dialog.maximized {
    resize: none;
    width: calc(100vw - 24px);
    height: calc(100vh - 24px);
    margin: 12px;
  }
  .composer-dialog::backdrop {
    background: var(--scrim);
    backdrop-filter: blur(2px);
  }
  .composer {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .compose-head {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    padding: 12px 12px 12px 18px;
    border-bottom: 1px solid var(--border);
  }
  .compose-head h2 {
    font-size: var(--text-md);
    font-weight: 720;
    letter-spacing: -0.015em;
  }
  .crypto-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 10.5px;
    font-weight: 700;
    white-space: nowrap;
  }
  .crypto-chip.off {
    background: var(--panel-2);
    color: var(--muted);
  }
  .head-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
  }
  .fields {
    flex: none;
    padding: 4px 18px 0;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    border-bottom: 1px solid var(--border);
  }
  .field-label {
    width: 52px;
    flex: none;
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--muted);
  }
  .bare-input,
  .from-select {
    flex: 1;
    min-width: 0;
    height: 38px;
    border: 0;
    background: transparent;
    outline: none;
    font-size: var(--text-sm);
    color: var(--text);
  }
  .from-wrap {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
  }
  .from-select {
    width: 100%;
    padding-right: 22px;
    appearance: none;
    cursor: pointer;
    font-weight: 600;
  }
  .from-wrap :global(svg) {
    position: absolute;
    right: 4px;
    color: var(--faint);
    pointer-events: none;
  }
  .bare-input::placeholder {
    color: var(--faint);
  }
  .subject-input {
    font-weight: 600;
  }
  .field-row:focus-within {
    border-bottom-color: var(--accent);
  }
  .mini-link {
    font-size: var(--text-xs);
    color: var(--accent);
    font-weight: 650;
    white-space: nowrap;
  }
  .crypto-note {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    margin: 8px 18px 0;
    padding: 7px 9px;
    border-radius: var(--radius-xs);
    background: var(--panel-2);
    color: var(--muted);
    font-size: 10.5px;
    line-height: 1.4;
  }
  .body {
    flex: 1;
    min-height: 160px;
    padding: 14px 18px;
    border: 0;
    outline: none;
    resize: none;
    background: transparent;
    font-family: var(--font-reading);
    font-size: 1rem;
    line-height: 1.62;
    font-variation-settings: 'opsz' 20, 'wght' 430;
  }
  .body::placeholder {
    color: var(--faint);
    font-family: var(--font-ui);
    font-size: var(--text-sm);
  }
  .attachments {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    flex: none;
    padding: 6px 18px;
    border-top: 1px solid var(--border);
  }
  .attach-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 280px;
    height: 27px;
    padding: 0 4px 0 9px;
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    background: var(--panel-2);
    font-size: var(--text-xs);
  }
  .attach-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .remove {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    color: var(--faint);
  }
  .remove:hover {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .send-error {
    display: flex;
    gap: 8px;
    flex: none;
    margin: 0 18px 8px;
    padding: 9px 11px;
    border-radius: var(--radius-xs);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: var(--text-xs);
  }
  .send-error p {
    margin-top: 2px;
    line-height: 1.4;
  }
  .compose-foot {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: none;
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    background: var(--panel-2);
  }
  .send {
    height: 36px;
    padding-left: 15px;
    padding-right: 10px;
    gap: 8px;
    border-radius: var(--radius-sm);
  }
  .send-kbd {
    display: inline-flex;
    gap: 2px;
    opacity: 0.72;
  }
  .send-kbd .kbd {
    height: 17px;
    min-width: 17px;
    font-size: 9px;
    background: transparent;
    color: inherit;
    border-color: currentColor;
  }
  .tolong {
    flex: 1;
  }
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: var(--text-xs);
    color: var(--muted);
    cursor: pointer;
    white-space: nowrap;
  }
  .toggle input {
    accent-color: var(--accent);
  }
  .schedule-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    padding: 7px 16px;
    border-top: 1px solid var(--border);
    background: var(--panel);
    color: var(--muted);
  }
  .schedule-input {
    flex: 1;
    min-width: 0;
    height: 30px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    background: var(--bg);
    color: var(--text);
    font-size: var(--text-xs);
  }
  .schedule-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 6px 0 8px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 10.5px;
    font-weight: 650;
    white-space: nowrap;
  }
  .schedule-chip button {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    color: inherit;
  }
  .schedule-chip button:hover {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }
  .icon-btn.scheduled {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .discard {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--warn-soft);
    color: var(--warn);
    font-size: var(--text-xs);
    border-bottom: 1px solid var(--border);
  }
  .discard span {
    flex: 1;
  }
  @media (max-width: 720px) {
    .composer-dialog {
      width: 100vw;
      height: 100vh;
      margin: 0;
      border-radius: 0;
      border: 0;
    }
  }
</style>
