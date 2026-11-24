use eframe::egui;

use crate::ui::ViewMode;

#[derive(Clone, Copy)]
pub enum FormatAction {
    Wrap(&'static str, &'static str),
    Prefix(&'static str),
    Heading(u8),
    Insert(&'static str),
}

pub enum ToolbarAction {
    NewNote,
    SaveNote,
    Format(FormatAction),
}

pub fn show(ui: &mut egui::Ui, view_mode: &mut ViewMode) -> Vec<ToolbarAction> {
    let mut actions = Vec::new();

    ui.horizontal_wrapped(|ui| {
        if icon_button(ui, egui_phosphor::regular::FILE_PLUS, "New note").clicked() {
            actions.push(ToolbarAction::NewNote);
        }
        if icon_button(ui, egui_phosphor::regular::FLOPPY_DISK, "Save / Save as").clicked() {
            actions.push(ToolbarAction::SaveNote);
        }

        ui.separator();

        if icon_button(ui, egui_phosphor::regular::TEXT_B, "Bold").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("**", "**")));
        }
        if icon_button(ui, egui_phosphor::regular::TEXT_ITALIC, "Italic").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("*", "*")));
        }
        if icon_button(
            ui,
            egui_phosphor::regular::TEXT_STRIKETHROUGH,
            "Strikethrough",
        )
        .clicked()
        {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("~~", "~~")));
        }
        if icon_button(
            ui,
            egui_phosphor::regular::TEXT_UNDERLINE,
            "Underline (HTML)",
        )
        .clicked()
        {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("<u>", "</u>")));
        }
        if icon_button(ui, egui_phosphor::regular::TEXT_SUPERSCRIPT, "Superscript").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("^", "^")));
        }
        if icon_button(ui, egui_phosphor::regular::TEXT_SUBSCRIPT, "Subscript").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("~", "~")));
        }
        if icon_button(ui, egui_phosphor::regular::CODE, "Inline code").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("`", "`")));
        }
        if icon_button(ui, egui_phosphor::regular::LINK, "Link").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("[", "](url)")));
        }
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_ONE, 1);
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_TWO, 2);
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_THREE, 3);
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_FOUR, 4);
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_FIVE, 5);
        heading_button(ui, &mut actions, egui_phosphor::regular::TEXT_H_SIX, 6);
        if icon_button(ui, egui_phosphor::regular::LIST_BULLETS, "Bulleted list").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Prefix("- ")));
        }
        if icon_button(ui, egui_phosphor::regular::LIST_NUMBERS, "Numbered list").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Prefix("1. ")));
        }
        if icon_button(ui, egui_phosphor::regular::LIST_CHECKS, "Task list").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Prefix("- [ ] ")));
        }
        if icon_button(ui, egui_phosphor::regular::QUOTES, "Block quote").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Prefix("> ")));
        }
        if icon_button(ui, egui_phosphor::regular::CODE_BLOCK, "Code block").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap("```\n", "\n```")));
        }
        if icon_button(ui, egui_phosphor::regular::IMAGE, "Image").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Wrap(
                "![",
                "](image-url)",
            )));
        }
        if icon_button(ui, egui_phosphor::regular::TABLE, "Table").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Insert(
                "| Column 1 | Column 2 |\n| --- | --- |\n| Value | Value |",
            )));
        }
        if icon_button(
            ui,
            egui_phosphor::regular::BRACKETS_CURLY,
            "Footnote reference",
        )
        .clicked()
        {
            actions.push(ToolbarAction::Format(FormatAction::Insert("[^1]")));
        }
        if icon_button(ui, egui_phosphor::regular::MINUS, "Horizontal rule").clicked() {
            actions.push(ToolbarAction::Format(FormatAction::Insert("\n\n---\n\n")));
        }

        ui.separator();

        align_button(
            ui,
            &mut actions,
            egui_phosphor::regular::TEXT_ALIGN_LEFT,
            "left",
            "Align left",
        );
        align_button(
            ui,
            &mut actions,
            egui_phosphor::regular::TEXT_ALIGN_CENTER,
            "center",
            "Align center",
        );
        align_button(
            ui,
            &mut actions,
            egui_phosphor::regular::TEXT_ALIGN_RIGHT,
            "right",
            "Align right",
        );

        ui.separator();

        view_button(
            ui,
            view_mode,
            ViewMode::Editor,
            egui_phosphor::regular::PENCIL_SIMPLE,
            "Editor",
        );
        view_button(
            ui,
            view_mode,
            ViewMode::Preview,
            egui_phosphor::regular::EYE,
            "Preview",
        );
        view_button(
            ui,
            view_mode,
            ViewMode::Split,
            egui_phosphor::regular::COLUMNS,
            "Both",
        );
    });

    actions
}

fn heading_button(ui: &mut egui::Ui, actions: &mut Vec<ToolbarAction>, icon: &str, level: u8) {
    if icon_button(ui, icon, &format!("Heading {level}")).clicked() {
        actions.push(ToolbarAction::Format(FormatAction::Heading(level)));
    }
}

fn align_button(
    ui: &mut egui::Ui,
    actions: &mut Vec<ToolbarAction>,
    icon: &str,
    alignment: &'static str,
    tooltip: &str,
) {
    if icon_button(ui, icon, tooltip).clicked() {
        let open_tag = match alignment {
            "left" => "<span data-notemd-align=\"left\" style=\"display:block;text-align:left;\">",
            "center" => {
                "<span data-notemd-align=\"center\" style=\"display:block;text-align:center;\">"
            }
            _ => "<span data-notemd-align=\"right\" style=\"display:block;text-align:right;\">",
        };
        actions.push(ToolbarAction::Format(FormatAction::Wrap(
            open_tag, "</span>",
        )));
    }
}

fn icon_button<'a>(ui: &mut egui::Ui, icon: &str, tooltip: &'a str) -> egui::Response {
    ui.button(egui::RichText::new(icon).size(18.0))
        .on_hover_text(tooltip)
}

fn view_button(
    ui: &mut egui::Ui,
    mode: &mut ViewMode,
    selected: ViewMode,
    icon: &str,
    tooltip: &str,
) {
    if ui
        .selectable_label(*mode == selected, egui::RichText::new(icon).size(18.0))
        .on_hover_text(tooltip)
        .clicked()
    {
        *mode = selected;
    }
}
