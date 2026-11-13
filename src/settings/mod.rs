//! Ajustes guardados en RON, en la carpeta de configuración estándar del sistema.

pub mod log;

use crate::editor::Tool;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    Flat,
    ThreeD,
    Preview,
    Library,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    System,
    Dark,
    Light,
}

/// Acciones que se pueden lanzar con un atajo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Action {
    New,
    Open,
    Save,
    SaveAs,
    Export,
    Undo,
    Redo,
    CopySelection,
    EraseSelection,
    Deselect,
    CopyParts,
    PasteParts,
    Pencil,
    Eraser,
    Fill,
    Picker,
    Gradient,
    Stamp,
    Select,
    ToggleMirror,
    SwapColors,
    ToggleView,
    QuickExport,
    CloseTab,
    NextTab,
    PrevTab,
    Lighten,
    Darken,
    ToggleOverlay,
    ViewFront,
    ViewBack,
    ViewSide,
    FocusPart,
}

pub const ACTIONS: [Action; 33] = [
    Action::New,
    Action::Open,
    Action::CloseTab,
    Action::NextTab,
    Action::PrevTab,
    Action::Save,
    Action::SaveAs,
    Action::Export,
    Action::QuickExport,
    Action::Undo,
    Action::Redo,
    Action::CopySelection,
    Action::EraseSelection,
    Action::Deselect,
    Action::CopyParts,
    Action::PasteParts,
    Action::Pencil,
    Action::Eraser,
    Action::Fill,
    Action::Picker,
    Action::Gradient,
    Action::Stamp,
    Action::Select,
    Action::Lighten,
    Action::Darken,
    Action::ToggleMirror,
    Action::SwapColors,
    Action::ToggleView,
    Action::ToggleOverlay,
    Action::ViewFront,
    Action::ViewBack,
    Action::ViewSide,
    Action::FocusPart,
];

/// Combinación de teclas. `ctrl` es Cmd en macOS. `key` es el nombre de la tecla en egui.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    pub key: String,
}

impl Shortcut {
    fn new(ctrl: bool, shift: bool, key: &str) -> Self {
        Self {
            ctrl,
            shift,
            alt: false,
            key: key.into(),
        }
    }
}

impl Action {
    pub fn default_shortcut(self) -> Option<Shortcut> {
        let (ctrl, shift, key) = match self {
            Action::New => (true, false, "N"),
            Action::Open => (true, false, "O"),
            Action::CloseTab => (true, false, "W"),
            Action::NextTab => (true, false, "Tab"),
            Action::PrevTab => (true, true, "Tab"),
            Action::Save => (true, false, "S"),
            Action::SaveAs => (true, true, "S"),
            Action::Export => (true, false, "E"),
            Action::QuickExport => (true, true, "E"),
            Action::Undo => (true, false, "Z"),
            Action::Redo => (true, true, "Z"),
            Action::CopySelection => (true, false, "C"),
            Action::EraseSelection => (false, false, "Delete"),
            Action::Deselect => (true, false, "D"),
            Action::CopyParts => (true, true, "C"),
            Action::PasteParts => (true, true, "V"),
            Action::Pencil => (false, false, "B"),
            Action::Eraser => (false, false, "E"),
            Action::Fill => (false, false, "F"),
            Action::Picker => (false, false, "I"),
            Action::Gradient => (false, false, "G"),
            Action::Stamp => (false, false, "K"),
            Action::Select => (false, false, "M"),
            Action::ToggleMirror => (false, false, "X"),
            Action::SwapColors => (false, false, "W"),
            Action::ToggleView => (false, false, "V"),
            Action::Lighten => (false, false, "L"),
            Action::Darken => (false, false, "O"),
            Action::ToggleOverlay => (false, false, "H"),
            Action::ViewFront => (false, false, "1"),
            Action::ViewBack => (false, false, "2"),
            Action::ViewSide => (false, false, "3"),
            Action::FocusPart => (false, false, "4"),
        };
        Some(Shortcut::new(ctrl, shift, key))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedColor {
    pub name: String,
    pub rgba: [u8; 4],
}

pub const RECENT_COLORS: usize = 16;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub pixel_size: f32,
    pub show_grid: bool,
    pub show_labels: bool,
    pub undo_levels: usize,
    /// Última vista usada (2D o 3D).
    pub view: View,
    /// Solo los atajos cambiados; `None` es «sin atajo». Lo que falta usa el valor por defecto.
    pub shortcuts: BTreeMap<Action, Option<Shortcut>>,
    /// Colores usados, el más reciente primero.
    pub recent_colors: Vec<[u8; 4]>,
    pub palette: Vec<NamedColor>,

    pub language: String,
    pub open_last: bool,
    pub last_file: Option<PathBuf>,
    /// Solo guarda proyectos que ya tienen archivo.
    pub autosave: bool,
    pub autosave_minutes: u32,
    pub start_tool: Tool,
    /// Transiciones de la interfaz instantáneas.
    pub reduce_motion: bool,
    /// Tamaño de toda la interfaz (1.0 = normal).
    pub ui_scale: f32,
    /// En la vista 3D, el lienzo 2D al lado.
    pub split_view: bool,

    /// Fondo de la vista 3D; `None` deja el del panel. La captura siempre es transparente.
    pub background_3d: Option<[u8; 3]>,
    /// 0 = color plano, 1 = luz direccional completa.
    pub lighting: f32,
    pub rotate_speed: f32,
    pub shadow: bool,

    pub export_dir: Option<PathBuf>,
    /// Admite `{name}` y `{model}`.
    pub export_name: String,
    pub confirm_overwrite: bool,

    /// `None`: la carpeta `skins` dentro de los datos de la app.
    pub library_dir: Option<PathBuf>,
    /// Ancho de las tarjetas, en puntos.
    pub card_size: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            pixel_size: 10.0,
            show_grid: true,
            show_labels: true,
            undo_levels: 100,
            view: View::ThreeD,
            shortcuts: BTreeMap::new(),
            recent_colors: Vec::new(),
            palette: Vec::new(),
            language: SYSTEM_LANGUAGE.into(),
            open_last: false,
            last_file: None,
            autosave: false,
            autosave_minutes: 5,
            start_tool: Tool::Pencil,
            reduce_motion: false,
            ui_scale: 1.0,
            split_view: false,
            background_3d: None,
            lighting: 1.0,
            rotate_speed: 1.0,
            shadow: true,
            export_dir: None,
            export_name: "{name}".into(),
            confirm_overwrite: true,
            library_dir: None,
            card_size: 170.0,
        }
    }
}

pub const PIXEL_SIZE: std::ops::RangeInclusive<f32> = 2.0..=40.0;
pub const UNDO_LEVELS: std::ops::RangeInclusive<usize> = 1..=1000;
pub const AUTOSAVE_MINUTES: std::ops::RangeInclusive<u32> = 1..=60;
pub const LIGHTING: std::ops::RangeInclusive<f32> = 0.0..=1.0;
pub const ROTATE_SPEED: std::ops::RangeInclusive<f32> = 0.2..=3.0;
pub const CARD_SIZE: std::ops::RangeInclusive<f32> = 120.0..=280.0;
/// Valor de `language` que sigue al idioma del sistema.
pub const SYSTEM_LANGUAGE: &str = "system";

impl Settings {
    /// Código del idioma a mostrar.
    pub fn resolved_language(&self) -> &str {
        if self.language == SYSTEM_LANGUAGE {
            crate::i18n::system_language()
        } else {
            &self.language
        }
    }

    /// Valores por defecto, pero conservando la paleta: es contenido del usuario.
    pub fn reset(&mut self) {
        let palette = std::mem::take(&mut self.palette);
        *self = Self {
            palette,
            ..Self::default()
        };
    }

    /// Nombre de archivo para «Exportar rápido», sin separadores de ruta y con `.png`.
    pub fn export_file_name(&self, name: &str, model: &str) -> String {
        let n: String = self
            .export_name
            .replace("{name}", name)
            .replace("{model}", model)
            .chars()
            .map(|c| {
                if c == '/' || c == '\\' || c == ':' {
                    '_'
                } else {
                    c
                }
            })
            .collect();
        let n = n.trim();
        let n = if n.is_empty() { "skin" } else { n };
        if n.to_ascii_lowercase().ends_with(".png") {
            n.to_string()
        } else {
            format!("{n}.png")
        }
    }

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

    pub fn shortcut(&self, action: Action) -> Option<Shortcut> {
        match self.shortcuts.get(&action) {
            Some(s) => s.clone(),
            None => action.default_shortcut(),
        }
    }

    pub fn action_for(&self, sc: &Shortcut) -> Option<Action> {
        ACTIONS
            .into_iter()
            .find(|&a| self.shortcut(a).as_ref() == Some(sc))
    }

    /// Asigna el atajo. Si otra acción ya lo usa, no cambia nada y devuelve esa acción.
    pub fn bind(&mut self, action: Action, sc: Option<Shortcut>) -> Result<(), Action> {
        if let Some(sc) = &sc
            && let Some(other) = self.action_for(sc).filter(|&o| o != action)
        {
            return Err(other);
        }
        if sc == action.default_shortcut() {
            self.shortcuts.remove(&action);
        } else {
            self.shortcuts.insert(action, sc);
        }
        Ok(())
    }

    pub fn remember_color(&mut self, c: [u8; 4]) {
        self.recent_colors.retain(|&x| x != c);
        self.recent_colors.insert(0, c);
        self.recent_colors.truncate(RECENT_COLORS);
    }

    fn clamped(mut self) -> Self {
        self.pixel_size = self
            .pixel_size
            .clamp(*PIXEL_SIZE.start(), *PIXEL_SIZE.end());
        self.undo_levels = self
            .undo_levels
            .clamp(*UNDO_LEVELS.start(), *UNDO_LEVELS.end());
        self.recent_colors.truncate(RECENT_COLORS);
        self.autosave_minutes = self
            .autosave_minutes
            .clamp(*AUTOSAVE_MINUTES.start(), *AUTOSAVE_MINUTES.end());
        self.lighting = self.lighting.clamp(*LIGHTING.start(), *LIGHTING.end());
        self.card_size = self.card_size.clamp(*CARD_SIZE.start(), *CARD_SIZE.end());
        self.rotate_speed = self
            .rotate_speed
            .clamp(*ROTATE_SPEED.start(), *ROTATE_SPEED.end());
        if self.language != SYSTEM_LANGUAGE
            && !crate::i18n::LANGUAGES.contains(&self.language.as_str())
        {
            self.language = SYSTEM_LANGUAGE.into();
        }
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
        // `start_view` ya no existe: un archivo viejo que lo trae se sigue leyendo.
        std::fs::write(
            &path,
            "(pixel_size: 999.0, undo_levels: 0, start_view: Last)",
        )
        .unwrap();
        let s = Settings::load_from(&path);
        assert_eq!((s.pixel_size, s.undo_levels, s.show_grid), (40.0, 1, true));
        assert!(!path.with_extension("ron.bak").exists());
    }

    #[test]
    fn defaults_have_no_clashes() {
        let s = Settings::default();
        for a in ACTIONS {
            let sc = s.shortcut(a).unwrap();
            assert_eq!(s.action_for(&sc), Some(a), "{sc:?}");
        }
    }

    #[test]
    fn bind_rejects_clash_and_persists() {
        let mut s = Settings::default();
        let ctrl_z = Action::Undo.default_shortcut();
        assert_eq!(s.bind(Action::Pencil, ctrl_z.clone()), Err(Action::Undo));
        assert_eq!(
            s.shortcut(Action::Pencil),
            Action::Pencil.default_shortcut()
        );
        // Liberar Ctrl+Z y dárselo al lápiz.
        s.bind(Action::Undo, None).unwrap();
        s.bind(Action::Pencil, ctrl_z.clone()).unwrap();
        assert_eq!(s.action_for(ctrl_z.as_ref().unwrap()), Some(Action::Pencil));
        let path = tmp("shortcuts");
        s.save_to(&path).unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.shortcut(Action::Undo), None);
        assert_eq!(loaded.shortcut(Action::Pencil), ctrl_z);
        // Volver al valor por defecto no deja entrada guardada.
        s.bind(Action::Undo, Action::Undo.default_shortcut())
            .unwrap_err();
        s.bind(Action::Pencil, Action::Pencil.default_shortcut())
            .unwrap();
        s.bind(Action::Undo, Action::Undo.default_shortcut())
            .unwrap();
        assert!(s.shortcuts.is_empty());
    }

    #[test]
    fn recent_colors_dedupe_and_cap() {
        let mut s = Settings::default();
        for i in 0..20u8 {
            s.remember_color([i, 0, 0, 255]);
        }
        s.remember_color([5, 0, 0, 255]);
        assert_eq!(s.recent_colors.len(), RECENT_COLORS);
        assert_eq!(s.recent_colors[0], [5, 0, 0, 255]);
        assert_eq!(s.recent_colors.iter().filter(|c| c[0] == 5).count(), 1);
    }

    #[test]
    fn export_names() {
        let mut s = Settings::default();
        assert_eq!(s.export_file_name("mia", "Alex"), "mia.png");
        s.export_name = "{name}-{model}".into();
        assert_eq!(s.export_file_name("a/b", "Steve"), "a_b-Steve.png");
        s.export_name = "  ".into();
        assert_eq!(s.export_file_name("x", "Steve"), "skin.png");
        s.export_name = "{name}.PNG".into();
        assert_eq!(s.export_file_name("x", "Steve"), "x.PNG");
    }

    #[test]
    fn reset_keeps_palette() {
        let mut s = Settings {
            show_grid: false,
            palette: vec![NamedColor {
                name: "piel".into(),
                rgba: [1, 2, 3, 255],
            }],
            ..Default::default()
        };
        s.reset();
        assert!(s.show_grid);
        assert_eq!(s.palette.len(), 1);
    }
}
