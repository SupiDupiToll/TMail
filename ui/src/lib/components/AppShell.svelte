<script lang="ts">
  import { app } from '../state.svelte';
  import { api, isTauri } from '../api';
  import Sidebar from './Sidebar.svelte';
  import MessageList from './MessageList.svelte';
  import Reader from './Reader.svelte';
  import Composer from './Composer.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import SettingsSheet from './SettingsSheet.svelte';
  import HelpSheet from './HelpSheet.svelte';
  import Toasts from './Toasts.svelte';

  interface ListApi {
    navigate: (delta: number) => void;
    openFocused: () => void;
    focusFirstUnread: () => void;
  }

  let listRef = $state<ListApi | null>(null);
  let sidebarOpen = $state(false);
  let pendingG = false;
  let gTimer: ReturnType<typeof setTimeout> | null = null;

  const currentFolder = $derived(
    app.snapshot?.folders.find((folder) => folder.id === app.selectedFolderId) ?? null
  );

  function focusList(): void {
    const viewport = document.querySelector<HTMLElement>('.virtual-viewport');
    viewport?.focus();
  }

  function navigateFolder(kind: string): void {
    const folder = app.snapshot?.folders.find((item) => item.kind === kind);
    if (folder) {
      void app.selectFolder(folder.id);
      sidebarOpen = false;
      requestAnimationFrame(focusList);
    }
  }

  function quit(): void {
    if (isTauri) void api.quit();
  }

  function closeReader(): void {
    app.selectedMailId = null;
    app.detail = null;
    app.clearSelection();
    requestAnimationFrame(focusList);
  }

  function handleKey(key: string, ctrl: boolean, shift: boolean): void {
    if (ctrl && key.toLowerCase() === 'k') {
      app.commandOpen = true;
      return;
    }
    if (ctrl && key.toLowerCase() === 'q') {
      quit();
      return;
    }
    if (key === 'Escape') {
      if (window.innerWidth <= 1120 && app.selectedMailId) closeReader();
      else if (sidebarOpen) sidebarOpen = false;
      else if (app.selectedIds.length) app.clearSelection();
      return;
    }
    if (key === '?') {
      app.helpOpen = true;
      return;
    }
    if (key === '/') {
      app.commandOpen = true;
      return;
    }
    if (key === 'g') {
      pendingG = true;
      if (gTimer) clearTimeout(gTimer);
      gTimer = setTimeout(() => (pendingG = false), 900);
      return;
    }
    if (pendingG) {
      pendingG = false;
      if (gTimer) clearTimeout(gTimer);
      const map: Record<string, string> = {
        i: 'inbox',
        d: 'draft',
        s: 'sent',
        a: 'archive',
        t: 'trash',
        w: 'custom'
      };
      if (map[key]) navigateFolder(map[key]);
      return;
    }
    switch (key) {
      case 'j':
        listRef?.navigate(1);
        break;
      case 'k':
        listRef?.navigate(-1);
        break;
      case 'c':
        app.compose('new');
        break;
      case 'r':
        if (app.detail) app.compose('reply');
        break;
      case 'a':
        if (app.detail) app.compose('reply_all');
        break;
      case 'f':
        if (app.detail) app.compose('forward');
        break;
      case 'u':
        void app.toggleUnread();
        break;
      case 'e':
        void app.archive();
        break;
      case '#':
        void app.trash();
        break;
      case 'x':
        if (app.selectedMailId) app.toggleSelected(app.selectedMailId);
        break;
      case 'R':
        void app.refreshCurrent();
        break;
      case 'Delete':
        if (currentFolder?.kind === 'trash') void app.deleteForever();
        else void app.trash();
        break;
    }
  }

  function onWindowKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    const target = event.target as HTMLElement | null;
    const typing = Boolean(
      target?.closest('input, textarea, select, [contenteditable="true"]')
    );
    const overlayOpen =
      app.commandOpen || app.composer.open || app.settingsOpen || app.helpOpen;
    if (typing || overlayOpen) return;
    if (event.altKey || event.metaKey) return;

    const inList = target?.closest('[role="listbox"]');
    const listKeys = ['j', 'k', 'ArrowDown', 'ArrowUp', 'PageDown', 'PageUp', 'Home', 'End', 'Enter'];
    if (inList && listKeys.includes(event.key)) return;
    if (event.key === 'Tab') return;
    if (!event.ctrlKey && !event.shiftKey) {
      const plain = ['j', 'k', 'c', 'r', 'a', 'f', 'u', 'e', '#', 'x', 'g', '?', '/', 'Escape'];
      if (!plain.includes(event.key) && event.key !== 'Delete') return;
    }
    if (event.ctrlKey) {
      if (!['k', 'q'].includes(event.key.toLowerCase())) return;
    }
    handleKey(event.key, event.ctrlKey, event.shiftKey);
    if (!['Tab', 'F5', 'F12'].includes(event.key)) event.preventDefault();
  }

  function onShortcutFromFrame(key: string, ctrl: boolean, shift: boolean): void {
    handleKey(key, ctrl, shift);
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<a class="skip-link" href="#message-list">Skip to message list</a>

<div class="shell">
  {#if sidebarOpen}
    <button class="drawer-scrim" aria-label="Close folders" onclick={() => (sidebarOpen = false)}></button>
  {/if}
  <div class="sidebar-wrap" class:open={sidebarOpen}>
    <Sidebar onNavigate={() => (sidebarOpen = false)} />
  </div>

  <main class="workspace" aria-label="Mail workspace">
    <div class="list-wrap" class:hidden-mobile={app.selectedMailId !== null}>
      <MessageList bind:this={listRef} onMenu={() => (sidebarOpen = true)} />
    </div>
    <div class="reader-wrap" class:open={app.selectedMailId !== null}>
      <Reader onShortcut={onShortcutFromFrame} onBack={closeReader} />
    </div>
  </main>
</div>

<Composer />
<CommandPalette />
<SettingsSheet />
<HelpSheet />
<Toasts />

<style>
  .skip-link {
    position: fixed;
    top: 8px;
    left: 8px;
    z-index: 100;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-contrast);
    font-size: var(--text-sm);
    font-weight: 700;
    transform: translateY(-200%);
    transition: transform var(--dur-fast) ease-out;
  }
  .skip-link:focus-visible {
    transform: translateY(0);
  }
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
    height: 100%;
    background: var(--bg);
  }
  .sidebar-wrap {
    min-width: 0;
    min-height: 0;
    height: 100%;
    z-index: 30;
    overflow: hidden;
  }
  .workspace {
    display: grid;
    grid-template-columns: var(--list-width) minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
  }
  .list-wrap {
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
  }
  .reader-wrap {
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
  }
  .drawer-scrim {
    display: none;
  }

  @media (max-width: 1120px) {
    .workspace {
      grid-template-columns: minmax(0, 1fr);
    }
    .reader-wrap {
      display: none;
    }
    .reader-wrap.open {
      display: block;
      position: fixed;
      inset: 0 0 0 var(--sidebar-width);
      z-index: 20;
    }
    .list-wrap.hidden-mobile {
      display: none;
    }
  }

  @media (max-width: 820px) {
    .shell {
      grid-template-columns: minmax(0, 1fr);
    }
    .sidebar-wrap {
      position: fixed;
      inset: 0 auto 0 0;
      width: var(--sidebar-width);
      transform: translateX(-102%);
      transition: transform var(--dur-base) var(--ease-out);
      box-shadow: var(--shadow);
    }
    .sidebar-wrap.open {
      transform: translateX(0);
    }
    .drawer-scrim {
      display: block;
      position: fixed;
      inset: 0;
      z-index: 29;
      background: var(--scrim);
      border: 0;
    }
    .reader-wrap.open {
      inset: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .sidebar-wrap {
      transition: none;
    }
  }
</style>
