use std::{str::FromStr, sync::Arc};

use camino::Utf8PathBuf;
use eframe::{egui, App, CreationContext};
use nif::{
    glam::{EulerRot, Quat, Vec3},
    Nif,
};

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        ..Default::default()
    };

    let path = std::env::args()
        .nth(1)
        .map(|s| Utf8PathBuf::from_str(&s).expect("invalid nif path"))
        .expect("no nif path");

    if path.extension().unwrap().to_lowercase() != "nif" {
        panic!("not a nif");
    }

    let nif_buf = std::fs::read(&path).unwrap();
    let nif = Nif::parse(&mut std::io::Cursor::new(nif_buf)).unwrap();

    eframe::run_native(
        "notskope",
        native_options,
        Box::new(|cc| Box::new(NifViewerApp::new(cc, nif))),
    );
}

struct NifViewerApp {
    nif: Nif,
    camera_distance: f32,
    camera_yaw: f32,
    camera_pitch: f32,
}

impl NifViewerApp {
    pub fn new(cc: &CreationContext, nif: Nif) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            nif,
            camera_distance: 5.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
        }
    }
}

impl App for NifViewerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let Self {
            nif,
            camera_distance,
            camera_yaw,
            camera_pitch,
            ..
        } = self;

        let render_state = frame.wgpu_render_state().unwrap();
        let mut egui_renderer = render_state.renderer.write();
        let renderer = egui_renderer
            .paint_callback_resources
            .entry()
            .or_insert_with(|| {
                let mut renderer = nif_wgpu::renderer::Renderer::new(
                    &render_state.device,
                    render_state.target_format,
                );
                let mesh = renderer.add_nif(nif, 0.0);
                renderer
                    .add_instance(
                        mesh,
                        nif_wgpu::glam::vec3(0.0, 0.0, 0.0),
                        nif_wgpu::glam::Quat::IDENTITY,
                        1.0,
                    )
                    .unwrap();
                renderer
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size_before_wrap();
            let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());

            *camera_yaw += response.drag_delta().x / 100.0;
            *camera_yaw %= std::f32::consts::TAU;
            *camera_pitch += response.drag_delta().y / -100.0;
            *camera_pitch = camera_pitch
                .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2)
                * 0.9999;

            let scroll_delta = ui.input().scroll_delta;
            if scroll_delta.length() > 0.0 {
                *camera_distance -= ui.input().scroll_delta.y / 50.0;
            }

            let pitch_quat = Quat::from_rotation_x(*camera_pitch);
            let yaw_quat = Quat::from_rotation_z(-*camera_yaw);
            let arm = -Vec3::Y * *camera_distance;

            renderer.camera.aspect_ratio = rect.aspect_ratio();
            renderer.camera.eye = yaw_quat.mul_quat(pitch_quat).mul_vec3(arm);

            let callback_fn = eframe::egui_wgpu::CallbackFn::new()
                .prepare(move |device, queue, _encoder, resources| {
                    let renderer: &mut nif_wgpu::renderer::Renderer = resources.get_mut().unwrap();
                    renderer.camera.update(queue);
                    renderer.prepare(device);
                    Default::default()
                })
                .paint(|_info, rpass, resources| {
                    let renderer: &nif_wgpu::renderer::Renderer = resources.get().unwrap();
                    renderer.render(rpass);
                });
            let callback = egui::PaintCallback {
                rect,
                callback: Arc::new(callback_fn),
            };
            ui.painter().add(callback);
        });
    }
}
