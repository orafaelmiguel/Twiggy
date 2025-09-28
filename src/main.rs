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
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
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
