#[cfg(test)]
mod config_tests {
    use super::*;
    use twiggy::config::{AppConfig, WindowConfig, ThemeConfig, ThemeType, GitConfig, UiConfig, PerformanceConfig, PanelSizes};
    use tempfile::TempDir;
    use std::path::PathBuf;

    fn create_test_config() -> AppConfig {
        AppConfig {
            window: WindowConfig {
                width: 1200.0,
                height: 800.0,
                maximized: false,
                position_x: Some(100.0),
                position_y: Some(50.0),
                remember_position: true,
            },
            theme: ThemeConfig {
                theme_type: ThemeType::Dark,
                font_size: 14.0,
                dark_mode: true,
                accent_color: "#007ACC".to_string(),
            },
            git: GitConfig {
                default_clone_path: Some(PathBuf::from("/tmp/repos")),
                max_commits: 1000,
                default_branch: "main".to_string(),
                auto_fetch: true,
                fetch_interval_minutes: 15,
            },
            ui: UiConfig {
                show_commit_graph: true,
                show_file_tree: true,
                show_diff_viewer: true,
                panel_sizes: PanelSizes {
                    left_panel_width: 250.0,
                    right_panel_width: 300.0,
                    bottom_panel_height: 200.0,
                },
            },
            performance: PerformanceConfig {
                enable_caching: true,
                cache_size_mb: 512,
                background_operations: true,
            },
            version: 1,
        }
    }

    #[test]
    fn test_config_default_values() {
        let config = AppConfig::default();
        
        assert_eq!(config.window.width, 1024.0);
        assert_eq!(config.window.height, 768.0);
        assert!(!config.window.maximized);
        assert!(config.window.position_x.is_none());
        assert!(config.window.position_y.is_none());
        assert!(config.window.remember_position);
        
        assert_eq!(config.theme.theme_type, ThemeType::System);
        assert_eq!(config.theme.font_size, 14.0);
        assert!(!config.theme.dark_mode);
        assert_eq!(config.theme.accent_color, "#007ACC");
        
        assert_eq!(config.git.max_commits, 1000);
        assert_eq!(config.git.default_branch, "main");
        assert!(config.git.auto_fetch);
        assert_eq!(config.git.fetch_interval_minutes, 30);
        
        assert!(config.ui.show_commit_graph);
        assert!(config.ui.show_file_tree);
        assert!(config.ui.show_diff_viewer);
        
        assert!(config.performance.enable_caching);
        assert_eq!(config.performance.cache_size_mb, 256);
        assert!(config.performance.background_operations);
        
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        
        assert!(json.contains("\"width\": 1200.0"));
        assert!(json.contains("\"height\": 800.0"));
        assert!(json.contains("\"theme_type\": \"Dark\""));
        assert!(json.contains("\"font_size\": 14.0"));
        assert!(json.contains("\"max_commits\": 1000"));
        assert!(json.contains("\"default_branch\": \"main\""));
        assert!(json.contains("\"cache_size_mb\": 512"));
        assert!(json.contains("\"version\": 1"));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "window": {
                "width": 1400.0,
                "height": 900.0,
                "maximized": true,
                "position_x": null,
                "position_y": null,
                "remember_position": false
            },
            "theme": {
                "theme_type": "Light",
                "font_size": 16.0,
                "dark_mode": false,
                "accent_color": "#FF5722"
            },
            "git": {
                "default_clone_path": null,
                "max_commits": 2000,
                "default_branch": "develop",
                "auto_fetch": false,
                "fetch_interval_minutes": 60
            },
            "ui": {
                "show_commit_graph": false,
                "show_file_tree": false,
                "show_diff_viewer": true,
                "panel_sizes": {
                    "left_panel_width": 200.0,
                    "right_panel_width": 350.0,
                    "bottom_panel_height": 150.0
                }
            },
            "performance": {
                "enable_caching": false,
                "cache_size_mb": 128,
                "background_operations": false
            },
            "version": 1
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.window.width, 1400.0);
        assert_eq!(config.window.height, 900.0);
        assert!(config.window.maximized);
        assert!(config.window.position_x.is_none());
        assert!(!config.window.remember_position);
        
        assert_eq!(config.theme.theme_type, ThemeType::Light);
        assert_eq!(config.theme.font_size, 16.0);
        assert!(!config.theme.dark_mode);
        assert_eq!(config.theme.accent_color, "#FF5722");
        
        assert!(config.git.default_clone_path.is_none());
        assert_eq!(config.git.max_commits, 2000);
        assert_eq!(config.git.default_branch, "develop");
        assert!(!config.git.auto_fetch);
        assert_eq!(config.git.fetch_interval_minutes, 60);
        
        assert!(!config.ui.show_commit_graph);
        assert!(!config.ui.show_file_tree);
        assert!(config.ui.show_diff_viewer);
        
        assert!(!config.performance.enable_caching);
        assert_eq!(config.performance.cache_size_mb, 128);
        assert!(!config.performance.background_operations);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_window_size() {
        let mut config = create_test_config();
        config.window.width = 50.0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Window width"));
    }

    #[test]
    fn test_config_validation_invalid_font_size() {
        let mut config = create_test_config();
        config.theme.font_size = 5.0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Font size"));
    }

    #[test]
    fn test_config_validation_invalid_cache_size() {
        let mut config = create_test_config();
        config.performance.cache_size_mb = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cache size"));
    }

    #[test]
    fn test_config_validation_invalid_git_settings() {
        let mut config = create_test_config();
        config.git.max_commits = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max commits"));
    }

    #[test]
    fn test_config_reset_to_defaults() {
        let mut config = create_test_config();
        
        config.window.width = 2000.0;
        config.theme.font_size = 20.0;
        config.git.max_commits = 5000;
        
        assert!(config.reset_to_defaults().is_ok());
        
        let default_config = AppConfig::default();
        assert_eq!(config.window.width, default_config.window.width);
        assert_eq!(config.theme.font_size, default_config.theme.font_size);
        assert_eq!(config.git.max_commits, default_config.git.max_commits);
    }

    #[test]
    fn test_config_migration_needed() {
        let mut config = create_test_config();
        config.version = 0;
        
        let migration_result = config.migrate_if_needed();
        assert!(migration_result.is_ok());
        assert!(migration_result.unwrap());
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_config_migration_not_needed() {
        let mut config = create_test_config();
        config.version = 1;
        
        let migration_result = config.migrate_if_needed();
        assert!(migration_result.is_ok());
        assert!(!migration_result.unwrap());
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_theme_type_equality() {
        assert_eq!(ThemeType::Light, ThemeType::Light);
        assert_eq!(ThemeType::Dark, ThemeType::Dark);
        assert_eq!(ThemeType::System, ThemeType::System);
        
        assert_ne!(ThemeType::Light, ThemeType::Dark);
        assert_ne!(ThemeType::Dark, ThemeType::System);
        assert_ne!(ThemeType::Light, ThemeType::System);
    }

    #[test]
    fn test_config_load_or_default() {
        let config = AppConfig::load_or_default();
        
        assert_eq!(config.window.width, 1024.0);
        assert_eq!(config.theme.theme_type, ThemeType::System);
        assert_eq!(config.git.default_branch, "main");
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_panel_sizes_defaults() {
        let panel_sizes = PanelSizes {
            left_panel_width: 250.0,
            right_panel_width: 300.0,
            bottom_panel_height: 200.0,
        };
        
        assert_eq!(panel_sizes.left_panel_width, 250.0);
        assert_eq!(panel_sizes.right_panel_width, 300.0);
        assert_eq!(panel_sizes.bottom_panel_height, 200.0);
    }
}