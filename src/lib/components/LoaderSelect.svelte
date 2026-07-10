<script lang="ts">
  import Dropdown from './ui/Dropdown.svelte';
  import type { Loader } from '../ipc/types';

  let { value = $bindable() }: { value: Loader } = $props();

  const options = [
    { value: 'vanilla', label: 'Vanilla', hint: 'Sin mods' },
    { value: 'fabric', label: 'Fabric', hint: 'Próximamente' },
    { value: 'forge', label: 'Forge', hint: 'Próximamente' },
    { value: 'neoforge', label: 'NeoForge', hint: 'Próximamente' },
  ];
</script>

<Dropdown bind:value {options} />
