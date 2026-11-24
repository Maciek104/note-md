use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(256.0)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("NoteMD, by Maciek104");
            });

            ui.separator();

            ui.add_space(10.0);

            ui.label("Your notes will appear here.");
        });
}