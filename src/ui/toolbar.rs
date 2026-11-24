use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {

        // Pierwsza kolumna
        ui.vertical(|ui| {
            if ui.button("New note").clicked() {
                println!("New note");
            }

            if ui.button("Open note").clicked() {
                println!("Open");
            }

            if ui.button("Delete note").clicked() {
                println!("Deleted");
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            if ui.button("Save note").clicked() {
                println!("Save note");
            }

            if ui.button("Save note as").clicked() {
                println!("Save note as");
            }
        });
    });
}