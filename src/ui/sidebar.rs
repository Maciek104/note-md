use eframe::egui;
use std::path::Path;

pub enum SidebarAction {
    Select(usize),
    Delete(usize),
}

pub fn show(
    ui: &mut egui::Ui,
    titles: &[String],
    selected: Option<usize>,
    saved_path: Option<&Path>,
    has_unsaved_changes: bool,
) -> Option<SidebarAction> {
    let mut action = None;

    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(256.0)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("NoteMD, by Maciek104");
            });

            ui.separator();

            if let Some(path) = saved_path {
                ui.label(egui::RichText::new("Saved to").small().weak());
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .small()
                        .weak(),
                );
            } else {
                ui.label(egui::RichText::new("Not saved yet").small().weak());
            }
            if has_unsaved_changes {
                ui.label(egui::RichText::new("Unsaved changes").small().weak());
            }

            ui.heading("History");
            egui::ScrollArea::vertical()
                .id_salt("note_history")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    for (index, title) in titles.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(selected == Some(index), title)
                                .clicked()
                            {
                                action = Some(SidebarAction::Select(index));
                            }
                            if ui
                                .small_button(egui_phosphor::regular::TRASH)
                                .on_hover_text("Delete note")
                                .clicked()
                            {
                                action = Some(SidebarAction::Delete(index));
                            }
                        });
                    }
                });
        });

    action
}
