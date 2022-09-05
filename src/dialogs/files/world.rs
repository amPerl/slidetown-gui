use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    time::Instant,
};

use camino::Utf8PathBuf;
use eframe::egui;
use nif::Nif;
use slidetown::parsers::{lbf::Lbf, lf::Lf, lif::Lif, lof::Lof, loi::Loi};

use crate::widgets::nif::{untextured_mesh::UntexturedMeshInstance, NifWidget};

use super::ProjectFileDialog;

#[derive(Debug)]
pub struct WorldDirDialog {
    dir_path: Utf8PathBuf,
    nif_widget: NifWidget,
    available_tracks: Vec<String>,
}

impl WorldDirDialog {
    fn load_track(&mut self, name: &str, frame: &mut eframe::Frame) {
        let render_state = frame.wgpu_render_state().unwrap();
        let Self {
            dir_path,
            nif_widget,
            ..
        } = self;

        nif_widget.clear_nifs(render_state);

        let track_start = Instant::now();

        let mut enabled_blocks = HashSet::new();

        let lif_path = dir_path.join(name).join("terrain0.LIF");
        let file = File::open(&lif_path).unwrap();
        let mut reader = BufReader::new(file);
        let lif = Lif::read(&mut reader).unwrap();
        for block in lif.blocks.iter() {
            if block.unk > 0 {
                enabled_blocks.insert(block.index);
            }
        }

        let lf_path = dir_path.join("terrain0.lf");
        let file = File::open(&lf_path).unwrap();
        let mut reader = BufReader::new(file);
        let lf = Lf::read(&mut reader).unwrap();
        for block in lf
            .blocks
            .iter()
            .filter(|b| enabled_blocks.contains(&b.index))
        {
            reader
                .seek(SeekFrom::Start(block.file_offset as _))
                .unwrap();
            let mut nif_data = vec![0u8; block.file_length as _];
            reader.read_exact(&mut nif_data).unwrap();
            let nif = Nif::parse(&mut Cursor::new(nif_data)).unwrap();
            nif_widget.add_nif(&nif, render_state, 0.0, Some("terrain".into()), None);
        }

        let lbf_path = dir_path.join("blockObj0.LBF");
        let file = File::open(&lbf_path).unwrap();
        let mut reader = BufReader::new(file);
        let lbf = Lbf::parse(&mut reader).unwrap();
        for (_, block) in lbf
            .blocks
            .iter()
            .enumerate()
            .filter(|b| enabled_blocks.contains(&(b.0 as _)))
        {
            for object in block.objects.iter() {
                reader
                    .seek(SeekFrom::Start(object.file_offset as _))
                    .unwrap();
                let mut nif_data = vec![0u8; object.file_length as _];
                reader.read_exact(&mut nif_data).unwrap();
                let nif = Nif::parse(&mut Cursor::new(nif_data)).unwrap();
                nif_widget.add_nif(&nif, render_state, 0.0, Some("blockObj".into()), None);
            }
        }

        let mut instances_by_model_index: HashMap<u32, Vec<UntexturedMeshInstance>> =
            HashMap::new();

        let loi_path = dir_path.join(name).join("object0.loI");
        let file = File::open(&loi_path).unwrap();
        let mut reader = BufReader::new(file);
        let loi = Loi::read(&mut reader, lf.block_count as _).unwrap();
        for block in loi
            .blocks
            .iter()
            .filter(|b| enabled_blocks.contains(&b.block_index))
        {
            for object in block.objects.iter() {
                let position = glam::vec3(object.position.0, object.position.1, object.position.2);
                let rm = object.rotation;
                let rotation_mat = glam::Mat3::from_cols_array_2d(&[
                    [rm.0 .0, rm.0 .1, rm.0 .2],
                    [rm.1 .0, rm.1 .1, rm.1 .2],
                    [rm.2 .0, rm.2 .1, rm.2 .2],
                ])
                .transpose();
                let rotation = glam::Quat::from_mat3(&rotation_mat);
                let scale = object.scale;
                let entry = instances_by_model_index.entry(object.model_table_index);
                entry.or_default().push(UntexturedMeshInstance {
                    position,
                    rotation,
                    scale,
                });
                // if object.object_extra_index >= 0 {
                //     let extra = loi
                //         .object_extras
                //         .get(object.object_extra_index as usize)
                //         .unwrap();
                //     dbg!(&object.position, &extra.position);
                // }
            }
        }

        let lof_path = dir_path.join("modeltable0.LOF");
        let file = File::open(&lof_path).unwrap();
        let mut reader = BufReader::new(file);
        let lof = Lof::read_without_data(&mut reader).unwrap();
        for model in lof.models.iter() {
            if !instances_by_model_index.contains_key(&model.index) {
                continue;
            }
            reader
                .seek(SeekFrom::Start(model.file_offset as _))
                .unwrap();
            let mut nif_data = vec![0u8; model.file_length as _];
            reader.read_exact(&mut nif_data).unwrap();
            let nif = Nif::parse(&mut Cursor::new(nif_data)).unwrap();
            let instances = instances_by_model_index.remove(&model.index);
            nif_widget.add_nif(
                &nif,
                render_state,
                0.0,
                Some(format!("modeltable_{}_{}", model.index, model.file_name)),
                instances,
            );
        }

        eprintln!(
            "finished loading track {:?}! total elapsed {}s",
            name,
            track_start.elapsed().as_secs_f64()
        );
    }
}

impl ProjectFileDialog for WorldDirDialog {
    fn create(dir_path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self {
        let render_state = frame.wgpu_render_state().unwrap();

        let mut available_tracks = vec!["Main".to_string()];
        for i in 1.. {
            let name = format!("Track{}", i);
            let path = dir_path.join(&name);
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.is_dir() {
                    available_tracks.push(name);
                }
            } else {
                break;
            }
        }

        let mut me = Self {
            dir_path,
            nif_widget: NifWidget::new(render_state),
            available_tracks,
        };
        me.load_track("Main", frame);
        me
    }

    fn title(&self) -> String {
        self.dir_path.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Self { nif_widget, .. } = self;

        let available_tracks = self.available_tracks.clone();

        let mut selected_track = None;
        ui.horizontal_top(|ui| {
            nif_widget.show(ui, frame, Some(egui::vec2(100.0, 0.0)));
            ui.separator();
            ui.vertical(|ui| {
                for track in available_tracks {
                    if ui.button(&track).clicked() {
                        selected_track = Some(track);
                    }
                }
            });
        });
        if let Some(track) = selected_track {
            self.load_track(&track, frame);
        }
    }
}
