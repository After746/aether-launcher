import { invoke } from '@tauri-apps/api/core';

class LaunchStore {
  launchingId = $state<string | null>(null);
  lastPid = $state<number | null>(null);
  error = $state<string | null>(null);

  isLaunching(id: string) {
    return this.launchingId === id;
  }

  async play(id: string): Promise<number | null> {
    console.log("INTENTANDO LANZAR INSTANCIA:", id);

    this.error = null;
    this.launchingId = id;

    try {
      const pid = await invoke<number>('launch_instance', { id });

      console.log("MINECRAFT LANZADO PID:", pid);

      this.lastPid = pid;
      return pid;

    } catch (err: any) {
      console.error("ERROR RUST LAUNCH:", err);
      this.error = String(err?.message ?? err);
      return null;

    } finally {
      this.launchingId = null;
    }
  }
}

export const launch = new LaunchStore();