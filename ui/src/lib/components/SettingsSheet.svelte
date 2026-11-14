<script lang="ts">
  import { app } from '../state.svelte';
  import { api } from '../api';
  import type { ThemePreference, Density } from '../state.svelte';
  import Icon from './Icon.svelte';

  let dialog = $state<HTMLDialogElement | null>(null);
  let clearing = $state(false);

  $effect(() => {
    if (app.settingsOpen && dialog && !dialog.open) {
      dialog.showModal();
    } else if (!app.settingsOpen && dialog?.open) {
      dialog.close();
    }
  });

  function close(): void {
    app.settingsOpen = false;
  }

  function onCancel(event: Event): void {
    event.preventDefault();
    close();
  }

  async function clearCache(): Promise<void> {
    clearing = true;
    try {
      await api.clearCache();
      app.toast('Encrypted cache cleared. Messages will reload when you open a folder.', 'success');
    } catch (error) {
      app.toast(app.message(error), 'error');
    } finally {
      clearing = false;
    }
  }

  const themes: { value: ThemePreference; label: string; icon: string }[] = [
    { value: 'system', label: 'System', icon: 'monitor' },
    { value: 'light', label: 'Light', icon: 'sun' },
    { value: 'dark', label: 'Dark', icon: 'moon' }
  ];

  const densities: { value: Density; label: string }[] = [
    { value: 'compact', label: 'Compact' },
    { value: 'comfortable', label: 'Comfortable' },
    { value: 'spacious', label: 'Spacious' }
  ];
</script>

<dialog bind:this={dialog} class="sheet-dialog" oncancel={onCancel} aria-labelledby="settings-title">
  <section class="sheet">
    <header class="sheet-head">
      <h2 id="settings-title">Settings</h2>
      <button class="icon-btn" aria-label="Close settings" onclick={close}>
        <Icon name="close" size={17} />
      </button>
    </header>

    <div class="sheet-body">
      <section class="group" aria-labelledby="appearance-label">
        <h3 id="appearance-label">Appearance</h3>
        <div class="setting">
          <div class="setting-copy">
            <span class="setting-name">Theme</span>
            <span class="setting-help">Follow the system or pick one.</span>
          </div>
          <div class="segmented" role="radiogroup" aria-label="Theme">
            {#each themes as item (item.value)}
              <button
                role="radio"
                aria-checked={app.theme === item.value}
                class:selected={app.theme === item.value}
                onclick={() => app.setTheme(item.value)}
              >
                <Icon name={item.icon} size={14} />
                {item.label}
              </button>
            {/each}
          </div>
        </div>
        <div class="setting">
          <div class="setting-copy">
            <span class="setting-name">Density</span>
          </div>
          <div class="segmented" role="radiogroup" aria-label="List density">
            {#each densities as item (item.value)}
              <button
                role="radio"
                aria-checked={app.density === item.value}
                class:selected={app.density === item.value}
                onclick={() => app.setDensity(item.value)}
              >
                {item.label}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <section class="group" aria-labelledby="privacy-label">
        <h3 id="privacy-label">Privacy</h3>
        <div class="setting column">
          <div class="setting-copy">
            <span class="setting-name">Remote images</span>
            <span class="setting-help">Blocked by default. Allow per sender.</span>
          </div>
          {#if app.remoteSenders.length === 0}
            <p class="empty-note">No sender is allowed. Messages show a banner with a per-sender choice.</p>
          {:else}
            <ul class="allow-list">
              {#each app.remoteSenders as sender (sender)}
                <li>
                  <span>{sender}</span>
                  <button class="mini-btn" onclick={() => app.revokeRemote(sender)}>Revoke</button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
        <div class="setting">
          <div class="setting-copy">
            <span class="setting-name">Local cache</span>
            <span class="setting-help">
              {app.snapshot?.cached_mails ?? 0} messages · encrypted
            </span>
          </div>
          <button class="btn btn-sm btn-danger" onclick={clearCache} disabled={clearing}>
            {clearing ? 'Clearing…' : 'Clear cache'}
          </button>
        </div>
      </section>

      <section class="group" aria-labelledby="account-label">
        <h3 id="account-label">Account</h3>
        <div class="setting">
          <div class="setting-copy">
            <span class="setting-name">{app.snapshot?.email ?? 'Not signed in'}</span>
            <span class="setting-help">{app.snapshot?.api_url ?? ''}</span>
          </div>
          <button class="btn btn-sm" onclick={() => app.logout()}>Sign out</button>
        </div>
        <div class="setting">
          <div class="setting-copy">
            <span class="setting-name">Session</span>
            <span class="setting-help">
              {app.snapshot?.session_persisted ? 'Stored in the system keyring' : 'Memory only; ends with the app'}
            </span>
          </div>
          <span class="chip" class:chip-warn={!app.snapshot?.session_persisted}>
            {app.snapshot?.session_persisted ? 'Keyring' : 'Memory'}
          </span>
        </div>
      </section>

      <section class="group" aria-labelledby="about-label">
        <h3 id="about-label">About</h3>
        <div class="about">
          <div class="about-mark"><Icon name="mailbox" size={20} /></div>
          <div>
            <p class="about-name">TMail <span class="faint">{app.info?.version}</span></p>
            <p class="about-copy">
              Unofficial. GPL-3.0-or-later. Not affiliated with Tutao GmbH.
            </p>
          </div>
        </div>
      </section>
    </div>
  </section>
</dialog>

<style>
  .sheet-dialog {
    width: min(560px, calc(100vw - 32px));
    max-width: none;
    height: calc(100vh - 40px);
    max-height: none;
    margin: 20px 20px 20px auto;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .sheet-dialog::backdrop {
    background: var(--scrim);
  }
  .sheet {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .sheet-head {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    padding: 12px 12px 12px 20px;
    border-bottom: 1px solid var(--border);
  }
  .sheet-head h2 {
    flex: 1;
    font-size: var(--text-lg);
    font-weight: 720;
    letter-spacing: -0.01em;
  }
  .sheet-body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 20px 26px;
  }
  .group {
    padding: 18px 0 8px;
    border-bottom: 1px solid var(--border);
  }
  .group:last-child {
    border-bottom: 0;
  }
  .group h3 {
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--faint);
    margin-bottom: 10px;
  }
  .setting {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 0;
  }
  .setting.column {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .setting-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .setting-name {
    font-size: var(--text-sm);
    font-weight: 650;
  }
  .setting-help {
    font-size: var(--text-xs);
    color: var(--muted);
    line-height: 1.45;
  }
  .segmented {
    display: flex;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--panel-2);
    flex: none;
  }
  .segmented button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 9px;
    border-radius: 3px;
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--muted);
  }
  .segmented button.selected {
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow-sm);
  }
  .empty-note {
    padding: 9px 11px;
    border-radius: var(--radius-sm);
    background: var(--panel-2);
    color: var(--muted);
    font-size: var(--text-xs);
    line-height: 1.5;
  }
  .allow-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .allow-list li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 9px;
    background: var(--panel-2);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
  }
  .allow-list li span {
    flex: 1;
  }
  .mini-btn {
    font-size: var(--text-xs);
    font-weight: 650;
    color: var(--accent);
    padding: 3px 5px;
    border-radius: var(--radius-xs);
  }
  .mini-btn:hover {
    background: var(--accent-soft);
  }
  .about {
    display: flex;
    gap: 12px;
    padding: 8px 0 4px;
  }
  .about-mark {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    flex: none;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .about-name {
    font-size: var(--text-sm);
    font-weight: 700;
  }
  .about-copy {
    margin-top: 5px;
    font-size: var(--text-xs);
    line-height: 1.5;
    color: var(--muted);
  }
</style>
