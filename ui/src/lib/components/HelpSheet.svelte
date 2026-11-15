<script lang="ts">
  import { app } from '../state.svelte';
  import Icon from './Icon.svelte';

  let dialog = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    if (app.helpOpen && dialog && !dialog.open) dialog.showModal();
    else if (!app.helpOpen && dialog?.open) dialog.close();
  });

  function close(): void {
    app.helpOpen = false;
  }

  const groups = [
    {
      name: 'Navigate',
      items: [
        ['j / ↓', 'Next message'],
        ['k / ↑', 'Previous message'],
        ['Enter', 'Open message'],
        ['g i', 'Go to Inbox'],
        ['g d', 'Go to Drafts'],
        ['g s', 'Go to Sent'],
        ['g a', 'Go to Archive'],
        ['g t', 'Go to Trash'],
        ['Esc', 'Back / close overlay']
      ]
    },
    {
      name: 'Act',
      items: [
        ['c', 'Compose'],
        ['r', 'Reply'],
        ['a', 'Reply all'],
        ['f', 'Forward'],
        ['u', 'Toggle read / unread'],
        ['e', 'Archive'],
        ['#', 'Move to trash'],
        ['x', 'Select message for bulk actions'],
        ['R', 'Refresh folder'],
        ['Delete', 'Delete permanently (in Trash)']
      ]
    },
    {
      name: 'Everywhere',
      items: [
        ['Ctrl K', 'Command palette and search'],
        ['/', 'Search'],
        ['?', 'This help'],
        ['Ctrl Enter', 'Send from the composer'],
        ['Ctrl Q', 'Quit TMail']
      ]
    }
  ];
</script>

<dialog
  bind:this={dialog}
  class="help-dialog"
  oncancel={(e) => { e.preventDefault(); close(); }}
  onclick={(event) => {
    if (event.target === dialog) close();
  }}
  aria-labelledby="help-title"
>
  <section class="help">
    <header class="help-head">
      <h2 id="help-title">Keyboard shortcuts</h2>
      <button class="icon-btn" aria-label="Close help" onclick={close}><Icon name="close" size={17} /></button>
    </header>
    <div class="help-body">
      {#each groups as group (group.name)}
        <section class="group" aria-label={group.name}>
          <h3>{group.name}</h3>
          <dl>
            {#each group.items as item (item[0])}
              <div class="row">
                <dt>
                  {#each item[0].split(' ') as key (key)}
                    <kbd class="kbd">{key}</kbd>
                  {/each}
                </dt>
                <dd>{item[1]}</dd>
              </div>
            {/each}
          </dl>
        </section>
      {/each}
      <p class="note">
        Shortcuts work while you are reading. Inside the message frame, the same keys are forwarded
        to the app; links remain clickable.
      </p>
    </div>
  </section>
</dialog>

<style>
  .help-dialog {
    width: min(760px, calc(100vw - 32px));
    max-width: none;
    max-height: calc(100vh - 48px);
    margin: auto;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .help-dialog::backdrop {
    background: var(--scrim);
  }
  .help {
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 48px);
  }
  .help-head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 12px 12px 20px;
    border-bottom: 1px solid var(--border);
  }
  .help-head h2 {
    flex: 1;
    font-size: var(--text-lg);
    font-weight: 720;
  }
  .help-body {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 24px;
    padding: 18px 20px 20px;
    overflow-y: auto;
  }
  .group h3 {
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--faint);
    margin-bottom: 8px;
  }
  dl {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 5px 0;
    border-top: 1px solid var(--border);
  }
  .row:first-of-type {
    border-top: 0;
  }
  dt {
    display: flex;
    gap: 2px;
  }
  dd {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--muted);
  }
  .note {
    grid-column: 1 / -1;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--panel-2);
    color: var(--muted);
    font-size: var(--text-xs);
    line-height: 1.5;
  }
  @media (max-width: 680px) {
    .help-body {
      grid-template-columns: 1fr;
    }
  }
</style>
