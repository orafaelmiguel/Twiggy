mod app;
mod config;
mod error;
mod git;
mod ui;

use eframe::egui;
use app::TwiggyApp;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Twiggy - Git Visualizer",
        options,
        Box::new(|_cc| {
            Box::new(TwiggyApp::new().unwrap_or_default())
        }),
    )
}
