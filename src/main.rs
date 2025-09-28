use eframe::egui;
use anyhow::Result;

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
        Box::new(|_cc| Box::new(TwiggyApp::default())),
    )
}

#[derive(Default)]
struct TwiggyApp {
}

impl eframe::App for TwiggyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("🌿 Twiggy");
                ui.label("Lightning-fast Git Visualization Tool");
                ui.add_space(20.0);
                ui.label("Phase 3: Basic egui Window - ✅ Currently Active");
                ui.separator();
                ui.label("Built with Rust + egui for maximum performance");
                ui.add_space(10.0);
                ui.small("Professional Git visualization for developers");
            });
        });
    }
}
