#[derive(thiserror::Error, Debug)]
pub enum TwiggyError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[allow(dead_code)]
    #[error("UI error: {message}")]
    Ui { message: String },
}

pub type Result<T> = std::result::Result<T, TwiggyError>;