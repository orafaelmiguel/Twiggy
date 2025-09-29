use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use crate::error::{Result, TwiggyError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub theme: ThemeConfig,
    pub git: GitConfig,
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
    pub recent_repositories: RecentRepositories,
    #[serde(default = "default_version")]
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub remember_position: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub theme_type: ThemeType,
    pub font_size: f32,
    pub dark_mode: bool,
    pub accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeType {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub default_clone_path: String,
    pub max_commits: usize,
    pub default_branch: String,
    pub auto_fetch: bool,
    pub show_stashes: bool,
    pub fetch_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_commit_graph: bool,
    pub show_file_tree: bool,
    pub show_diff_viewer: bool,
    pub panel_sizes: PanelSizes,
    pub menu_preferences: MenuPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuPreferences {
    pub show_menu_bar: bool,
    pub show_keyboard_shortcuts: bool,
    pub compact_menus: bool,
    pub show_icons: bool,
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
    pub enable_background_operations: bool,
    pub max_background_threads: usize,
    pub target_fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file_enabled: bool,
    pub console_enabled: bool,
    pub max_file_size: u64,
    pub max_files: usize,
    pub log_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_enabled: true,
            console_enabled: cfg!(debug_assertions),
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
            log_directory: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRepositories {
    pub repositories: Vec<RecentRepository>,
    pub max_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRepository {
    pub path: PathBuf,
    pub name: String,
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

impl Default for RecentRepositories {
    fn default() -> Self {
        Self {
            repositories: Vec::new(),
            max_count: 10,
        }
    }
}

impl RecentRepositories {
    pub fn add_repository(&mut self, path: PathBuf, name: String) {
        self.repositories.retain(|r| r.path != path);
        
        self.repositories.insert(0, RecentRepository {
            path,
            name,
            last_opened: chrono::Utc::now(),
        });
        
        if self.repositories.len() > self.max_count {
            self.repositories.truncate(self.max_count);
        }
    }
    
    pub fn remove_repository(&mut self, path: &Path) {
        self.repositories.retain(|r| r.path != path);
    }
    
    pub fn clear(&mut self) {
        self.repositories.clear();
    }
    
    pub fn validate_and_clean(&mut self) {
        self.repositories.retain(|r| r.path.exists());
    }
}

fn default_version() -> u32 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 1200.0,
                height: 800.0,
                maximized: false,
                position_x: None,
                position_y: None,
                remember_position: true,
            },
            theme: ThemeConfig {
                theme_type: ThemeType::System,
                font_size: 14.0,
                dark_mode: false,
                accent_color: "#007ACC".to_string(),
            },
            git: GitConfig {
                default_clone_path: std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string()),
                max_commits: 1000,
                default_branch: "main".to_string(),
                auto_fetch: true,
                show_stashes: false,
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
                menu_preferences: MenuPreferences {
                    show_menu_bar: true,
                    show_keyboard_shortcuts: true,
                    compact_menus: false,
                    show_icons: false,
                },
            },
            performance: PerformanceConfig {
                enable_caching: true,
                cache_size_mb: 100,
                enable_background_operations: true,
                max_background_threads: 4,
                target_fps: 60,
            },
            logging: LoggingConfig::default(),
            recent_repositories: RecentRepositories::default(),
            version: 1,
        }
    }
}

impl AppConfig {
    pub fn copy_for_temp(&self) -> Self {
        self.clone()
    }

    pub fn apply_from_temp(&mut self, temp_config: &AppConfig) -> Result<()> {
        temp_config.validate()?;
        *self = temp_config.clone();
        Ok(())
    }

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

        let mut config: Self = serde_json::from_str(&config_content).map_err(|e| {
            tracing::warn!("Failed to parse config file, using defaults: {}", e);
            TwiggyError::Config {
                message: format!("Invalid configuration format: {}", e),
            }
        })?;

        config.validate()?;
        
        if config.migrate_if_needed()? {
            tracing::info!("Configuration migrated to newer version");
            config.save()?;
        }
        
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
        
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            TwiggyError::Serialization {
                operation: "config serialization".to_string(),
                source: e,
            }
        })?;
        
        std::fs::write(&config_path, content).map_err(|e| {
            TwiggyError::FileSystem {
                path: config_path.display().to_string(),
                source: e,
            }
        })?;
        
        tracing::info!("Configuration saved to {}", config_path.display());
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.window.width < 400.0 || self.window.width > 4000.0 {
            return Err(TwiggyError::Validation {
                field: "window.width".to_string(),
                message: "Window width must be between 400 and 4000 pixels".to_string(),
            });
        }

        if self.window.height < 300.0 || self.window.height > 3000.0 {
            return Err(TwiggyError::Validation {
                field: "window.height".to_string(),
                message: "Window height must be between 300 and 3000 pixels".to_string(),
            });
        }

        if self.theme.font_size < 8.0 || self.theme.font_size > 32.0 {
            return Err(TwiggyError::Validation {
                field: "theme.font_size".to_string(),
                message: "Font size must be between 8 and 32 points".to_string(),
            });
        }

        if self.git.max_commits == 0 || self.git.max_commits > 10000 {
            return Err(TwiggyError::Validation {
                field: "git.max_commits".to_string(),
                message: "Max commits must be between 1 and 10000".to_string(),
            });
        }

        if self.git.fetch_interval_minutes == 0 || self.git.fetch_interval_minutes > 1440 {
            return Err(TwiggyError::Validation {
                field: "git.fetch_interval_minutes".to_string(),
                message: "Fetch interval must be between 1 and 1440 minutes".to_string(),
            });
        }

        if self.performance.cache_size_mb == 0 || self.performance.cache_size_mb > 2048 {
            return Err(TwiggyError::Validation {
                field: "performance.cache_size_mb".to_string(),
                message: "Cache size must be between 1 and 2048 MB".to_string(),
            });
        }

        if self.performance.max_background_threads == 0 || self.performance.max_background_threads > 16 {
            return Err(TwiggyError::Validation {
                field: "performance.max_background_threads".to_string(),
                message: "Max background threads must be between 1 and 16".to_string(),
            });
        }

        if self.performance.target_fps < 30 || self.performance.target_fps > 144 {
            return Err(TwiggyError::Validation {
                field: "performance.target_fps".to_string(),
                message: "Target FPS must be between 30 and 144".to_string(),
            });
        }

        if self.ui.panel_sizes.left_panel_width < 100.0 || self.ui.panel_sizes.left_panel_width > 800.0 {
            return Err(TwiggyError::Validation {
                field: "ui.panel_sizes.left_panel_width".to_string(),
                message: "Left panel width must be between 100 and 800 pixels".to_string(),
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
        if self.version < 1 {
            self.version = 1;
            return Ok(true);
        }
        Ok(false)
    }

    fn config_file_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("dev", "twiggy", "Twiggy")
            .ok_or_else(|| TwiggyError::Config {
                message: "Cannot determine config directory".to_string(),
            })?;
        
        Ok(project_dirs.config_dir().join("config.json"))
    }
}