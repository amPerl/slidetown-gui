use camino::Utf8PathBuf;

pub fn prompt_game_directory() -> Option<Utf8PathBuf> {
    native_dialog::FileDialog::new()
        .show_open_single_dir()
        .ok()
        .flatten()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
}

pub fn prompt_open_project_file() -> Option<Utf8PathBuf> {
    native_dialog::FileDialog::new()
        .add_filter("Slidetown Project", &["stproj"])
        .show_open_single_file()
        .ok()
        .flatten()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
}

pub fn prompt_save_project_file(existing_path: Option<Utf8PathBuf>) -> Option<Utf8PathBuf> {
    let mut dialog = native_dialog::FileDialog::new().add_filter("Slidetown Project", &["stproj"]);

    if let Some(existing_path) = existing_path.as_ref() {
        dialog = dialog.set_location(existing_path);
    }

    dialog
        .show_save_single_file()
        .ok()
        .flatten()
        .map(|path| Utf8PathBuf::from_path_buf(path).ok())
        .flatten()
}
