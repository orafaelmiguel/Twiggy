use std::path::PathBuf;
use tracing_subscriber::{
    fmt::{self, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use tracing_appender::{non_blocking, rolling};
use directories::ProjectDirs;
use crate::error::{Result, TwiggyError};

pub struct LoggingConfig {
    pub log_level: String,
    pub log_to_file: bool,
    pub log_to_console: bool,
    pub log_directory: PathBuf,
    pub max_log_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let log_directory = ProjectDirs::from("com", "twiggy", "twiggy")
            .map(|dirs| dirs.data_dir().join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"));

        Self {
            log_level: "info".to_string(),
            log_to_file: true,
            log_to_console: true,
            log_directory,
            max_log_files: 10,
        }
    }
}

pub fn initialize_logging(config: LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .map_err(|e| TwiggyError::Config {
            message: format!("Invalid log level '{}': {}", config.log_level, e),
        })?;

    let mut layers = Vec::new();

    if config.log_to_console {
        let console_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(true)
            .with_line_number(true)
            .with_timer(LocalTime::rfc_3339())
            .with_filter(env_filter.clone());
        
        layers.push(console_layer.boxed());
    }

    if config.log_to_file {
        std::fs::create_dir_all(&config.log_directory).map_err(|e| {
            TwiggyError::FileSystem {
                path: config.log_directory.display().to_string(),
                source: e,
            }
        })?;

        let file_appender = rolling::daily(&config.log_directory, "twiggy.log");
        let (non_blocking_appender, _guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(non_blocking_appender)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(LocalTime::rfc_3339())
            .json()
            .with_filter(env_filter.clone());

        layers.push(file_layer.boxed());

        cleanup_old_logs(&config.log_directory, config.max_log_files)?;
    }

    tracing_subscriber::registry()
        .with(layers)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_level = %config.log_level,
        log_to_file = config.log_to_file,
        log_to_console = config.log_to_console,
        log_directory = %config.log_directory.display(),
        "Twiggy logging initialized"
    );

    Ok(())
}

fn cleanup_old_logs(log_dir: &PathBuf, max_files: usize) -> Result<()> {
    let entries = std::fs::read_dir(log_dir).map_err(|e| TwiggyError::FileSystem {
        path: log_dir.display().to_string(),
        source: e,
    })?;

    let mut log_files: Vec<_> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "log" {
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                Some((path, modified))
            } else {
                None
            }
        })
        .collect();

    if log_files.len() <= max_files {
        return Ok(());
    }

    log_files.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, _) in log_files.iter().skip(max_files) {
        if let Err(e) = std::fs::remove_file(path) {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "Failed to remove old log file"
            );
        } else {
            tracing::debug!(
                path = %path.display(),
                "Removed old log file"
            );
        }
    }

    Ok(())
}

pub fn log_performance<T, F>(operation: &str, func: F) -> T
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = func();
    let duration = start.elapsed();
    
    tracing::debug!(
        operation = operation,
        duration_ms = duration.as_millis(),
        "Performance measurement"
    );
    
    result
}

#[macro_export]
macro_rules! log_error {
    ($error:expr) => {
        tracing::error!(
            error = %$error,
            error_code = $error.error_code(),
            recoverable = $error.is_recoverable(),
            "Application error occurred"
        );
    };
}

#[macro_export]
macro_rules! log_error_with_context {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            error_code = $error.error_code(),
            recoverable = $error.is_recoverable(),
            context = $context,
            "Application error occurred"
        );
    };
}