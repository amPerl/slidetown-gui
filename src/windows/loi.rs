use std::path::PathBuf;

use eframe::egui;
use slidetown::parsers::agt;
use slidetown::parsers::loi;

#[derive(Debug)]
pub struct LoiWindow {
    open: bool,
    title: String,
    path: PathBuf,
    agt_entry: agt::Entry,
    data: Option<loi::Loi>,
    show_empty_blocks: bool,
}

impl LoiWindow {
    pub fn from_agt_entry(path: PathBuf, agt_entry: agt::Entry) -> Self {
        Self {
            open: false,
            title: agt_entry.path.clone(),
            path,
            agt_entry,
            data: None,
            show_empty_blocks: false,
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

        let loi_bytes = agt_reader.read_entry(&self.agt_entry).unwrap();
        let mut loi_cursor = std::io::Cursor::new(loi_bytes);

        let loi = loi::Loi::parse(&mut loi_cursor).unwrap();

        self.title = format!("{} ({})", self.agt_entry.path, loi.header.version_date);

        self.data = Some(loi);
    }

    pub fn display(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.open, &self.title);

        if !self.open {
            return;
        }

        if self.data.is_none() {
            self.load();
        }

        let show_empty_blocks = &mut self.show_empty_blocks;
        let data = self.data.as_mut().unwrap();

        egui::Window::new(&self.title)
            .open(&mut self.open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(show_empty_blocks, "Show empty blocks");
                        ui.label(format!("{} blocks", data.blocks.len()));

                        let block_iter = data.blocks.iter().filter(|b| {
                            if *show_empty_blocks {
                                true
                            } else {
                                b.object_count > 0
                            }
                        });

                        for block in block_iter {
                            ui.collapsing(
                                format!(
                                    "Block {} ({} objects)",
                                    block.block_index, block.object_count
                                ),
                                |ui| {
                                    for object in block.objects.iter() {
                                        ui.collapsing(
                                            format!(
                                                "Object {} model {} ({}, {}, {})",
                                                object.object_index,
                                                object.model_table_index,
                                                object.position.0,
                                                object.position.1,
                                                object.position.2
                                            ),
                                            |ui| {
                                                ui.label(format!(
                                                    "Extra data index: {}",
                                                    object.object_extra_index
                                                ));
                                                ui.label(format!("unknown1: {}", object.unknown1));
                                                ui.label(format!("unknown2: {}", object.unknown2));
                                                ui.label(format!("unknown3: {}", object.unknown3));
                                                ui.label(format!("unknown4: {}", object.unknown4));
                                                ui.label(format!("unknown7: {}", object.unknown7));
                                                ui.label(format!("unknown8: {}", object.unknown8));
                                                ui.label(format!("unknown9: {}", object.unknown9));
                                                ui.label(format!(
                                                    "unknown11: {}",
                                                    object.unknown11
                                                ));
                                            },
                                        );
                                    }
                                },
                            );
                        }
                    });
                });
            });
    }
}
