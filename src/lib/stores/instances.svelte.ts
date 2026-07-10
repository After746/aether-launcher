import { invoke } from '@tauri-apps/api/core';
import type { Instance, InstanceSummary, CreateInput } from '../ipc/types';

// Rust es la fuente de verdad. Este store cachea el indice en memoria.
class InstancesStore {
  items = $state<InstanceSummary[]>([]);
  selectedId = $state<string | null>(null);
  loading = $state(false);
  loaded = $state(false);

  selected = $derived(this.items.find((i) => i.id === this.selectedId) ?? null);

  async load() {
    this.loading = true;
    try {
      this.items = await invoke<InstanceSummary[]>('list_instances');
      if ((!this.selectedId || !this.items.some((i) => i.id === this.selectedId)) && this.items.length) {
        const fav = this.items.find((i) => i.favorite);
        const recent = [...this.items].sort((a, b) => (b.last_played ?? 0) - (a.last_played ?? 0))[0];
        this.selectedId = (fav ?? recent ?? this.items[0]).id;
      }
    } finally {
      this.loading = false;
      this.loaded = true;
    }
  }

  select(id: string) { this.selectedId = id; }
  get(id: string): Promise<Instance> { return invoke<Instance>('get_instance', { id }); }

  async create(input: CreateInput): Promise<Instance> {
    const created = await invoke<Instance>('create_instance', { input });
    await this.load();
    this.selectedId = created.id;
    return created;
  }
  async update(id: string, patch: Record<string, unknown>): Promise<Instance> {
    const updated = await invoke<Instance>('update_instance', { id, patch });
    await this.load();
    return updated;
  }
  async remove(id: string): Promise<void> {
    await invoke('delete_instance', { id });
    if (this.selectedId === id) this.selectedId = null;
    await this.load();
  }
  async duplicate(id: string): Promise<Instance> {
    const copy = await invoke<Instance>('duplicate_instance', { id });
    await this.load();
    return copy;
  }
  toggleFavorite(id: string, value: boolean) { return this.update(id, { favorite: value }); }
}

export const instances = new InstancesStore();
