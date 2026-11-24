use eframe::egui;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::ui::ViewMode;
use crate::ui::editor;
use crate::ui::preview;
use crate::ui::sidebar;
use crate::ui::toolbar::{self, ToolbarAction};

struct Note {
    id: String,
    path: Option<PathBuf>,
    content: String,
    saved_content: Option<String>,
}

impl Note {
    fn title(&self) -> String {
        self.content
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| {
                line.trim()
                    .strip_prefix("# ")
                    .unwrap_or(line.trim())
                    .to_owned()
            })
            .filter(|title| !title.is_empty())
            .or_else(|| {
                self.path
                    .as_ref()
                    .and_then(|path| path.file_stem())
                    .map(|title| title.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "Untitled note".to_owned())
    }

    fn is_dirty(&self) -> bool {
        self.path.is_some() && self.saved_content.as_deref() != Some(&self.content)
    }
}

pub struct NoteMD {
    notes_dir: Option<PathBuf>,
    notes: Vec<Note>,
    next_note_id: u64,
    selected: Option<usize>,
    view_mode: ViewMode,
    editor_id: Option<egui::Id>,
    error: Option<String>,
}

impl Default for NoteMD {
    fn default() -> Self {
        Self::new()
    }
}

impl NoteMD {
    fn new() -> Self {
        let mut app = Self {
            notes_dir: None,
            notes: Vec::new(),
            next_note_id: 0,
            selected: None,
            view_mode: ViewMode::Split,
            editor_id: None,
            error: None,
        };

        match app.initialize_notes() {
            Ok(()) => {}
            Err(error) => {
                app.error = Some(error);
                app.notes.push(Note {
                    id: "draft-0".to_owned(),
                    path: None,
                    content: String::from(
                        "# Welcome to NoteMD\n\nStart writing your notes here...",
                    ),
                    saved_content: None,
                });
                app.next_note_id = 1;
                app.selected = Some(0);
            }
        }

        app
    }

    fn initialize_notes(&mut self) -> Result<(), String> {
        let home_dir = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .ok_or_else(|| "Could not locate the user's Documents folder.".to_owned())?;
        let notes_dir = home_dir.join("Documents").join("NoteMD");
        fs::create_dir_all(&notes_dir)
            .map_err(|error| format!("Could not create notes directory: {error}"))?;

        let mut paths = fs::read_dir(&notes_dir)
            .map_err(|error| format!("Could not read note history: {error}"))?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|error| format!("Could not read a note entry: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|path| path.extension().is_some_and(|extension| extension == "md"));
        paths.sort_by(|left, right| right.file_name().cmp(&left.file_name()));

        self.notes = paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let content = fs::read_to_string(&path)
                    .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
                Ok(Note {
                    id: format!("saved-{index}"),
                    path: Some(path),
                    saved_content: Some(content.clone()),
                    content,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        self.notes_dir = Some(notes_dir);
        self.next_note_id = self.notes.len() as u64;
        if self.notes.is_empty() {
            self.add_unsaved_note(String::from(
                "# Welcome to NoteMD\n\nStart writing your notes here...",
            ));
        } else {
            self.selected = Some(0);
        }
        Ok(())
    }

    fn add_unsaved_note(&mut self, content: String) {
        let id = format!("draft-{}", self.next_note_id);
        self.next_note_id += 1;
        self.notes.insert(
            0,
            Note {
                id,
                path: None,
                content,
                saved_content: None,
            },
        );
        self.selected = Some(0);
    }

    fn save_selected(&mut self) -> Result<(), String> {
        let index = self
            .selected
            .ok_or_else(|| "There is no note selected.".to_owned())?;
        let notes_dir = self
            .notes_dir
            .clone()
            .ok_or_else(|| "Notes storage is unavailable.".to_owned())?;
        let note = &mut self.notes[index];

        if let Some(path) = &note.path {
            fs::write(path, &note.content)
                .map_err(|error| format!("Could not save {}: {error}", path.display()))?;
            note.saved_content = Some(note.content.clone());
            return Ok(());
        }

        let stem = safe_file_stem(&note.title());
        let mut suffix = 1_u64;
        loop {
            let file_name = if suffix == 1 {
                format!("{stem}.md")
            } else {
                format!("{stem} ({suffix}).md")
            };
            let path = notes_dir.join(file_name);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    if let Err(error) = file.write_all(note.content.as_bytes()) {
                        drop(file);
                        return match fs::remove_file(&path) {
                            Ok(()) => Err(format!("Could not save note: {error}")),
                            Err(cleanup_error) => Err(format!(
                                "Could not save note: {error}; could not remove incomplete file {}: {cleanup_error}",
                                path.display()
                            )),
                        };
                    }
                    note.path = Some(path);
                    note.saved_content = Some(note.content.clone());
                    return Ok(());
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    suffix += 1;
                }
                Err(error) => {
                    return Err(format!(
                        "Could not save note as {}: {error}",
                        path.display()
                    ));
                }
            }
        }
    }

    fn delete_note(&mut self, index: usize) -> Result<(), String> {
        let note = self
            .notes
            .get(index)
            .ok_or_else(|| "The selected note no longer exists.".to_owned())?;
        if let Some(path) = &note.path {
            fs::remove_file(path).map_err(|error| format!("Could not delete note: {error}"))?;
        }
        self.notes.remove(index);

        if self.notes.is_empty() {
            self.add_unsaved_note(String::from("# New note\n\n"));
        } else {
            self.selected = Some(index.min(self.notes.len() - 1));
        }
        Ok(())
    }

    fn report_result(&mut self, result: Result<(), String>) {
        self.error = result.err();
    }
}

impl eframe::App for NoteMD {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let titles: Vec<String> = self.notes.iter().map(Note::title).collect();
        let selected_note = self.selected.and_then(|index| self.notes.get(index));
        let saved_path = selected_note.and_then(|note| note.path.as_deref());
        let has_unsaved_changes = selected_note.is_some_and(Note::is_dirty);
        if let Some(action) =
            sidebar::show(ui, &titles, self.selected, saved_path, has_unsaved_changes)
        {
            match action {
                sidebar::SidebarAction::Select(index) => self.selected = Some(index),
                sidebar::SidebarAction::Delete(index) => {
                    let result = self.delete_note(index);
                    self.report_result(result);
                }
            }
        }

        let actions = toolbar::show(ui, &mut self.view_mode);
        for action in actions {
            match action {
                ToolbarAction::NewNote => {
                    self.add_unsaved_note(String::from("# New note\n\n"));
                }
                ToolbarAction::SaveNote => {
                    let result = self.save_selected();
                    self.report_result(result);
                }
                ToolbarAction::Format(format) => {
                    if let (Some(index), Some(editor_id)) = (self.selected, self.editor_id) {
                        editor::apply_format(
                            ui.ctx(),
                            &mut self.notes[index].content,
                            editor_id,
                            format,
                        );
                    }
                }
            }
        }

        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }

        let Some(index) = self.selected else {
            return;
        };
        let note_id = self.notes[index].path.as_ref().map_or_else(
            || self.notes[index].id.clone(),
            |path| path.to_string_lossy().into_owned(),
        );
        let image_base_dir = self.notes[index]
            .path
            .as_ref()
            .and_then(|path| path.parent())
            .map(|path| path.to_path_buf())
            .or_else(|| self.notes_dir.clone());
        match self.view_mode {
            ViewMode::Editor => {
                self.editor_id = Some(editor::show(ui, &mut self.notes[index].content, &note_id));
            }
            ViewMode::Preview => {
                preview::show(ui, &self.notes[index].content, image_base_dir.as_deref());
            }
            ViewMode::Split => {
                ui.columns(2, |columns| {
                    self.editor_id = Some(editor::show(
                        &mut columns[0],
                        &mut self.notes[index].content,
                        &note_id,
                    ));
                    preview::show(
                        &mut columns[1],
                        &self.notes[index].content,
                        image_base_dir.as_deref(),
                    );
                });
            }
        }
    }
}

fn safe_file_stem(title: &str) -> String {
    let stem: String = title
        .trim()
        .chars()
        .take(100)
        .map(|character| {
            if matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) {
                '_'
            } else {
                character
            }
        })
        .collect();
    let stem = stem.trim().trim_end_matches('.').trim();
    if stem.is_empty() {
        "Untitled note".to_owned()
    } else if matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        format!("_{stem}")
    } else {
        stem.to_owned()
    }
}
