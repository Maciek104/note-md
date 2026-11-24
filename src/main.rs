mod app;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("NoteMD"),
        ..Default::default()
    };

    eframe::run_native(
        "NoteMD",
        options,
        Box::new(|_cc| {
            Ok(Box::new(app::NoteMD::default()))
        }),
    )
}