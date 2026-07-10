//! Aether Launcher - nucleo de la aplicacion de escritorio (Tauri 2).

mod download;
mod error;
mod install;
mod instance;
mod launch;
mod minecraft;
mod runtime;

use std::sync::Mutex;

/// Estado global. Indice en memoria + cliente HTTP reutilizado (pool keep-alive).
pub struct AppState {
    pub index: Mutex<Vec<instance::InstanceSummary>>,
    pub http: reqwest::Client,
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let index = instance::store::load_index().unwrap_or_default();
    let http = reqwest::Client::builder()
        .user_agent(concat!("AetherLauncher/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("no se pudo construir el cliente HTTP");

    tauri::Builder::default()
        .manage(AppState { index: Mutex::new(index), http })
        .manage(install::commands::InstallRegistry::default())
        .invoke_handler(tauri::generate_handler![
            app_version,
            instance::commands::list_instances,
            instance::commands::get_instance,
            instance::commands::create_instance,
            instance::commands::update_instance,
            instance::commands::delete_instance,
            instance::commands::duplicate_instance,
            instance::commands::import_instance,
            install::commands::install_instance,
            install::commands::cancel_install,
            install::commands::list_mc_versions,
            launch::commands::launch_instance,
            runtime::commands::resolve_runtime,
            runtime::commands::required_java_for_version
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar Aether Launcher");
}
