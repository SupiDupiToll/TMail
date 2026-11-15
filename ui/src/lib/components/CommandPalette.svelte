<script lang="ts">
  import { app } from '../state.svelte';
  import { api } from '../api';
  import type { SearchHit } from '../types';
  import { senderLabel, formatListDate } from '../format';
  import Icon from './Icon.svelte';

  interface Command {
    id: string;
    label: string;
    hint?: string;
    icon: string;
    group: string;
    run: () => void;
  }

  let dialog = $state<HTMLDialogElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);
  let query = $state('');
  let active = $state(0);
  let hits = $state<SearchHit[]>([]);
  let searching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  const commands = $derived<Command[]>([
    { id: 'compose', label: 'Compose new message', hint: 'c', icon: 'compose', group: 'Actions', run: () => app.compose('new') },
    { id: 'reply', label: 'Reply', hint: 'r', icon: 'reply', group: 'Actions', run: () => app.compose('reply') },
    { id: 'reply-all', label: 'Reply all', hint: 'a', icon: 'reply-all', group: 'Actions', run: () => app.compose('reply_all') },
    { id: 'forward', label: 'Forward', hint: 'f', icon: 'forward', group: 'Actions', run: () => app.compose('forward') },
    { id: 'archive', label: 'Archive selected', hint: 'e', icon: 'archive', group: 'Actions', run: () => app.archive() },
    { id: 'trash', label: 'Move selected to trash', hint: '#', icon: 'trash', group: 'Actions', run: () => app.trash() },
    { id: 'unread', label: 'Toggle read / unread', hint: 'u', icon: 'clock', group: 'Actions', run: () => app.toggleUnread() },
    { id: 'refresh', label: 'Refresh folder', hint: 'R', icon: 'refresh', group: 'Actions', run: () => app.refreshCurrent() },
    ...(app.snapshot?.folders ?? []).map((folder) => ({
      id: `go-${folder.id}`,
      label: `Go to ${folder.name || folder.kind}`,
      icon: folder.kind === 'custom' ? 'folder' : folder.kind,
      group: 'Go to',
      run: () => app.selectFolder(folder.id)
    })),
    { id: 'theme', label: 'Toggle light / dark', icon: 'sun', group: 'App', run: () => app.setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark') },
    { id: 'settings', label: 'Open settings', icon: 'settings', group: 'App', run: () => (app.settingsOpen = true) },
    { id: 'help', label: 'Keyboard shortcuts', hint: '?', icon: 'command', group: 'App', run: () => (app.helpOpen = true) },
    { id: 'logout', label: 'Sign out', icon: 'arrow-left', group: 'App', run: () => app.logout() }
  ]);

  const filtered = $derived(
    query.trim()
      ? commands.filter((command) =>
          `${command.label} ${command.group}`.toLowerCase().includes(query.trim().toLowerCase())
        )
      : commands
  );

  const total = $derived(filtered.length + hits.length);

  $effect(() => {
    if (app.commandOpen && dialog && !dialog.open) {
      dialog.showModal();
      query = '';
      hits = [];
      active = 0;
      requestAnimationFrame(() => input?.focus());
    } else if (!app.commandOpen && dialog?.open) {
      dialog.close();
    }
  });

  $effect(() => {
    const value = query.trim();
    if (searchTimer) clearTimeout(searchTimer);
    if (value.length < 2) {
      hits = [];
      searching = false;
      return;
    }
    searching = true;
    searchTimer = setTimeout(async () => {
      try {
        hits = await api.search(value, 20);
      } catch {
        hits = [];
      } finally {
        searching = false;
      }
    }, 170);
    return () => {
      if (searchTimer) clearTimeout(searchTimer);
    };
  });

  $effect(() => {
    active = 0;
  });

  function close(): void {
    app.commandOpen = false;
    app.closeSearch();
  }

  function run(index: number): void {
    if (index < filtered.length) {
      filtered[index]?.run();
    } else {
      const hit = hits[index - filtered.length];
      if (hit) void app.openFromSearch(hit);
    }
    close();
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      close();
    } else if (event.key === 'ArrowDown') {
      event.preventDefault();
      active = (active + 1) % Math.max(1, total);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      active = (active - 1 + Math.max(1, total)) % Math.max(1, total);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      run(active);
    }
  }

  function onCancel(event: Event): void {
    event.preventDefault();
    close();
  }

  function groupStart(index: number): boolean {
    return index === 0 || filtered[index].group !== filtered[index - 1].group;
  }
</script>

<dialog
  bind:this={dialog}
  class="palette-dialog"
  oncancel={onCancel}
  onkeydown={onkeydown}
  onclick={(event) => {
    if (event.target === dialog) close();
  }}
  aria-label="Command palette"
>
  <div class="palette">
    <div class="search-row">
      <Icon name="search" size={18} />
      <input
        bind:this={input}
        bind:value={query}
        class="search"
        placeholder="Search mail or run a command…"
        aria-label="Search mail or run a command"
        role="combobox"
        aria-expanded="true"
        aria-autocomplete="list"
        aria-haspopup="listbox"
        aria-controls="palette-results"
        aria-activedescendant={total ? `palette-item-${active}` : undefined}
        autocomplete="off"
        spellcheck="false"
      />
      {#if searching}<span class="spinner" aria-label="Searching"></span>{/if}
      <kbd class="kbd">Esc</kbd>
    </div>

    <div id="palette-results" class="results" role="listbox" aria-label="Results">
      {#each filtered as command, index (command.id)}
        {#if groupStart(index)}
          <div class="group">{command.group}</div>
        {/if}
        <button
          id={`palette-item-${index}`}
          class="result"
          class:active={active === index}
          role="option"
          tabindex="-1"
          aria-selected={active === index}
          onmousemove={() => (active = index)}
          onclick={() => run(index)}
        >
          <span class="result-icon"><Icon name={command.icon} size={16} /></span>
          <span class="result-label">{command.label}</span>
          {#if command.hint}<kbd class="kbd">{command.hint}</kbd>{/if}
        </button>
      {/each}

      {#if hits.length > 0}
        <div class="group">Messages{searching ? ' · searching…' : ''}</div>
        {#each hits as hit, offset (hit.mail.id)}
          {@const index = filtered.length + offset}
          <button
            id={`palette-item-${index}`}
            class="result message"
            class:active={active === index}
            role="option"
            tabindex="-1"
            aria-selected={active === index}
            onmousemove={() => (active = index)}
            onclick={() => run(index)}
          >
            <span class="result-icon"><Icon name="inbox" size={16} /></span>
            <span class="result-text">
              <span class="result-label">
                {hit.mail.subject}
                <span class="faint"> · {senderLabel(hit.mail.sender.name, hit.mail.sender.address)}</span>
              </span>
              <span class="snippet">{hit.snippet}</span>
            </span>
            <span class="faint tiny tabular">{formatListDate(hit.mail.received_ms)}</span>
          </button>
        {/each}
      {/if}

      {#if filtered.length === 0 && hits.length === 0}
        <div class="none">
          {#if searching}
            Searching…
          {:else if query.trim().length >= 2}
            No mail or command matches “{query.trim()}”.
          {:else}
            Type to search, or pick an action.
          {/if}
        </div>
      {/if}
    </div>

    <footer class="palette-foot">
      <span><kbd class="kbd">↑</kbd><kbd class="kbd">↓</kbd> move</span>
      <span><kbd class="kbd">↵</kbd> select</span>
      <span><kbd class="kbd">Esc</kbd> close</span>
    </footer>
  </div>
</dialog>

<style>
  .palette-dialog {
    width: min(640px, calc(100vw - 40px));
    max-width: none;
    margin: 12vh auto auto;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .palette-dialog::backdrop {
    background: var(--scrim);
    backdrop-filter: blur(2px);
  }
  .search-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    color: var(--faint);
  }
  .search {
    flex: 1;
    min-width: 0;
    height: 28px;
    border: 0;
    background: transparent;
    outline: none;
    font-size: var(--text-md);
    color: var(--text);
  }
  .search::placeholder {
    color: var(--faint);
  }
  .results {
    max-height: 52vh;
    overflow-y: auto;
    padding: 6px;
  }
  .group {
    padding: 9px 9px 4px;
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .result {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 34px;
    padding: 5px 9px;
    border-radius: var(--radius-sm);
    color: var(--text);
    text-align: left;
    font-size: var(--text-sm);
  }
  .result.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .result-icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-xs);
    background: var(--panel-2);
    color: var(--muted);
    flex: none;
  }
  .result.active .result-icon {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .result-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
  }
  .result.message {
    align-items: flex-start;
    padding-top: 7px;
    padding-bottom: 7px;
  }
  .result-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }
  .snippet {
    color: var(--muted);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .none {
    padding: 22px 12px;
    text-align: center;
    color: var(--muted);
    font-size: var(--text-sm);
  }
  .palette-foot {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    background: var(--panel-2);
    font-size: 10.5px;
    color: var(--muted);
  }
  .palette-foot span {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
</style>
