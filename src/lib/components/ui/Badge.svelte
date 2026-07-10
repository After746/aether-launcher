<script lang="ts">
  import type { Snippet } from 'svelte';

  let { variant = 'neutral', children }: {
    variant?: 'neutral' | 'accent' | 'ok' | 'warn';
    children: Snippet;
  } = $props();
</script>

<span class="badge {variant}">{@render children()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
    border: 1px solid var(--stroke);
    white-space: nowrap;
  }
  .neutral { background: var(--panel); color: var(--muted); }
  .accent { background: var(--accent-soft); color: var(--accent); border-color: transparent; }
  .ok { background: rgba(70, 211, 154, 0.14); color: var(--ok); border-color: transparent; }
  .warn { background: rgba(255, 180, 84, 0.14); color: var(--warn); border-color: transparent; }
</style>