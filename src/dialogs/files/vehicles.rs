use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
};

use camino::Utf8PathBuf;
use eframe::egui;
use nif::{blocks::Block, Nif};
use slidetown::{
    parsers::xlt::{
        vehicle_list::{VehicleList, VehicleListEntry},
        visual_item_list::{VisualItemCategory, VisualItemList, VisualItemListEntry},
        Xlt,
    },
    xlt::InitConfiguration,
};

use crate::widgets::nif::{untextured_mesh::UntexturedMeshInstance, NifWidget};

use super::ProjectFileDialog;

#[derive(Debug)]
pub struct VehiclesDialog {
    data_dir_path: Utf8PathBuf,
    nif_widget: NifWidget,
    init: InitConfiguration,
    active_vehicle: VehicleListEntry,
    active_vehicle_items: HashMap<VisualItemCategory, VisualItemListEntry>,
    active_vehicle_compatible_items: HashMap<VisualItemCategory, Vec<VisualItemListEntry>>,
}

impl VehiclesDialog {
    fn load_vehicle(&mut self, vehicle: &VehicleListEntry, reset: bool, frame: &mut eframe::Frame) {
        let render_state = frame.wgpu_render_state().unwrap();
        let Self {
            data_dir_path,
            nif_widget,
            init,
            active_vehicle,
            active_vehicle_items,
            active_vehicle_compatible_items,
            ..
        } = self;

        if reset || active_vehicle.id != vehicle.id {
            *active_vehicle = vehicle.clone();
            *active_vehicle_items = init
                .vehicle_default_items(vehicle.id)
                .into_iter()
                .map(|(k, v)| (k, v.clone()))
                .collect();
        }

        let mut compatible_items = HashMap::new();
        for entry in init.vehicle_compatible_items(vehicle.id) {
            compatible_items
                .entry(entry.category)
                .or_insert(Vec::new())
                .push(entry.clone());
        }
        *active_vehicle_compatible_items = compatible_items;

        nif_widget.clear_nifs(render_state);

        let mut wheels = Vec::new();

        let player_vehicle_dir = data_dir_path.join("vehicle").join("player");

        let body_nif_path = player_vehicle_dir.join(&vehicle.file_name);
        dbg!(&body_nif_path);
        let nif = Nif::parse(&mut BufReader::new(File::open(&body_nif_path).unwrap())).unwrap();
        nif_widget.add_nif(
            &nif,
            render_state,
            0.0,
            Some(vehicle.file_name.clone()),
            None,
        );

        for block in nif.blocks.iter() {
            if let Block::NiNode(ni_node) = block {
                if ni_node.name.value == "wheels" {
                    eprintln!("body has wheels block!");
                    if let Some(wheels_children) = block.children(&nif.blocks) {
                        wheels.clear();
                        for (child_ref, child) in wheels_children {
                            if let Block::NiNode(ni_node) = child {
                                wheels.push(ni_node.translation.clone());
                            }
                        }
                    }
                }
            }
        }

        let dm_nif_path = player_vehicle_dir.join(&vehicle.file_name.replace(".nif", "_dm.nif"));
        dbg!(&dm_nif_path);
        let nif = Nif::parse(&mut BufReader::new(File::open(&dm_nif_path).unwrap())).unwrap();

        for block in nif.blocks.iter() {
            if let Block::NiNode(ni_node) = block {
                if ni_node.name.value == "wheels" {
                    eprintln!("body dm has wheels block!");
                    if let Some(wheels_children) = block.children(&nif.blocks) {
                        wheels.clear();
                        for (child_ref, child) in wheels_children {
                            if let Block::NiNode(ni_node) = child {
                                wheels.push(ni_node.translation.clone());
                            }
                        }
                    }
                }
            }
        }

        for (category, item) in active_vehicle_items.iter() {
            let mut item_id_plus_nif = item.item_id.clone();
            item_id_plus_nif.push_str(".nif");

            let item_nif_path = match category {
                VisualItemCategory::AeroBumper => player_vehicle_dir.join(&item_id_plus_nif),
                VisualItemCategory::AeroHood => player_vehicle_dir.join(&item_id_plus_nif),
                VisualItemCategory::AeroKit => player_vehicle_dir.join(&item_id_plus_nif),
                // VisualItemCategory::Spoiler => Some(),
                _ => continue,
            };

            dbg!(&item_nif_path);
            let nif = Nif::parse(&mut BufReader::new(File::open(&item_nif_path).unwrap())).unwrap();
            nif_widget.add_nif(&nif, render_state, 0.0, Some(item_id_plus_nif), None);

            for block in nif.blocks.iter() {
                if let Block::NiNode(ni_node) = block {
                    if ni_node.name.value == "wheels" {
                        eprintln!("item {:?} has wheels block!", item.name);
                        if let Some(wheels_children) = block.children(&nif.blocks) {
                            wheels.clear();
                            for (child_ref, child) in wheels_children {
                                if let Block::NiNode(ni_node) = child {
                                    wheels.push(ni_node.translation.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(wheel) = active_vehicle_items.get(&VisualItemCategory::Tire) {
            let mut scale = 1.0;
            if let Some(tire_info) = init.tire_info(wheel.category_item_id) {
                let tire_diameter = (tire_info.diameter as f32) / 100.0;
                let vehicle_tire_diameter = (vehicle.tire_scale_front as f32) / 100.0;
                scale = vehicle_tire_diameter / tire_diameter;
            }

            let mut wheel_id_plus_nif = wheel.item_id.clone();
            wheel_id_plus_nif.push_str(".nif");
            let wheel_nif_path = player_vehicle_dir.join("wheel").join(&wheel_id_plus_nif);
            let nif =
                Nif::parse(&mut BufReader::new(File::open(&wheel_nif_path).unwrap())).unwrap();
            nif_widget.add_nif(
                &nif,
                render_state,
                0.0,
                Some(wheel_id_plus_nif),
                Some(
                    wheels
                        .iter()
                        .map(|t| UntexturedMeshInstance {
                            position: t.into(),
                            rotation: nif::glam::Quat::IDENTITY,
                            scale,
                        })
                        .collect(),
                ),
            )
        }
    }
}

impl ProjectFileDialog for VehiclesDialog {
    fn create(data_dir_path: Utf8PathBuf, frame: &mut eframe::Frame) -> Self {
        let render_state = frame.wgpu_render_state().unwrap();

        let init_path = data_dir_path.join("Init");
        let vehicle_list_bytes = std::fs::read(init_path.join("VehicleList.xlt")).unwrap();
        let vehicle_list_xlt = Xlt::read(&mut Cursor::new(&vehicle_list_bytes)).unwrap();
        let visual_item_list_bytes = std::fs::read(init_path.join("VisualItem.xlt")).unwrap();
        let visual_item_list_xlt = Xlt::read(&mut Cursor::new(&visual_item_list_bytes)).unwrap();
        let vshop_item_list_bytes = std::fs::read(init_path.join("VShopItem.xlt")).unwrap();
        let vshop_item_list_xlt = Xlt::read(&mut Cursor::new(&vshop_item_list_bytes)).unwrap();
        let tire_list_bytes = std::fs::read(init_path.join("TireList.xlt")).unwrap();
        let tire_list_xlt = Xlt::read(&mut Cursor::new(&tire_list_bytes)).unwrap();
        let spoiler_list_bytes = std::fs::read(init_path.join("SpoilerList.xlt")).unwrap();
        let spoiler_list_xlt = Xlt::read(&mut Cursor::new(&spoiler_list_bytes)).unwrap();

        let init = InitConfiguration::from_xlts(
            &vehicle_list_xlt,
            &visual_item_list_xlt,
            &vshop_item_list_xlt,
            &tire_list_xlt,
            &spoiler_list_xlt,
        )
        .unwrap();

        let first_vehicle = init.vehicle_list.entries[0].clone();

        let mut me = Self {
            data_dir_path,
            nif_widget: NifWidget::new(render_state),
            init,
            active_vehicle: first_vehicle.clone(),
            active_vehicle_items: Default::default(),
            active_vehicle_compatible_items: Default::default(),
        };
        me.load_vehicle(&first_vehicle, true, frame);
        me.nif_widget.reset_camera_from_bounds();
        me
    }

    fn title(&self) -> String {
        let parent_dir_name = self.data_dir_path.parent().unwrap().file_name().unwrap();
        format!("{} (Vehicles)", parent_dir_name)
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Self {
            nif_widget,
            active_vehicle,
            active_vehicle_items,
            active_vehicle_compatible_items,
            init,
            ..
        } = self;

        let mut newly_selected_item = None;
        let mut selected_vehicle_id = active_vehicle.id;

        ui.horizontal_top(|ui| {
            nif_widget.show(ui, frame, Some(egui::vec2(200.0, 0.0)));
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Select vehicle:");
                        egui::ComboBox::from_id_source("Select vehicle")
                            .selected_text(&active_vehicle.name)
                            .show_ui(ui, |ui| {
                                for vehicle in init.player_vehicles().iter() {
                                    if !vehicle.enabled || vehicle.aero_set {
                                        continue;
                                    }
                                    if vehicle.id == active_vehicle.id {
                                        continue;
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected_vehicle_id,
                                            vehicle.id,
                                            &vehicle.name,
                                        )
                                        .clicked()
                                    {
                                        selected_vehicle_id = vehicle.id;
                                    }
                                }
                            });
                        let mut categories =
                            active_vehicle_compatible_items.keys().collect::<Vec<_>>();
                        categories.sort_unstable_by_key(|c| format!("{:?}", c));
                        for category in categories {
                            let items = &active_vehicle_compatible_items[category];
                            ui.collapsing(format!("{:?}", category), |ui| {
                                for item in items {
                                    let mut was_enabled = false;
                                    let mut enabled = false;
                                    if let Some(active_item) = active_vehicle_items.get(category) {
                                        if active_item.id == item.id {
                                            enabled = true;
                                            was_enabled = true;
                                        }
                                    }
                                    ui.checkbox(&mut enabled, &item.name);
                                    if !was_enabled && enabled {
                                        newly_selected_item = Some(item.clone());
                                    }
                                }
                            });
                        }
                    });
                });
        });
        if selected_vehicle_id != active_vehicle.id {
            let selected_vehicle = init
                .player_vehicles()
                .iter()
                .find(|v| v.id == selected_vehicle_id)
                .map(|v| (*v).clone())
                .unwrap();
            self.load_vehicle(&selected_vehicle, true, frame);
        } else if let Some(item) = newly_selected_item {
            active_vehicle_items.insert(item.category, item);
            let vehicle = active_vehicle.clone();
            self.load_vehicle(&vehicle, false, frame);
        }
    }
}
