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
  let menuEl: HTMLDivElement;
  let openUpward = $state(false);
  let menuStyle = $state('');

  const current = $derived(options.find((o) => o.value === value) ?? null);

  const MENU_MAX_HEIGHT = 240;
  const GAP = 6;
  const VIEWPORT_MARGIN = 12;

  // El menú se porta a document.body para escapar del `overflow: hidden`
  // (y del containing block que crea el `backdrop-filter`) del modal padre.
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }

  function updatePosition() {
    if (!root) return;
    const rect = root.getBoundingClientRect();
    const spaceBelow = window.innerHeight - rect.bottom - VIEWPORT_MARGIN;
    const spaceAbove = rect.top - VIEWPORT_MARGIN;

    openUpward = spaceBelow < MENU_MAX_HEIGHT && spaceAbove > spaceBelow;

    const available = Math.max(120, Math.min(MENU_MAX_HEIGHT, openUpward ? spaceAbove : spaceBelow));

    menuStyle = openUpward
      ? `left:${rect.left}px; width:${rect.width}px; bottom:${window.innerHeight - rect.top + GAP}px; max-height:${available}px;`
      : `left:${rect.left}px; width:${rect.width}px; top:${rect.bottom + GAP}px; max-height:${available}px;`;
  }

  function toggle() {
    if (!open) updatePosition();
    open = !open;
  }
  function choose(v: string) {
    value = v;
    open = false;
  }
  function onWindowClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (root && !root.contains(t) && menuEl && !menuEl.contains(t)) open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
  function onReposition() {
    if (open) updatePosition();
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onKey} onresize={onReposition} onscroll={onReposition} />

<div class="dd" bind:this={root}>
  <button type="button" class="trigger" class:open onclick={toggle}>
    <span class="value" class:placeholder={!current}>{current ? current.label : placeholder}</span>
    <span class="chev" class:open><Icon name="chevron" size={16} /></span>
  </button>

  {#if open}
    <div
      class="menu"
      class:up={openUpward}
      style={menuStyle}
      use:portal
      bind:this={menuEl}
      transition:slide={{ duration: 160 }}>
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
    position: fixed;
    z-index: 1000;
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
