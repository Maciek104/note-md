use eframe::egui;
use pulldown_cmark::{
    Alignment as TableAlignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use std::path::Path;

struct ListState {
    ordered: bool,
    next_number: u64,
}

#[derive(Default)]
struct TableState {
    alignments: Vec<TableAlignment>,
    rows: Vec<Vec<egui::text::LayoutJob>>,
    row: Vec<egui::text::LayoutJob>,
    header: bool,
}

pub fn show(ui: &mut egui::Ui, content: &str, image_base_dir: Option<&Path>) {
    ui.vertical(|ui| {
        ui.heading("Preview");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("preview_scroll_area")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                render_markdown(ui, content, image_base_dir);
            });
    });
}

fn render_markdown(ui: &mut egui::Ui, content: &str, image_base_dir: Option<&Path>) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_WIKILINKS;

    let mut job = egui::text::LayoutJob::default();
    let mut table_cell = None;
    let mut bold_depth = 0_usize;
    let mut italic_depth = 0_usize;
    let mut strikethrough_depth = 0_usize;
    let mut superscript_depth = 0_usize;
    let mut subscript_depth = 0_usize;
    let mut underline_depth = 0_usize;
    let mut link_url: Option<String> = None;
    let mut image_url: Option<String> = None;
    let mut image_alt: Option<String> = None;
    let mut heading_size: Option<f32> = None;
    let mut blockquote_depth = 0;
    let mut code_block: Option<(String, Option<String>)> = None;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut paragraph_alignment = egui::Align::Min;
    let mut alignment_span = false;
    let mut table: Option<TableState> = None;
    let mut table_id = 0;

    for event in Parser::new_ext(content, options) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_job(ui, &mut job, paragraph_alignment);
                heading_size = Some(heading_font_size(level));
                bold_depth += 1;
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_job(ui, &mut job, paragraph_alignment);
                heading_size = None;
                bold_depth = bold_depth.saturating_sub(1);
                ui.add_space(4.0);
            }
            Event::Start(Tag::Paragraph) => {
                paragraph_alignment = egui::Align::Min;
                if job.text.is_empty() {
                    if blockquote_depth > 0 {
                        append_marker(
                            ui,
                            &mut job,
                            &mut table_cell,
                            &"│ ".repeat(blockquote_depth),
                        );
                    }
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if table_cell.is_none() {
                    flush_job(ui, &mut job, paragraph_alignment);
                    ui.add_space(6.0);
                }
            }
            Event::Start(Tag::BlockQuote(_)) => blockquote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => {
                blockquote_depth = blockquote_depth.saturating_sub(1);
                ui.add_space(4.0);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_job(ui, &mut job, paragraph_alignment);
                let language = match kind {
                    CodeBlockKind::Fenced(language) if !language.is_empty() => {
                        Some(language.to_string())
                    }
                    _ => None,
                };
                code_block = Some((String::new(), language));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((code, language)) = code_block.take() {
                    egui::Frame::default()
                        .fill(ui.visuals().extreme_bg_color)
                        .inner_margin(8.0)
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            if let Some(language) = language {
                                ui.label(egui::RichText::new(language).small().weak());
                            }
                            ui.label(egui::RichText::new(code.trim_end()).monospace().size(13.0));
                        });
                    ui.add_space(6.0);
                }
            }
            Event::Start(Tag::List(start)) => list_stack.push(ListState {
                ordered: start.is_some(),
                next_number: start.unwrap_or(1),
            }),
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
                ui.add_space(4.0);
            }
            Event::Start(Tag::Item) => {
                if !job.text.is_empty() {
                    flush_job(ui, &mut job, paragraph_alignment);
                }
                let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                let marker = if let Some(list) = list_stack.last_mut() {
                    if list.ordered {
                        let marker = format!("{indent}{}. ", list.next_number);
                        list.next_number += 1;
                        marker
                    } else {
                        format!("{indent}• ")
                    }
                } else {
                    "• ".to_owned()
                };
                append_marker(ui, &mut job, &mut table_cell, &marker);
            }
            Event::End(TagEnd::Item) => {
                flush_job(ui, &mut job, paragraph_alignment);
            }
            Event::Start(Tag::Emphasis) => italic_depth += 1,
            Event::End(TagEnd::Emphasis) => italic_depth = italic_depth.saturating_sub(1),
            Event::Start(Tag::Strong) => bold_depth += 1,
            Event::End(TagEnd::Strong) => bold_depth = bold_depth.saturating_sub(1),
            Event::Start(Tag::Strikethrough) => strikethrough_depth += 1,
            Event::End(TagEnd::Strikethrough) => {
                strikethrough_depth = strikethrough_depth.saturating_sub(1);
            }
            Event::Start(Tag::Superscript) => superscript_depth += 1,
            Event::End(TagEnd::Superscript) => {
                superscript_depth = superscript_depth.saturating_sub(1);
            }
            Event::Start(Tag::Subscript) => subscript_depth += 1,
            Event::End(TagEnd::Subscript) => subscript_depth = subscript_depth.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) => link_url = Some(dest_url.to_string()),
            Event::End(TagEnd::Link) => link_url = None,
            Event::Start(Tag::Image { dest_url, .. }) => {
                image_url = Some(dest_url.to_string());
                if table_cell.is_some() {
                    append_marker(ui, &mut job, &mut table_cell, "🖼 ");
                } else {
                    flush_job(ui, &mut job, paragraph_alignment);
                    image_alt = Some(String::new());
                }
            }
            Event::End(TagEnd::Image) => {
                if let Some(url) = image_url.take() {
                    if let Some(alt) = image_alt.take() {
                        let max_width = ui.available_width().min(640.0);
                        ui.add(
                            egui::Image::from_uri(resolve_image_uri(&url, image_base_dir))
                                .max_width(max_width)
                                .max_height(400.0),
                        );
                        if !alt.is_empty() {
                            ui.label(egui::RichText::new(alt).small().weak());
                        }
                    } else {
                        append_styled(
                            &mut job,
                            &mut table_cell,
                            &format!(" ({url})"),
                            egui::text::TextFormat {
                                color: ui.visuals().weak_text_color(),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            Event::TaskListMarker(checked) => append_marker(
                ui,
                &mut job,
                &mut table_cell,
                if checked { "☑ " } else { "☐ " },
            ),
            Event::Text(text) => {
                if let Some(alt) = image_alt.as_mut() {
                    alt.push_str(&text);
                } else if let Some((code, _)) = code_block.as_mut() {
                    code.push_str(&text);
                } else {
                    append_styled(
                        &mut job,
                        &mut table_cell,
                        &text,
                        make_format(
                            ui,
                            bold_depth,
                            italic_depth,
                            strikethrough_depth,
                            superscript_depth,
                            subscript_depth,
                            underline_depth,
                            link_url.is_some(),
                            heading_size,
                        ),
                    );
                }
            }
            Event::Code(text) => append_styled(
                &mut job,
                &mut table_cell,
                &text,
                egui::text::TextFormat {
                    font_id: egui::FontId::monospace(13.5),
                    color: ui.visuals().text_color(),
                    background: ui.visuals().code_bg_color,
                    ..Default::default()
                },
            ),
            Event::InlineMath(math) => append_code(ui, &mut job, &mut table_cell, &math),
            Event::DisplayMath(math) => {
                flush_job(ui, &mut job, paragraph_alignment);
                egui::Frame::default()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(math).monospace());
                    });
            }
            Event::FootnoteReference(label) => append_styled(
                &mut job,
                &mut table_cell,
                &format!("[^{label}]"),
                egui::text::TextFormat {
                    color: ui.visuals().weak_text_color(),
                    ..Default::default()
                },
            ),
            Event::SoftBreak | Event::HardBreak => append_styled(
                &mut job,
                &mut table_cell,
                "\n",
                make_format(
                    ui,
                    bold_depth,
                    italic_depth,
                    strikethrough_depth,
                    superscript_depth,
                    subscript_depth,
                    underline_depth,
                    link_url.is_some(),
                    heading_size,
                ),
            ),
            Event::Rule => {
                flush_job(ui, &mut job, paragraph_alignment);
                ui.separator();
            }
            Event::InlineHtml(html) => {
                if let Some(alignment) = parse_alignment(&html) {
                    paragraph_alignment = alignment;
                    alignment_span = true;
                } else if alignment_span && html.trim().eq_ignore_ascii_case("</span>") {
                    alignment_span = false;
                } else if html.trim().eq_ignore_ascii_case("<u>") {
                    underline_depth += 1;
                } else if html.trim().eq_ignore_ascii_case("</u>") {
                    underline_depth = underline_depth.saturating_sub(1);
                } else if html.trim().eq_ignore_ascii_case("<br>")
                    || html.trim().eq_ignore_ascii_case("<br/>")
                {
                    append_styled(&mut job, &mut table_cell, "\n", Default::default());
                }
            }
            Event::Html(html) => {
                if let Some(alignment) = parse_alignment(&html) {
                    paragraph_alignment = alignment;
                    alignment_span = true;
                } else {
                    append_styled(
                        &mut job,
                        &mut table_cell,
                        &html,
                        egui::text::TextFormat {
                            color: ui.visuals().weak_text_color(),
                            ..Default::default()
                        },
                    );
                }
            }
            Event::Start(Tag::Table(alignments)) => {
                flush_job(ui, &mut job, paragraph_alignment);
                table = Some(TableState {
                    alignments,
                    ..Default::default()
                });
            }
            Event::Start(Tag::TableHead) => {
                if let Some(table) = table.as_mut() {
                    table.header = true;
                    table.row.clear();
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = table.as_mut() {
                    if !table.row.is_empty() {
                        table.rows.push(std::mem::take(&mut table.row));
                    }
                    table.header = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(table) = table.as_mut() {
                    table.row.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = table.as_mut() {
                    if !table.row.is_empty() {
                        table.rows.push(std::mem::take(&mut table.row));
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                table_cell = Some(egui::text::LayoutJob::default());
                if table.as_ref().is_some_and(|table| table.header) {
                    bold_depth += 1;
                }
            }
            Event::End(TagEnd::TableCell) => {
                if table.as_ref().is_some_and(|table| table.header) {
                    bold_depth = bold_depth.saturating_sub(1);
                }
                if let (Some(table), Some(cell)) = (table.as_mut(), table_cell.take()) {
                    table.row.push(cell);
                }
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                flush_job(ui, &mut job, paragraph_alignment);
                append_marker(ui, &mut job, &mut table_cell, &format!("[^{label}]: "));
            }
            Event::Start(Tag::DefinitionListDefinition) => {
                append_marker(ui, &mut job, &mut table_cell, "  : ");
            }
            Event::End(TagEnd::DefinitionListDefinition) => {
                flush_job(ui, &mut job, paragraph_alignment);
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = table.take() {
                    render_table(ui, table, table_id);
                    table_id += 1;
                }
            }
            _ => {}
        }
    }

    flush_job(ui, &mut job, paragraph_alignment);
}

fn heading_font_size(level: HeadingLevel) -> f32 {
    match level {
        HeadingLevel::H1 => 28.0,
        HeadingLevel::H2 => 24.0,
        HeadingLevel::H3 => 21.0,
        HeadingLevel::H4 => 18.0,
        HeadingLevel::H5 => 16.0,
        HeadingLevel::H6 => 14.0,
    }
}

fn make_format(
    ui: &egui::Ui,
    bold_depth: usize,
    italic_depth: usize,
    strikethrough_depth: usize,
    superscript_depth: usize,
    subscript_depth: usize,
    underline_depth: usize,
    is_link: bool,
    heading_size: Option<f32>,
) -> egui::text::TextFormat {
    let emphasized = bold_depth > 0;
    let size = heading_size.unwrap_or(if emphasized { 15.5 } else { 14.0 })
        * if superscript_depth > 0 || subscript_depth > 0 {
            0.8
        } else {
            1.0
        };
    let link_color = egui::Color32::from_rgb(80, 150, 235);
    egui::text::TextFormat {
        font_id: egui::FontId::proportional(size),
        color: if is_link {
            link_color
        } else {
            ui.visuals().text_color()
        },
        italics: italic_depth > 0,
        strikethrough: if strikethrough_depth > 0 {
            egui::Stroke::new(1.0, ui.visuals().text_color())
        } else {
            egui::Stroke::NONE
        },
        underline: if is_link || underline_depth > 0 {
            egui::Stroke::new(1.0, link_color)
        } else {
            egui::Stroke::NONE
        },
        valign: if superscript_depth > 0 {
            egui::Align::Min
        } else if subscript_depth > 0 {
            egui::Align::Max
        } else {
            egui::Align::Center
        },
        ..Default::default()
    }
}

fn append_marker(
    ui: &egui::Ui,
    job: &mut egui::text::LayoutJob,
    table_cell: &mut Option<egui::text::LayoutJob>,
    marker: &str,
) {
    append_styled(
        job,
        table_cell,
        marker,
        egui::text::TextFormat {
            color: ui.visuals().text_color(),
            ..Default::default()
        },
    );
}

fn append_code(
    ui: &egui::Ui,
    job: &mut egui::text::LayoutJob,
    table_cell: &mut Option<egui::text::LayoutJob>,
    text: &str,
) {
    append_styled(
        job,
        table_cell,
        text,
        egui::text::TextFormat {
            font_id: egui::FontId::monospace(13.5),
            color: ui.visuals().text_color(),
            background: ui.visuals().code_bg_color,
            ..Default::default()
        },
    );
}

fn append_styled(
    job: &mut egui::text::LayoutJob,
    table_cell: &mut Option<egui::text::LayoutJob>,
    text: &str,
    format: egui::text::TextFormat,
) {
    let target = match table_cell.as_mut() {
        Some(cell) => cell,
        None => job,
    };
    target.append(text, 0.0, format);
}

fn flush_job(ui: &mut egui::Ui, job: &mut egui::text::LayoutJob, alignment: egui::Align) {
    if job.text.is_empty() {
        return;
    }

    let mut rendered_job = std::mem::take(job);
    rendered_job.halign = alignment;
    ui.add(egui::Label::new(rendered_job).wrap().halign(alignment));
}

fn parse_alignment(html: &str) -> Option<egui::Align> {
    if !html.contains("data-notemd-align") {
        return None;
    }
    if html.contains("\"center\"") || html.contains("'center'") {
        Some(egui::Align::Center)
    } else if html.contains("\"right\"") || html.contains("'right'") {
        Some(egui::Align::Max)
    } else {
        Some(egui::Align::Min)
    }
}

fn resolve_image_uri(uri: &str, base_dir: Option<&Path>) -> String {
    if uri.contains("://") || uri.starts_with("data:") {
        return uri.to_owned();
    }

    let path = Path::new(uri);
    let Some(base_dir) = base_dir else {
        return uri.to_owned();
    };
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };
    let path = path.to_string_lossy().replace('\\', "/");
    let encoded_path = encode_uri_path(&path);

    if path.starts_with('/') {
        format!("file://{encoded_path}")
    } else {
        format!("file:///{encoded_path}")
    }
}

fn encode_uri_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn render_table(ui: &mut egui::Ui, table: TableState, id: usize) {
    egui::Grid::new(("markdown_preview_table", id))
        .striped(true)
        .show(ui, |ui| {
            for row in table.rows {
                for (column, mut cell) in row.into_iter().enumerate() {
                    let alignment = match table.alignments.get(column) {
                        Some(TableAlignment::Center) => egui::Align::Center,
                        Some(TableAlignment::Right) => egui::Align::Max,
                        _ => egui::Align::Min,
                    };
                    cell.halign = alignment;
                    ui.add(egui::Label::new(cell).wrap().halign(alignment));
                }
                ui.end_row();
            }
        });
    ui.add_space(6.0);
}

#[cfg(test)]
mod tests {
    use super::{encode_uri_path, parse_alignment, resolve_image_uri};
    use eframe::egui;
    use std::path::Path;

    #[test]
    fn resolves_relative_image_paths_from_note_directory() {
        let base_dir = std::env::temp_dir().join("NoteMD");
        let expected_path = base_dir.join("images").join("cover image.png");
        let expected_path = expected_path.to_string_lossy().replace('\\', "/");
        let prefix = if expected_path.starts_with('/') {
            "file://"
        } else {
            "file:///"
        };

        assert_eq!(
            resolve_image_uri("images/cover image.png", Some(&base_dir)),
            format!("{prefix}{}", encode_uri_path(&expected_path))
        );
    }

    #[test]
    fn preserves_remote_image_uris() {
        let uri = "https://example.com/cover.png";
        assert_eq!(resolve_image_uri(uri, Some(Path::new("notes"))), uri);
    }

    #[test]
    fn reads_note_alignment_markers() {
        assert_eq!(
            parse_alignment(
                r#"<span data-notemd-align="center" style="display:block;text-align:center;">"#
            ),
            Some(egui::Align::Center)
        );
        assert_eq!(
            parse_alignment(r#"<span data-notemd-align="right">"#),
            Some(egui::Align::Max)
        );
    }
}
