<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from '../components/ui/Icon.svelte';
  import { theme } from '../stores/theme.svelte';

  const appWindow = getCurrentWindow();
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="mark">&#9670;</span>
    <span class="name">Aether</span>
  </div>

  <div class="controls">
    <button class="ctrl" title="Cambiar tema" onclick={() => theme.toggle()}>
      <Icon name={theme.current === 'dark' ? 'sun' : 'moon'} size={16} />
    </button>
    <button class="ctrl" title="Minimizar" onclick={() => appWindow.minimize()}>
      <Icon name="minus" size={16} />
    </button>
    <button class="ctrl" title="Maximizar" onclick={() => appWindow.toggleMaximize()}>
      <Icon name="square" size={14} />
    </button>
    <button class="ctrl danger" title="Cerrar" onclick={() => appWindow.close()}>
      <Icon name="close" size={16} />
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: 44px;
    flex: 0 0 44px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 18px;
    user-select: none;
    border-bottom: 1px solid var(--stroke);
  }
  .brand { display: flex; align-items: center; gap: 8px; }
  .mark { color: var(--accent); font-size: 14px; }
  .name {
    font-weight: 700;
    letter-spacing: 0.4px;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .controls { display: flex; gap: 2px; }
  .ctrl {
    width: 34px;
    height: 30px;
    display: grid;
    place-items: center;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 8px;
    cursor: pointer;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .ctrl:hover { background: var(--panel); color: var(--text); }
  .ctrl.danger:hover { background: #e5484d; color: #fff; }
</style>