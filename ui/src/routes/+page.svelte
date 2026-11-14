<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '$lib/state.svelte';
  import { isMock, isTauri } from '$lib/api';
  import Login from '$lib/components/Login.svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import Icon from '$lib/components/Icon.svelte';

  onMount(() => {
    void app.init();
    return () => app.destroy();
  });
</script>

<svelte:head>
  <title>TMail — unofficial Tuta Mail client</title>
  <meta
    name="description"
    content="An unofficial, keyboard-first desktop client for Tuta Mail."
  />
</svelte:head>

{#if !isTauri && !isMock}
  <main class="unsupported">
    <span class="mark"><Icon name="mailbox" size={26} /></span>
    <h1>This is the TMail desktop app</h1>
    <p>
      The browser build has no access to the mail core. Start the desktop binary, or run the
      development server with <code>?mock=1</code> to preview the interface with synthetic data.
    </p>
  </main>
{:else if app.auth === 'booting'}
  <main class="boot" aria-busy="true" aria-label="Starting TMail">
    <div class="boot-mark"><Icon name="mailbox" size={24} /></div>
    <div class="boot-copy">
      <span class="boot-name">TMail</span>
      <span class="boot-state">Opening the encrypted cache…</span>
    </div>
    <span class="spinner"></span>
  </main>
{:else if app.auth === 'signed_in'}
  <AppShell />
{:else}
  <Login />
{/if}

<style>
  .boot,
  .unsupported {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    height: 100%;
    padding: 32px;
    text-align: center;
    background: var(--bg);
  }
  .boot {
    flex-direction: row;
    gap: 14px;
  }
  .boot-mark {
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .boot-copy {
    display: flex;
    flex-direction: column;
    text-align: left;
  }
  .boot-name {
    font-size: var(--text-lg);
    font-weight: 750;
    letter-spacing: -0.015em;
  }
  .boot-state {
    font-size: var(--text-xs);
    color: var(--muted);
  }
  .boot .spinner {
    margin-left: 10px;
  }
  .unsupported {
    max-width: 52ch;
    margin: 0 auto;
  }
  .unsupported .mark {
    display: grid;
    place-items: center;
    width: 54px;
    height: 54px;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--accent);
  }
  .unsupported h1 {
    font-size: var(--text-xl);
    font-weight: 720;
    letter-spacing: -0.02em;
  }
  .unsupported p {
    color: var(--muted);
    font-size: var(--text-sm);
    line-height: 1.6;
  }
  code {
    font-family: var(--font-mono);
    font-size: 0.9em;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--panel-2);
  }
</style>
