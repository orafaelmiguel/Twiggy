mod app;
mod config;
mod error;
mod git;
mod ui;
mod logging;

use crate::{app::TwiggyApp, config::AppConfig};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::rolling;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load().unwrap_or_default();
    
    if let Err(e) = setup_logging(&config.logging) {
        eprintln!("Failed to initialize logging: {}", e);
        return Err(e);
    }
    
    tracing::info!("Starting Twiggy v{}", env!("CARGO_PKG_VERSION"));
    
    let app = match TwiggyApp::new() {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("Failed to initialize application: {}", e);
            return Err(Box::new(e));
        }
    };
    
    tracing::info!("Launching GUI application");
    
    let viewport_builder = egui::ViewportBuilder::default()
        .with_min_inner_size([800.0, 600.0])
        .with_title("Twiggy");
    
    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "Twiggy",
        native_options,
        Box::new(move |cc| {
            let mut app = app;
            app.apply_initial_window_config(&cc.egui_ctx);
            Box::new(app)
        }),
    );
    
    tracing::info!("Application shutdown");
    
    result.map_err(|e| {
        tracing::error!("GUI application failed: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })
}

fn setup_logging(config: &config::LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let level = match config.level {
        config::LogLevel::Error => tracing::Level::ERROR,
        config::LogLevel::Warn => tracing::Level::WARN,
        config::LogLevel::Info => tracing::Level::INFO,
        config::LogLevel::Debug => tracing::Level::DEBUG,
        config::LogLevel::Trace => tracing::Level::TRACE,
    };
    
    let env_filter = EnvFilter::from_default_env()
        .add_directive(level.into());
    
    let mut layers = Vec::new();
    
    if config.console_enabled {
        let console_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);
        layers.push(console_layer.boxed());
    }
    
    if config.file_enabled {
        let log_dir = get_log_directory(config)?;
        std::fs::create_dir_all(&log_dir)?;
        
        let file_appender = rolling::daily(log_dir, "twiggy.log");
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);
        layers.push(file_layer.boxed());
    }
    
    if layers.is_empty() {
        return Err("No logging outputs enabled".into());
    }
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .init();
    
    Ok(())
}

fn get_log_directory(config: &config::LoggingConfig) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(ref custom_dir) = config.log_directory {
        return Ok(PathBuf::from(custom_dir));
    }
    
    let project_dirs = directories::ProjectDirs::from("dev", "twiggy", "Twiggy")
        .ok_or("Could not determine project directories")?;
    
    Ok(project_dirs.data_dir().join("logs"))
}
