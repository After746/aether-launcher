<script lang="ts">
  let { value = 0, indeterminate = false }: { value?: number; indeterminate?: boolean } = $props();
  const pct = $derived(Math.max(0, Math.min(100, value)));
</script>

<div class="track">
  <div class="fill" class:indeterminate style="width: {indeterminate ? 100 : pct}%"></div>
</div>

<style>
  .track {
    width: 100%;
    height: 8px;
    border-radius: 999px;
    background: var(--panel-strong);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    box-shadow: 0 0 16px var(--accent-glow);
    transition: width 180ms var(--ease);
  }
  .fill.indeterminate {
    width: 40% !important;
    animation: indet 1.2s var(--ease) infinite;
  }
  @keyframes indet {
    0% { margin-left: -40%; }
    100% { margin-left: 100%; }
  }
</style>
