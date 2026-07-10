<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children, class: klass = '' }: { children: Snippet; class?: string } = $props();
</script>

<div class="card {klass}">{@render children()}</div>

<style>
  .card {
    background: var(--glass);
    border: 1px solid var(--stroke);
    border-radius: var(--radius-lg);
    padding: 22px;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    box-shadow: var(--shadow-soft);
  }
</style>