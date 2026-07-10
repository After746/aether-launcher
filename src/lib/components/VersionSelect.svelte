<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Dropdown from './ui/Dropdown.svelte';
  import type { VersionOption } from '../ipc/types';

  let { value = $bindable() }: { value: string } = $props();

  let versions = $state<VersionOption[]>([]);
  let showSnapshots = $state(false);
  let loading = $state(true);

  const options = $derived(
    versions
      .filter((v) => showSnapshots || v.kind === 'release')
      .map((v) => ({ value: v.id, label: v.id, hint: v.kind === 'release' ? '' : v.kind })),
  );

  async function load() {
    loading = true;
    try {
      versions = await invoke<VersionOption[]>('list_mc_versions');
      if (!value && versions.length) {
        value = versions.find((v) => v.kind === 'release')?.id ?? versions[0].id;
      }
    } catch {
      versions = [];
    } finally {
      loading = false;
    }
  }
  load();
</script>

<div class="vs">
  {#if loading}
    <div class="loading">Cargando versiones…</div>
  {:else}
    <Dropdown bind:value {options} placeholder="Elige versión" />
  {/if}
  <label class="snap">
    <input type="checkbox" bind:checked={showSnapshots} />
    <span>Mostrar snapshots</span>
  </label>
</div>

<style>
  .vs { display: flex; flex-direction: column; gap: 8px; }
  .loading {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke);
    background: var(--panel);
    color: var(--muted);
    font-size: 13px;
  }
  .snap { display: flex; align-items: center; gap: 7px; font-size: 12px; color: var(--muted); cursor: pointer; }
  .snap input { accent-color: var(--accent); }
</style>
