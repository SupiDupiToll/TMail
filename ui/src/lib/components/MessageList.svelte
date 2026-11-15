<script lang="ts">
  import { app } from '../state.svelte';
  import type { MailSummary } from '../types';
  import { senderLabel, plural } from '../format';
  import VirtualList from './VirtualList.svelte';
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';

  interface Props {
    onMenu?: () => void;
  }
  let { onMenu }: Props = $props();

  const folder = $derived(
    app.snapshot?.folders.find((item) => item.id === app.selectedFolderId) ?? null
  );

  const folderName = $derived(
    app.searchMode
      ? `Search: ${app.searchQuery}`
      : folder?.name || (folder?.kind ? capitalise(folder.kind) : 'Mail')
  );

  const displayedMails = $derived.by(() => {
    if (!app.searchMode) return app.mails;
    return app.searchHits
      .map((hit) => hit.mail)
      .filter((mail) => {
        if (app.searchFilter === 'unread' && !mail.unread) return false;
        if (app.searchFilter === 'attachments' && !mail.has_attachments) return false;
        if (app.searchFolder && !mail.folder_ids.includes(app.searchFolder)) return false;
        return true;
      });
  });

  const rows = $derived(displayedMails);

  const focusedIndex = $derived(
    app.selectedMailId
      ? rows.findIndex((mail) => mail.id === app.selectedMailId)
      : -1
  );

  let localFocus = $state(-1);

  $effect(() => {
    if (focusedIndex >= 0) localFocus = focusedIndex;
  });

  $effect(() => {
    // Folder switches reset the keyboard cursor to the first row.
    app.selectedFolderId;
    localFocus = -1;
  });

  function capitalise(value: string): string {
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function rowHeight(): number {
    return app.density === 'comfortable' ? 70 : app.density === 'spacious' ? 82 : 58;
  }

  function choose(index: number, open = true): void {
    localFocus = index;
    const mail = rows[index];
    if (!mail) return;
    if (open) void app.openMail(mail.id);
    else app.selectedMailId = mail.id;
  }

  export function navigate(delta: number): void {
    if (!rows.length) return;
    const base = localFocus >= 0 ? localFocus : app.selectedMailId ? focusedIndex : -1;
    const next = Math.min(rows.length - 1, Math.max(0, base + delta));
    choose(next);
  }

  export function openFocused(): void {
    if (localFocus >= 0) choose(localFocus);
  }

  export function focusFirstUnread(): void {
    const index = rows.findIndex((mail) => mail.unread);
    choose(index >= 0 ? index : 0);
  }

  function emptyTitle(kind?: string): string {
    switch (kind) {
      case 'sent':
        return 'Nothing sent yet';
      case 'drafts':
        return 'No drafts';
      case 'trash':
        return 'Trash is empty';
      case 'archive':
        return 'Archive is empty';
      case 'spam':
        return 'No spam';
      case 'custom':
        return 'This folder is empty';
      default:
        return 'Inbox zero';
    }
  }

  function emptyBody(kind?: string): string {
    switch (kind) {
      case 'sent':
        return 'Messages you send will appear here.';
      case 'drafts':
        return 'Start writing and save a draft to keep it here.';
      case 'trash':
        return 'Deleted messages stay here until you remove them forever.';
      case 'archive':
        return 'Archived mail is kept out of the way until you need it.';
      case 'spam':
        return 'Nothing has been classified as spam.';
      case 'custom':
        return 'Move a message here or write a new one.';
      default:
        return 'Nothing needs your attention. Press c to write, or Ctrl+K for everything else.';
    }
  }
</script>

<section id="message-list" tabindex="-1" class="list-pane" aria-label={`${folderName} message list`}>
  <header class="list-head">
    <div class="title-row">
      <button class="icon-btn menu-btn" aria-label="Open folders" onclick={() => onMenu?.()}>
        <Icon name="menu" size={17} />
      </button>
      <h1>{folderName}</h1>
      <span class="count tabular">{displayedMails.length}</span>
      <div class="grow"></div>
      <button
        class="icon-btn"
        aria-label="Search mail"
        title="Search (Ctrl+K)"
        onclick={() => (app.commandOpen = true)}
      >
        <Icon name="search" size={16} />
      </button>
      {#if app.searchMode}
        <button class="icon-btn" aria-label="Clear search" title="Clear search" onclick={() => app.closeSearch()}>
          <Icon name="close" size={16} />
        </button>
      {/if}
      <button
        class="icon-btn"
        aria-label="Refresh folder"
        title="Refresh (R)"
        onclick={() => app.refreshCurrent()}
      >
        <Icon name="refresh" size={16} />
      </button>
      <button
        class="icon-btn compose-quick"
        aria-label="Compose new message"
        title="Compose (c)"
        onclick={() => app.compose('new')}
      >
        <Icon name="compose" size={17} />
      </button>
    </div>
    {#if app.refreshing}
      <div class="refresh-line" aria-hidden="true"></div>
    {/if}
  </header>

  {#if app.searchMode}
    <div class="search-filters" role="toolbar" aria-label="Search filters">
      <button
        class="filter-chip"
        class:active={app.searchFilter === 'all'}
        aria-pressed={app.searchFilter === 'all'}
        onclick={() => (app.searchFilter = 'all')}
      >All</button>
      <button
        class="filter-chip"
        class:active={app.searchFilter === 'unread'}
        aria-pressed={app.searchFilter === 'unread'}
        onclick={() => (app.searchFilter = 'unread')}
      >Unread</button>
      <button
        class="filter-chip"
        class:active={app.searchFilter === 'attachments'}
        aria-pressed={app.searchFilter === 'attachments'}
        onclick={() => (app.searchFilter = 'attachments')}
      >Attachments</button>
      <select class="filter-select" aria-label="Limit search to folder" bind:value={app.searchFolder}>
        <option value="">All folders</option>
        {#each app.snapshot?.folders ?? [] as option (option.id)}
          {#if option.kind !== 'label'}
            <option value={option.id}>{option.name || option.kind}</option>
          {/if}
        {/each}
      </select>
    </div>
  {/if}

  {#if app.error}
    <div class="inline-error" role="alert">
      <span>{app.error}</span>
      <button class="btn btn-sm" onclick={() => app.refreshCurrent()}>Retry</button>
    </div>
  {/if}

  {#if app.selectedIds.length > 0}
    <div class="bulk" role="toolbar" aria-label="Selected messages">
      <span class="bulk-count">{plural(app.selectedIds.length, 'selected')}</span>
      <div class="grow"></div>
      <button class="btn btn-sm" onclick={() => app.toggleUnread()}>Read/Unread</button>
      <button class="btn btn-sm" onclick={() => app.archive()}>Archive</button>
      <button class="btn btn-sm btn-danger" onclick={() => app.trash()}>Trash</button>
      <button class="btn btn-sm btn-ghost" onclick={() => app.clearSelection()}>Clear</button>
    </div>
  {/if}

  <div class="list-body">
    {#if app.loadingFolder && app.mails.length === 0}
      <div class="skeleton" aria-label="Loading messages" aria-busy="true">
        {#each Array(9) as _, i}
          <div class="skeleton-row" style:--i={i}>
            <div class="sk-avatar"></div>
            <div class="sk-lines">
              <div class="sk-line short"></div>
              <div class="sk-line"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else if displayedMails.length === 0}
      <div class="empty">
        <div class="empty-mark" aria-hidden="true">
          <Icon name={folder?.kind ?? 'inbox'} size={26} />
        </div>
        <h2>{app.searchMode ? 'No matching mail' : emptyTitle(folder?.kind)}</h2>
        <p>{app.searchMode ? 'Try a different search or clear the filters.' : emptyBody(folder?.kind)}</p>
        {#if folder?.kind === 'inbox' || folder?.kind === 'custom'}
          <button class="btn btn-primary" onclick={() => app.compose('new')}>
            <Icon name="compose" size={15} />
            Write a message
          </button>
        {/if}
      </div>
    {:else}
      <VirtualList
        items={displayedMails}
        rowHeight={rowHeight()}
        selectedIndex={focusedIndex}
        focusedIndex={localFocus}
        label={`${folderName} messages`}
        onFocusIndex={(index) => choose(index)}
        onActivate={(index) => choose(index)}
      >
        {#snippet children(mail: MailSummary, index: number)}
          <div
            class="mail-row"
            class:unread={mail.unread}
            class:checked={app.selectedIds.includes(mail.id)}
          >
            <div class="avatar-wrap">
              <Avatar
                name={app.contactName(mail.sender.address) ?? mail.sender.name}
                address={mail.sender.address}
                size={34}
              />
            </div>
            <div class="content">
              <div class="line">
                <span class="sender">
                  {app.contactName(mail.sender.address) ?? senderLabel(mail.sender.name, mail.sender.address)}
                </span>
                {#if mail.confidential}
                  <span class="mini-flag" title="End-to-end encrypted" aria-label="End-to-end encrypted">
                    <Icon name="shield" size={12} />
                  </span>
                {/if}
                {#if mail.has_attachments}
                  <span class="mini-flag" title={`${mail.attachment_count} attachment(s)`} aria-label={`${mail.attachment_count} attachment(s)`}>
                    <Icon name="attachment" size={12} />
                  </span>
                {/if}
                {#if mail.is_draft}
                  <span class="chip">Draft</span>
                {/if}
                <span class="grow"></span>
                {#if mail.unread}
                  <span class="unread-dot" aria-label="Unread"></span>
                {/if}
                {#if app.selectedIds.includes(mail.id)}
                  <span class="check"><Icon name="check" size={11} /></span>
                {/if}
              </div>
              <div class="line">
                <span class="subject">{mail.subject}</span>
              </div>
            </div>
          </div>
        {/snippet}
      </VirtualList>
    {/if}
  </div>
</section>

<style>
  .list-pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--panel);
    border-right: 1px solid var(--border);
    container-type: inline-size;
  }
  .list-head {
    position: relative;
    flex: none;
    padding: 12px 12px 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 34px;
  }
  h1 {
    font-size: var(--text-md);
    font-weight: 720;
    letter-spacing: -0.015em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .count {
    color: var(--faint);
    font-size: var(--text-xs);
    font-weight: 600;
    padding: 1px 7px;
    border-radius: var(--radius-pill);
    background: var(--panel-2);
  }
  .menu-btn {
    display: none;
    margin-left: -6px;
  }
  .compose-quick {
    display: none;
  }
  .refresh-line {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: -1px;
    height: 2px;
    border-radius: 2px;
    background: var(--accent);
    transform-origin: left center;
    animation: progress 1.1s var(--ease-out) infinite;
  }
  @keyframes progress {
    0% { transform: scaleX(0); opacity: 0.8; }
    60% { transform: scaleX(0.7); opacity: 1; }
    100% { transform: scaleX(1); opacity: 0; }
  }
  .search-filters {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }
  .filter-chip {
    padding: 3px 9px;
    border-radius: var(--radius-pill);
    background: var(--panel-2);
    color: var(--muted);
    font-size: var(--text-xs);
    font-weight: 650;
    white-space: nowrap;
  }
  .filter-chip.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .filter-select {
    height: 26px;
    max-width: 160px;
    margin-left: auto;
    padding: 0 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    background: var(--panel-2);
    color: var(--muted);
    font-size: 10.5px;
  }
  .bulk {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    margin: 8px 10px 0;
    padding: 7px 8px 7px 12px;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
  }
  .bulk-count {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--accent);
  }
  .list-body {
    flex: 1;
    min-height: 0;
  }
  .mail-row {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 100%;
    min-width: 0;
    height: 100%;
    padding: 0 10px 0 8px;
    border-radius: var(--radius-md);
    cursor: default;
    transition: background var(--dur-fast) ease-out;
  }
  .avatar-wrap {
    position: relative;
    flex: none;
  }
  .content {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .sender,
  .subject {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sender {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--muted);
  }
  .subject {
    font-size: var(--text-sm);
    color: var(--faint);
  }
  .mail-row.unread .sender {
    color: var(--text);
    font-weight: 760;
  }
  .mail-row.unread .subject {
    color: var(--text);
    font-weight: 560;
  }
  .mini-flag {
    display: inline-flex;
    color: var(--faint);
    flex: none;
  }
  .mail-row.unread .mini-flag {
    color: var(--brass);
  }
  .unread-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
    flex: none;
    margin-left: 2px;
  }
  .check {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-contrast);
    flex: none;
  }
  .mail-row.checked {
    background: var(--accent-soft);
  }
  :global([role='option']:hover) .mail-row {
    background: var(--hover-bg);
  }
  :global([role='option'].selected) .mail-row,
  :global([role='option'].focused) .mail-row {
    background: var(--selected-bg);
    box-shadow: inset 0 0 0 1px var(--selected-border);
  }
  .inline-error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 8px 10px 0;
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    color: var(--danger);
    background: var(--danger-soft);
  }
  .inline-error span {
    flex: 1;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100%;
    padding: 40px 28px;
    text-align: center;
    color: var(--muted);
  }
  .empty-mark {
    display: grid;
    place-items: center;
    width: 58px;
    height: 58px;
    border-radius: var(--radius-md);
    background: var(--panel-2);
    color: var(--faint);
    margin-bottom: 2px;
  }
  .empty h2 {
    font-size: var(--text-md);
    font-weight: 720;
    color: var(--text);
  }
  .empty p {
    max-width: 34ch;
    font-size: var(--text-sm);
    line-height: 1.5;
  }
  .empty .btn {
    margin-top: 6px;
  }
  .skeleton {
    padding: 4px 6px;
  }
  .skeleton-row {
    display: flex;
    align-items: center;
    gap: 11px;
    height: var(--row-height);
    padding: 0 10px;
    opacity: calc(0.4 + var(--i) * 0.045);
  }
  .sk-avatar {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: var(--panel-3);
    flex: none;
  }
  .sk-lines {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }
  .sk-line {
    height: 7px;
    border-radius: 3px;
    background: var(--panel-3);
    width: 72%;
  }
  .sk-line.short {
    width: 38%;
  }
  @container (max-width: 380px) {
    .mini-flag {
      display: none;
    }
    .compose-quick {
      display: inline-flex;
    }
  }
  @media (max-width: 820px) {
    .menu-btn {
      display: inline-flex;
    }
  }
</style>
