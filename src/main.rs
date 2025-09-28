mod app;
mod ui;

use app::MainApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Egui Template",
        options,
        Box::new(|_cc| Ok(Box::new(MainApp::default()))),
    )
}