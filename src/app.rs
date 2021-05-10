use std::path::{Path, PathBuf};

use eframe::{egui, epi};

use crate::windows::{self, AgtWindow};

#[derive(Debug)]
pub enum GameDirectoryEntry {
    Agt(windows::AgtWindow),
    Unknown(String),
}

impl GameDirectoryEntry {
    pub fn display(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        match self {
            GameDirectoryEntry::Agt(ref mut agt_window) => {
                agt_window.display(ctx, ui);
            }
            GameDirectoryEntry::Unknown(label) => {
                ui.label(label as &str);
            }
        }
    }
}

#[derive(Debug)]
pub struct OpenGameDirectory {
    pub path: PathBuf,
    pub entries: Vec<GameDirectoryEntry>,
}

impl OpenGameDirectory {
    fn crawl_dir(mut entries: &mut Vec<GameDirectoryEntry>, root_path: &str, path: &Path) {
        for thing in std::fs::read_dir(path).unwrap() {
            let thing = thing.unwrap();
            if thing.file_type().unwrap().is_dir() {
                OpenGameDirectory::crawl_dir(&mut entries, &root_path, &thing.path());
                continue;
            }

            let file_name = thing.file_name();
            let file_name_str = file_name.to_str().unwrap();

            let thing_path = thing.path();
            let thing_path_str = thing_path.display().to_string();
            let thing_path_str_relative =
                thing_path_str.replacen(&format!("{}\\", root_path), "", 1);

            if file_name_str.ends_with(".agt") || file_name_str.starts_with("Patch.0") {
                entries.push(GameDirectoryEntry::Agt(AgtWindow::new(
                    thing_path,
                    Some(thing_path_str_relative),
                )));
            } else {
                // entries.push(GameDirectoryEntry::Unknown(thing_path_str_relative));
            }
        }
    }

    pub fn from_dialog() -> Option<Self> {
        let path = native_dialog::FileDialog::new()
            .show_open_single_dir()
            .unwrap();

        if let Some(path) = path {
            let mut entries = Vec::new();

            OpenGameDirectory::crawl_dir(&mut entries, &path.display().to_string(), &path);

            Some(OpenGameDirectory { path, entries })
        } else {
            None
        }
    }
}

pub struct SlidetownApp {
    game_dir: Option<OpenGameDirectory>,
}

impl Default for SlidetownApp {
    fn default() -> Self {
        Self { game_dir: None }
    }
}

impl epi::App for SlidetownApp {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        self.game_dir = OpenGameDirectory::from_dialog();
                    }
                });
            });
        });

        if let Some(game_dir) = &mut self.game_dir {
            egui::SidePanel::left("side_panel", 320.0).show(ctx, |ui| {
                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    ui.heading(format!("{}", game_dir.path.display()));
                    for entry in game_dir.entries.iter_mut() {
                        entry.display(ctx, ui);
                    }
                });
            });
        }
    }

    fn name(&self) -> &str {
        "slidetown"
    }
}
