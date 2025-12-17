//Brought to you by KpolitX - https://github.com/DarkFeed2005
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct Note {
    id: usize,
    title: String,
    content: String,
    created_at: String,
}

struct NoteApp {
    notes: Vec<Note>,
    selected_note: Option<usize>,
    new_note_title: String,
    new_note_content: String,
    search_query: String,
    data_file: PathBuf,
}

impl Default for NoteApp {
    fn default() -> Self {
        let data_file = Self::get_data_file_path();
        let notes = Self::load_notes(&data_file);
        
        Self {
            notes,
            selected_note: None,
            new_note_title: String::new(),
            new_note_content: String::new(),
            search_query: String::new(),
            data_file,
        }
    }
}

impl NoteApp {
    fn get_data_file_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".note_app_data.json");
        path
    }

    fn load_notes(path: &PathBuf) -> Vec<Note> {
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        }
    }

    fn save_notes(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.notes) {
            let _ = fs::write(&self.data_file, json);
        }
    }

    fn create_note(&mut self) {
        if !self.new_note_title.is_empty() {
            let id = self.notes.len();
            let note = Note {
                id,
                title: self.new_note_title.clone(),
                content: self.new_note_content.clone(),
                created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            self.notes.push(note);
            self.selected_note = Some(id);
            self.new_note_title.clear();
            self.new_note_content.clear();
            self.save_notes();
        }
    }

    fn delete_note(&mut self, id: usize) {
        self.notes.retain(|note| note.id != id);
        // Reassign IDs
        for (idx, note) in self.notes.iter_mut().enumerate() {
            note.id = idx;
        }
        if let Some(selected) = self.selected_note {
            if selected == id {
                self.selected_note = None;
            }
        }
        self.save_notes();
    }

    fn update_note(&mut self, id: usize, content: String) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
            note.content = content;
            self.save_notes();
        }
    }

    fn filtered_notes(&self) -> Vec<&Note> {
        if self.search_query.is_empty() {
            self.notes.iter().collect()
        } else {
            self.notes
                .iter()
                .filter(|note| {
                    note.title.to_lowercase().contains(&self.search_query.to_lowercase())
                        || note.content.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }
}
//Brought to you by KpolitX - https://github.com/DarkFeed2005
impl eframe::App for NoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sidebar
        egui::SidePanel::left("sidebar")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("📝 My Notes");
                    ui.add_space(10.0);
                });

                ui.separator();

                // Search bar
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.search_query);
                });

                ui.add_space(5.0);
                ui.separator();

                // Notes list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect note data first to avoid borrow checker issues
                    let filtered: Vec<(usize, String, String, bool)> = self.filtered_notes()
                        .iter()
                        .map(|note| (
                            note.id,
                            note.title.clone(),
                            note.created_at.clone(),
                            self.selected_note == Some(note.id)
                        ))
                        .collect();
                    
                    let mut note_to_select = None;
                    let mut note_to_delete = None;

                    for (id, title, created_at, is_selected) in filtered {
                        let response = ui.selectable_label(
                            is_selected,
                            format!("📄 {}", title),
                        );

                        if response.clicked() {
                            note_to_select = Some(id);
                        }

                        if is_selected {
                            ui.horizontal(|ui| {
                                ui.label(format!("   📅 {}", created_at));
                                if ui.small_button("🗑").clicked() {
                                    note_to_delete = Some(id);
                                }
                            });
                        }

                        ui.add_space(5.0);
                    }

                    // Apply changes after the loop
                    if let Some(id) = note_to_select {
                        self.selected_note = Some(id);
                    }
                    if let Some(id) = note_to_delete {
                        self.delete_note(id);
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.button("➕ New Note").clicked() {
                        self.selected_note = None;
                    }
                    ui.add_space(5.0);
                });
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            if let Some(note_id) = self.selected_note {
                // Edit existing note
                if let Some(note) = self.notes.iter().find(|n| n.id == note_id).cloned() {
                    ui.heading(&note.title);
                    ui.label(format!("Created: {}", note.created_at));
                    ui.separator();
                    ui.add_space(10.0);

                    let mut content = note.content.clone();
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut content)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .font(egui::TextStyle::Body),
                    );

                    if response.changed() {
                        self.update_note(note_id, content);
                    }
                }
            } else {
                // Create new note
                ui.vertical_centered(|ui| {
                    ui.heading("Create New Note");
                });
                
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_note_title);
                });

                ui.add_space(10.0);

                ui.label("Content:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.new_note_content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15),
                );

                ui.add_space(10.0);

                if ui.button("Create Note").clicked() {
                    self.create_note();
                }
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Note Taking App by KpolitX",
        options,
        Box::new(|_cc| Ok(Box::new(NoteApp::default()))),
    )
}
//Brought to you by KpolitX - https://github.com/DarkFeed2005