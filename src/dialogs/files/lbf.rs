use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
};

use camino::Utf8PathBuf;
use eframe::egui;
use nif::Nif;
use slidetown::parsers::lbf::Lbf;

use crate::widgets::nif::NifWidget;

use super::ProjectFileDialog;

#[derive(Debug)]
pub struct LbfFileDialog {
    path: Utf8PathBuf,
    nif_widget: NifWidget,
}

impl ProjectFileDialog for LbfFileDialog {
    fn create(path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self {
        let file = File::open(&path).unwrap();
        let mut reader = BufReader::new(file);

        let render_state = frame.wgpu_render_state().unwrap();
        let mut nif_widget = NifWidget::new(render_state);

        let data = Lbf::parse(&mut reader).unwrap();
        for block in data.blocks.iter() {
            for object in block.objects.iter() {
                reader
                    .seek(SeekFrom::Start(object.file_offset as _))
                    .unwrap();

                let mut nif_data = vec![0u8; object.file_length as _];
                reader.read_exact(&mut nif_data).unwrap();

                let nif = Nif::parse(&mut Cursor::new(nif_data)).unwrap();

                nif_widget.add_nif(&nif, render_state, 0.0, None, None);
            }
        }

        nif_widget.reset_camera_from_bounds();

        Self { path, nif_widget }
    }

    fn title(&self) -> String {
        self.path.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Self { nif_widget, .. } = self;

        ui.horizontal_top(|ui| {
            nif_widget.show(ui, frame, None);
        });
    }
}
