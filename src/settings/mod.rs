//! Ajustes básicos guardados en RON, en la carpeta de configuración estándar del sistema.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    System,
    Dark,
    Light,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub pixel_size: f32,
    pub show_grid: bool,
    pub undo_levels: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            pixel_size: 10.0,
            show_grid: true,
            undo_levels: 100,
        }
    }
}

pub const PIXEL_SIZE: std::ops::RangeInclusive<f32> = 2.0..=40.0;
pub const UNDO_LEVELS: std::ops::RangeInclusive<usize> = 1..=1000;

impl Settings {
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "deepslate-studio")
            .map(|d| d.config_dir().join("settings.ron"))
    }

    /// Nunca falla: si el archivo no existe se usan valores por defecto, y si está dañado
    /// se renombra a `.bak` para no perderlo.
    pub fn load_from(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        match ron::from_str::<Self>(&text) {
            Ok(s) => s.clamped(),
            Err(_) => {
                let _ = std::fs::rename(path, path.with_extension("ron.bak"));
                Self::default()
            }
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text =
            ron::ser::to_string_pretty(self, Default::default()).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }

    fn clamped(mut self) -> Self {
        self.pixel_size = self
            .pixel_size
            .clamp(*PIXEL_SIZE.start(), *PIXEL_SIZE.end());
        self.undo_levels = self
            .undo_levels
            .clamp(*UNDO_LEVELS.start(), *UNDO_LEVELS.end());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("deepslate-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("settings.ron")
    }

    #[test]
    fn roundtrip_and_missing() {
        let path = tmp("roundtrip");
        assert_eq!(Settings::load_from(&path), Settings::default());
        let s = Settings {
            show_grid: false,
            undo_levels: 7,
            ..Default::default()
        };
        s.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path), s);
    }

    #[test]
    fn corrupt_file_is_kept_as_backup() {
        let path = tmp("corrupt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "esto no es ron (").unwrap();
        assert_eq!(Settings::load_from(&path), Settings::default());
        assert!(path.with_extension("ron.bak").exists());
    }

    #[test]
    fn partial_and_out_of_range() {
        let path = tmp("partial");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "(pixel_size: 999.0, undo_levels: 0)").unwrap();
        let s = Settings::load_from(&path);
        assert_eq!((s.pixel_size, s.undo_levels, s.show_grid), (40.0, 1, true));
    }
}
