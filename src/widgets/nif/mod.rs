use std::sync::Arc;

use dolly::{
    prelude::{Position, Smooth, YawPitch},
    rig::CameraRig,
};
use eframe::{egui, egui_wgpu};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};
use nif::Nif;

use self::{
    camera::Camera, light::Light, nif_render_resources::NifRenderResources,
    untextured_mesh::UntexturedMeshInstance,
};

mod camera;
mod light;
mod nif_render_resources;
mod texture;
pub mod untextured_mesh;
mod untextured_mesh_pipeline;

#[derive(Debug)]
pub struct NifWidget {
    light: Light,
    dolly_camera: CameraRig,
    camera: Camera,
    model_rotation: glam::Quat,
}

impl NifWidget {
    pub fn new(render_state: &eframe::egui_wgpu::RenderState) -> Self {
        let light = Light::default();
        let camera = Camera::default();
        let nif_render_resources = NifRenderResources::new(
            &render_state.device,
            render_state.target_format,
            &light,
            &camera,
        );

        render_state
            .egui_rpass
            .write()
            .paint_callback_resources
            .insert(nif_render_resources);

        Self {
            light,
            dolly_camera: CameraRig::builder()
                .with(Position::new(dolly::glam::Vec3::Z * 100.0))
                .with(YawPitch::new().yaw_degrees(135.0).pitch_degrees(-45.0))
                .with(Smooth::new_position_rotation(1.0, 1.0))
                .build(),
            camera,
            model_rotation: glam::Quat::IDENTITY,
        }
    }

    pub fn clear_nifs(&mut self, render_state: &eframe::egui_wgpu::RenderState) {
        let paint_callback_resources =
            &mut render_state.egui_rpass.write().paint_callback_resources;

        let nif_render_resources = paint_callback_resources
            .get_mut::<NifRenderResources>()
            .unwrap();

        nif_render_resources.clear_nifs();
    }

    pub fn set_nif(
        &mut self,
        nif: &Nif,
        render_state: &eframe::egui_wgpu::RenderState,
        lod_distance: f32,
        group: Option<String>,
        instances: Option<Vec<UntexturedMeshInstance>>,
    ) {
        let paint_callback_resources =
            &mut render_state.egui_rpass.write().paint_callback_resources;

        let nif_render_resources = paint_callback_resources
            .get_mut::<NifRenderResources>()
            .unwrap();

        nif_render_resources.set_nif(nif, lod_distance, group, instances);

        let bounds = nif_render_resources.combined_bounds;
        let horiz_distance = bounds[0].max(bounds[1]) / 2.0;
        self.dolly_camera.driver_mut::<Position>().position =
            dolly::glam::Vec3::new(horiz_distance, horiz_distance, bounds[2] * 2.0);
    }

    pub fn add_nif(
        &mut self,
        nif: &Nif,
        render_state: &eframe::egui_wgpu::RenderState,
        lod_distance: f32,
        group: Option<String>,
        instances: Option<Vec<UntexturedMeshInstance>>,
    ) {
        let paint_callback_resources =
            &mut render_state.egui_rpass.write().paint_callback_resources;

        let nif_render_resources = paint_callback_resources
            .get_mut::<NifRenderResources>()
            .unwrap();

        nif_render_resources.add_nif(nif, lod_distance, group, instances);

        let bounds = nif_render_resources.combined_bounds;
        let horiz_distance = bounds[0].max(bounds[1]) / 2.0;
        self.dolly_camera.driver_mut::<Position>().position =
            dolly::glam::Vec3::new(horiz_distance, horiz_distance, bounds[2] * 2.0);
        self.light.position = [bounds[0] * -2.0, bounds[1] * 2.0, bounds[2] * 2.0];
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
        space_aside: Option<egui::Vec2>,
    ) {
        let dt = ui.input().stable_dt;

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // Create canvas
            let mut canvas_size = ui.available_size_before_wrap();
            if let Some(space_aside) = space_aside {
                canvas_size -= space_aside;
            }
            let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::drag());

            // Set new aspect ratio, canvas could've been resized
            self.camera.aspect_ratio = rect.aspect_ratio();

            let dolly_transform = {
                // rotation from dragging mouse
                let drag_horizontal = response.drag_delta().x.min(359.0);
                let drag_vertical = response.drag_delta().y.min(359.0);
                let yaw_speed = -1.0 / 3.0;
                let pitch_speed = -1.0 / 3.0;

                // position from keyboard
                let input = ui.input();
                let forward = if input.key_down(egui::Key::W) {
                    1.0
                } else if input.key_down(egui::Key::S) {
                    -1.0
                } else {
                    0.0
                };
                let right = if input.key_down(egui::Key::D) {
                    1.0
                } else if input.key_down(egui::Key::A) {
                    -1.0
                } else {
                    0.0
                };
                let up = if input.key_down(egui::Key::Q) {
                    1.0
                } else if input.key_down(egui::Key::E) {
                    -1.0
                } else {
                    0.0
                };
                let boost = if input.modifiers.shift { 1.0 } else { 0.0 };

                let move_vec = self.dolly_camera.final_transform.rotation
                    * dolly::glam::Vec3::new(right, forward, up).clamp_length_max(1.0)
                    * 10.0f32.powf(boost);

                self.dolly_camera
                    .driver_mut::<YawPitch>()
                    .rotate_yaw_pitch(drag_horizontal * yaw_speed, drag_vertical * pitch_speed);

                self.dolly_camera
                    .driver_mut::<Position>()
                    .translate(move_vec * dt * 100.0);

                self.dolly_camera.update(dt)
            };

            self.camera.eye = dolly_transform.position;
            self.camera.up = dolly_transform.up();
            self.camera.target = dolly_transform.position + dolly_transform.forward();

            // Scroll to zoom in and out
            // if response.hovered() {
            //     let mut speed_mult = 0.02;
            //     if ui.input().modifiers.alt {
            //         speed_mult = 0.2;
            //     }
            //     self.camera.eye.z += ui.input().scroll_delta.y * speed_mult;
            // }
            // self.camera.eye.x = (self.camera_yaw.cos() * self.camera_distance) as f32;
            // self.camera.eye.y = (self.camera_yaw.sin() * self.camera_distance) as f32;

            // let light_yaw = self.camera_yaw as f32 + FRAC_PI_2 + FRAC_PI_4;
            // self.light.position[0] = light_yaw.cos() * self.camera_distance as f32 * 2.0;
            // self.light.position[1] = light_yaw.sin() * self.camera_distance as f32 * 2.0;

            let camera = self.camera.clone();
            let light = self.light;
            let model_rotation = self.model_rotation;

            // Wgpu rendering
            let cb = egui_wgpu::CallbackFn::new()
                .prepare(move |device, queue, paint_callback_resources| {
                    let resources: &mut NifRenderResources =
                        paint_callback_resources.get_mut().unwrap();
                    resources.prepare(device, queue, &camera, &light, &model_rotation);
                })
                .paint(move |_info, rpass, paint_callback_resources| {
                    let resources: &NifRenderResources = paint_callback_resources.get().unwrap();
                    resources.paint(rpass)
                });
            let callback = egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            };
            ui.painter().add(callback);

            let gizmo = Gizmo::new("nif_gizmo")
                .view_matrix(self.camera.build_view_matrix().to_cols_array_2d())
                .projection_matrix(self.camera.build_projection_matrix().to_cols_array_2d())
                .model_matrix(glam::Mat4::from_quat(model_rotation).to_cols_array_2d())
                .orientation(GizmoOrientation::Local)
                .mode(GizmoMode::Rotate)
                .viewport(rect);

            if let Some(gizmo_response) = gizmo.interact(ui) {
                let transform_mat: glam::Mat4 =
                    glam::Mat4::from_cols_array_2d(&gizmo_response.transform);
                let transform_rot = transform_mat.to_scale_rotation_translation().1;
                self.model_rotation = transform_rot;
            }

            // Keep animating (non-reactive)
            response.ctx.request_repaint();
        });
    }
}
