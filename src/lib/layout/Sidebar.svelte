<script lang="ts">
  import { router } from '../stores/router.svelte';
  import { NAV } from '../router/routes';
  import Icon from '../components/ui/Icon.svelte';

  const version = '0.1.0';
</script>

<aside class="sidebar">
  <div class="brand">
    <span class="logo-mark">◆</span>
    <div class="brand-text">
      <span class="brand-name">Aether</span>
      <span class="brand-sub">Launcher</span>
    </div>
  </div>

  <nav class="nav">
    {#each NAV as item (item.id)}
      <button
        class="nav-item"
        class:active={router.current === item.id}
        onclick={() => router.navigate(item.id)}>
        <span class="indicator"></span>
        <span class="nav-ic"><Icon name={item.icon} size={19} /></span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="foot">
    <button class="account">
      <span class="avatar"><Icon name="user" size={18} /></span>
      <span class="acc-text">
        <b>Invitado</b>
        <small>Iniciar sesión</small>
      </span>
    </button>
    <span class="version">v{version}</span>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-w);
    flex: 0 0 var(--sidebar-w);
    display: flex;
    flex-direction: column;
    padding: 18px 14px;
    border-right: 1px solid var(--stroke);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px 18px;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--stroke);
  }
  .logo-mark {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 12px;
    color: #fff;
    font-size: 15px;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: 0 8px 24px var(--accent-glow);
  }
  .brand-text { display: flex; flex-direction: column; line-height: 1.15; }
  .brand-name {
    font-weight: 800;
    font-size: 17px;
    letter-spacing: 0.3px;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .brand-sub { font-size: 11px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); }

  .nav { flex: 1; display: flex; flex-direction: column; gap: 4px; padding-top: 8px; }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 11px 14px;
    border: none;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-weight: 500;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease);
  }
  .nav-item:hover { background: var(--panel); color: var(--text); }
  .nav-item:hover .nav-ic { transform: translateX(2px); }
  .nav-item.active { color: var(--text); background: var(--panel-strong); }
  .nav-ic { display: grid; place-items: center; transition: transform var(--dur) var(--ease); }
  .indicator {
    position: absolute;
    left: 0;
    top: 50%;
    width: 3px;
    height: 20px;
    border-radius: 3px;
    background: linear-gradient(var(--accent), var(--accent-2));
    transform: translateY(-50%) scaleY(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease);
  }
  .nav-item.active .indicator { transform: translateY(-50%) scaleY(1); }
  .label { font-size: 14px; }

  .foot { display: flex; flex-direction: column; gap: 10px; padding-top: 14px; border-top: 1px solid var(--stroke); }
  .account {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 9px 11px;
    border: 1px solid var(--stroke);
    background: var(--panel);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    color: var(--text);
    transition: background var(--dur) var(--ease), border-color var(--dur) var(--ease), transform var(--dur) var(--ease);
  }
  .account:hover { background: var(--panel-strong); border-color: var(--stroke-strong); transform: translateY(-1px); }
  .avatar {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    color: var(--muted);
    background: var(--panel-strong);
  }
  .acc-text { display: flex; flex-direction: column; line-height: 1.2; }
  .acc-text b { font-size: 13px; }
  .acc-text small { font-size: 11px; color: var(--muted); }
  .version { font-size: 11px; color: var(--muted); text-align: center; letter-spacing: 0.5px; }
</style>
