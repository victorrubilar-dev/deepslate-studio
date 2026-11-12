// Deepslate Studio: editor de skins de escritorio. Licencia GPLv3.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod editor;
mod i18n;
mod library;
mod project;
mod render3d;
mod settings;
mod ui;

fn icon() -> eframe::egui::IconData {
    let img = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("el ícono embebido es un PNG válido")
        .into_rgba8();
    let (width, height) = img.dimensions();
    eframe::egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result {
    // winit 0.30 no recibe archivos arrastrados en Wayland. Si hay XWayland, se abre con X11
    // para que arrastrar y soltar funcione.
    // shortcut: sacar esto cuando winit soporte arrastrar y soltar en Wayland.
    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_some() && std::env::var_os("WAYLAND_DISPLAY").is_some() {
        // SAFETY: es lo primero de `main`, antes de crear cualquier hilo.
        unsafe { std::env::remove_var("WAYLAND_DISPLAY") };
    }
    if let Some(p) = settings::log::path() {
        settings::log::init(&p);
    }
    log::info!("Deepslate Studio {}", env!("CARGO_PKG_VERSION"));
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Deepslate Studio")
            // Debe coincidir con el nombre del archivo .desktop para que Wayland muestre el ícono.
            .with_app_id("deepslate-studio")
            .with_inner_size([1100.0, 760.0])
            .with_icon(icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Deepslate Studio",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
}
