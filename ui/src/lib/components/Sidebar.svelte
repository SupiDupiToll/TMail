<script lang="ts">
  import { app } from '../state.svelte';
  import type { FolderNode } from '../types';
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';

  interface Props {
    onNavigate?: () => void;
  }
  let { onNavigate }: Props = $props();

  const folders = $derived(app.snapshot?.folders ?? []);
  const systemFolders = $derived(
    folders.filter((folder) => folder.level === 0 && folder.kind !== 'custom')
  );
  const customFolders = $derived(folders.filter((folder) => folder.kind === 'custom'));

  function unreadFor(folder: FolderNode): number {
    return app.snapshot?.unread.find((entry) => entry.folder_id === folder.id)?.unread ?? 0;
  }

  function label(folder: FolderNode): string {
    if (folder.name) return folder.name;
    return folder.kind.charAt(0).toUpperCase() + folder.kind.slice(1);
  }

  function select(folder: FolderNode): void {
    void app.selectFolder(folder.id);
    onNavigate?.();
  }
</script>

<aside class="sidebar" aria-label="Mailbox">
  <div class="account">
    <Avatar name={app.snapshot?.email ?? 'TMail'} address={app.snapshot?.email ?? 'tmail'} size={36} />
    <div class="account-text">
      <span class="account-email" title={app.snapshot?.email ?? ''}>{app.snapshot?.email ?? 'Not signed in'}</span>
      {#if app.synthetic}<span class="synthetic">preview</span>{/if}
    </div>
    <button class="icon-btn settings-btn" aria-label="Settings" title="Settings" onclick={() => (app.settingsOpen = true)}>
      <Icon name="settings" size={17} />
    </button>
    <button
      class="icon-btn collapse-btn"
      aria-label={app.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      title={app.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      onclick={() => app.toggleSidebar()}
    >
      <Icon name={app.sidebarCollapsed ? 'chevron' : 'chevron-left'} size={16} />
    </button>
  </div>

  <div class="compose-wrap">
    <button
      class="compose"
      aria-label={app.sidebarCollapsed ? 'Compose new message' : undefined}
      title="Compose (c)"
      onclick={() => app.compose('new')}
    >
      <Icon name="compose" size={16} />
      <span class="compose-label">Compose</span>
      <kbd class="kbd">c</kbd>
    </button>
  </div>

  <nav class="folders" aria-label="Folders">
    {#each systemFolders as folder (folder.id)}
      <button
        class="folder"
        class:active={app.selectedFolderId === folder.id}
        aria-label={app.sidebarCollapsed ? label(folder) : undefined}
        title={label(folder)}
        aria-current={app.selectedFolderId === folder.id ? 'page' : undefined}
        onclick={() => select(folder)}
      >
        <Icon name={folder.kind} size={16} />
        <span class="folder-name">{label(folder)}</span>
        {#if unreadFor(folder) > 0}
          <span class="badge tabular" aria-label={`${unreadFor(folder)} unread`}>{unreadFor(folder)}</span>
        {/if}
      </button>
    {/each}

    {#if customFolders.length > 0}
      <div class="section-label">Folders</div>
      {#each customFolders as folder (folder.id)}
        <button
          class="folder custom"
          class:active={app.selectedFolderId === folder.id}
          style:padding-left={`${12 + folder.level * 15}px`}
          aria-label={app.sidebarCollapsed ? folder.name : undefined}
          title={folder.name}
          aria-current={app.selectedFolderId === folder.id ? 'page' : undefined}
          onclick={() => select(folder)}
        >
          <Icon name="folder" size={15} />
          <span class="folder-name">{folder.name}</span>
          {#if unreadFor(folder) > 0}
            <span class="badge tabular" aria-label={`${unreadFor(folder)} unread`}>{unreadFor(folder)}</span>
          {/if}
        </button>
      {/each}
    {/if}
  </nav>

  <div class="sidebar-foot">
    <button class="foot-action shortcuts" onclick={() => (app.helpOpen = true)} title="Keyboard shortcuts">
      <span class="kbd">?</span>
      <span class="foot-label">Shortcuts</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    height: 100%;
    padding: 0 10px;
    background: var(--panel);
    border-right: 1px solid var(--border);
  }
  .account {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    padding: 12px 4px 10px;
    min-height: var(--topbar-height);
  }
  .account-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    gap: 2px;
  }
  .account-email {
    font-size: var(--text-sm);
    font-weight: 680;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .synthetic {
    align-self: flex-start;
    padding: 0 5px;
    border-radius: var(--radius-pill);
    background: var(--brass-soft);
    color: var(--brass);
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    font-size: 8.5px;
  }
  .compose-wrap {
    padding: 4px 2px 10px;
  }
  .compose {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 40px;
    padding: 0 10px 0 13px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-contrast);
    font-size: var(--text-sm);
    font-weight: 700;
    transition:
      background var(--dur-fast) ease-out,
      transform var(--dur-fast) ease-out;
  }
  .compose:hover {
    background: var(--accent-hover);
  }
  .compose:active {
    transform: scale(0.99);
  }
  .compose .kbd {
    margin-left: auto;
    background: transparent;
    border-color: color-mix(in srgb, var(--accent-contrast) 35%, transparent);
    color: var(--accent-contrast);
  }
  .folders {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 2px 0 12px;
  }
  .folder {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 36px;
    padding: 0 11px;
    border-radius: var(--radius-sm);
    color: var(--muted);
    font-size: var(--text-sm);
    text-align: left;
    transition:
      background var(--dur-fast) ease-out,
      color var(--dur-fast) ease-out;
  }
  .folder:hover {
    background: var(--hover-bg);
    color: var(--text);
  }
  .folder.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 720;
  }
  .folder-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    border-radius: var(--radius-pill);
    background: var(--accent);
    color: var(--accent-contrast);
    font-size: 10.5px;
    font-weight: 750;
    line-height: 20px;
    text-align: center;
  }
  .folder:not(.active) .badge {
    background: var(--panel-3);
    color: var(--muted);
  }
  .section-label {
    margin: 16px 11px 5px;
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .sidebar-foot {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    padding: 9px 2px 11px;
    border-top: 1px solid var(--border);
  }
  .foot-action {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 6px;
    border-radius: var(--radius-xs);
    color: var(--muted);
    font-size: var(--text-xs);
  }
  .foot-action:hover {
    background: var(--hover-bg);
    color: var(--text);
  }

  /* Icon-only state. The width itself comes from the shell's CSS variable. */
  :global([data-sidebar='collapsed']) .sidebar {
    padding: 0 8px;
  }
  :global([data-sidebar='collapsed']) .account {
    justify-content: center;
    padding: 12px 0 10px;
    gap: 0;
  }
  :global([data-sidebar='collapsed']) .account :global(.avatar),
  :global([data-sidebar='collapsed']) .account-text,
  :global([data-sidebar='collapsed']) .settings-btn,
  :global([data-sidebar='collapsed']) .compose-label,
  :global([data-sidebar='collapsed']) .compose .kbd,
  :global([data-sidebar='collapsed']) .folder-name,
  :global([data-sidebar='collapsed']) .section-label,
  :global([data-sidebar='collapsed']) .foot-label {
    display: none;
  }
  :global([data-sidebar='collapsed']) .compose {
    width: 40px;
    height: 40px;
    padding: 0;
    justify-content: center;
    margin: 0 auto;
  }
  :global([data-sidebar='collapsed']) .folder {
    justify-content: center;
    padding: 0 !important;
    width: 40px;
    margin: 0 auto;
  }
  :global([data-sidebar='collapsed']) .folder .badge {
    display: none;
  }
  :global([data-sidebar='collapsed']) .collapse-btn {
    width: 38px;
    height: 38px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  :global([data-sidebar='collapsed']) .sidebar-foot {
    display: none;
  }
</style>
