<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title = '', description = '', action, children }: {
    title?: string;
    description?: string;
    action?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section class="panel">
  {#if title}
    <header class="panel-head">
      <div class="panel-titles">
        <h3>{title}</h3>
        {#if description}<p>{description}</p>{/if}
      </div>
      {#if action}<div class="panel-action">{@render action()}</div>{/if}
    </header>
  {/if}
  <div class="panel-body">{@render children()}</div>
</section>

<style>
  .panel {
    height: 100%;
    background: var(--glass);
    border: 1px solid var(--stroke);
    border-radius: var(--radius-lg);
    padding: 22px;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    box-shadow: var(--shadow-soft);
  }
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 18px;
  }
  .panel-titles h3 { font-size: 15px; font-weight: 700; }
  .panel-titles p { color: var(--muted); font-size: 12.5px; margin-top: 3px; }
  .panel-action { flex: 0 0 auto; }
</style>