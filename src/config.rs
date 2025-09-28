use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::error::{Result, TwiggyError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub theme: ThemeConfig,
    pub git: GitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub dark_mode: bool,
    pub font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub default_branch: String,
    pub show_merge_commits: bool,
    pub max_commits: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 1200.0,
                height: 800.0,
                maximized: false,
            },
            theme: ThemeConfig {
                dark_mode: true,
                font_size: 14.0,
            },
            git: GitConfig {
                default_branch: "main".to_string(),
                show_merge_commits: true,
                max_commits: 1000,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("com", "twiggy", "twiggy")
            .ok_or_else(|| TwiggyError::Config {
                message: "Could not determine config directory".to_string(),
            })?;
        
        Ok(project_dirs.config_dir().join("config.json"))
    }
}