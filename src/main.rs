use std::str::FromStr;

use camino::Utf8PathBuf;

mod app;
mod dialogs;
mod project;
mod storage;
mod widgets;

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        ..Default::default()
    };
    let quick_open_path = std::env::args()
        .skip(1)
        .flat_map(|s| Utf8PathBuf::from_str(&s).ok())
        .next();
    eframe::run_native(
        "slidetown",
        native_options,
        Box::new(|cc| Box::new(app::SlidetownApp::new(cc, quick_open_path))),
    );
}
