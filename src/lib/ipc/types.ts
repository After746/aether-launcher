// Tipos espejo del modelo Rust. Sincronizar con src-tauri/src/instance/mod.rs.

export type Loader = 'vanilla' | 'fabric' | 'forge' | 'neoforge';
export type InstallStatus =
  | 'created' | 'installing' | 'cancelled' | 'ready' | 'error' | 'corrupt';

export interface InstanceSummary {
  id: string;
  name: string;
  mc_version: string;
  loader: Loader;
  ram_mb: number;
  icon: string | null;
  last_played: number | null;
  playtime_secs: number;
  mod_count: number;
  favorite: boolean;
  status: InstallStatus;
  total_size_bytes: number;
}

export interface Instance extends InstanceSummary {
  loader_version: string | null;
  path: string;
  java_path: string | null;
  java_major: number | null;
  created_at: number;
  installed_at: number | null;
  last_error: string | null;
}

export interface CreateInput {
  name: string;
  mc_version: string;
  loader: Loader;
  loader_version?: string | null;
  ram_mb?: number | null;
  icon?: string | null;
  java_path?: string | null;
}

export interface VersionOption {
  id: string;
  kind: string;
  release_time: string;
}

export type InstallEvent =
  | { kind: 'started'; total_files: number; total_bytes: number }
  | { kind: 'phase'; phase: string }
  | {
      kind: 'progress';
      files_done: number;
      total_files: number;
      bytes_done: number;
      total_bytes: number;
      current_file: string;
      speed_bps: number;
    }
  | { kind: 'done' }
  | { kind: 'cancelled' }
  | { kind: 'failed'; message: string };

export const LOADER_LABEL: Record<Loader, string> = {
  vanilla: 'Vanilla',
  fabric: 'Fabric',
  forge: 'Forge',
  neoforge: 'NeoForge',
};

export const STATUS_LABEL: Record<InstallStatus, string> = {
  created: 'Sin instalar',
  installing: 'Instalando',
  cancelled: 'Cancelada',
  ready: 'Lista',
  error: 'Error',
  corrupt: 'Dañada',
};

export const PHASE_LABEL: Record<string, string> = {
  starting: 'Preparando',
  manifest: 'Obteniendo versiones',
  version_json: 'Leyendo versión',
  libraries: 'Descargando librerías',
  client: 'Descargando cliente',
  assets: 'Descargando recursos',
  natives: 'Extrayendo nativos',
  done: 'Completado',
};
