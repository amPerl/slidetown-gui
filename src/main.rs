mod app;
mod windows;

fn main() {
    let app = app::SlidetownApp::default();
    eframe::run_native(Box::new(app));
}
