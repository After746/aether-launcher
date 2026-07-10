<script lang="ts">
  import Titlebar from './Titlebar.svelte';
  import Sidebar from './Sidebar.svelte';
  import { router } from '../stores/router.svelte';
  import HomeView from '../views/HomeView.svelte';
  import InstancesView from '../views/InstancesView.svelte';
  import SettingsView from '../views/SettingsView.svelte';
  import { fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  const views = {
    home: HomeView,
    instances: InstancesView,
    settings: SettingsView,
  };

  const Current = $derived(views[router.current]);
</script>

<div class="shell">
  <Titlebar />
  <div class="body">
    <Sidebar />
    <main class="content">
      {#key router.current}
        <div class="view" in:fade={{ duration: 220, easing: cubicOut }}>
          <Current />
        </div>
      {/key}
    </main>
  </div>
</div>

<style>
  .shell { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
  .body { flex: 1; display: flex; min-height: 0; }
  .content { flex: 1; overflow-y: auto; padding: 32px 40px; }
  .view { height: 100%; }
</style>