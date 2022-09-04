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
    eframe::run_native(
        "slidetown",
        native_options,
        Box::new(|cc| Box::new(app::SlidetownApp::new(cc))),
    );
}
