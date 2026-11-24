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
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "phosphor".to_owned(),
                egui::FontData::from_static(egui_phosphor::bytes::regular::FONT).into(),
            );
            let proportional_fonts = fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .expect("default proportional font family exists");
            let insert_at = proportional_fonts.len().min(1);
            proportional_fonts.insert(insert_at, "phosphor".into());
            cc.egui_ctx.set_fonts(fonts);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app::NoteMD::default()))
        }),
    )
}
