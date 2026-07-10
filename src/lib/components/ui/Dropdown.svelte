<script lang="ts">
  import Icon from './Icon.svelte';
  import { slide } from 'svelte/transition';

  export type Option = { value: string; label: string; hint?: string };

  let { value = $bindable(), options, placeholder = 'Selecciona…' }: {
    value: string;
    options: Option[];
    placeholder?: string;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;

  const current = $derived(options.find((o) => o.value === value) ?? null);

  function choose(v: string) {
    value = v;
    open = false;
  }
  function onWindowClick(e: MouseEvent) {
    if (root && !root.contains(e.target as Node)) open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onKey} />

<div class="dd" bind:this={root}>
  <button type="button" class="trigger" class:open onclick={() => (open = !open)}>
    <span class="value" class:placeholder={!current}>{current ? current.label : placeholder}</span>
    <span class="chev" class:open><Icon name="chevron" size={16} /></span>
  </button>

  {#if open}
    <div class="menu" transition:slide={{ duration: 160 }}>
      {#each options as opt (opt.value)}
        <button
          type="button"
          class="opt"
          class:active={opt.value === value}
          onclick={() => choose(opt.value)}>
          <span class="opt-label">{opt.label}</span>
          {#if opt.hint}<span class="opt-hint">{opt.hint}</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dd { position: relative; width: 100%; }
  .trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 12px;
    font: inherit;
    color: var(--text);
    background: var(--panel);
    border: 1px solid var(--stroke);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .trigger:hover { background: var(--panel-strong); }
  .trigger.open { border-color: var(--accent); background: var(--panel-strong); }
  .value { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .value.placeholder { color: var(--muted); }
  .chev { color: var(--muted); transition: transform var(--dur) var(--ease); }
  .chev.open { transform: rotate(90deg); }

  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    max-height: 240px;
    overflow-y: auto;
    padding: 6px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke-strong);
    background: var(--glass-strong);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    box-shadow: var(--shadow-lg);
  }
  .opt {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 9px 11px;
    border: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    text-align: left;
    border-radius: 9px;
    cursor: pointer;
    transition: background var(--dur) var(--ease);
  }
  .opt:hover { background: var(--panel); }
  .opt.active { background: var(--accent-soft); color: var(--accent); }
  .opt-hint { font-size: 11px; color: var(--muted); }
</style>
