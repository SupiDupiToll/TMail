<script lang="ts">
  import type { Contact } from '../api';
  import Avatar from './Avatar.svelte';

  interface Props {
    contacts: Contact[];
    query: string;
    open: boolean;
    onPick: (address: string) => void;
  }
  let { contacts, query, open, onPick }: Props = $props();

  const matches = $derived.by(() => {
    const value = query.trim().toLowerCase();
    if (value.length < 2) return [];
    return contacts
      .filter((contact) =>
        contact.name.toLowerCase().includes(value) ||
        contact.company.toLowerCase().includes(value) ||
        contact.addresses.some((address) => address.toLowerCase().includes(value))
      )
      .slice(0, 6);
  });
</script>

{#if open && matches.length > 0}
  <div class="suggestions" role="listbox" aria-label="Contact suggestions">
    {#each matches as contact (contact.addresses[0])}
      <button
        type="button"
        class="suggestion"
        role="option"
        aria-selected="false"
        onmousedown={(event) => event.preventDefault()}
        onclick={() => contact.addresses[0] && onPick(contact.addresses[0])}
      >
        <Avatar name={contact.name} address={contact.addresses[0] ?? contact.name} size={26} />
        <span class="suggestion-text">
          <span class="suggestion-name">{contact.name}</span>
          <span class="suggestion-address">{contact.addresses.join(', ')}</span>
        </span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .suggestions {
    position: absolute;
    top: calc(100% + 2px);
    left: 52px;
    right: 10px;
    z-index: 30;
    padding: 4px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--panel);
    box-shadow: var(--shadow);
    max-height: 240px;
    overflow-y: auto;
  }
  .suggestion {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 6px 8px;
    border-radius: var(--radius-xs);
    text-align: left;
    color: var(--text);
  }
  .suggestion:hover,
  .suggestion:focus-visible {
    background: var(--accent-soft);
  }
  .suggestion-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .suggestion-name {
    font-size: var(--text-sm);
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .suggestion-address {
    font-size: var(--text-xs);
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
