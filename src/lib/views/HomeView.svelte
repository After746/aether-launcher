<script lang="ts">
  import { onMount } from 'svelte';
  import Panel from '../components/ui/Panel.svelte';
  import Badge from '../components/ui/Badge.svelte';
  import Icon from '../components/ui/Icon.svelte';
  import InstallProgress from '../components/InstallProgress.svelte';
  import { router } from '../stores/router.svelte';
  import { instances } from '../stores/instances.svelte';
  import { install } from '../stores/install.svelte';
  import { LOADER_LABEL, STATUS_LABEL } from '../ipc/types';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  onMount(() => { if (!instances.loaded) instances.load(); });

  const sel = $derived(instances.selected);
  const total = $derived(instances.items.length);
  const totalMods = $derived(instances.items.reduce((a, i) => a + i.mod_count, 0));
  const totalHours = $derived(Math.floor(instances.items.reduce((a, i) => a + i.playtime_secs, 0) / 3600));
  const totalGb = $derived(
    (instances.items.reduce((a, i) => a + (i.total_size_bytes ?? 0), 0) / 1024 / 1024 / 1024).toFixed(1),
  );
  const selInstalling = $derived(!!sel && install.isInstalling(sel.id));

  const quick = [
    { icon: 'plus', label: 'Nueva instancia', primary: true, go: () => router.navigate('instances') },
    { icon: 'grid', label: 'Mis instancias', primary: false, go: () => router.navigate('instances') },
    { icon: 'package', label: 'Mods', primary: false, go: () => router.navigate('instances') },
    { icon: 'settings', label: 'Ajustes', primary: false, go: () => router.navigate('settings') },
  ];

  function lastPlayedLabel(ts: number | null): string {
    if (!ts) return 'Nunca';
    return new Date(ts * 1000).toLocaleDateString();
  }

  function primaryAction() {
    if (!sel) { router.navigate('instances'); return; }
    if (sel.status === 'ready') {
      // El lanzamiento real de la JVM llega en la Fase 4.
      return;
    }
    install.start(sel.id);
  }

  const ctaLabel = $derived(!sel ? 'CREAR INSTANCIA' : sel.status === 'ready' ? 'JUGAR' : 'INSTALAR');
  const ctaSub = $derived(
    !sel ? 'Configura tu primer perfil'
      : sel.status === 'ready' ? sel.mc_version
      : sel.status === 'error' ? 'Reintentar instalación'
      : 'Descargar ' + sel.mc_version,
  );
</script>

<div class="home">
  <section class="featured" in:fly={{ y: 18, duration: 380, easing: cubicOut }}>
    <div class="featured-glow"></div>
    <div class="featured-info">
      {#if sel}
        <span class="eyebrow">{sel.favorite ? 'Tu favorita' : 'Continúa jugando'}</span>
        <h1>{sel.name}</h1>
        <div class="meta">
          <span class="chip"><Icon name="layers" size={14} /> {sel.mc_version}</span>
          <span class="chip"><Icon name="package" size={14} /> {LOADER_LABEL[sel.loader]}</span>
          <span class="chip"><Icon name="package" size={14} /> {sel.mod_count} mods</span>
          <span class="chip"><Icon name="clock" size={14} /> {lastPlayedLabel(sel.last_played)}</span>
        </div>
      {:else}
        <span class="eyebrow">Empieza aquí</span>
        <h1>Tu primera instancia te espera</h1>
        <p class="featured-sub">Crea un perfil, elige tu versión y despega en segundos.</p>
      {/if}
    </div>

    {#if !selInstalling}
      <div class="featured-cta">
        {#if sel}<Badge variant={sel.status === 'ready' ? 'ok' : 'warn'}>{STATUS_LABEL[sel.status]}</Badge>{/if}
        <button class="play" onclick={primaryAction}>
          <Icon name={sel && sel.status === 'ready' ? 'play' : sel ? 'download' : 'plus'} size={28} />
          <span class="play-text">
            <b>{ctaLabel}</b>
            <small>{ctaSub}</small>
          </span>
        </button>
      </div>
    {/if}
  </section>

  {#if selInstalling && sel}
    <section in:fly={{ y: 14, duration: 320, easing: cubicOut }}>
      <Panel title="Instalando {sel.name}" description="No cierres Aether hasta terminar.">
        <InstallProgress id={sel.id} />
      </Panel>
    </section>
  {/if}

  <section class="quick" in:fly={{ y: 18, duration: 380, delay: 80, easing: cubicOut }}>
    {#each quick as q, i (q.label)}
      <button class="q-card" class:primary={q.primary} onclick={q.go} style="--d:{i * 45}ms">
        <span class="q-icon"><Icon name={q.icon} size={20} /></span>
        <span class="q-label">{q.label}</span>
        <span class="q-arrow"><Icon name="chevron" size={16} /></span>
      </button>
    {/each}
  </section>

  <div class="grid">
    <div in:fly={{ y: 18, duration: 380, delay: 150, easing: cubicOut }}>
      <Panel title="Actividad reciente" description="Tu historial aparecerá aquí cuando juegues.">
        <div class="empty-row">
          <span class="empty-ic"><Icon name="clock" size={22} /></span>
          <div>
            <b>Tu historial te espera</b>
            <p>Cuando juegues, verás aquí tus últimas sesiones.</p>
          </div>
        </div>
      </Panel>
    </div>

    <div in:fly={{ y: 18, duration: 380, delay: 210, easing: cubicOut }}>
      <Panel title="Novedades" description="Noticias y actualizaciones de Aether.">
        {#snippet action()}<Badge variant="accent">Próximamente</Badge>{/snippet}
        <article class="news-item">
          <span class="news-dot"></span>
          <div>
            <b>Bienvenido a Aether Launcher</b>
            <p>Estamos construyendo el launcher más rápido y elegante. Pronto: noticias en vivo.</p>
          </div>
        </article>
      </Panel>
    </div>

    <div class="span-2" in:fly={{ y: 18, duration: 380, delay: 270, easing: cubicOut }}>
      <Panel title="Estadísticas" description="Tu resumen crecerá a medida que juegues.">
        <div class="stats">
          <div class="stat"><span class="stat-ic"><Icon name="grid" size={18} /></span><b>{total}</b><span>Instancias</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="package" size={18} /></span><b>{totalMods}</b><span>Mods</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="clock" size={18} /></span><b>{totalHours} h</b><span>Jugadas</span></div>
          <div class="stat"><span class="stat-ic"><Icon name="download" size={18} /></span><b>{totalGb} GB</b><span>Descargado</span></div>
        </div>
      </Panel>
    </div>
  </div>
</div>

<style>
  .home { display: flex; flex-direction: column; gap: 22px; max-width: 1140px; margin: 0 auto; }

  .featured {
    position: relative; display: flex; align-items: center; justify-content: space-between;
    gap: 24px; padding: 38px 40px; border-radius: var(--radius-xl);
    border: 1px solid var(--stroke);
    background: linear-gradient(120deg, var(--accent-soft), transparent 60%), var(--glass);
    backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
    box-shadow: var(--shadow-soft); overflow: hidden;
  }
  .featured-glow {
    position: absolute; top: -40%; right: -10%; width: 380px; height: 380px; border-radius: 50%;
    background: radial-gradient(circle, var(--accent-glow), transparent 70%); filter: blur(20px); pointer-events: none;
  }
  .featured-info { position: relative; z-index: 1; min-width: 0; }
  .eyebrow { display: inline-block; font-size: 11px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); margin-bottom: 10px; }
  .featured h1 { font-size: 30px; font-weight: 800; letter-spacing: -0.5px; }
  .featured-sub { color: var(--muted); font-size: 15px; margin-top: 12px; max-width: 420px; }
  .meta { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 16px; }
  .chip { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 999px; font-size: 12.5px; color: var(--muted); background: var(--panel); border: 1px solid var(--stroke); }

  .featured-cta { position: relative; z-index: 1; display: flex; flex-direction: column; align-items: flex-end; gap: 12px; flex: 0 0 auto; }
  .play {
    display: inline-flex; align-items: center; gap: 15px; padding: 22px 38px;
    border: none; border-radius: var(--radius-lg); color: #fff; cursor: pointer;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 16px 48px var(--accent-glow);
    transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
  }
  .play:hover { transform: translateY(-3px) scale(1.02); box-shadow: 0 22px 64px var(--accent-glow); }
  .play:active { transform: translateY(-1px) scale(0.99); }
  .play-text { display: flex; flex-direction: column; align-items: flex-start; line-height: 1.1; }
  .play-text b { font-size: 24px; font-weight: 800; letter-spacing: 1.5px; }
  .play-text small { font-size: 12px; opacity: 0.85; }

  .quick { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
  .q-card {
    display: flex; align-items: center; gap: 12px; padding: 16px 18px;
    border: 1px solid var(--stroke); border-radius: var(--radius-lg); background: var(--glass);
    backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); color: var(--text); cursor: pointer; text-align: left;
    animation: rise 0.42s var(--ease) both; animation-delay: var(--d);
    transition: transform var(--dur) var(--ease), border-color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .q-card:hover { transform: translateY(-3px); border-color: var(--stroke-strong); background: var(--glass-strong); }
  .q-card:active { transform: translateY(-1px) scale(0.99); }
  .q-card.primary { border-color: transparent; background: var(--accent-soft); }
  .q-card.primary .q-label { color: var(--accent); }
  .q-icon { display: grid; place-items: center; width: 40px; height: 40px; border-radius: 12px; color: var(--accent); background: var(--accent-soft); flex: 0 0 auto; }
  .q-card.primary .q-icon { color: #fff; background: linear-gradient(135deg, var(--accent), var(--accent-2)); }
  .q-label { flex: 1; font-weight: 600; font-size: 13.5px; }
  .q-arrow { color: var(--muted); transition: transform var(--dur) var(--ease); }
  .q-card:hover .q-arrow { transform: translateX(3px); color: var(--text); }

  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; }
  .grid > div { min-width: 0; }
  .span-2 { grid-column: 1 / -1; }

  .empty-row { display: flex; align-items: center; gap: 14px; padding: 8px 0; }
  .empty-ic { display: grid; place-items: center; width: 46px; height: 46px; border-radius: 13px; color: var(--accent); background: var(--accent-soft); flex: 0 0 auto; }
  .empty-row b { font-size: 14px; }
  .empty-row p { color: var(--muted); font-size: 13px; margin-top: 2px; }

  .news-item { display: flex; gap: 12px; }
  .news-dot { width: 9px; height: 9px; margin-top: 6px; border-radius: 50%; background: linear-gradient(var(--accent), var(--accent-2)); flex: 0 0 auto; box-shadow: 0 0 12px var(--accent-glow); }
  .news-item b { font-size: 14px; }
  .news-item p { color: var(--muted); font-size: 13px; margin-top: 3px; }

  .stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
  .stat { display: flex; flex-direction: column; gap: 4px; padding: 16px; border-radius: var(--radius-md); background: var(--panel); border: 1px solid var(--stroke); }
  .stat-ic { color: var(--accent); margin-bottom: 4px; }
  .stat b { font-size: 22px; font-weight: 800; }
  .stat span { color: var(--muted); font-size: 12px; }

  @keyframes rise { from { opacity: 0; transform: translateY(12px); } to { opacity: 1; transform: translateY(0); } }

  @media (max-width: 880px) {
    .quick { grid-template-columns: repeat(2, 1fr); }
    .grid { grid-template-columns: 1fr; }
    .stats { grid-template-columns: repeat(2, 1fr); }
    .featured { flex-direction: column; align-items: flex-start; }
    .featured-cta { align-items: flex-start; }
  }
</style>
