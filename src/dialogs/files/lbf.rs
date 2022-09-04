use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    time::Instant,
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
    data: Lbf,
    nif_widget: NifWidget,
}

impl ProjectFileDialog for LbfFileDialog {
    fn create(path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self {
        let file = File::open(&path).unwrap();
        let mut reader = BufReader::new(file);

        let render_state = frame.wgpu_render_state().unwrap();
        let mut nif_widget = NifWidget::new(render_state);

        let lbf_start = Instant::now();

        let data = Lbf::parse(&mut reader).unwrap();
        for block in data.blocks.iter() {
            // let block_start = Instant::now();

            for object in block.objects.iter() {
                reader
                    .seek(SeekFrom::Start(object.file_offset as _))
                    .unwrap();

                // let seek_time = block_start.elapsed().as_micros();

                let mut nif_data = vec![0u8; object.file_length as _];
                reader.read_exact(&mut nif_data).unwrap();

                // let buffer_time = block_start.elapsed().as_micros() - seek_time;

                let nif = Nif::parse(&mut Cursor::new(nif_data)).unwrap();

                // let parse_time = block_start.elapsed().as_micros() - buffer_time;

                nif_widget.add_nif(&nif, render_state, 0.0, None, None);
            }

            // let add_time = block_start.elapsed().as_micros() - parse_time;
            // eprintln!(
            //     "added block at {:?}, seek {}us, buffer {}us, parse {}us, add {}us",
            //     (block.position_x, block.position_y),
            //     seek_time,
            //     buffer_time,
            //     parse_time,
            //     add_time
            // );
        }

        eprintln!(
            "finished! total elapsed {}s",
            lbf_start.elapsed().as_secs_f64()
        );

        Self {
            path,
            data,
            nif_widget,
        }
    }

    fn title(&self) -> String {
        self.path.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Self {
            data: _,
            nif_widget,
            ..
        } = self;

        ui.horizontal_top(|ui| {
            nif_widget.show(ui, frame, None);
        });
    }
}
