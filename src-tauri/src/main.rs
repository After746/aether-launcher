// Aether Launcher â€” punto de entrada del binario de escritorio.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    aether_launcher_lib::run();
}