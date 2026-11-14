<script lang="ts">
  import { app } from '../state.svelte';
  import Icon from './Icon.svelte';
</script>

<div class="toasts" aria-live="polite" aria-atomic="false">
  {#each app.toasts as toast (toast.id)}
    <div class="toast" class:error={toast.kind === 'error'} class:success={toast.kind === 'success'} role="status">
      <span class="mark" aria-hidden="true">
        <Icon name={toast.kind === 'error' ? 'bolt' : toast.kind === 'success' ? 'check' : 'clock'} size={15} />
      </span>
      <span class="message">{toast.message}</span>
      {#if toast.action}
        <button class="action" onclick={() => { toast.action?.run(); app.dismissToast(toast.id); }}>
          {toast.action.label}
        </button>
      {/if}
      <button class="close" aria-label="Dismiss" onclick={() => app.dismissToast(toast.id)}>
        <Icon name="close" size={12} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 90;
    display: flex;
    flex-direction: column;
    gap: 7px;
    max-width: min(420px, calc(100vw - 32px));
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 8px 8px 11px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    font-size: var(--text-sm);
    pointer-events: auto;
    animation: enter var(--dur-base) var(--ease-out);
  }
  @keyframes enter {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .mark {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .toast.error .mark {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .toast.success .mark {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .message {
    flex: 1;
    min-width: 0;
    line-height: 1.35;
  }
  .action {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--accent);
    padding: 4px 6px;
    border-radius: var(--radius-xs);
    white-space: nowrap;
  }
  .action:hover {
    background: var(--accent-soft);
  }
  .close {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-xs);
    color: var(--faint);
    flex: none;
  }
  .close:hover {
    background: var(--hover-bg);
    color: var(--text);
  }
</style>
