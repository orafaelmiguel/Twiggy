use eframe::{egui, App};
use crate::ui;

#[derive(Default)]
pub struct MainApp;

impl App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_main_panel(ctx);
    }
}