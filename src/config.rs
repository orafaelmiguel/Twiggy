use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::error::{Result, TwiggyError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeConfig,
    pub git: GitConfig,
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub dark_mode: bool,
    pub accent_color: String,
    pub font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub default_branch: String,
    pub auto_fetch: bool,
    pub fetch_interval_minutes: u32,
    pub max_commits_display: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_commit_graph: bool,
    pub show_file_tree: bool,
    pub show_diff_viewer: bool,
    pub panel_sizes: PanelSizes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSizes {
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_caching: bool,
    pub cache_size_mb: usize,
    pub background_operations: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                dark_mode: true,
                accent_color: "#4A90E2".to_string(),
                font_size: 14.0,
            },
            git: GitConfig {
                default_branch: "main".to_string(),
                auto_fetch: false,
                fetch_interval_minutes: 15,
                max_commits_display: 1000,
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
                cache_size_mb: 100,
                background_operations: true,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            tracing::info!("Config file not found, creating default configuration");
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_content = std::fs::read_to_string(&config_path).map_err(|e| {
            TwiggyError::FileSystem {
                path: config_path.display().to_string(),
                source: e,
            }
        })?;

        let config: Self = serde_json::from_str(&config_content).map_err(|e| {
            tracing::warn!("Failed to parse config file, using defaults: {}", e);
            TwiggyError::Config {
                message: format!("Invalid configuration format: {}", e),
            }
        })?;

        config.validate()?;
        
        tracing::info!("Configuration loaded successfully from {}", config_path.display());
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("Failed to load configuration: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TwiggyError::FileSystem {
                    path: parent.display().to_string(),
                    source: e,
                }
            })?;
        }

        let config_json = serde_json::to_string_pretty(self).map_err(|e| {
            TwiggyError::Serialization {
                operation: "config serialization".to_string(),
                source: e,
            }
        })?;

        std::fs::write(&config_path, config_json).map_err(|e| {
            TwiggyError::FileSystem {
                path: config_path.display().to_string(),
                source: e,
            }
        })?;

        tracing::info!("Configuration saved to {}", config_path.display());
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.theme.font_size < 8.0 || self.theme.font_size > 32.0 {
            return Err(TwiggyError::Validation {
                field: "theme.font_size".to_string(),
                message: "Font size must be between 8.0 and 32.0".to_string(),
            });
        }

        if self.git.fetch_interval_minutes == 0 {
            return Err(TwiggyError::Validation {
                field: "git.fetch_interval_minutes".to_string(),
                message: "Fetch interval must be greater than 0".to_string(),
            });
        }

        if self.git.max_commits_display == 0 || self.git.max_commits_display > 10000 {
            return Err(TwiggyError::Validation {
                field: "git.max_commits_display".to_string(),
                message: "Max commits display must be between 1 and 10000".to_string(),
            });
        }

        if self.performance.cache_size_mb == 0 || self.performance.cache_size_mb > 1024 {
            return Err(TwiggyError::Validation {
                field: "performance.cache_size_mb".to_string(),
                message: "Cache size must be between 1 and 1024 MB".to_string(),
            });
        }

        if self.ui.panel_sizes.left_panel_width < 100.0 || self.ui.panel_sizes.left_panel_width > 500.0 {
            return Err(TwiggyError::Validation {
                field: "ui.panel_sizes.left_panel_width".to_string(),
                message: "Left panel width must be between 100.0 and 500.0".to_string(),
            });
        }

        Ok(())
    }

    pub fn reset_to_defaults(&mut self) -> Result<()> {
        *self = Self::default();
        self.save()?;
        tracing::info!("Configuration reset to defaults");
        Ok(())
    }

    pub fn migrate_if_needed(&mut self) -> Result<bool> {
        let mut migrated = false;

        if self.git.default_branch == "master" {
            self.git.default_branch = "main".to_string();
            migrated = true;
            tracing::info!("Migrated default branch from 'master' to 'main'");
        }

        if migrated {
            self.save()?;
            tracing::info!("Configuration migration completed");
        }

        Ok(migrated)
    }

    fn config_file_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("com", "twiggy", "twiggy")
            .ok_or_else(|| TwiggyError::Config {
                message: "Unable to determine configuration directory".to_string(),
            })?;

        Ok(project_dirs.config_dir().join("config.json"))
    }
}