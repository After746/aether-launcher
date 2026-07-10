<script lang="ts">
  import Icon from './ui/Icon.svelte';
  import Badge from './ui/Badge.svelte';
  import ProgressBar from './ui/ProgressBar.svelte';
  import { LOADER_LABEL, STATUS_LABEL, PHASE_LABEL } from '../ipc/types';
  import type { InstanceSummary, InstallStatus } from '../ipc/types';
  import { install } from '../stores/install.svelte';

  let { instance, onplay, onedit, onduplicate, ondelete, onfavorite, oninstall }: {
    instance: InstanceSummary;
    onplay: () => void;
    onedit: () => void;
    onduplicate: () => void;
    ondelete: () => void;
    onfavorite: () => void;
    oninstall: () => void;
  } = $props();

  const statusVariant: Record<InstallStatus, 'neutral' | 'accent' | 'ok' | 'warn'> = {
    created: 'warn',
    installing: 'accent',
    cancelled: 'warn',
    ready: 'ok',
    error: 'warn',
    corrupt: 'warn',
  };

  const installing = $derived(install.isInstalling(instance.id));
  const isReady = $derived(instance.status === 'ready');
</script>

<div class="card">
  <button class="fav" class:on={instance.favorite} onclick={onfavorite}
    title={instance.favorite ? 'Quitar de favoritas' : 'Marcar como favorita'}>
    <Icon name="star" size={16} />
  </button>

  <div class="thumb">{instance.name.charAt(0).toUpperCase()}</div>

  <div class="info">
    <b class="name" title={instance.name}>{instance.name}</b>
    <div class="tags">
      <span class="tag">{instance.mc_version}</span>
      <span class="tag">{LOADER_LABEL[instance.loader]}</span>
    </div>
    <div class="status">
      <Badge variant={statusVariant[instance.status]}>{STATUS_LABEL[instance.status]}</Badge>
      <span class="mods">{instance.mod_count} mods</span>
    </div>
  </div>

  {#if installing}
    <div class="installing">
      <div class="ins-head">
        <span>{PHASE_LABEL[install.phase] ?? 'Instalando'}</span>
        <span class="ins-pct">{install.percent.toFixed(0)}%</span>
      </div>
      <ProgressBar value={install.percent} indeterminate={install.totalBytes === 0} />
      <button class="cancel" onclick={() => install.cancel(instance.id)}>Cancelar</button>
    </div>
  {:else}
    <div class="actions">
      {#if isReady}
        <button class="play" onclick={onplay}><Icon name="play" size={15} /> Jugar</button>
      {:else}
        <button class="play install" onclick={oninstall}>
          <Icon name="download" size={15} /> Instalar
        </button>
      {/if}
      <div class="icons">
        <button title="Editar" onclick={onedit}><Icon name="pencil" size={15} /></button>
        <button title="Duplicar" onclick={onduplicate}><Icon name="copy" size={15} /></button>
        <button class="danger" title="Eliminar" onclick={ondelete}><Icon name="trash" size={15} /></button>
      </div>
    </div>
  {/if}
</div>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--stroke);
    background: var(--glass);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: var(--shadow-soft);
    transition: transform var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .card:hover { transform: translateY(-3px); border-color: var(--stroke-strong); }

  .fav {
    position: absolute; top: 14px; right: 14px;
    display: grid; place-items: center; width: 30px; height: 30px;
    border: none; border-radius: 9px; background: var(--panel); color: var(--muted); cursor: pointer;
    transition: color var(--dur) var(--ease), background var(--dur) var(--ease);
  }
  .fav:hover { background: var(--panel-strong); color: var(--text); }
  .fav.on { color: var(--warn); }

  .thumb {
    display: grid; place-items: center; width: 54px; height: 54px;
    border-radius: 14px; font-size: 24px; font-weight: 800; color: #fff;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 8px 24px var(--accent-glow);
  }

  .info { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  .name { font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tags { display: flex; flex-wrap: wrap; gap: 6px; }
  .tag { font-size: 11.5px; padding: 3px 9px; border-radius: 999px; color: var(--muted); background: var(--panel); border: 1px solid var(--stroke); }
  .status { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .mods { font-size: 12px; color: var(--muted); }

  .actions { display: flex; align-items: center; gap: 8px; margin-top: 2px; }
  .play {
    flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 7px;
    padding: 10px; border: none; border-radius: var(--radius-md);
    font: inherit; font-weight: 700; font-size: 13px; color: #fff; cursor: pointer;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 8px 22px var(--accent-glow);
    transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
  }
  .play:hover { transform: translateY(-2px); box-shadow: 0 12px 30px var(--accent-glow); }
  .play:active { transform: translateY(0) scale(0.99); }
  .play.install { background: var(--panel-strong); color: var(--text); box-shadow: none; border: 1px solid var(--stroke-strong); }
  .play.install:hover { background: var(--panel); box-shadow: none; }

  .icons { display: flex; gap: 4px; }
  .icons button {
    display: grid; place-items: center; width: 36px; height: 36px;
    border: 1px solid var(--stroke); border-radius: var(--radius-md);
    background: var(--panel); color: var(--muted); cursor: pointer;
    transition: color var(--dur) var(--ease), background var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .icons button:hover { background: var(--panel-strong); color: var(--text); border-color: var(--stroke-strong); }
  .icons button.danger:hover { background: rgba(229, 72, 77, 0.16); color: #ff7a8a; border-color: transparent; }

  .installing { display: flex; flex-direction: column; gap: 8px; }
  .ins-head { display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; }
  .ins-pct { font-weight: 800; color: var(--accent); }
  .cancel {
    align-self: flex-end; margin-top: 2px; padding: 5px 12px;
    border: 1px solid var(--stroke); border-radius: 8px;
    background: transparent; color: var(--muted); font: inherit; font-size: 12px; cursor: pointer;
    transition: color var(--dur) var(--ease), border-color var(--dur) var(--ease);
  }
  .cancel:hover { color: #ff7a8a; border-color: rgba(229, 72, 77, 0.4); }
</style>
