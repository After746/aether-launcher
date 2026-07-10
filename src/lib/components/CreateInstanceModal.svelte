<script lang="ts">
  import Modal from './ui/Modal.svelte';
  import Button from './ui/Button.svelte';
  import VersionSelect from './VersionSelect.svelte';
  import LoaderSelect from './LoaderSelect.svelte';
  import { instances } from '../stores/instances.svelte';
  import type { Instance, Loader } from '../ipc/types';

  let { open = false, edit = null, onclose }: {
    open?: boolean;
    edit?: Instance | null;
    onclose?: () => void;
  } = $props();

  let name = $state('');
  let mcVersion = $state('');
  let loader = $state<Loader>('vanilla');
  let ram = $state(4096);
  let saving = $state(false);
  let error = $state<string | null>(null);

  const isEdit = $derived(!!edit);

  $effect(() => {
    if (open) {
      error = null;
      if (edit) {
        name = edit.name;
        mcVersion = edit.mc_version;
        loader = edit.loader;
        ram = edit.ram_mb;
      } else {
        name = '';
        mcVersion = '';
        loader = 'vanilla';
        ram = 4096;
      }
    }
  });

  async function submit() {
    if (!name.trim()) { error = 'Ponle un nombre a la instancia.'; return; }
    if (!mcVersion) { error = 'Elige una versión de Minecraft.'; return; }
    if (loader !== 'vanilla') {
      error = 'Por ahora solo Vanilla está disponible. El resto llega pronto.';
      return;
    }
    saving = true;
    error = null;
    try {
      if (isEdit && edit) {
        await instances.update(edit.id, { name: name.trim(), mc_version: mcVersion, loader, ram_mb: ram });
      } else {
        await instances.create({ name: name.trim(), mc_version: mcVersion, loader, ram_mb: ram });
      }
      onclose?.();
    } catch (e: any) {
      error = String(e?.message ?? e);
    } finally {
      saving = false;
    }
  }
</script>

<Modal {open} title={isEdit ? 'Editar instancia' : 'Nueva instancia'} onclose={() => onclose?.()}>
  <div class="form">
    <label>
      <span>Nombre</span>
      <input type="text" bind:value={name} placeholder="Mi mundo" />
    </label>

    <div class="row">
      <div class="field">
        <span class="lbl">Versión de Minecraft</span>
        <VersionSelect bind:value={mcVersion} />
      </div>
      <div class="field">
        <span class="lbl">Loader</span>
        <LoaderSelect bind:value={loader} />
      </div>
    </div>

    <label>
      <span>Memoria asignada: <b>{(ram / 1024).toFixed(1)} GB</b></span>
      <input type="range" min="1024" max="16384" step="512" bind:value={ram} />
    </label>

    {#if error}<p class="err">{error}</p>{/if}
  </div>

  {#snippet footer()}
    <Button variant="ghost" onclick={() => onclose?.()}>Cancelar</Button>
    <Button variant="primary" disabled={saving} onclick={submit}>
      {saving ? 'Guardando…' : isEdit ? 'Guardar cambios' : 'Crear instancia'}
    </Button>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: 16px; }
  .row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: start; }
  .field { display: flex; flex-direction: column; gap: 7px; }
  label { display: flex; flex-direction: column; gap: 7px; }
  label > span, .lbl { font-size: 13px; font-weight: 600; color: var(--text); }
  input[type='text'] {
    font: inherit;
    color: var(--text);
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--stroke);
    background: var(--panel);
    transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  input[type='text']:focus { outline: none; border-color: var(--accent); background: var(--panel-strong); }
  input[type='range'] { width: 100%; accent-color: var(--accent); }
  .err { color: #ff7a8a; font-size: 13px; }
</style>
