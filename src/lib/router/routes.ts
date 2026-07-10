import type { RouteId } from '../stores/router.svelte';

export type NavItem = { id: RouteId; label: string; icon: string };

export const NAV: NavItem[] = [
  { id: 'home', label: 'Inicio', icon: 'home' },
  { id: 'instances', label: 'Instancias', icon: 'grid' },
  { id: 'settings', label: 'Ajustes', icon: 'settings' },
];