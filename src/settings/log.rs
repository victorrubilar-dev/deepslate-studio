//! Registro en un archivo de texto, para «Ajustes → Avanzado → Ver registro».
//! Se vacía en cada arranque. De otras librerías (wgpu, etc.) solo guarda avisos y errores.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct FileLogger(Mutex<std::fs::File>);

impl log::Log for FileLogger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        let own = m.target().starts_with(env!("CARGO_CRATE_NAME"));
        m.level()
            <= if own {
                log::Level::Info
            } else {
                log::Level::Warn
            }
    }

    fn log(&self, r: &log::Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        if let Ok(mut f) = self.0.lock() {
            let _ = writeln!(f, "[{secs}] {} {}: {}", r.level(), r.target(), r.args());
        }
    }

    fn flush(&self) {
        if let Ok(mut f) = self.0.lock() {
            let _ = f.flush();
        }
    }
}

pub fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "deepslate-studio").map(|d| d.data_dir().to_path_buf())
}

pub fn cache_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "deepslate-studio").map(|d| d.cache_dir().to_path_buf())
}

pub fn path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("registro.log"))
}

/// Si no se puede crear el archivo, la app sigue sin registro.
pub fn init(path: &Path) {
    let file = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::File::create(path));
    if let Ok(f) = file
        && log::set_boxed_logger(Box::new(FileLogger(Mutex::new(f)))).is_ok()
    {
        log::set_max_level(log::LevelFilter::Info);
    }
}
