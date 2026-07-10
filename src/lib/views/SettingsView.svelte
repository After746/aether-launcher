<script lang="ts">
  import Panel from '../components/ui/Panel.svelte';
  import Badge from '../components/ui/Badge.svelte';
  import { theme } from '../stores/theme.svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
</script>

<div class="page">
  <header class="page-head" in:fly={{ y: 14, duration: 320, easing: cubicOut }}>
    <h2>Ajustes</h2>
    <p>Configura Aether Launcher a tu gusto.</p>
  </header>

  <div class="stack">
    <div in:fly={{ y: 16, duration: 340, delay: 60, easing: cubicOut }}>
      <Panel title="Apariencia" description="Cambia entre tema oscuro y claro.">
        <div class="seg">
          <button class:active={theme.current === 'dark'} onclick={() => theme.set('dark')}>Oscuro</button>
          <button class:active={theme.current === 'light'} onclick={() => theme.set('light')}>Claro</button>
        </div>
      </Panel>
    </div>

    <div in:fly={{ y: 16, duration: 340, delay: 120, easing: cubicOut }}>
      <Panel title="Java y rendimiento" description="Memoria asignada y argumentos de la JVM.">
        {#snippet action()}<Badge>Próximamente</Badge>{/snippet}
        <p class="soon">Disponible cuando implementemos el lanzamiento de Minecraft.</p>
      </Panel>
    </div>

    <div in:fly={{ y: 16, duration: 340, delay: 180, easing: cubicOut }}>
      <Panel title="Descargas" description="Concurrencia y carpeta de datos.">
        {#snippet action()}<Badge>Próximamente</Badge>{/snippet}
        <p class="soon">Se configurará junto al motor de descargas.</p>
      </Panel>
    </div>

    <div in:fly={{ y: 16, duration: 340, delay: 240, easing: cubicOut }}>
      <Panel title="Cuenta" description="Inicia sesión con tu cuenta de Microsoft.">
        {#snippet action()}<Badge>Próximamente</Badge>{/snippet}
        <p class="soon">La autenticación llegará en una fase posterior.</p>
      </Panel>
    </div>
  </div>
</div>

<style>
  .page { max-width: 760px; margin: 0 auto; }
  .page-head { margin-bottom: 24px; }
  .page-head h2 { font-size: 26px; font-weight: 700; }
  .page-head p { color: var(--muted); margin-top: 4px; }
  .stack { display: flex; flex-direction: column; gap: 16px; }

  .seg {
    display: inline-flex;
    background: var(--panel);
    border-radius: 10px;
    padding: 3px;
    border: 1px solid var(--stroke);
  }
  .seg button {
    border: none;
    background: transparent;
    color: var(--muted);
    padding: 8px 20px;
    border-radius: 8px;
    cursor: pointer;
    font: inherit;
    font-weight: 500;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .seg button.active { background: var(--panel-strong); color: var(--text); }
  .soon { color: var(--muted); font-size: 13px; }
</style>
