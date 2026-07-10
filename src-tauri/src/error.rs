//! Tipo de error unificado. Todo error del backend se serializa a la UI
//! como { code, message } estable y legible.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AetherError {
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("error al serializar TOML: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("error al leer TOML: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("error JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("error HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("recurso no encontrado: {0}")]
    NotFound(String),

    #[error("estado invalido: {0}")]
    InvalidState(String),

    #[error("no implementado: {0}")]
    NotImplemented(String),

    #[error("hash SHA1 invalido para {path}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

impl Serialize for AetherError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let code = match self {
            AetherError::Io(_) => "IO",
            AetherError::TomlSer(_) | AetherError::TomlDe(_) => "TOML",
            AetherError::Json(_) => "JSON",
            AetherError::Http(_) => "HTTP",
            AetherError::NotFound(_) => "NOT_FOUND",
            AetherError::InvalidState(_) => "INVALID_STATE",
            AetherError::NotImplemented(_) => "NOT_IMPLEMENTED",
            AetherError::HashMismatch { .. } => "HASH_MISMATCH",
        };

        #[derive(Serialize)]
        struct Payload<'a> {
            code: &'a str,
            message: String,
        }

        Payload {
            code,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type Result<T> = std::result::Result<T, AetherError>;