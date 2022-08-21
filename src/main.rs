mod app;
mod dialogs;
mod project;
mod storage;

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "slidetown",
        native_options,
        Box::new(|cc| Box::new(app::SlidetownApp::new(cc))),
    );
}
