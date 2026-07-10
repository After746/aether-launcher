<script lang="ts">
  import ProgressBar from './ui/ProgressBar.svelte';
  import Button from './ui/Button.svelte';
  import { install } from '../stores/install.svelte';
  import { PHASE_LABEL } from '../ipc/types';
  import { fade } from 'svelte/transition';

  let { id }: { id: string } = $props();

  function fmtBytes(n: number): string {
    if (n < 1024) return n + ' B';
    if (n < 1024 * 1024) return (n / 1024).toFixed(0) + ' KB';
    if (n < 1024 * 1024 * 1024) return (n / 1024 / 1024).toFixed(1) + ' MB';
    return (n / 1024 / 1024 / 1024).toFixed(2) + ' GB';
  }
  const speed = $derived(fmtBytes(install.speedBps) + '/s');
  const eta = $derived(() => {
    if (install.speedBps <= 0) return '—';
    const remaining = install.totalBytes - install.bytesDone;
    const secs = Math.max(0, Math.round(remaining / install.speedBps));
    if (secs < 60) return secs + 's';
    return Math.floor(secs / 60) + 'm ' + (secs % 60) + 's';
  });
</script>

<div class="ip" in:fade={{ duration: 200 }}>
  <div class="head">
    <span class="phase">{PHASE_LABEL[install.phase] ?? 'Instalando'}</span>
    <span class="pct">{install.percent.toFixed(0)}%</span>
  </div>

  <ProgressBar value={install.percent} indeterminate={install.totalBytes === 0} />

  <div class="detail">
    <span class="file" title={install.currentFile}>{install.currentFile || '…'}</span>
    <span class="nums">{install.filesDone}/{install.totalFiles}</span>
  </div>

  <div class="foot">
    <div class="meta">
      <span>{fmtBytes(install.bytesDone)} / {fmtBytes(install.totalBytes)}</span>
      <span class="dot">•</span>
      <span>{speed}</span>
      <span class="dot">•</span>
      <span>ETA {eta()}</span>
    </div>
    <Button variant="ghost" onclick={() => install.cancel(id)}>Cancelar</Button>
  </div>
</div>

<style>
  .ip { display: flex; flex-direction: column; gap: 12px; }
  .head { display: flex; align-items: center; justify-content: space-between; }
  .phase { font-weight: 700; font-size: 14px; }
  .pct { font-weight: 800; font-size: 15px; color: var(--accent); }
  .detail { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .file {
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: ui-monospace, monospace;
  }
  .nums { font-size: 12px; color: var(--muted); flex: 0 0 auto; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .meta { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--muted); }
  .dot { opacity: 0.5; }
</style>
