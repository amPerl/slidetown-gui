use camino::Utf8PathBuf;
use eframe::egui;

use crate::project::ProjectFilesEntry;

pub mod lbf;
pub mod levelmodifier;
pub mod lf;
pub mod nif;
pub mod vehicles;
pub mod world;

pub fn create_dialog_for_file(
    path: &Utf8PathBuf,
    frame: &mut eframe::Frame,
) -> Box<dyn ProjectFileDialog> {
    let file_name = path.file_stem().map(str::to_lowercase);
    let extension = path.extension().map(str::to_lowercase);

    match (file_name.as_deref(), extension.as_deref()) {
        (Some("levelmodifier") | Some("oldlevelmodifier"), Some("dat")) => Box::new(
            levelmodifier::LevelmodifierFileDialog::create(path.clone(), frame),
        ),
        (_, Some("nif")) => Box::new(nif::NifFileDialog::create(path.clone(), frame)),
        (_, Some("lf")) => Box::new(lf::LfFileDialog::create(path.clone(), frame)),
        (_, Some("lbf")) => Box::new(lbf::LbfFileDialog::create(path.clone(), frame)),
        (None, _) => panic!("no filename ({:?}, {:?})", file_name, extension),
        (_, _) => Box::new(PlaceholderFileDialog::create(path.clone(), frame)),
    }
}

pub enum DirDialogKind {
    World,
    Vehicles,
}

impl std::fmt::Display for DirDialogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirDialogKind::World => f.write_str("World"),
            DirDialogKind::Vehicles => f.write_str("Vehicles"),
        }
    }
}

impl DirDialogKind {
    pub fn create_dialog(
        &self,
        path: &Utf8PathBuf,
        frame: &mut eframe::Frame,
    ) -> Box<dyn ProjectFileDialog> {
        match self {
            DirDialogKind::World => Box::new(world::WorldDirDialog::create(path.clone(), frame)),
            DirDialogKind::Vehicles => {
                Box::new(vehicles::VehiclesDialog::create(path.clone(), frame))
            }
        }
    }
}

pub fn get_dir_dialog(_path: &Utf8PathBuf, entries: &[ProjectFilesEntry]) -> Option<DirDialogKind> {
    // let file_name = path.file_name().map(str::to_lowercase).unwrap();

    fn has_file(name: &str, entries: &[ProjectFilesEntry]) -> bool {
        entries.iter().any(|e| match e {
            ProjectFilesEntry::File(f) => f.file_name().unwrap() == name,
            _ => false,
        })
    }

    fn has_dir(name: &str, entries: &[ProjectFilesEntry]) -> bool {
        entries.iter().any(|e| match e {
            ProjectFilesEntry::Directory((d, _)) => d.file_name().unwrap() == name,
            _ => false,
        })
    }

    if has_file("terrain0.lf", entries) && has_file("blockObj0.LBF", entries) {
        return Some(DirDialogKind::World);
    }

    if has_dir("vehicle", entries) && has_dir("Init", entries) {
        return Some(DirDialogKind::Vehicles);
    }

    None
}

pub trait ProjectFileDialog: std::fmt::Debug {
    fn create(path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self
    where
        Self: Sized;
    fn title(&self) -> String;
    fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame);
}

#[derive(Debug)]
pub struct PlaceholderFileDialog(Utf8PathBuf);

impl ProjectFileDialog for PlaceholderFileDialog {
    fn create(path: Utf8PathBuf, _frame: &mut eframe::Frame) -> Self {
        Self(path)
    }

    fn title(&self) -> String {
        self.0.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("There is no defined viewer for this file type.");
    }
}
