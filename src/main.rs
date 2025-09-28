mod app;
mod config;
mod error;
mod git;
mod ui;
mod logging;

use crate::{app::TwiggyApp, logging::{initialize_logging, LoggingConfig}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logging_config = LoggingConfig::default();
    
    if let Err(e) = initialize_logging(logging_config) {
        eprintln!("Failed to initialize logging: {}", e);
        return Err(Box::new(e));
    }
    
    tracing::info!("Starting Twiggy Git Visualizer v{}", env!("CARGO_PKG_VERSION"));
    
    let app = match TwiggyApp::new() {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("Failed to initialize application: {}", e);
            return Err(Box::new(e));
        }
    };
    
    tracing::info!("Launching GUI application");
    
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([app.config().window.width, app.config().window.height])
        .with_min_inner_size([800.0, 600.0]);
    
    if app.config().window.maximized {
        viewport_builder = viewport_builder.with_maximized(true);
    }
    
    if app.config().window.remember_position {
        if let (Some(x), Some(y)) = (app.config().window.position_x, app.config().window.position_y) {
            viewport_builder = viewport_builder.with_position([x, y]);
        }
    }
    
    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };
    
    eframe::run_native(
        "Twiggy - Git Visualizer",
        native_options,
        Box::new(move |_cc| Box::new(app)),
    ).map_err(|e| {
        tracing::error!("GUI application failed: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })
}
