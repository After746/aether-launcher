import { invoke, Channel } from '@tauri-apps/api/core';
import type { InstallEvent } from '../ipc/types';
import { instances } from './instances.svelte';

// Estado de una instalacion en curso. La UI se suscribe a este store.
class InstallStore {
  activeId = $state<string | null>(null);
  phase = $state<string>('');
  filesDone = $state(0);
  totalFiles = $state(0);
  bytesDone = $state(0);
  totalBytes = $state(0);
  currentFile = $state('');
  speedBps = $state(0);
  error = $state<string | null>(null);

  percent = $derived(this.totalBytes > 0 ? (this.bytesDone / this.totalBytes) * 100 : 0);

  isInstalling(id: string) {
    return this.activeId === id;
  }

  reset() {
    this.phase = '';
    this.filesDone = 0;
    this.totalFiles = 0;
    this.bytesDone = 0;
    this.totalBytes = 0;
    this.currentFile = '';
    this.speedBps = 0;
    this.error = null;
  }

  async start(id: string) {
    this.reset();
    this.activeId = id;

    const channel = new Channel<InstallEvent>();
    channel.onmessage = (e) => {
      switch (e.kind) {
        case 'started':
          this.totalFiles = e.total_files;
          this.totalBytes = e.total_bytes;
          break;
        case 'phase':
          this.phase = e.phase;
          break;
        case 'progress':
          this.filesDone = e.files_done;
          this.totalFiles = e.total_files;
          this.bytesDone = e.bytes_done;
          this.totalBytes = e.total_bytes;
          this.currentFile = e.current_file;
          this.speedBps = e.speed_bps;
          break;
        case 'failed':
          this.error = e.message;
          break;
      }
    };

    try {
      await invoke('install_instance', { id, onEvent: channel });
    } catch (err: any) {
      this.error = String(err?.message ?? err);
    } finally {
      this.activeId = null;
      await instances.load();
    }
  }

  async cancel(id: string) {
    await invoke('cancel_install', { id });
  }
}

export const install = new InstallStore();
