<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  let { open = false, title = '', onclose, children, footer }: {
    open?: boolean;
    title?: string;
    onclose?: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  function close() { onclose?.(); }
  function onkeydown(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
</script>

<svelte:window onkeydown={open ? onkeydown : undefined} />

{#if open}
  <div class="overlay" transition:fade={{ duration: 160 }} onclick={close}>
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      in:scale={{ start: 0.94, duration: 240, easing: cubicOut }}
      onclick={(e) => e.stopPropagation()}>
      <header class="modal-head">
        <h3>{title}</h3>
        <button class="x" aria-label="Cerrar" onclick={close}>
          <Icon name="close" size={18} />
        </button>
      </header>
      <div class="modal-body">{@render children()}</div>
      {#if footer}<footer class="modal-foot">{@render footer()}</footer>{/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(6, 8, 14, 0.55);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
  }
  .modal {
    width: min(460px, 100%);
    border-radius: var(--radius-xl);
    border: 1px solid var(--stroke);
    background: var(--glass-strong);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }
  .modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 22px;
    border-bottom: 1px solid var(--stroke);
  }
  .modal-head h3 { font-size: 17px; font-weight: 700; }
  .x {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 9px;
    cursor: pointer;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .x:hover { background: var(--panel); color: var(--text); }
  .modal-body { padding: 22px; color: var(--text); }
  .modal-foot {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 16px 22px;
    border-top: 1px solid var(--stroke);
  }
</style>