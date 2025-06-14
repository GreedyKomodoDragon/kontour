use std::fmt;

/// Custom error types for the application
#[derive(Debug, Clone)]
pub enum KubeconfigError {
    /// Error when kubeconfig file is not found
    NotFound(String),
    /// Error when a referenced file path doesn't exist
    FileNotFound(String),
    /// Error when kubeconfig content is invalid
    InvalidContent(String),
    /// Error when storing kubeconfig fails
    StorageError(String),
    /// Error when creating Kubernetes client fails
    ClientCreationError(String),
    /// IO related errors
    IoError(String),
}

impl fmt::Display for KubeconfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KubeconfigError::NotFound(path) => write!(f, "Kubeconfig not found: {}", path),
            KubeconfigError::FileNotFound(path) => write!(f, "File not found: {}", path),
            KubeconfigError::InvalidContent(msg) => write!(f, "Invalid kubeconfig content: {}", msg),
            KubeconfigError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            KubeconfigError::ClientCreationError(msg) => write!(f, "Client creation failed: {}", msg),
            KubeconfigError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for KubeconfigError {}

impl From<std::io::Error> for KubeconfigError {
    fn from(err: std::io::Error) -> Self {
        KubeconfigError::IoError(err.to_string())
    }
}

impl From<kube::Error> for KubeconfigError {
    fn from(err: kube::Error) -> Self {
        KubeconfigError::ClientCreationError(err.to_string())
    }
}

/// Result type for kubeconfig operations
pub type KubeconfigResult<T> = Result<T, KubeconfigError>;
