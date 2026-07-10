<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    variant = 'primary',
    disabled = false,
    onclick,
    children,
  }: {
    variant?: 'primary' | 'ghost' | 'glass';
    disabled?: boolean;
    onclick?: () => void;
    children: Snippet;
  } = $props();
</script>

<button class="btn {variant}" {disabled} {onclick}>
  {@render children()}
</button>

<style>
  .btn {
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    padding: 10px 20px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    color: var(--text);
    transition:
      transform var(--dur) var(--ease),
      background var(--dur) var(--ease),
      box-shadow var(--dur) var(--ease),
      border-color var(--dur) var(--ease);
  }
  .btn:active:not(:disabled) { transform: translateY(0) scale(0.98); }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }

  .primary {
    color: #fff;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 10px 30px var(--accent-glow);
  }
  .primary:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 16px 44px var(--accent-glow);
  }

  .ghost { background: transparent; border-color: var(--stroke); }
  .ghost:hover:not(:disabled) { background: var(--panel); border-color: var(--stroke-strong); }

  .glass {
    background: var(--glass);
    border-color: var(--stroke);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }
  .glass:hover:not(:disabled) { background: var(--glass-strong); transform: translateY(-1px); }
</style>