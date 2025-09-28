#[cfg(test)]
mod error_handling_tests {
    use twiggy::error::{TwiggyError, Result};
    use twiggy::config::AppConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_error_codes() {
        let git_error = TwiggyError::Git { 
            message: "Repository not found".to_string(),
            source: git2::Error::from_str("test error")
        };
        assert_eq!(git_error.error_code(), 1000);

        let io_error = TwiggyError::Io { 
            operation: "file read".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")
        };
        assert_eq!(io_error.error_code(), 2000);

        let config_error = TwiggyError::Config { 
            message: "Invalid configuration".to_string()
        };
        assert_eq!(config_error.error_code(), 3000);
    }

    #[test]
    fn test_error_user_messages() {
        let git_error = TwiggyError::Git { 
            message: "Repository not found".to_string(),
            source: git2::Error::from_str("test error")
        };
        assert!(git_error.user_message().contains("Git operation failed"));

        let config_error = TwiggyError::Config { 
            message: "Invalid theme setting".to_string()
        };
        assert!(config_error.user_message().contains("Configuration issue"));
    }

    #[test]
    fn test_error_recovery() {
        let recoverable_error = TwiggyError::Config { 
            message: "Missing theme setting".to_string()
        };
        assert!(recoverable_error.is_recoverable());

        let non_recoverable_error = TwiggyError::Git { 
            message: "Repository corrupted".to_string(),
            source: git2::Error::from_str("test error")
        };
        assert!(!non_recoverable_error.is_recoverable());
    }

    #[test]
    fn test_error_propagation() {
        fn inner_function() -> Result<String> {
            Err(TwiggyError::Io { 
                operation: "file read".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")
            })
        }

        fn outer_function() -> Result<String> {
            inner_function()?;
            Ok("success".to_string())
        }

        let result = outer_function();
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert_eq!(error.error_code(), 2000);
        }
    }

    #[test]
    fn test_config_error_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        fs::write(&config_path, "invalid json content").unwrap();
        
        let config = AppConfig::load_or_default();
        assert_eq!(config.theme.font_size, 14.0);
    }

    #[test]
    fn test_error_display_formatting() {
        let error = TwiggyError::Git { 
            message: "Failed to fetch".to_string(),
            source: git2::Error::from_str("network error")
        };
        
        let display_string = format!("{}", error);
        assert!(display_string.contains("Git error"));
        assert!(display_string.contains("Failed to fetch"));
    }

    #[test]
    fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let invalid_config = r#"{"ui": {"font_size": -5.0}}"#;
        fs::write(&config_path, invalid_config).unwrap();
        
        let config = AppConfig::load_or_default();
        assert_eq!(config.theme.font_size, 14.0);
    }

    #[test]
    fn test_error_context_preservation() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let twiggy_error = TwiggyError::from(io_error);
        
        assert_eq!(twiggy_error.error_code(), 2000);
        assert!(twiggy_error.user_message().contains("File operation"));
    }
}