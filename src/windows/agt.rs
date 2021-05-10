use std::path::PathBuf;

use eframe::egui;
use slidetown::parsers::*;

use super::LevelModifierWindow;

#[derive(Debug)]
pub enum AgtEntry {
    LevelModifier(LevelModifierWindow),
    Unknown(agt::Entry),
}

#[derive(Debug)]
struct AgtData {
    pub header: agt::Header,
    pub entries: Vec<AgtEntry>,
}

#[derive(Debug)]
pub struct AgtWindow {
    open: bool,
    title: String,
    path: PathBuf,
    data: Option<AgtData>,
}

impl AgtWindow {
    pub fn new(path: PathBuf, title: Option<String>) -> Self {
        Self {
            open: false,
            title: title.unwrap_or(path.file_name().unwrap().to_string_lossy().to_string()),
            data: None,
            path,
        }
    }

    pub fn load(&mut self) {
        let npluto_key: &[u8] = &[
            0x01, 0x05, 0x06, 0x02, 0x04, 0x03, 0x07, 0x08, 0x01, 0x05, 0x06, 0x0F, 0x04, 0x03,
            0x07, 0x0C, 0x31, 0x85, 0x76, 0x39, 0x34, 0x3D, 0x30, 0xE8, 0x67, 0x36, 0x36, 0x32,
            0x3E, 0x33, 0x34, 0x3B, 0x11, 0x15, 0x16, 0x16, 0x14, 0x13, 0x1D, 0x18, 0x11, 0x03,
            0x06, 0x0C, 0x04, 0x03, 0x06, 0x08, 0x2E, 0x55, 0x26, 0x23, 0x2A, 0x23, 0x2E, 0x28,
            0x21, 0x21, 0x26, 0x27, 0x2E, 0x00, 0x2D, 0x2D, 0xCF, 0xA5, 0x06, 0x02, 0x04, 0x0F,
            0x07, 0x18, 0xE1, 0x15, 0x36, 0x18, 0x60, 0x13, 0x1A, 0x19, 0x11, 0x15, 0x16, 0x10,
            0x12, 0x13, 0x17, 0x38, 0xF1, 0x25,
        ];
        let mut agt_file = std::fs::File::open(&self.path).unwrap();
        let mut agt_reader = slidetown::parsers::agt::AgtReader::new(&mut agt_file, &npluto_key);

        let header = slidetown::parsers::agt::Header::parse(&mut agt_reader).unwrap();

        let mut raw_entries = slidetown::parsers::agt::Entry::parse_entries(
            &mut agt_reader,
            header.file_count as usize,
        )
        .unwrap();

        let mut agt_entries = Vec::new();

        loop {
            let popped = raw_entries.pop();
            if let Some(entry) = popped {
                let entry_filename = entry
                    .path
                    .split('\\')
                    .last()
                    .unwrap_or(&entry.path)
                    .to_lowercase();
                if entry_filename == "levelmodifier.dat" {
                    agt_entries.push(AgtEntry::LevelModifier(
                        LevelModifierWindow::from_agt_entry(self.path.clone(), entry),
                    ));
                } else {
                    agt_entries.push(AgtEntry::Unknown(entry));
                }
            } else {
                break;
            }
        }

        agt_entries.reverse();

        self.data = Some(AgtData {
            header,
            entries: agt_entries,
        });
    }

    pub fn display(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.open, &self.title);

        if !self.open {
            return;
        }

        if self.data.is_none() {
            self.load();
        }

        let data = self.data.as_mut().unwrap();

        egui::Window::new(&self.title)
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    ui.label(format!("{} files:", data.header.file_count));

                    for entry in data.entries.iter_mut() {
                        match entry {
                            AgtEntry::LevelModifier(ref mut lm_window) => {
                                lm_window.display(ctx, ui);
                            }
                            AgtEntry::Unknown(ref mut entry) => {
                                ui.label(&entry.path);
                            }
                        }
                    }
                });
            });
    }
}
