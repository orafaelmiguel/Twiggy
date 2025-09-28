use eframe::egui;
use crate::config::AppConfig;

#[derive(Default)]
pub struct TwiggyApp {
    config: AppConfig,
}

impl TwiggyApp {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self {
            config: AppConfig::load()?,
        })
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    #[allow(dead_code)]
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }
}

impl eframe::App for TwiggyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("🌿 Twiggy");
                ui.label("Lightning-fast Git Visualization Tool");
                ui.add_space(20.0);
                ui.label("Phase 4: Modular Architecture - ✅ Currently Active");
                ui.separator();
                ui.label("Built with Rust + egui for maximum performance");
                ui.add_space(10.0);
                ui.small("Professional Git visualization for developers");
                ui.add_space(20.0);
                ui.label("🏗️ Modular Structure Complete:");
                ui.label("• Application Logic (app.rs)");
                ui.label("• Error Handling (error.rs)");
                ui.label("• Configuration (config.rs)");
                ui.label("• Git Operations (git/)");
                ui.label("• UI Components (ui/)");
            });
        });
    }
}