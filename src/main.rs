// Deepslate Studio: editor de skins de escritorio. Licencia GPLv3.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod editor;
mod i18n;
mod project;
mod settings;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Deepslate Studio")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Deepslate Studio",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
}
