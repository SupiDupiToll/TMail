<script lang="ts">
  import { initials } from '../format';

  interface Props {
    name: string;
    address: string;
    size?: number;
  }
  let { name, address, size = 28 }: Props = $props();

  const hash = $derived(
    Array.from(address.toLowerCase()).reduce((acc, ch) => (acc * 31 + ch.charCodeAt(0)) % 360, 7)
  );
  const label = $derived(initials(name, address));
</script>

<span
  class="avatar"
  style:width={`${size}px`}
  style:height={`${size}px`}
  style:font-size={`${Math.max(10, Math.round(size * 0.36))}px`}
  style:--hue={hash}
  aria-hidden="true"
>{label}</span>

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    border-radius: 50%;
    background: hsl(var(--hue) 18% 90%);
    color: hsl(var(--hue) 28% 26%);
    font-weight: 700;
    letter-spacing: 0.01em;
    user-select: none;
  }
  :global(:root[data-theme='dark']) .avatar {
    background: hsl(var(--hue) 20% 22%);
    color: hsl(var(--hue) 45% 82%);
  }
</style>
