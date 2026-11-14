<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';

  interface Props {
    items: T[];
    rowHeight: number;
    selectedIndex?: number;
    focusedIndex?: number;
    label: string;
    overscan?: number;
    onActivate?: (index: number) => void;
    onFocusIndex?: (index: number) => void;
    children: Snippet<[T, number]>;
  }

  let {
    items,
    rowHeight,
    selectedIndex = -1,
    focusedIndex = -1,
    label,
    overscan = 8,
    onActivate,
    onFocusIndex,
    children
  }: Props = $props();

  let viewport = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let height = $state(600);

  const start = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - overscan));
  const visibleCount = $derived(Math.ceil(height / rowHeight) + overscan * 2);
  const end = $derived(Math.min(items.length, start + visibleCount));
  const slice = $derived(items.slice(start, end));

  $effect(() => {
    if (!viewport) return;
    const ro = new ResizeObserver((entries) => {
      height = entries[0]?.contentRect.height ?? height;
    });
    ro.observe(viewport);
    height = viewport.clientHeight || height;
    return () => ro.disconnect();
  });

  // Keep the active row inside the viewport without fighting the user's scroll.
  $effect(() => {
    if (!viewport || focusedIndex < 0) return;
    const top = focusedIndex * rowHeight;
    const bottom = top + rowHeight;
    const viewTop = viewport.scrollTop;
    const viewBottom = viewTop + viewport.clientHeight;
    if (top < viewTop) viewport.scrollTop = top;
    else if (bottom > viewBottom) viewport.scrollTop = bottom - viewport.clientHeight;
  });

  function onscroll(): void {
    scrollTop = viewport?.scrollTop ?? 0;
  }

  function onclick(event: MouseEvent, index: number): void {
    const row = (event.target as HTMLElement).closest<HTMLElement>('[data-index]');
    if (!row) return;
    onFocusIndex?.(index);
  }

  function ondblclick(index: number): void {
    onActivate?.(index);
  }

  function onkeydown(event: KeyboardEvent): void {
    if (items.length === 0) return;
    const current = focusedIndex >= 0 ? focusedIndex : 0;
    let next: number | null = null;
    switch (event.key) {
      case 'ArrowDown':
      case 'j':
        next = Math.min(items.length - 1, current + 1);
        break;
      case 'ArrowUp':
      case 'k':
        next = Math.max(0, current - 1);
        break;
      case 'PageDown':
        next = Math.min(items.length - 1, current + Math.floor(height / rowHeight) - 1);
        break;
      case 'PageUp':
        next = Math.max(0, current - Math.floor(height / rowHeight) + 1);
        break;
      case 'Home':
        next = 0;
        break;
      case 'End':
        next = items.length - 1;
        break;
      case 'Enter':
        onActivate?.(current);
        event.preventDefault();
        return;
      default:
        return;
    }
    event.preventDefault();
    onFocusIndex?.(next);
  }
</script>

<div
  bind:this={viewport}
  class="virtual-viewport"
  role="listbox"
  aria-label={label}
  tabindex="0"
  aria-activedescendant={focusedIndex >= 0 ? `mail-row-${focusedIndex}` : undefined}
  {onscroll}
  {onkeydown}
  onclick={(event) => onclick(event, focusedIndex)}
>
  <div class="spacer" style:height={`${start * rowHeight}px`} aria-hidden="true"></div>
  {#each slice as item, i (start + i)}
    {@const index = start + i}
    <div
      id={`mail-row-${index}`}
      data-index={index}
      role="option"
      tabindex="-1"
      aria-selected={index === selectedIndex}
      class:selected={index === selectedIndex}
      class:focused={index === focusedIndex}
      style:height={`${rowHeight}px`}
      onclick={() => onFocusIndex?.(index)}
      ondblclick={() => ondblclick(index)}
      onkeydown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onActivate?.(index);
        }
      }}
    >
      {@render children(item, index)}
    </div>
  {/each}
  <div
    class="spacer"
    style:height={`${Math.max(0, items.length - end) * rowHeight}px`}
    aria-hidden="true"
  ></div>
</div>

<style>
  .virtual-viewport {
    position: relative;
    overflow-y: auto;
    overflow-x: hidden;
    height: 100%;
    outline: none;
    scrollbar-gutter: stable;
  }
  .virtual-viewport:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }
  .spacer {
    width: 1px;
  }
  [role='option'] {
    position: relative;
    display: flex;
    align-items: stretch;
    cursor: default;
    padding: 2px 6px;
  }
</style>
