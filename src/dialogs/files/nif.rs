use camino::Utf8PathBuf;
use eframe::egui;
use nif::{blocks::Block, common::BlockRef, Nif};

use crate::widgets::nif::NifWidget;

use super::ProjectFileDialog;

#[derive(Debug)]
pub struct NifFileDialog {
    path: Utf8PathBuf,
    data: Nif,
    nif_widget: NifWidget,
    lod_distance: f32,
}

impl ProjectFileDialog for NifFileDialog {
    fn create(path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self {
        let data_buf = std::fs::read(&path).unwrap();
        let data = Nif::parse(&mut std::io::Cursor::new(data_buf)).unwrap();

        let render_state = frame.wgpu_render_state().unwrap();
        let mut nif_widget = NifWidget::new(render_state);
        nif_widget.set_nif(&data, render_state, 0.0, None, None);

        nif_widget.reset_camera_from_bounds();

        Self {
            path,
            data,
            nif_widget,
            lod_distance: 0.0,
        }
    }

    fn title(&self) -> String {
        self.path.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Self {
            data,
            nif_widget,
            lod_distance,
            ..
        } = self;

        ui.horizontal_top(|ui| {
            nif_widget.show(ui, frame, Some(egui::vec2(320.0, 0.0)));
            ui.separator();
            ui.vertical(|ui| {
                ui.label("Simulated distance (LOD)");
                if ui
                    .add(egui::Slider::new(lod_distance, 0.0..=500.0))
                    .changed()
                {
                    nif_widget.set_nif(
                        data,
                        frame.wgpu_render_state().unwrap(),
                        *lod_distance,
                        None,
                        None,
                    );
                }
                ui.separator();
                fn add_node(
                    ui: &mut egui::Ui,
                    block_ref: BlockRef,
                    block: &Block,
                    blocks: &[Block],
                ) {
                    let properties = block.properties(blocks).unwrap_or_default();
                    let children = block.children(blocks).unwrap_or_default();
                    let extra_data = block.extra_data(blocks).unwrap_or_default();

                    let mut specialized_data_blocks = Vec::new();
                    match block {
                        Block::NiTextureEffect(block) => {
                            if let Some(source_texture) = block.source_texture_ref.get(blocks) {
                                specialized_data_blocks
                                    .push((block.source_texture_ref, source_texture));
                            }
                        }
                        Block::NiTriShape(block) => {
                            if let Some(tri_shape_data) = block.data_ref.get(blocks) {
                                specialized_data_blocks.push((block.data_ref, tri_shape_data));
                            }
                        }
                        _ => {}
                    }

                    if !properties.is_empty()
                        || !children.is_empty()
                        || !extra_data.is_empty()
                        || !specialized_data_blocks.is_empty()
                    {
                        egui::CollapsingHeader::new(format!(
                            "{} (id {})",
                            block.name(),
                            block_ref.0
                        ))
                        .show(ui, |ui| {
                            ui.style_mut().wrap = Some(true);

                            for (property_ref, property_block) in properties {
                                ui.label(format!(
                                    "{} (id {})",
                                    property_block.name(),
                                    property_ref.0
                                ));
                            }

                            for (child_ref, child_block) in children {
                                add_node(ui, child_ref, child_block, blocks);
                            }

                            for (extra_data_ref, extra_data_block) in extra_data {
                                add_node(ui, extra_data_ref, extra_data_block, blocks);
                            }

                            for (specialized_data_ref, specialized_data_block) in
                                specialized_data_blocks
                            {
                                add_node(ui, specialized_data_ref, specialized_data_block, blocks);
                            }
                        });
                    } else {
                        ui.label(format!("{} (id {})", block.name(), block_ref.0));
                    }
                }
                let root_block = data.blocks.get(0).unwrap();
                add_node(ui, BlockRef(0), root_block, &data.blocks);
            });
        });
    }
}
