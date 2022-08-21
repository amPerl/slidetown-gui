use camino::Utf8PathBuf;
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::{project::Project, storage::prompt_game_directory};

pub enum NewProjectDialogResult {
    Created(Project),
    Canceled,
    Idle,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct NewProjectDialog {
    game_dir: Option<Utf8PathBuf>,
    key_dir: Option<Utf8PathBuf>,
}

impl NewProjectDialog {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn show(&mut self, ctx: &egui::Context) -> NewProjectDialogResult {
        let mut result = NewProjectDialogResult::Idle;

        let mut open = true;
        egui::Window::new("New Project")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Game directory:");
                    if let Some(game_dir) = &self.game_dir {
                        ui.add_enabled(false, egui::Label::new(game_dir.as_str()));
                    }
                    if ui.button("Choose").clicked() {
                        self.game_dir = prompt_game_directory();
                    }

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Custom encryption key (optional):").weak());
                    if let Some(key_dir) = &self.key_dir {
                        ui.add_enabled(false, egui::Label::new(key_dir.as_str()));
                    }
                    if ui.button("Choose").clicked() {
                        dbg!(&self.key_dir);
                    }

                    ui.add_space(12.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.add_enabled_ui(self.game_dir.is_some(), |ui| {
                            if ui.button("Create Project").clicked() {
                                let game_dir = self.game_dir.as_ref().unwrap().clone();
                                result = NewProjectDialogResult::Created(Project::new(game_dir));
                            }
                        });
                    });
                });
            });

        if open {
            result
        } else {
            NewProjectDialogResult::Canceled
        }
    }
}
