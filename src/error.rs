#[derive(thiserror::Error, Debug)]
pub enum TwiggyError {
    #[error("Git error: {message}")]
    Git { 
        message: String, 
        #[source] 
        source: git2::Error 
    },
    
    #[error("IO error during {operation}")]
    Io { 
        operation: String, 
        #[source] 
        source: std::io::Error 
    },
    
    #[error("Configuration error: {message}")]
    Config { 
        message: String 
    },
    
    #[error("UI error: {message}")]
    Ui { 
        message: String 
    },
    
    #[error("Application error: {message}")]
    Application { 
        message: String 
    },
    
    #[error("Serialization error: {operation}")]
    Serialization { 
        operation: String, 
        #[source] 
        source: serde_json::Error 
    },
    
    #[error("File system error: {path}")]
    FileSystem { 
        path: String, 
        #[source] 
        source: std::io::Error 
    },
    
    #[error("Network error: {message}")]
    Network { 
        message: String 
    },
    
    #[error("Validation error: {field} - {message}")]
    Validation { 
        field: String, 
        message: String 
    },
}

impl TwiggyError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Git { .. } => 1000,
            Self::Io { .. } => 2000,
            Self::Config { .. } => 3000,
            Self::Ui { .. } => 4000,
            Self::Application { .. } => 5000,
            Self::Serialization { .. } => 6000,
            Self::FileSystem { .. } => 7000,
            Self::Network { .. } => 8000,
            Self::Validation { .. } => 9000,
        }
    }
    
    pub fn user_message(&self) -> String {
        match self {
            Self::Git { message, .. } => {
                format!("Git operation failed: {}", message)
            },
            Self::Io { operation, .. } => {
                format!("File operation '{}' failed. Please check file permissions.", operation)
            },
            Self::Config { message } => {
                format!("Configuration issue: {}. Using default settings.", message)
            },
            Self::Ui { message } => {
                format!("Interface error: {}", message)
            },
            Self::Application { message } => {
                format!("Application error: {}", message)
            },
            Self::Serialization { operation, .. } => {
                format!("Data processing error during {}. Please try again.", operation)
            },
            Self::FileSystem { path, .. } => {
                format!("Cannot access file: {}. Check if the file exists and you have permission.", path)
            },
            Self::Network { message } => {
                format!("Network error: {}. Check your internet connection.", message)
            },
            Self::Validation { field, message } => {
                format!("Invalid {}: {}", field, message)
            },
        }
    }
    
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Git { .. } => false,
            Self::Io { .. } => true,
            Self::Config { .. } => true,
            Self::Ui { .. } => true,
            Self::Application { .. } => false,
            Self::Serialization { .. } => true,
            Self::FileSystem { .. } => true,
            Self::Network { .. } => true,
            Self::Validation { .. } => true,
        }
    }
    
    pub fn suggested_action(&self) -> Option<String> {
        match self {
            Self::Git { .. } => Some("Check repository status and try again".to_string()),
            Self::Io { .. } => Some("Verify file permissions and disk space".to_string()),
            Self::Config { .. } => Some("Reset to default configuration".to_string()),
            Self::Ui { .. } => Some("Restart the application".to_string()),
            Self::Application { .. } => Some("Report this issue to support".to_string()),
            Self::Serialization { .. } => Some("Check file format and try again".to_string()),
            Self::FileSystem { .. } => Some("Ensure file exists and is accessible".to_string()),
            Self::Network { .. } => Some("Check network connection and retry".to_string()),
            Self::Validation { .. } => Some("Correct the input and try again".to_string()),
        }
    }
}

impl From<git2::Error> for TwiggyError {
    fn from(error: git2::Error) -> Self {
        Self::Git {
            message: error.message().to_string(),
            source: error,
        }
    }
}

impl From<std::io::Error> for TwiggyError {
    fn from(error: std::io::Error) -> Self {
        Self::Io {
            operation: "unknown".to_string(),
            source: error,
        }
    }
}

impl From<serde_json::Error> for TwiggyError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            operation: "JSON processing".to_string(),
            source: error,
        }
    }
}

pub type Result<T> = std::result::Result<T, TwiggyError>;