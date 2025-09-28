use eframe::egui;

pub fn render_main_panel(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Hello World!");
        ui.separator();
        ui.label("Boilerplate for Egui");
    });
}