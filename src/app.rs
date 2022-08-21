use camino::{Utf8Path, Utf8PathBuf};
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::{
    dialogs::new_project::{NewProjectDialog, NewProjectDialogResult},
    project::{Project, ProjectFilePath},
    storage,
};

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct SlidetownApp {
    recent_projects: Vec<ProjectFilePath>,
    current_project: Option<Project>,

    new_project_dialog: Option<NewProjectDialog>,
}

impl SlidetownApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for SlidetownApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.style_mut().spacing.item_spacing.y = 4.0;

                    if ui.button("New Project").clicked() {
                        self.create_new_project_dialog();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Open Project").clicked() {
                        self.prompt_open_project_file();
                        ui.close_menu();
                    }

                    let recent_projects = self.recent_projects.clone();
                    ui.menu_button("Open Recent", |ui| {
                        for recent_project in recent_projects.iter() {
                            let project_path_str = recent_project.as_str();
                            let button = egui::Button::new(project_path_str).wrap(false);
                            if ui.add(button).clicked() {
                                self.open_project_file(recent_project);
                                ui.close_menu();
                            }
                        }
                    });

                    ui.separator();

                    let save_button = egui::Button::new("Save Project");
                    if ui
                        .add_enabled(self.current_project.is_some(), save_button)
                        .clicked()
                    {
                        match self.save_project() {
                            Ok(_) => {
                                self.bump_recent_project(None);
                            }
                            Err(err) => {
                                eprintln!("Failed to save project: {:?}", err);
                            }
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        let mut new_project = None;
        if let Some(mut new_project_dialog) = self.new_project_dialog.take() {
            match new_project_dialog.show(ctx) {
                NewProjectDialogResult::Created(project) => {
                    new_project = Some(project);
                }
                NewProjectDialogResult::Idle => {
                    self.new_project_dialog = Some(new_project_dialog);
                }
                _ => {}
            }
        }
        if let Some(new_project) = new_project.take() {
            self.set_current_project(new_project);
            self.new_project_dialog = None;
        }
    }
}

impl SlidetownApp {
    fn bump_recent_project(&mut self, project: Option<&Project>) {
        let project = project.or(self.current_project.as_ref()).unwrap();
        if let Some(path) = project.last_path() {
            let existing_idx = self
                .recent_projects
                .iter()
                .enumerate()
                .find(|(_idx, el)| path == **el)
                .map(|t| t.0);
            if let Some(existing_idx) = existing_idx {
                let existing = self.recent_projects.remove(existing_idx);
                self.recent_projects.insert(0, existing);
            } else {
                self.recent_projects.insert(0, path);
            }
        }
    }

    fn create_new_project_dialog(&mut self) {
        self.new_project_dialog = Some(NewProjectDialog::new());
    }

    fn open_project_file(&mut self, path: &Utf8Path) {
        match Project::from_file(path) {
            Ok(project) => self.set_current_project(project),
            Err(err) => eprintln!("failed to load project from file: {:?}", err),
        }
    }

    fn prompt_open_project_file(&mut self) {
        if let Some(path) = storage::prompt_open_project_file() {
            self.open_project_file(&path)
        }
    }

    fn set_current_project(&mut self, project: Project) {
        self.bump_recent_project(Some(&project));
        self.current_project = Some(project);
    }

    fn save_project(&mut self) -> anyhow::Result<()> {
        if let Some(project) = self.current_project.as_mut() {
            if let Some(path) = storage::prompt_save_project_file(project.last_path()) {
                project.save_file(path)?;
            }
        }

        Ok(())
    }
}
