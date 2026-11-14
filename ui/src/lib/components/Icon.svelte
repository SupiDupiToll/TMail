<script lang="ts">
  interface Props {
    name: string;
    size?: number;
    stroke?: number;
    label?: string;
  }
  let { name, size = 18, stroke = 1.5, label }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden={label ? undefined : 'true'}
  role={label ? 'img' : undefined}
  aria-label={label}
>
  {#if name === 'search'}
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-3.2-3.2" />
  {:else if name === 'compose'}
    <path d="M4 20h4l10.5-10.5a2.1 2.1 0 0 0-3-3L5 17v3Z" />
    <path d="m13.5 7.5 3 3" />
  {:else if name === 'inbox'}
    <path d="M4 13h4l1.5 2.5h5L16 13h4" />
    <path d="M4 13 6 5h12l2 8v6H4v-6Z" />
  {:else if name === 'draft'}
    <path d="M6 3h8l4 4v14H6z" />
    <path d="M14 3v4h4M9 13h6M9 17h4" />
  {:else if name === 'sent'}
    <path d="M21 3 10 14" />
    <path d="m21 3-7 18-4-7-7-4 18-7Z" />
  {:else if name === 'trash'}
    <path d="M3 6h18M8 6V4h8v2M6 6l1 15h10l1-15" />
    <path d="M10 11v6M14 11v6" />
  {:else if name === 'archive'}
    <path d="M3 4h18v4H3zM5 8v12h14V8" />
    <path d="M10 12h4" />
  {:else if name === 'spam'}
    <path d="M12 3 2.8 20h18.4L12 3Z" />
    <path d="M12 9v5M12 17.2v.1" />
  {:else if name === 'folder'}
    <path d="M3 6h6l2 2h10v11H3z" />
  {:else if name === 'chevron'}
    <path d="m9 6 6 6-6 6" />
  {:else if name === 'chevron-down'}
    <path d="m6 9 6 6 6-6" />
  {:else if name === 'reply'}
    <path d="M9 7 4 12l5 5" />
    <path d="M4 12h9a7 7 0 0 1 7 7v1" />
  {:else if name === 'reply-all'}
    <path d="m7 7-5 5 5 5M13 7l-5 5 5 5" />
    <path d="M8 12h7a6 6 0 0 1 6 6v1" />
  {:else if name === 'forward'}
    <path d="m15 7 5 5-5 5" />
    <path d="M20 12h-9a7 7 0 0 0-7 7v1" />
  {:else if name === 'attachment'}
    <path d="m20 11-8.5 8.5a5 5 0 0 1-7-7L14 3a3.5 3.5 0 0 1 5 5l-9.5 9.5a2 2 0 0 1-3-3L15 6" />
  {:else if name === 'more'}
    <circle cx="5" cy="12" r="1" fill="currentColor" stroke="none" />
    <circle cx="12" cy="12" r="1" fill="currentColor" stroke="none" />
    <circle cx="19" cy="12" r="1" fill="currentColor" stroke="none" />
  {:else if name === 'close'}
    <path d="M5 5l14 14M19 5 5 19" />
  {:else if name === 'check'}
    <path d="m4 12 5 5L20 6" />
  {:else if name === 'menu'}
    <path d="M4 6h16M4 12h16M4 18h16" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6 1.7 1.7 0 0 0 10 3V2.8h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />
  {:else if name === 'command'}
    <path d="M9 6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6Z" />
  {:else if name === 'arrow-left'}
    <path d="M20 12H4M10 6l-6 6 6 6" />
  {:else if name === 'download'}
    <path d="M12 3v12M7 10l5 5 5-5M4 20h16" />
  {:else if name === 'shield'}
    <path d="M12 3 4.5 6v6c0 4.5 3 7.6 7.5 9 4.5-1.4 7.5-4.5 7.5-9V6L12 3Z" />
    <path d="m9 12 2 2 4-4" />
  {:else if name === 'clock'}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 7v5l3 2" />
  {:else if name === 'image-off'}
    <path d="M3 3l18 18M5 5h14v14H5z" />
    <path d="m5 16 4-4 3 3M15 13l1-1 3 3" />
  {:else if name === 'image'}
    <path d="M5 5h14v14H5zM5 16l4-4 3 3 2-2 5 5" />
    <circle cx="9" cy="9" r="1.4" />
  {:else if name === 'sun'}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  {:else if name === 'moon'}
    <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5Z" />
  {:else if name === 'monitor'}
    <path d="M3 5h18v11H3zM9 20h6M12 16v4" />
  {:else if name === 'mailbox'}
    <path d="M4 10h16v10H4zM4 10l2-6h12l2 6" />
    <path d="M9 20v-5h6v5" />
  {:else if name === 'key'}
    <circle cx="8" cy="15" r="4" />
    <path d="m11 12 8-8M16 4l3 3M14 6l3 3" />
  {:else if name === 'refresh'}
    <path d="M20 12a8 8 0 1 1-2.3-5.7M20 4v5h-5" />
  {:else if name === 'bolt'}
    <path d="M13 2 4 14h6l-1 8 9-12h-6l1-8Z" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'smile'}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M8.5 14.5a4.5 4.5 0 0 0 7 0M9 9.5h.01M15 9.5h.01" />
  {:else}
    <circle cx="12" cy="12" r="8" />
  {/if}
</svg>

<style>
  svg {
    flex: none;
    display: block;
  }
</style>
