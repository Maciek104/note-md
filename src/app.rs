use eframe::egui;

use crate::ui::sidebar;
use crate::ui::toolbar;

#[derive(Default)]
pub struct NoteMD;

impl eframe::App for NoteMD {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        sidebar::show(ui);
        toolbar::show(ui);
    }
}