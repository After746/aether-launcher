//! Ejecucion real del proceso de Minecraft a partir de un LaunchPlan.
//!
//! Construye el Command con el Java resuelto, los argumentos JVM, la mainClass
//! y los argumentos de juego, usando la carpeta de la instancia como
//! current_dir. stdout/stderr quedan redirigidos por piped para poder
//! capturarlos en fases posteriores.
//!
//! No toca frontend, comandos Tauri ni autenticacion.

use super::{prepare_launch, LaunchPlan};
use crate::error::{AetherError, Result};
use crate::instance::Instance;
use std::process::{Child, Command, Stdio};

/// Proceso de Minecraft ya lanzado. Envuelve el Child para que las fases
/// siguientes puedan leer stdout/stderr o esperar/matar el proceso.
pub struct LaunchedProcess {
    pub child: Child,
    pub pid: u32,
}

/// Ejecuta un LaunchPlan ya construido: arranca la JVM sin bloquear.
///
/// - command[0] es el binario de Java; el resto es jvm + mainClass + game.
/// - current_dir = carpeta de la instancia (game_dir).
/// - stdout/stderr = piped (capturables desde el Child).
pub fn execute_launch_plan(plan: &LaunchPlan) -> Result<LaunchedProcess> {
    // Salvaguarda: el argv siempre debe traer al menos el binario de Java.
    let (program, args) = plan
        .command
        .split_first()
        .ok_or_else(|| AetherError::InvalidState("LaunchPlan sin comando".into()))?;

    let child = Command::new(program)
        .args(args)
        .current_dir(&plan.game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => AetherError::NotFound(format!(
                "no se encontro el ejecutable de Java en '{}'",
                plan.java.display()
            )),
            _ => AetherError::InvalidState(format!("no se pudo iniciar Minecraft: {e}")),
        })?;

    let pid = child.id();
    Ok(LaunchedProcess { child, pid })
}

/// Camino completo: prepara el LaunchPlan de la instancia y lo ejecuta.
/// Envoltorio conveniente para la fase de UI (aun sin exponer via Tauri).
pub fn launch_instance(instance: &Instance) -> Result<LaunchedProcess> {
    let plan = prepare_launch(instance)?;
    execute_launch_plan(&plan)
}