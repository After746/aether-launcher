# Aether Launcher

El launcher de Minecraft mas moderno, rapido y elegante.

**Stack:** Tauri 2 (Rust) - Svelte 5 - TypeScript - Vite.

## Requisitos
- Node 18+
- Rust (stable) â€” https://rustup.rs
- Linux: dependencias de WebKitGTK.

## Puesta en marcha
    npm install
    npx tauri dev     # ventana con hot reload
    npx tauri build   # binario de release optimizado

## Estructura
- `src/` â€” frontend Svelte 5 (UI, stores, design system).
- `src-tauri/` â€” backend Rust (ventana, comandos, nucleo).