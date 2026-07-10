//! JSON detallado de una version: librerias (con reglas por SO), assets,
//! client jar, natives y hint de Java recomendado.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDetail {
    pub id: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    pub assets: String,
    pub downloads: ClientDownloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    // --- agregado: preparacion de lanzamiento ---
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Formato moderno (>= 1.13). Ausente en versiones antiguas.
    #[serde(default)]
    pub arguments: Option<Arguments>,
    /// Formato legacy (< 1.13): string plano de argumentos de juego.
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub total_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientDownloads {
    pub client: Artifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

/// Nombre del SO segun la convencion de Mojang.
pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

impl Library {
    /// Evalua las reglas allow/disallow para el SO actual.
    pub fn allowed(&self) -> bool {
        if self.rules.is_empty() {
            return true;
        }
        let mut allow = false;
        for rule in &self.rules {
            let matches = match &rule.os {
                Some(os) => os.name.as_deref().map_or(true, |n| n == current_os()),
                None => true,
            };
            if matches {
                allow = rule.action == "allow";
            }
        }
        allow
    }

    /// Clave de native para el SO actual (p.ej. "natives-windows"), si aplica.
    pub fn native_classifier(&self) -> Option<String> {
        self.natives.as_ref().and_then(|m| m.get(current_os()).cloned())
    }
}

/// Objeto individual del asset index.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: std::collections::HashMap<String, AssetObject>,
}

/// Resumen serializable de una version para la UI (selector).
#[derive(Debug, Clone, Serialize)]
pub struct VersionOption {
    pub id: String,
    pub kind: String,
    pub release_time: String,
}

/// Bloque `arguments` del version.json moderno.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub jvm: Vec<ArgElement>,
    #[serde(default)]
    pub game: Vec<ArgElement>,
}

/// Un argumento: string simple o condicional (reglas + valor).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgElement {
    Simple(String),
    Conditional { rules: Vec<Rule>, value: ArgValue },
}

/// El `value` de un argumento condicional puede ser uno o varios strings.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    Single(String),
    Many(Vec<String>),
}

/// Evalua allow/disallow para el entorno actual (SO + arch + features).
/// Usada tanto por librerias como por argumentos.
pub fn rules_allow(rules: &[Rule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allow = false;
    for rule in rules {
        if rule_matches(rule) {
            allow = rule.action == "allow";
        }
    }
    allow
}

fn rule_matches(rule: &Rule) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != current_os() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch != std::env::consts::ARCH {
                return false;
            }
        }
    }
    // No habilitamos ningun feature (demo, custom_resolution, quick_play...).
    // Un feature esperado en `true` nunca aplica todavia.
    if let Some(features) = &rule.features {
        for expected in features.values() {
            if *expected {
                return false;
            }
        }
    }
    true
}
