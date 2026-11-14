<script lang="ts">
  import { app } from '../state.svelte';
  import Icon from './Icon.svelte';

  let email = $state('');
  let password = $state('');
  let totp = $state('');
  let showPassword = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let emailInput = $state<HTMLInputElement | null>(null);
  let totpInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (app.auth === 'totp') {
      totpInput?.focus();
    } else {
      emailInput?.focus();
    }
  });

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (busy) return;
    error = null;
    busy = true;
    try {
      if (app.auth === 'totp') {
        const code = Number(totp.replace(/\s/g, ''));
        if (!Number.isInteger(code) || code < 0 || code > 999999) {
          error = 'Enter the six-digit code from your authenticator app.';
          return;
        }
        await app.submitTotp(code);
      } else {
        if (!email.trim() || !password) {
          error = 'Enter your address and password.';
          return;
        }
        await app.login(email, password);
      }
    } finally {
      busy = false;
    }
  }

  async function back(): Promise<void> {
    await app.cancelTotp();
    totp = '';
    password = '';
    error = null;
  }

  function onTotpInput(event: Event): void {
    const input = event.target as HTMLInputElement;
    totp = input.value.replace(/[^\d]/g, '').slice(0, 6);
    if (totp.length === 6 && !busy) {
      void app.submitTotp(Number(totp));
    }
  }

  const isDark = $derived(
    app.theme === 'dark' ||
      (app.theme === 'system' && document?.documentElement?.dataset?.theme === 'dark')
  );

  function toggleTheme(): void {
    app.setTheme(isDark ? 'light' : 'dark');
  }
</script>

<main class="login">
  <button
    class="icon-btn theme-toggle"
    aria-label={isDark ? 'Switch to light theme' : 'Switch to dark theme'}
    title="Toggle theme"
    onclick={toggleTheme}
  >
    <Icon name={isDark ? 'sun' : 'moon'} size={17} />
  </button>

  <section class="card" aria-label="Sign in">
    <header class="brand">
      <span class="mark" aria-hidden="true"><Icon name="mailbox" size={22} /></span>
      <div class="brand-copy">
        <span class="name">TMail</span>
        <span class="sub">Unofficial client for Tuta Mail</span>
      </div>
    </header>

    {#if app.auth === 'totp'}
      <div class="intro">
        <h1>Two-factor authentication</h1>
        <p>Enter the six-digit code from your authenticator app.</p>
      </div>
      <form onsubmit={submit} novalidate>
        <div class="field">
          <label class="label" for="totp">Code</label>
          <input
            bind:this={totpInput}
            id="totp"
            class="input code-input tabular"
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="6"
            placeholder="000000"
            bind:value={totp}
            oninput={onTotpInput}
          />
        </div>
        {#if error || app.error}
          <div class="error" role="alert">{error ?? app.error}</div>
        {/if}
        <button class="btn btn-primary submit" type="submit" disabled={busy || totp.length !== 6}>
          {#if busy}<span class="spinner"></span>{/if}
          {busy ? 'Verifying…' : 'Verify'}
        </button>
        <button class="btn btn-ghost back" type="button" onclick={back}>
          <Icon name="arrow-left" size={14} />
          Use a different account
        </button>
      </form>
    {:else}
      <div class="intro">
        <h1>Sign in</h1>
        <p>Your password is only used to unlock the session and is never stored.</p>
      </div>
      <form onsubmit={submit} novalidate>
        <div class="field">
          <label class="label" for="email">Mail address</label>
          <input
            bind:this={emailInput}
            id="email"
            class="input"
            type="email"
            autocomplete="username"
            spellcheck="false"
            placeholder="you@tuta.com"
            bind:value={email}
          />
        </div>
        <div class="field">
          <label class="label" for="password">Password</label>
          <div class="password-wrap">
            <input
              id="password"
              class="input"
              type={showPassword ? 'text' : 'password'}
              autocomplete="current-password"
              bind:value={password}
            />
            <button
              class="peek"
              type="button"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
              aria-pressed={showPassword}
              onclick={() => (showPassword = !showPassword)}
            >
              <Icon name={showPassword ? 'eye-off' : 'eye'} size={16} />
            </button>
          </div>
        </div>
        {#if error || app.error}
          <div class="error" role="alert">{error ?? app.error}</div>
        {/if}
        {#if app.bootError}
          <div class="notice" role="status">{app.bootError}</div>
        {/if}
        <button class="btn btn-primary submit" type="submit" disabled={busy}>
          {#if busy}<span class="spinner"></span>{/if}
          {busy ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
      <p class="keyring">
        <Icon name="shield" size={13} />
        Session stored in your system keyring
      </p>
    {/if}

    {#if app.synthetic}
      <div class="demo" role="status">
        <strong>Preview</strong>
        <span>Not connected to Tuta. The mailbox is synthetic.</span>
      </div>
    {/if}
  </section>

  <footer class="legal">
    <span>TMail {app.info?.version ?? ''}</span>
    <span aria-hidden="true">·</span>
    <span>GPL-3.0</span>
    <span aria-hidden="true">·</span>
    <span>Not affiliated with Tutao GmbH</span>
  </footer>
</main>

<style>
  .login {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
    min-height: 100%;
    padding: 28px;
    background: var(--bg);
  }
  .theme-toggle {
    position: absolute;
    top: 16px;
    right: 16px;
    color: var(--faint);
  }
  .card {
    width: 100%;
    max-width: 400px;
    padding: 26px 26px 22px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--panel);
    box-shadow: var(--shadow-sm);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    margin-bottom: 26px;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-contrast);
    flex: none;
  }
  .brand-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }
  .name {
    font-size: 1.2rem;
    font-weight: 760;
    letter-spacing: -0.02em;
  }
  .sub {
    font-size: 11px;
    color: var(--faint);
    font-weight: 600;
  }
  .intro h1 {
    font-size: 1.55rem;
    line-height: 1.15;
    letter-spacing: -0.025em;
    font-weight: 740;
  }
  .intro p {
    margin-top: 6px;
    color: var(--muted);
    font-size: var(--text-sm);
    line-height: 1.5;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    margin-top: 20px;
  }
  .input {
    border-radius: var(--radius-sm);
    background: var(--bg);
  }
  .password-wrap {
    position: relative;
  }
  .password-wrap .input {
    padding-right: 44px;
  }
  .peek {
    position: absolute;
    right: 5px;
    top: 50%;
    transform: translateY(-50%);
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-xs);
    color: var(--faint);
  }
  .peek:hover {
    background: var(--hover-bg);
    color: var(--text);
  }
  .submit {
    height: 42px;
    margin-top: 2px;
    font-size: var(--text-base);
    border-radius: var(--radius-sm);
  }
  .code-input {
    height: 48px;
    padding-left: 0.32em;
    text-align: center;
    font-size: 1.3rem;
    font-weight: 700;
    letter-spacing: 0.32em;
  }
  .error {
    padding: 9px 11px;
    border-radius: var(--radius-xs);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: var(--text-sm);
    line-height: 1.4;
  }
  .notice {
    padding: 9px 11px;
    border-radius: var(--radius-xs);
    background: var(--warn-soft);
    color: var(--warn);
    font-size: var(--text-xs);
  }
  .back {
    color: var(--muted);
    font-weight: 600;
  }
  .keyring {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin-top: 16px;
    font-size: 10.5px;
    color: var(--faint);
  }
  .demo {
    display: flex;
    align-items: baseline;
    gap: 6px;
    margin-top: 16px;
    padding: 9px 11px;
    border-radius: var(--radius-xs);
    background: var(--brass-soft);
    color: var(--brass);
    font-size: var(--text-xs);
    line-height: 1.4;
  }
  .demo strong {
    font-weight: 750;
  }
  .legal {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    color: var(--faint);
    flex-wrap: wrap;
    justify-content: center;
  }
  @media (max-width: 500px) {
    .login {
      padding: 16px;
    }
    .card {
      padding: 22px 18px 18px;
      border-radius: var(--radius-md);
    }
  }
</style>
