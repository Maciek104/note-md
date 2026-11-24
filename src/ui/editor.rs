use eframe::egui;

use crate::ui::toolbar::FormatAction;

pub fn show(ui: &mut egui::Ui, content: &mut String, note_id: &str) -> egui::Id {
    let mut editor_id = None;

    ui.vertical(|ui| {
        ui.heading("Editor");

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt(("editor_scroll_area", note_id))
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(content)
                        .id_salt(("note_editor", note_id))
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .font(egui::TextStyle::Monospace)
                        .frame(egui::Frame::NONE),
                );
                editor_id = Some(response.id);
            });
    });

    editor_id.expect("the editor text field is always rendered")
}

pub fn apply_format(
    ctx: &egui::Context,
    content: &mut String,
    editor_id: egui::Id,
    action: FormatAction,
) -> bool {
    let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) else {
        return false;
    };

    let cursor_range = state.cursor.char_range();
    let selected_range = cursor_range
        .map(|range| range.as_sorted_char_range())
        .unwrap_or_else(|| {
            let end = egui::text::CharIndex(content.chars().count());
            end..end
        });
    let start = byte_offset(content, selected_range.start.0);
    let end = byte_offset(content, selected_range.end.0);

    match action {
        FormatAction::Wrap(prefix, suffix) => {
            let selected = content[start..end].to_owned();
            let replacement = format!("{prefix}{selected}{suffix}");
            content.replace_range(start..end, &replacement);

            let cursor_start = start + prefix.len();
            let cursor_end = cursor_start + selected.len();
            let range = if selected.is_empty() {
                egui::text::CCursorRange::one(egui::text::CCursor::new(
                    content[..cursor_start].chars().count(),
                ))
            } else {
                egui::text::CCursorRange::two(
                    egui::text::CCursor::new(content[..cursor_start].chars().count()),
                    egui::text::CCursor::new(content[..cursor_end].chars().count()),
                )
            };
            state.cursor.set_char_range(Some(range));
        }
        FormatAction::Prefix(prefix) => {
            let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
            let last_selected_char = if end > line_start && content[..end].ends_with('\n') {
                end - 1
            } else {
                end
            };
            let line_end = content[last_selected_char..]
                .find('\n')
                .map_or(content.len(), |index| last_selected_char + index);
            let selected_lines = content[line_start..line_end].to_owned();
            let mut formatted_lines = String::new();
            for line in selected_lines.split_inclusive('\n') {
                let (line_text, line_break) = line
                    .strip_suffix('\n')
                    .map_or((line, ""), |text| (text, "\n"));
                if !line_text.is_empty() && !line_text.starts_with(prefix) {
                    formatted_lines.push_str(prefix);
                }
                formatted_lines.push_str(line_text);
                formatted_lines.push_str(line_break);
            }
            content.replace_range(line_start..line_end, &formatted_lines);
            let cursor = content[..line_start + formatted_lines.len()]
                .chars()
                .count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(cursor),
                )));
        }
        FormatAction::Heading(level) => {
            let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
            let line_end = content[line_start..]
                .find('\n')
                .map_or(content.len(), |index| line_start + index);
            let line = &content[line_start..line_end];
            let old_prefix_len = line
                .chars()
                .take_while(|character| *character == '#')
                .count()
                .min(6);
            let old_prefix_len = if old_prefix_len > 0 && line[old_prefix_len..].starts_with(' ') {
                old_prefix_len + 1
            } else {
                0
            };
            let prefix = format!("{} ", "#".repeat(usize::from(level.clamp(1, 6))));
            content.replace_range(line_start..line_start + old_prefix_len, &prefix);
            let cursor = content[..line_start + prefix.len()].chars().count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(cursor),
                )));
        }
        FormatAction::Insert(insert) => {
            content.replace_range(start..end, insert);
            let cursor = content[..start + insert.len()].chars().count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(cursor),
                )));
        }
    }

    state.store(ctx, editor_id);
    ctx.memory_mut(|memory| memory.request_focus(editor_id));
    true
}

fn byte_offset(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map_or(text.len(), |(byte_offset, _)| byte_offset)
}
