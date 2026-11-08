use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tunnel error: {0}")]
    Tunnel(String),
    #[error("Wisp error: {0}")]
    Wisp(String),
    #[error("Proxy bind error: {0}")]
    ProxyBind(String),
    #[error("Hardware fingerprint error: {0}")]
    Hardware(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
