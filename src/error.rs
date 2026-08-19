use thiserror::Error;

#[derive(Debug, Error)]
pub enum K3dError {
    #[error("failed to load model {path}: {reason}")]
    Model { path: String, reason: String },
    #[error("invalid model: {0}")]
    InvalidModel(String),
}
