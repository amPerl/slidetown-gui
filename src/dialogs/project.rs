use std::{collections::HashMap, fmt::Debug};

use camino::Utf8PathBuf;
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::{
    dialogs::files::{create_dialog_for_file, get_dir_dialog, ProjectFileDialog},
    project::{Project, ProjectFilesEntry},
};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ProjectDialog {
    project: Project,
    #[serde(skip)]
    files: Option<Vec<ProjectFilesEntry>>,
    #[serde(skip)]
    open_files: HashMap<Utf8PathBuf, Box<dyn ProjectFileDialog>>,
    #[serde(skip)]
    active_file: Option<Utf8PathBuf>,
}

impl ProjectDialog {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            files: None,
            open_files: Default::default(),
            active_file: None,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn project_mut(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            project,
            files,
            open_files,
            active_file,
        } = self;

        egui::SidePanel::new(egui::panel::Side::Left, "project_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label("Project");
                ui.heading(project.game_dir().file_name().unwrap_or("untitled"));
                ui.separator();
                egui::containers::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if files.is_none() {
                            *files = Some(project.enumerate_files());
                        } else {
                            let entries = files.as_ref().unwrap();
                            fn render_entries(
                                ui: &mut egui::Ui,
                                frame: &mut eframe::Frame,
                                entries: &[ProjectFilesEntry],
                                open_entries: &mut HashMap<Utf8PathBuf, Box<dyn ProjectFileDialog>>,
                            ) {
                                for entry in entries {
                                    match entry {
                                        ProjectFilesEntry::File(file) => {
                                            let mut is_open = open_entries.contains_key(file);
                                            let was_open = is_open;
                                            ui.checkbox(&mut is_open, file.file_name().unwrap());
                                            if was_open && !is_open {
                                                open_entries.remove(file);
                                            } else if !was_open && is_open {
                                                open_entries.insert(
                                                    file.clone(),
                                                    create_dialog_for_file(file, frame),
                                                );
                                            }
                                        }
                                        ProjectFilesEntry::Directory((dir, inner_entries)) => {
                                            ui.collapsing(dir.file_name().unwrap(), |ui| {
                                                match get_dir_dialog(dir, inner_entries) {
                                                    Some(kind) => {
                                                        let mut is_open =
                                                            open_entries.contains_key(dir);
                                                        let was_open = is_open;
                                                        ui.checkbox(
                                                            &mut is_open,
                                                            format!(
                                                                "({}) {}",
                                                                kind,
                                                                dir.file_name().unwrap()
                                                            ),
                                                        );
                                                        if was_open && !is_open {
                                                            open_entries.remove(dir);
                                                        } else if !was_open && is_open {
                                                            open_entries.insert(
                                                                dir.clone(),
                                                                kind.create_dialog(dir, frame),
                                                            );
                                                        }
                                                    }
                                                    None => {}
                                                }
                                                render_entries(
                                                    ui,
                                                    frame,
                                                    inner_entries,
                                                    open_entries,
                                                );
                                            });
                                        }
                                    }
                                }
                            }
                            render_entries(ui, frame, entries, open_files);
                        }
                    });
            });

        if !open_files.is_empty() {
            // reset active_file in case the first file was just opened or the currently active file was closed
            if active_file.is_none() || !open_files.contains_key(active_file.as_ref().unwrap()) {
                *active_file = Some(open_files.iter().next().unwrap().0.clone());
            }

            egui::CentralPanel::default()
                .frame(egui::Frame {
                    inner_margin: egui::style::Margin::same(0.0),
                    rounding: eframe::epaint::Rounding::none(),
                    fill: ctx.style().visuals.window_fill(),
                    stroke: Default::default(),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    egui::Frame {
                        inner_margin: egui::style::Margin {
                            left: 4.0,
                            top: 4.0,
                            right: 4.0,
                            bottom: 2.0,
                        },
                        stroke: egui::Stroke::none(),
                        shadow: egui::epaint::Shadow {
                            extrusion: 0.0,
                            color: egui::Color32::TRANSPARENT,
                        },
                        ..egui::Frame::menu(ui.style())
                    }
                    .show(ui, |ui| {
                        ui.set_width(ui.available_size_before_wrap().x);
                        ui.horizontal_wrapped(|ui| {
                            for (key, dialog) in open_files.iter_mut() {
                                if ui
                                    .selectable_label(
                                        active_file.as_ref().unwrap() == key,
                                        dialog.title(),
                                    )
                                    .clicked()
                                {
                                    *active_file = Some(key.clone());
                                }
                            }
                        });
                    });

                    ui.add(
                        egui::Separator::default()
                            .spacing(ui.style().visuals.window_stroke().width),
                    );

                    egui::Frame {
                        inner_margin: egui::style::Margin::symmetric(4.0, 2.0),
                        ..egui::Frame::default()
                    }
                    .show(ui, |ui| {
                        if let Some(active_file) = active_file {
                            open_files
                                .get_mut(active_file)
                                .unwrap()
                                .show(ctx, ui, frame);
                        }
                    });
                    // let mut closed_dialogs = Vec::new();
                    // for (key, dialog) in open_files.iter_mut() {
                    //     if let ProjectFileDialogResult::Closed = dialog.show(ctx, ui) {
                    //         closed_dialogs.push(key.clone());
                    //     }
                    // }
                    // for closed_dialog in closed_dialogs.into_iter() {
                    //     open_files.remove(&closed_dialog);
                    // }
                });
        }
    }
}
