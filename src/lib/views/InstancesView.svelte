<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '../components/ui/Button.svelte';
  import Modal from '../components/ui/Modal.svelte';
  import InstanceCard from '../components/InstanceCard.svelte';
  import CreateInstanceModal from '../components/CreateInstanceModal.svelte';
  import { instances } from '../stores/instances.svelte';
  import { install } from '../stores/install.svelte';
  import { router } from '../stores/router.svelte';
  import type { Instance, InstanceSummary } from '../ipc/types';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { launch } from '../stores/launch.svelte';

  let showForm = $state(false);
  let editing = $state<Instance | null>(null);
  let toDelete = $state<InstanceSummary | null>(null);
  let busy = $state(false);

  onMount(() => { if (!instances.loaded) instances.load(); });

  function openCreate() { editing = null; showForm = true; }
  async function openEdit(s: InstanceSummary) { editing = await instances.get(s.id); showForm = true; }
  function playOne(s: InstanceSummary) { instances.select(s.id); launch.play(s.id); }
  function installOne(s: InstanceSummary) { instances.select(s.id); install.start(s.id); }

  async function confirmDelete() {
    if (!toDelete) return;
    busy = true;
    try { await instances.remove(toDelete.id); toDelete = null; }
    finally { busy = false; }
  }
</script>

<div class="page">
  <header class="page-head" in:fly={{ y: 14, duration: 320, easing: cubicOut }}>
    <div>
      <h2>Instancias</h2>
      <p>Gestiona tus perfiles de Minecraft.</p>
    </div>
    <Button variant="glass" onclick={openCreate}>+ Nueva instancia</Button>
  </header>

  {#if instances.items.length === 0}
    <div class="empty" in:fade={{ duration: 260 }}>
      <div class="orb"></div>
      <h3>Tu primera instancia te espera</h3>
      <p>Crea un perfil, elige versión y loader, y prepárate para jugar.</p>
      <div class="empty-cta"><Button variant="primary" onclick={openCreate}>Crear instancia</Button></div>
    </div>
  {:else}
    <div class="grid">
      {#each instances.items as inst, i (inst.id)}
        <div in:fly={{ y: 16, duration: 320, delay: i * 45, easing: cubicOut }}>
          <InstanceCard
            instance={inst}
            onplay={() => playOne(inst)}
            oninstall={() => installOne(inst)}
            onedit={() => openEdit(inst)}
            onduplicate={() => instances.duplicate(inst.id)}
            ondelete={() => (toDelete = inst)}
            onfavorite={() => instances.toggleFavorite(inst.id, !inst.favorite)} />
        </div>
      {/each}
    </div>
  {/if}
</div>

<CreateInstanceModal open={showForm} edit={editing} onclose={() => (showForm = false)} />

<Modal open={!!toDelete} title="Eliminar instancia" onclose={() => (toDelete = null)}>
  <p class="del-note">
    ¿Seguro que quieres eliminar <b>{toDelete?.name}</b>? Se borrarán todos sus
    archivos de forma permanente. Esta acción no se puede deshacer.
  </p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (toDelete = null)}>Cancelar</Button>
    <Button variant="primary" disabled={busy} onclick={confirmDelete}>{busy ? 'Eliminando…' : 'Eliminar'}</Button>
  {/snippet}
</Modal>

{#if launch.error}
  <p class="launch-error">No se pudo iniciar: {launch.error}</p>
  {/if}

<style>
  .page { display: flex; flex-direction: column; height: 100%; max-width: 1100px; margin: 0 auto; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; margin-bottom: 28px; }
  .page-head h2 { font-size: 26px; font-weight: 700; }
  .page-head p { color: var(--muted); margin-top: 4px; }
  .launch-error { color: #ff7a8a; font-size: 13px; margin-bottom: 12px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 18px; align-content: start; }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; gap: 6px; padding-bottom: 40px; }
  .orb { width: 92px; height: 92px; border-radius: 50%; margin-bottom: 20px; background: radial-gradient(circle at 30% 30%, var(--accent), var(--accent-2)); box-shadow: 0 0 60px var(--accent-glow); animation: float 4s ease-in-out infinite; }
  @keyframes float { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-10px); } }
  .empty h3 { font-size: 19px; font-weight: 700; }
  .empty p { color: var(--muted); max-width: 360px; }
  .empty-cta { margin-top: 18px; }
  .del-note { color: var(--muted); font-size: 14px; line-height: 1.6; }
  .del-note b { color: var(--text); }
</style>
