//! Ventana principal: menús, panel de herramientas, lienzo 2D y diálogos.

mod settings_window;

use crate::core::{FACES, LAYERS, Layer, LoadError, Model, PARTS, Part, Skin, convert_model, rect};
use crate::editor::colors::{palette_from, parse_hex, to_hex};
use crate::editor::{ERASE, Editor, Mirror, PartMask, Tool};
use crate::i18n::I18n;
use crate::project::{self, Project, ProjectError};
use crate::render3d::{Camera, PRESETS, Preset, Renderer3d, Scene, pick, project};
use crate::settings::{Action, NamedColor, Settings, Shortcut, StartView, Theme, View};
use eframe::egui::{
    self, Color32, ColorImage, Key, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle,
    TextureOptions, Vec2, color_picker, ecolor::Hsva,
};
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
enum After {
    New,
    Open,
    Close,
}

enum Dialog {
    Discard(After),
    Convert(Skin),
    /// Píxeles translúcidos y si es exportación rápida.
    Export(usize, bool),
    Overwrite(PathBuf),
    ResetSettings,
    Error(String),
    CopyParts(PartMask),
    PasteParts(PartMask),
}

#[derive(Clone, Copy, PartialEq)]
enum PreviewBase {
    Mannequin,
    Current,
    Loaded,
}

/// Estado del previsualizador. Nada de esto se guarda: la skin y la capa cargadas
/// solo viven en memoria.
struct PreviewState {
    base: PreviewBase,
    skin: Option<Skin>,
    skin_rev: u64,
    cape: bool,
    cape_color: [u8; 3],
    cape_img: Option<RgbaImage>,
    cape_rev: u64,
}

#[derive(Clone, Copy, PartialEq)]
enum MirrorMode {
    Off,
    All,
    Selected,
}

#[derive(Clone, Copy)]
enum Choice {
    Accept,
    Alternate,
    Cancel,
}

pub struct App {
    i18n: I18n,
    settings: Settings,
    settings_path: Option<PathBuf>,
    settings_changed: bool,
    show_settings: bool,
    editor: Editor,
    path: Option<PathBuf>,
    tool: Tool,
    color: Hsva,
    texture: Option<TextureHandle>,
    texture_rev: Option<u64>,
    preview: Option<TextureHandle>,
    painting: bool,
    last_px: Option<(u32, u32)>,
    dialog: Option<Dialog>,
    status: String,
    closing: bool,
    renderer: Option<Renderer3d>,
    /// Qué skin hay en la GPU: (origen, revisión).
    uploaded: Option<(u8, u64)>,
    cape_uploaded: Option<u64>,
    pv: PreviewState,
    last_autosave: f64,
    /// Contenido del registro mientras su ventana está abierta.
    log_text: Option<String>,
    camera: Camera,
    selected: Option<Part>,
    /// Color secundario, el final del degradado.
    color2: Hsva,
    hex: String,
    mirror: MirrorMode,
    /// Donde empezó el arrastre de la selección o del degradado.
    drag_from: Option<(u32, u32)>,
    /// Lo que estampa el sello; se conserva al abrir otro archivo.
    stamp: Option<RgbaImage>,
    /// Partes copiadas; se conservan al abrir otro archivo para pegarlas en él.
    clip: Option<(Skin, PartMask)>,
    /// Acción que espera una tecla en Ajustes → Atajos.
    binding: Option<Action>,
    bind_msg: String,
    new_color_name: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings_path = Settings::path();
        let settings = settings_path
            .as_deref()
            .map(Settings::load_from)
            .unwrap_or_default();
        cc.egui_ctx.set_theme(theme_pref(settings.theme));
        let mut settings = settings;
        match settings.start_view {
            StartView::Last => {}
            StartView::Flat => settings.view = View::Flat,
            StartView::ThreeD => settings.view = View::ThreeD,
        }
        if cc.wgpu_render_state.is_none() {
            log::warn!("sin wgpu: la vista 3D no estará disponible");
        }
        let last = settings
            .open_last
            .then(|| settings.last_file.clone())
            .flatten();
        let start_tool = settings.start_tool;
        let mut app = Self {
            i18n: I18n::spanish(),
            editor: Editor::new(Skin::blank(Model::Steve), settings.undo_levels),
            settings,
            settings_path,
            settings_changed: false,
            show_settings: false,
            path: None,
            tool: Tool::Pencil,
            color: Hsva::from_srgba_unmultiplied([196, 92, 52, 255]),
            texture: None,
            texture_rev: None,
            preview: None,
            painting: false,
            last_px: None,
            dialog: None,
            status: String::new(),
            closing: false,
            // Solo es None si eframe no arrancó con wgpu; entonces la vista 3D muestra un aviso.
            renderer: cc.wgpu_render_state.as_ref().map(Renderer3d::new),
            uploaded: None,
            cape_uploaded: None,
            pv: PreviewState {
                base: PreviewBase::Mannequin,
                skin: None,
                skin_rev: 0,
                cape: true,
                cape_color: [255, 255, 255],
                cape_img: None,
                cape_rev: 0,
            },
            last_autosave: 0.0,
            log_text: None,
            camera: Camera::default(),
            selected: None,
            color2: Hsva::from_srgba_unmultiplied([255, 255, 255, 255]),
            hex: String::new(),
            mirror: MirrorMode::Off,
            drag_from: None,
            stamp: None,
            clip: None,
            binding: None,
            bind_msg: String::new(),
            new_color_name: String::new(),
        };
        app.tool = start_tool;
        if let Some(path) = last.filter(|p| p.exists()) {
            app.open_path(path);
        }
        app
    }

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.editor = Editor::new(project.skin, self.settings.undo_levels);
        self.editor.set_pose(project.pose);
        self.editor.mark_saved();
        self.path = path;
        self.texture_rev = None;
        self.uploaded = None;
    }

    // ---- Archivo ----

    fn request(&mut self, ctx: &egui::Context, after: After) {
        if self.editor.is_dirty() {
            self.dialog = Some(Dialog::Discard(after));
        } else {
            self.run(ctx, after);
        }
    }

    fn run(&mut self, ctx: &egui::Context, after: After) {
        match after {
            After::New => self.set_project(
                Project {
                    skin: Skin::blank(Model::Steve),
                    pose: Default::default(),
                },
                None,
            ),
            After::Open => self.open(),
            After::Close => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn open(&mut self) {
        if let Some(path) = png_dialog().pick_file() {
            self.open_path(path);
        }
    }

    fn open_path(&mut self, path: PathBuf) {
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                return self.error(
                    self.i18n
                        .tr_args("error-io", &[("detail", e.to_string().into())]),
                );
            }
        };
        match project::load(&bytes) {
            Ok(project) => {
                // Un PNG suelto se importa sin ruta, para que "Guardar" no lo sobrescriba sin preguntar.
                let is_project = project::is_project(&bytes);
                let key = if is_project {
                    "status-opened"
                } else {
                    "status-imported"
                };
                self.status = self.i18n.tr_args(key, &[("file", file_name(&path).into())]);
                log::info!("abierto {}", path.display());
                self.remember_file(&path);
                self.set_project(project, is_project.then_some(path));
            }
            Err(e) => self.error(self.load_error(e)),
        }
    }

    fn load_error(&self, e: ProjectError) -> String {
        let i = &self.i18n;
        match e {
            ProjectError::Skin(LoadError::Decode) => i.tr("error-not-png"),
            ProjectError::Skin(LoadError::BadSize(w, h)) => {
                i.tr_args("error-size", &[("w", w.into()), ("h", h.into())])
            }
            ProjectError::NewerVersion(v) => i.tr_args("error-newer", &[("version", v.into())]),
            ProjectError::Corrupt => i.tr("error-corrupt"),
        }
    }

    /// Devuelve `true` si quedó guardado.
    fn save(&mut self, save_as: bool) -> bool {
        let path = match (&self.path, save_as) {
            (Some(p), false) => p.clone(),
            _ => match png_dialog().set_file_name("skin.png").save_file() {
                Some(p) => with_png_ext(p),
                None => return false,
            },
        };
        match std::fs::write(&path, project::save(self.editor.skin(), self.editor.pose())) {
            Ok(()) => {
                self.editor.mark_saved();
                self.status = self
                    .i18n
                    .tr_args("status-saved", &[("file", file_name(&path).into())]);
                log::info!("guardado {}", path.display());
                self.remember_file(&path);
                self.path = Some(path);
                true
            }
            Err(e) => {
                self.error(
                    self.i18n
                        .tr_args("error-io", &[("detail", e.to_string().into())]),
                );
                false
            }
        }
    }

    fn remember_file(&mut self, path: &Path) {
        if self.settings.last_file.as_deref() != Some(path) {
            self.settings.last_file = Some(path.to_path_buf());
            self.settings_changed = true;
        }
    }

    /// Nombre para exportar según el formato de Ajustes.
    fn export_name(&self) -> String {
        let name = self
            .path
            .as_deref()
            .and_then(Path::file_stem)
            .map_or_else(|| "skin".into(), |n| n.to_string_lossy().into_owned());
        let model = format!("{:?}", self.editor.skin().model);
        self.settings.export_file_name(&name, &model)
    }

    /// `quick`: sin diálogo, directo a la carpeta predeterminada (si hay una).
    fn export(&mut self, quick: bool) {
        let dir = self.settings.export_dir.clone();
        let path = match (&dir, quick) {
            (Some(dir), true) => {
                let path = dir.join(self.export_name());
                if path.exists() && self.settings.confirm_overwrite {
                    self.dialog = Some(Dialog::Overwrite(path));
                    return;
                }
                path
            }
            _ => {
                let mut d = png_dialog().set_file_name(self.export_name());
                if let Some(dir) = &dir {
                    d = d.set_directory(dir);
                }
                match d.save_file() {
                    Some(p) => with_png_ext(p),
                    None => return,
                }
            }
        };
        self.write_export(&path);
    }

    fn write_export(&mut self, path: &Path) {
        match std::fs::write(path, self.editor.skin().to_png()) {
            Ok(()) => {
                log::info!("exportado {}", path.display());
                self.status = self
                    .i18n
                    .tr_args("status-exported", &[("file", file_name(path).into())])
            }
            Err(e) => self.error(
                self.i18n
                    .tr_args("error-io", &[("detail", e.to_string().into())]),
            ),
        }
    }

    fn request_export(&mut self, quick: bool) {
        match self.editor.skin().translucent_base_pixels() {
            0 => self.export(quick),
            n => self.dialog = Some(Dialog::Export(n, quick)),
        }
    }

    /// Guarda cada N minutos si hay cambios y el proyecto ya tiene archivo.
    fn autosave(&mut self, ctx: &egui::Context) {
        if !self.settings.autosave {
            return;
        }
        let period = self.settings.autosave_minutes as f64 * 60.0;
        let now = ctx.input(|i| i.time);
        ctx.request_repaint_after(std::time::Duration::from_secs(30));
        if now - self.last_autosave < period {
            return;
        }
        if self.editor.is_dirty()
            && self.path.is_some()
            && self.dialog.is_none()
            && !self.painting
            && self.save(false)
        {
            log::info!("autoguardado");
        }
        self.last_autosave = now;
    }

    fn error(&mut self, msg: String) {
        log::error!("{msg}");
        self.status = self
            .i18n
            .tr_args("status-error", &[("detail", msg.clone().into())]);
        self.dialog = Some(Dialog::Error(msg));
    }

    // ---- Entrada global ----

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.painting
            || self.dialog.is_some()
            || self.binding.is_some()
            || ctx.egui_wants_keyboard_input()
        {
            return;
        }
        let pressed = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } => Some(shortcut_of(*key, *modifiers)),
                _ => None,
            })
        });
        if let Some(action) = pressed.and_then(|sc| self.settings.action_for(&sc)) {
            self.act(ctx, action);
        }
    }

    fn act(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::New => self.request(ctx, After::New),
            Action::Open => self.request(ctx, After::Open),
            Action::Save => {
                self.save(false);
            }
            Action::SaveAs => {
                self.save(true);
            }
            Action::Export => self.request_export(false),
            Action::QuickExport => self.request_export(true),
            Action::Undo => self.editor.undo(),
            Action::Redo => self.editor.redo(),
            Action::CopySelection => {
                if let Some(img) = self.editor.copy_selection() {
                    self.stamp = Some(img);
                    self.tool = Tool::Stamp;
                    self.status = self.i18n.tr("status-stamp-copied");
                }
            }
            Action::EraseSelection => self.editor.erase_selection(),
            Action::Deselect => self.editor.set_selection(None),
            Action::CopyParts => {
                let mask = match self.selected {
                    Some(p) => {
                        let mut m = [[false; 2]; 6];
                        m[p as usize] = [true; 2];
                        m
                    }
                    None => [[true; 2]; 6],
                };
                self.dialog = Some(Dialog::CopyParts(mask));
            }
            Action::PasteParts => {
                if let Some((_, mask)) = &self.clip {
                    self.dialog = Some(Dialog::PasteParts(*mask));
                }
            }
            Action::Pencil => self.tool = Tool::Pencil,
            Action::Eraser => self.tool = Tool::Eraser,
            Action::Fill => self.tool = Tool::Fill,
            Action::Picker => self.tool = Tool::Picker,
            Action::Gradient => self.tool = Tool::Gradient,
            Action::Stamp => self.tool = Tool::Stamp,
            Action::Select => self.tool = Tool::Select,
            Action::ToggleMirror => {
                self.mirror = if self.mirror == MirrorMode::Off {
                    MirrorMode::All
                } else {
                    MirrorMode::Off
                }
            }
            Action::SwapColors => std::mem::swap(&mut self.color, &mut self.color2),
            Action::ToggleView => {
                self.settings.view = match self.settings.view {
                    View::Flat => View::ThreeD,
                    View::ThreeD | View::Preview => View::Flat,
                };
                self.settings_changed = true;
            }
        }
    }

    fn can(&self, action: Action) -> bool {
        match action {
            Action::Undo => self.editor.can_undo(),
            Action::Redo => self.editor.can_redo(),
            Action::CopySelection | Action::EraseSelection | Action::Deselect => {
                self.editor.selection().is_some()
            }
            Action::PasteParts => self.clip.is_some(),
            _ => true,
        }
    }

    /// Entrada de menú con su atajo a la derecha.
    fn menu_item(&mut self, ui: &mut egui::Ui, action: Action) {
        let mut button = egui::Button::new(action_name(&self.i18n, action));
        if let Some(sc) = self.settings.shortcut(action) {
            button = button.shortcut_text(shortcut_label(&self.i18n, &sc));
        }
        if ui.add_enabled(self.can(action), button).clicked() {
            let ctx = ui.ctx().clone();
            self.act(&ctx, action);
        }
    }

    fn handle_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.closing && self.editor.is_dirty()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.dialog = Some(Dialog::Discard(After::Close));
        }
    }

    // ---- Paneles ----

    fn menu(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(self.i18n.tr("menu-file"), |ui| {
                self.menu_item(ui, Action::New);
                self.menu_item(ui, Action::Open);
                ui.separator();
                self.menu_item(ui, Action::Save);
                self.menu_item(ui, Action::SaveAs);
                self.menu_item(ui, Action::Export);
                self.menu_item(ui, Action::QuickExport);
            });
            ui.menu_button(self.i18n.tr("menu-edit"), |ui| {
                self.menu_item(ui, Action::Undo);
                self.menu_item(ui, Action::Redo);
                ui.separator();
                self.menu_item(ui, Action::CopySelection);
                self.menu_item(ui, Action::EraseSelection);
                self.menu_item(ui, Action::Deselect);
                ui.separator();
                self.menu_item(ui, Action::CopyParts);
                self.menu_item(ui, Action::PasteParts);
            });
            if ui.button(self.i18n.tr("menu-settings")).clicked() {
                self.show_settings = true;
            }
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let name = self
                .path
                .as_deref()
                .map(file_name)
                .unwrap_or_else(|| self.i18n.tr("untitled"));
            let mark = if self.editor.is_dirty() { " •" } else { "" };
            ui.label(format!("{name}{mark}"));
            ui.separator();
            ui.label(&self.status);
        });
    }

    fn side(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.side_tools(ui);
            ui.separator();
            self.side_colors(ui);
            ui.separator();
            self.side_parts(ui);
        });
    }

    fn side_tools(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        ui.heading(i.tr("tools"));
        ui.horizontal_wrapped(|ui| {
            for (tool, action) in [
                (Tool::Pencil, Action::Pencil),
                (Tool::Eraser, Action::Eraser),
                (Tool::Fill, Action::Fill),
                (Tool::Picker, Action::Picker),
                (Tool::Gradient, Action::Gradient),
                (Tool::Stamp, Action::Stamp),
                (Tool::Select, Action::Select),
            ] {
                let mut resp = ui.selectable_value(&mut self.tool, tool, action_name(i, action));
                if let Some(sc) = self.settings.shortcut(action) {
                    resp = resp.on_hover_text(shortcut_label(i, &sc));
                }
                if tool == Tool::Stamp && self.stamp.is_none() {
                    resp.on_hover_text(i.tr("stamp-empty"));
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(i.tr("mirror"));
            ui.selectable_value(&mut self.mirror, MirrorMode::Off, i.tr("mirror-off"));
            ui.selectable_value(&mut self.mirror, MirrorMode::All, i.tr("mirror-all"));
            ui.selectable_value(&mut self.mirror, MirrorMode::Selected, i.tr("mirror-part"));
        });
        if self.mirror == MirrorMode::Selected && self.selected.is_none() {
            ui.weak(i.tr("mirror-pick-part"));
        }
    }

    fn side_colors(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        ui.heading(i.tr("color"));
        ui.horizontal(|ui| {
            color_picker::color_edit_button_hsva(
                ui,
                &mut self.color,
                color_picker::Alpha::OnlyBlend,
            )
            .on_hover_text(i.tr("color-primary"));
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.hex)
                    .desired_width(80.0)
                    .font(egui::TextStyle::Monospace),
            );
            if resp.changed()
                && let Some(c) = parse_hex(&self.hex)
            {
                self.color = Hsva::from_srgba_unmultiplied(c);
            }
            if !resp.has_focus() {
                self.hex = to_hex(self.color.to_srgba_unmultiplied());
            }
            resp.on_hover_text(i.tr("color-hex"));
        });
        ui.horizontal(|ui| {
            color_picker::color_edit_button_hsva(
                ui,
                &mut self.color2,
                color_picker::Alpha::OnlyBlend,
            )
            .on_hover_text(i.tr("color-secondary"));
            if ui.button(action_name(i, Action::SwapColors)).clicked() {
                std::mem::swap(&mut self.color, &mut self.color2);
            }
        });

        if !self.settings.recent_colors.is_empty() {
            ui.label(i.tr("color-recent"));
            ui.horizontal_wrapped(|ui| {
                for c in self.settings.recent_colors.clone() {
                    if swatch(ui, c).on_hover_text(to_hex(c)).clicked() {
                        self.color = Hsva::from_srgba_unmultiplied(c);
                    }
                }
            });
        }

        let mut import = false;
        egui::CollapsingHeader::new(i.tr("palette"))
            .default_open(true)
            .show(ui, |ui| {
                let mut remove = None;
                ui.horizontal_wrapped(|ui| {
                    for (k, entry) in self.settings.palette.iter_mut().enumerate() {
                        let resp = swatch(ui, entry.rgba).on_hover_text(format!(
                            "{}  {}",
                            entry.name,
                            to_hex(entry.rgba)
                        ));
                        if resp.clicked() {
                            self.color = Hsva::from_srgba_unmultiplied(entry.rgba);
                        }
                        resp.context_menu(|ui| {
                            ui.label(i.tr("palette-name"));
                            if ui.text_edit_singleline(&mut entry.name).changed() {
                                self.settings_changed = true;
                            }
                            if ui.button(i.tr("palette-remove")).clicked() {
                                remove = Some(k);
                                ui.close();
                            }
                        });
                    }
                });
                if let Some(k) = remove {
                    self.settings.palette.remove(k);
                    self.settings_changed = true;
                }
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_color_name)
                            .desired_width(110.0)
                            .hint_text(i.tr("palette-name")),
                    );
                    if ui.button(i.tr("palette-add")).clicked() {
                        let rgba = self.color.to_srgba_unmultiplied();
                        let name = match self.new_color_name.trim() {
                            "" => to_hex(rgba),
                            n => n.to_string(),
                        };
                        self.settings.palette.push(NamedColor { name, rgba });
                        self.new_color_name.clear();
                        self.settings_changed = true;
                    }
                });
                import = ui.button(i.tr("palette-import")).clicked();
                if !self.settings.palette.is_empty() {
                    ui.weak(i.tr("palette-hint"));
                }
            });
        if import {
            self.import_palette();
        }
    }

    /// Agrega a la paleta los colores más usados de un PNG (una skin u otra imagen).
    fn import_palette(&mut self) {
        let Some(path) = png_dialog().pick_file() else {
            return;
        };
        let img = match std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|b| image::load_from_memory(&b).map_err(|e| e.to_string()))
        {
            Ok(img) => img.to_rgba8(),
            Err(e) => return self.error(self.i18n.tr_args("error-io", &[("detail", e.into())])),
        };
        let mut added = 0;
        for rgba in palette_from(&img, PALETTE_IMPORT_MAX) {
            if !self.settings.palette.iter().any(|c| c.rgba == rgba) {
                self.settings.palette.push(NamedColor {
                    name: to_hex(rgba),
                    rgba,
                });
                added += 1;
            }
        }
        self.settings_changed = true;
        self.status = self
            .i18n
            .tr_args("status-palette-imported", &[("count", added.into())]);
    }

    fn side_parts(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        let model = self.editor.skin().model;
        let other = if model == Model::Steve {
            Model::Alex
        } else {
            Model::Steve
        };
        ui.label(i.tr_args("model-current", &[("model", format!("{model:?}").into())]));
        if ui
            .button(i.tr_args("model-convert", &[("model", format!("{other:?}").into())]))
            .clicked()
        {
            self.preview = None;
            self.dialog = Some(Dialog::Convert(convert_model(self.editor.skin(), other)));
        }
        ui.separator();

        ui.heading(i.tr("layers"));
        let layer_names = [i.tr("layer-base"), i.tr("layer-overlay")];
        egui::Grid::new("visibility").show(ui, |ui| {
            ui.label("");
            for n in &layer_names {
                ui.label(n);
            }
            ui.end_row();
            for part in PARTS {
                let name = part_name(i, part);
                let is_sel = self.selected == Some(part);
                let text = if is_sel {
                    egui::RichText::new(&name).strong().color(ACCENT)
                } else {
                    egui::RichText::new(&name)
                };
                if ui
                    .selectable_label(is_sel, text)
                    .on_hover_text(i.tr("part-select-hint"))
                    .clicked()
                {
                    self.selected = (!is_sel).then_some(part);
                }
                for (layer, layer_name) in LAYERS.into_iter().zip(&layer_names) {
                    let mut on = self.editor.is_visible(part, layer);
                    let resp = ui
                        .checkbox(&mut on, "")
                        .on_hover_text(format!("{name}: {layer_name}"));
                    if resp.changed() {
                        self.editor.set_visible(part, layer, on);
                    }
                }
                ui.end_row();
            }
        });
    }

    // ---- Lienzo ----

    fn update_texture(&mut self, ctx: &egui::Context) {
        let rev = self.editor.revision();
        if self.texture_rev == Some(rev) {
            return;
        }
        let editor = &self.editor;
        let img = skin_image(editor.skin(), |p, l| editor.is_visible(p, l));
        match &mut self.texture {
            Some(t) => t.set(img, TextureOptions::NEAREST),
            None => self.texture = Some(ctx.load_texture("skin", img, TextureOptions::NEAREST)),
        }
        self.texture_rev = Some(rev);
    }

    fn central(&mut self, ui: &mut egui::Ui) {
        let before = self.settings.view;
        let view3d = self.settings.view != View::Flat;
        ui.horizontal_wrapped(|ui| {
            let i = &self.i18n;
            ui.selectable_value(&mut self.settings.view, View::Flat, i.tr("view-2d"));
            ui.selectable_value(&mut self.settings.view, View::ThreeD, i.tr("view-3d"));
            ui.selectable_value(&mut self.settings.view, View::Preview, i.tr("view-preview"));
            if view3d {
                ui.separator();
                ui.label(i.tr("pose"));
                let current = Preset::matching(self.editor.pose())
                    .map_or_else(|| i.tr("pose-custom"), |p| pose_name(i, p));
                let mut chosen = None;
                egui::ComboBox::from_id_salt("pose")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for p in PRESETS {
                            if ui.button(pose_name(i, p)).clicked() {
                                chosen = Some(p);
                            }
                        }
                    });
                if let Some(p) = chosen {
                    self.editor.set_pose(p.pose());
                }
                if ui.button(i.tr("view-reset")).clicked() {
                    self.camera = Camera::default();
                }
                if ui.button(i.tr("capture")).clicked() {
                    self.capture();
                }
            }
        });
        if self.settings.view == View::Preview {
            self.preview_bar(ui);
        }
        if self.settings.view != before {
            self.settings_changed = true;
        }
        ui.separator();
        match self.settings.view {
            View::Flat => self.canvas(ui),
            View::ThreeD => self.view3d(ui, false),
            View::Preview => self.view3d(ui, true),
        }
    }

    fn preview_bar(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        let (mut load_skin, mut load_cape) = (false, false);
        ui.horizontal_wrapped(|ui| {
            ui.label(i.tr("preview-base"));
            ui.selectable_value(
                &mut self.pv.base,
                PreviewBase::Mannequin,
                i.tr("preview-mannequin"),
            );
            ui.selectable_value(
                &mut self.pv.base,
                PreviewBase::Current,
                i.tr("preview-current"),
            );
            if self.pv.skin.is_some() {
                ui.selectable_value(
                    &mut self.pv.base,
                    PreviewBase::Loaded,
                    i.tr("preview-loaded"),
                );
            }
            load_skin = ui.button(i.tr("preview-load-skin")).clicked();
            ui.separator();
            ui.checkbox(&mut self.pv.cape, i.tr("preview-cape"));
            if self.pv.cape {
                if self.pv.cape_img.is_none() {
                    let before = self.pv.cape_color;
                    ui.color_edit_button_srgb(&mut self.pv.cape_color)
                        .on_hover_text(i.tr("preview-cape-color"));
                    if before != self.pv.cape_color {
                        self.pv.cape_rev += 1;
                    }
                    load_cape = ui.button(i.tr("preview-load-cape")).clicked();
                } else if ui.button(i.tr("preview-remove-cape")).clicked() {
                    self.pv.cape_img = None;
                    self.pv.cape_rev += 1;
                }
            }
        });
        ui.weak(i.tr("preview-notice"));
        if load_skin {
            self.load_preview_skin();
        }
        if load_cape {
            self.load_preview_cape();
        }
    }

    fn read_png(&mut self) -> Option<(PathBuf, Vec<u8>)> {
        let path = png_dialog().pick_file()?;
        match std::fs::read(&path) {
            Ok(b) => Some((path, b)),
            Err(e) => {
                self.error(
                    self.i18n
                        .tr_args("error-io", &[("detail", e.to_string().into())]),
                );
                None
            }
        }
    }

    /// Solo para mirar: no se guarda ni cambia el proyecto.
    fn load_preview_skin(&mut self) {
        let Some((_, bytes)) = self.read_png() else {
            return;
        };
        match Skin::from_png(&bytes) {
            Ok(skin) => {
                self.pv.skin = Some(skin);
                self.pv.skin_rev += 1;
                self.pv.base = PreviewBase::Loaded;
            }
            Err(e) => self.error(self.load_error(ProjectError::Skin(e))),
        }
    }

    /// La capa queda solo en memoria; la app nunca la guarda.
    fn load_preview_cape(&mut self) {
        let Some((_, bytes)) = self.read_png() else {
            return;
        };
        let cape = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .ok()
            .and_then(|img| crate::core::cape_texture(&img.to_rgba8()));
        match cape {
            Some(img) => {
                self.pv.cape_img = Some(img);
                self.pv.cape_rev += 1;
            }
            None => self.error(self.i18n.tr("error-cape")),
        }
    }

    fn model_3d(&self, preview: bool) -> Model {
        match (preview, self.pv.base, &self.pv.skin) {
            (true, PreviewBase::Loaded, Some(s)) => s.model,
            _ => self.editor.skin().model,
        }
    }

    /// Sube a la GPU la skin (y la capa) que toca mostrar, si cambiaron.
    fn sync_3d(&mut self, preview: bool) {
        let Some(r) = &self.renderer else {
            return;
        };
        let model = self.editor.skin().model;
        let key = match (preview, self.pv.base, &self.pv.skin) {
            (true, PreviewBase::Mannequin, _) => (1, model as u64),
            (true, PreviewBase::Loaded, Some(_)) => (2, self.pv.skin_rev),
            _ => (0, self.editor.revision()),
        };
        if self.uploaded != Some(key) {
            match key.0 {
                1 => r.upload_skin(&Skin::mannequin(model).pixels),
                2 => r.upload_skin(&self.pv.skin.as_ref().expect("clave 2").pixels),
                _ => r.upload_skin(&self.editor.skin().pixels),
            }
            self.uploaded = Some(key);
        }
        if preview && self.cape_uploaded != Some(self.pv.cape_rev) {
            match &self.pv.cape_img {
                Some(img) => r.upload_cape(img),
                None => {
                    let [cr, cg, cb] = self.pv.cape_color;
                    r.upload_cape(&RgbaImage::from_pixel(64, 32, Rgba([cr, cg, cb, 255])));
                }
            }
            self.cape_uploaded = Some(self.pv.cape_rev);
        }
    }

    fn view3d(&mut self, ui: &mut egui::Ui, preview: bool) {
        if self.renderer.is_none() {
            ui.label(self.i18n.tr("view3d-unavailable"));
            return;
        }
        self.sync_3d(preview);
        let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        if resp.dragged() {
            let d = resp.drag_delta() * 0.5 * self.settings.rotate_speed;
            self.camera.yaw += d.x;
            self.camera.pitch = (self.camera.pitch + d.y).clamp(-89.0, 89.0);
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.camera.distance = (self.camera.distance - scroll * 0.1).clamp(30.0, 160.0);
        }
        let aspect = area.width() / area.height().max(1.0);
        let painter = ui.painter_at(area);
        if let Some([r, g, b]) = self.settings.background_3d {
            painter.rect_filled(area, 0.0, Color32::from_rgb(r, g, b));
        }
        if self.settings.shadow {
            // Sombra simple: una elipse en el suelo, proyectada con la misma cámara.
            let to_screen = |[nx, ny]: [f32; 2]| {
                area.center() + Vec2::new(nx * area.width(), -ny * area.height()) / 2.0
            };
            let pts: Option<Vec<Pos2>> = (0..32)
                .map(|k| {
                    let a = k as f32 / 32.0 * std::f32::consts::TAU;
                    project(&self.camera, aspect, [a.cos() * 8.0, 0.0, a.sin() * 5.0])
                        .map(to_screen)
                })
                .collect();
            if let Some(pts) = pts {
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    Color32::from_black_alpha(70),
                    Stroke::NONE,
                ));
            }
        }
        let model = self.model_3d(preview);
        let editor = &self.editor;
        let visible = |p, l| preview || editor.is_visible(p, l);
        if !preview
            && resp.clicked()
            && let Some(pos) = resp.interact_pointer_pos()
        {
            let nx = (pos.x - area.min.x) / area.width() * 2.0 - 1.0;
            let ny = 1.0 - (pos.y - area.min.y) / area.height() * 2.0;
            self.selected = pick(
                model,
                editor.pose(),
                &self.camera,
                &visible,
                aspect,
                (nx, ny),
            );
        }
        let scene = Scene {
            model,
            pose: editor.pose(),
            camera: &self.camera,
            selected: if preview { None } else { self.selected },
            visible: &visible,
            cape: preview && self.pv.cape,
            lighting: self.settings.lighting,
        };
        let ppp = ui.ctx().pixels_per_point();
        let size = [area.width(), area.height()].map(|v| (v * ppp).round() as u32);
        let tex = self
            .renderer
            .as_mut()
            .expect("comprobado arriba")
            .render(size, &scene);
        painter.image(
            tex,
            area,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        if !preview {
            resp.on_hover_text(self.i18n.tr("view3d-hint"));
        }
    }

    fn capture(&mut self) {
        if self.renderer.is_none() {
            return;
        }
        let Some(path) = png_dialog().set_file_name("captura.png").save_file() else {
            return;
        };
        let path = with_png_ext(path);
        let preview = self.settings.view == View::Preview;
        self.sync_3d(preview);
        let editor = &self.editor;
        let visible = |p, l| preview || editor.is_visible(p, l);
        let scene = Scene {
            model: self.model_3d(preview),
            pose: editor.pose(),
            camera: &self.camera,
            selected: None,
            visible: &visible,
            cape: preview && self.pv.cape,
            lighting: self.settings.lighting,
        };
        let renderer = self.renderer.as_ref().expect("comprobado arriba");
        let Some(img) = renderer.capture(1024, &scene) else {
            return self.error(self.i18n.tr("error-capture"));
        };
        let result = img
            .save_with_format(&path, image::ImageFormat::Png)
            .map_err(|e| e.to_string());
        match result {
            Ok(()) => {
                self.status = self
                    .i18n
                    .tr_args("status-captured", &[("file", file_name(&path).into())])
            }
            Err(e) => self.error(self.i18n.tr_args("error-io", &[("detail", e.into())])),
        }
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        self.update_texture(ui.ctx());
        let px = self.settings.pixel_size;
        egui::ScrollArea::both().show(ui, |ui| {
            let (area, resp) =
                ui.allocate_exact_size(Vec2::splat(64.0 * px), Sense::click_and_drag());
            let painter = ui.painter_at(area);
            let v = ui.visuals();
            let to_screen = |r: crate::core::Rect| {
                Rect::from_min_size(
                    area.min + Vec2::new(r.x as f32, r.y as f32) * px,
                    Vec2::new(r.w as f32, r.h as f32) * px,
                )
            };
            let model = self.editor.skin().model;
            let visible_faces: Vec<Rect> = PARTS
                .iter()
                .flat_map(|&p| LAYERS.map(move |l| (p, l)))
                .filter(|&(p, l)| self.editor.is_visible(p, l))
                .flat_map(|(p, l)| FACES.map(move |f| rect(p, l, f, model)))
                .map(to_screen)
                .collect();

            // Fondo: zona sin uso más oscura que las caras pintables.
            painter.rect_filled(area, 0.0, v.extreme_bg_color);
            for r in &visible_faces {
                painter.rect_filled(*r, 0.0, v.widgets.inactive.bg_fill);
            }
            if let Some(t) = &self.texture {
                painter.image(
                    t.id(),
                    area,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            if self.settings.show_grid && px >= 6.0 {
                let grid = Stroke::new(1.0, v.weak_text_color().gamma_multiply(0.25));
                for k in 0..=64 {
                    let o = k as f32 * px;
                    painter.vline(area.min.x + o, area.y_range(), grid);
                    painter.hline(area.x_range(), area.min.y + o, grid);
                }
            }
            let outline = Stroke::new(1.0, v.widgets.noninteractive.bg_stroke.color);
            for r in &visible_faces {
                painter.rect_stroke(*r, 0.0, outline, StrokeKind::Inside);
            }
            if let Some(part) = self.selected {
                for layer in LAYERS
                    .into_iter()
                    .filter(|&l| self.editor.is_visible(part, l))
                {
                    for face in FACES {
                        let r = to_screen(rect(part, layer, face, model));
                        painter.rect_stroke(r, 0.0, Stroke::new(2.0, ACCENT), StrokeKind::Inside);
                    }
                }
            }

            let pointer = resp.interact_pointer_pos().or(resp.hover_pos());
            let hover = pointer.and_then(|p| {
                let t = (p - area.min) / px;
                (t.x >= 0.0 && t.y >= 0.0 && t.x < 64.0 && t.y < 64.0)
                    .then_some((t.x as u32, t.y as u32))
            });
            // Para arrastres que salen del lienzo: el píxel más cercano dentro.
            let clamped = pointer.map(|p| {
                let t = (p - area.min) / px;
                (t.x.clamp(0.0, 63.0) as u32, t.y.clamp(0.0, 63.0) as u32)
            });

            let marquee = |r: crate::core::Rect| {
                let r = to_screen(r);
                painter.rect_stroke(r, 0.0, Stroke::new(3.0, Color32::BLACK), StrokeKind::Inside);
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::WHITE), StrokeKind::Inside);
            };
            if let Some(sel) = self.editor.selection() {
                marquee(sel);
            }
            let center =
                |(x, y): (u32, u32)| area.min + Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * px;
            match (self.tool, self.drag_from, clamped) {
                (Tool::Gradient, Some(a), Some(b)) => {
                    painter.line_segment([center(a), center(b)], Stroke::new(2.0, ACCENT));
                }
                (Tool::Stamp, _, _) => {
                    if let (Some(img), Some((x, y))) = (&self.stamp, hover) {
                        marquee(crate::core::Rect {
                            x,
                            y,
                            w: img.width().min(64 - x),
                            h: img.height().min(64 - y),
                        });
                    }
                }
                _ => {}
            }
            self.handle_canvas_input(&resp, hover, clamped);
        });
    }

    fn primary(&self) -> Rgba<u8> {
        Rgba(self.color.to_srgba_unmultiplied())
    }

    fn remember(&mut self, c: Rgba<u8>) {
        self.settings.remember_color(c.0);
        self.settings_changed = true;
    }

    fn handle_canvas_input(
        &mut self,
        resp: &egui::Response,
        at: Option<(u32, u32)>,
        clamped: Option<(u32, u32)>,
    ) {
        self.editor.set_mirror(match self.mirror {
            MirrorMode::Off => Mirror::Off,
            MirrorMode::All => Mirror::All,
            MirrorMode::Selected => self.selected.map_or(Mirror::Off, Mirror::Part),
        });
        match self.tool {
            Tool::Pencil | Tool::Eraser => {
                let down = resp.is_pointer_button_down_on()
                    && resp.ctx.input(|i| i.pointer.primary_down());
                if down && !self.painting {
                    self.editor.begin_stroke();
                    self.painting = true;
                    self.last_px = None;
                    if self.tool == Tool::Pencil {
                        self.remember(self.primary());
                    }
                }
                if !self.painting {
                    return;
                }
                if !down {
                    self.editor.end_stroke();
                    self.painting = false;
                    return;
                }
                let color = if self.tool == Tool::Eraser {
                    ERASE
                } else {
                    self.primary()
                };
                if let Some(p) = at {
                    self.editor.paint_line(self.last_px.unwrap_or(p), p, color);
                }
                self.last_px = at;
            }
            Tool::Fill => {
                if let (true, Some((x, y))) = (resp.clicked(), at) {
                    let c = self.primary();
                    self.editor.fill(x, y, c);
                    self.remember(c);
                }
            }
            Tool::Picker => {
                if let (true, Some((x, y))) = (resp.clicked(), at) {
                    self.color = Hsva::from_srgba_unmultiplied(self.editor.pick(x, y).0);
                }
            }
            Tool::Select => {
                if resp.drag_started() {
                    self.drag_from = at;
                }
                if let (Some(a), Some(b)) = (self.drag_from, clamped)
                    && resp.dragged()
                {
                    self.editor.set_selection(Some(crate::core::Rect {
                        x: a.0.min(b.0),
                        y: a.1.min(b.1),
                        w: a.0.abs_diff(b.0) + 1,
                        h: a.1.abs_diff(b.1) + 1,
                    }));
                }
                if resp.drag_stopped() {
                    self.drag_from = None;
                }
                if resp.clicked() {
                    self.editor.set_selection(None);
                }
            }
            Tool::Gradient => {
                if resp.drag_started() {
                    self.drag_from = at;
                }
                if resp.drag_stopped()
                    && let (Some(a), Some(b)) = (self.drag_from.take(), clamped)
                {
                    let c0 = self.primary();
                    self.editor
                        .gradient(a, b, c0, Rgba(self.color2.to_srgba_unmultiplied()));
                    self.remember(c0);
                }
            }
            Tool::Stamp => {
                if let (true, Some(at), Some(img)) = (resp.clicked(), at, &self.stamp) {
                    self.editor.stamp(at, img);
                }
            }
        }
    }

    // ---- Ventanas ----

    fn dialogs(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog.take() else {
            return;
        };
        let i = &self.i18n;
        let buttons = |ui: &mut egui::Ui, labels: &[(Choice, String)]| {
            let mut picked = None;
            ui.horizontal(|ui| {
                for (c, label) in labels {
                    if ui.button(label).clicked() {
                        picked = Some(*c);
                    }
                }
            });
            picked
        };
        let id = egui::Id::unique("dialog");
        let mut dialog = dialog;
        let resp = egui::Modal::new(id).show(ctx, |ui| match &mut dialog {
            Dialog::Discard(_) => {
                ui.label(i.tr("discard-question"));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("discard-save")),
                        (Choice::Alternate, i.tr("discard-discard")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::Convert(skin) => {
                ui.heading(i.tr_args(
                    "convert-title",
                    &[("model", format!("{:?}", skin.model).into())],
                ));
                ui.label(i.tr("convert-note"));
                let preview = self.preview.get_or_insert_with(|| {
                    ctx.load_texture(
                        "preview",
                        skin_image(skin, |_, _| true),
                        TextureOptions::NEAREST,
                    )
                });
                let before = self.texture.as_ref().map(|t| t.id());
                ui.horizontal(|ui| {
                    for (label, tex) in [
                        (i.tr("convert-before"), before),
                        (i.tr("convert-after"), Some(preview.id())),
                    ] {
                        ui.vertical(|ui| {
                            ui.label(label);
                            if let Some(tex) = tex {
                                ui.image((tex, Vec2::splat(256.0)));
                            }
                        });
                    }
                });
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("convert-apply")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::Export(n, _) => {
                ui.label(i.tr_args("export-translucent", &[("count", (*n).into())]));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("export-anyway")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::Overwrite(path) => {
                ui.label(i.tr_args("export-overwrite", &[("file", file_name(path).into())]));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("export-replace")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::ResetSettings => {
                ui.label(i.tr("settings-reset-question"));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("settings-reset")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::Error(msg) => {
                ui.label(msg.as_str());
                buttons(ui, &[(Choice::Cancel, i.tr("ok"))])
            }
            Dialog::CopyParts(mask) => {
                ui.heading(i.tr("parts-copy-title"));
                ui.label(i.tr("parts-copy-note"));
                mask_grid(ui, i, mask, &[[true; 2]; 6]);
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("parts-copy")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::PasteParts(mask) => {
                ui.heading(i.tr("parts-paste-title"));
                ui.label(i.tr("parts-paste-note"));
                let copied = self.clip.as_ref().map_or([[false; 2]; 6], |c| c.1);
                mask_grid(ui, i, mask, &copied);
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("parts-paste")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
        });
        let choice = match resp.inner {
            Some(c) => c,
            None if resp.should_close() => Choice::Cancel,
            None => {
                self.dialog = Some(dialog);
                return;
            }
        };
        match (dialog, choice) {
            (Dialog::Discard(after), Choice::Accept) => {
                if self.save(false) {
                    self.run(ctx, after);
                }
            }
            (Dialog::Discard(after), Choice::Alternate) => self.run(ctx, after),
            (Dialog::Convert(skin), Choice::Accept) => self.editor.replace(skin),
            (Dialog::Export(_, quick), Choice::Accept) => self.export(quick),
            (Dialog::Overwrite(path), Choice::Accept) => self.write_export(&path),
            (Dialog::ResetSettings, Choice::Accept) => {
                self.settings.reset();
                ctx.set_theme(theme_pref(self.settings.theme));
                self.editor.set_limit(self.settings.undo_levels);
                self.settings_changed = true;
            }
            (Dialog::CopyParts(mask), Choice::Accept) => {
                self.clip = Some((self.editor.skin().clone(), mask));
                self.status = self.i18n.tr("status-parts-copied");
            }
            (Dialog::PasteParts(mask), Choice::Accept) => {
                if let Some((skin, _)) = &self.clip {
                    self.editor.paste_parts(skin, &mask);
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_close(&ctx);
        self.shortcuts(&ctx);
        self.autosave(&ctx);
        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("tools")
            .default_size(260.0)
            .show(ui, |ui| self.side(ui));
        egui::CentralPanel::default_margins().show(ui, |ui| self.central(ui));
        self.settings_window(&ctx);
        self.log_window(&ctx);
        self.dialogs(&ctx);
    }
}

/// Casillas parte × capa. Las que no están en `allowed` quedan desactivadas.
fn mask_grid(ui: &mut egui::Ui, i: &I18n, mask: &mut PartMask, allowed: &PartMask) {
    egui::Grid::new("mask").show(ui, |ui| {
        ui.label("");
        ui.label(i.tr("layer-base"));
        ui.label(i.tr("layer-overlay"));
        ui.end_row();
        for part in PARTS {
            ui.label(part_name(i, part));
            for layer in LAYERS {
                let ok = allowed[part as usize][layer as usize];
                let cell = &mut mask[part as usize][layer as usize];
                *cell &= ok;
                ui.add_enabled(ok, egui::Checkbox::without_text(cell));
            }
            ui.end_row();
        }
    });
}

fn swatch(ui: &mut egui::Ui, c: [u8; 4]) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
    let p = ui.painter();
    p.rect_filled(
        r,
        2.0,
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
    );
    let stroke = if resp.hovered() {
        Stroke::new(2.0, ACCENT)
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };
    p.rect_stroke(r, 2.0, stroke, StrokeKind::Inside);
    resp
}

const PALETTE_IMPORT_MAX: usize = 32;

fn shortcut_of(key: Key, m: egui::Modifiers) -> Shortcut {
    Shortcut {
        ctrl: m.command,
        shift: m.shift,
        alt: m.alt,
        key: key.name().into(),
    }
}

fn shortcut_label(i: &I18n, sc: &Shortcut) -> String {
    let mut parts = Vec::new();
    for (on, k) in [
        (sc.ctrl, i.tr("key-ctrl")),
        (sc.shift, i.tr("key-shift")),
        (sc.alt, i.tr("key-alt")),
    ] {
        if on {
            parts.push(k);
        }
    }
    parts.push(Key::from_name(&sc.key).map_or(sc.key.clone(), |k| k.symbol_or_name().into()));
    parts.join("+")
}

fn action_name(i: &I18n, a: Action) -> String {
    match a {
        Action::New => i.tr("file-new"),
        Action::Open => i.tr("file-open"),
        Action::Save => i.tr("file-save"),
        Action::SaveAs => i.tr("file-save-as"),
        Action::Export => i.tr("file-export"),
        Action::QuickExport => i.tr("file-quick-export"),
        Action::Undo => i.tr("edit-undo"),
        Action::Redo => i.tr("edit-redo"),
        Action::CopySelection => i.tr("edit-copy-selection"),
        Action::EraseSelection => i.tr("edit-erase-selection"),
        Action::Deselect => i.tr("edit-deselect"),
        Action::CopyParts => i.tr("edit-copy-parts"),
        Action::PasteParts => i.tr("edit-paste-parts"),
        Action::Pencil => i.tr("tool-pencil"),
        Action::Eraser => i.tr("tool-eraser"),
        Action::Fill => i.tr("tool-fill"),
        Action::Picker => i.tr("tool-picker"),
        Action::Gradient => i.tr("tool-gradient"),
        Action::Stamp => i.tr("tool-stamp"),
        Action::Select => i.tr("tool-select"),
        Action::ToggleMirror => i.tr("action-toggle-mirror"),
        Action::SwapColors => i.tr("action-swap-colors"),
        Action::ToggleView => i.tr("action-toggle-view"),
    }
}

fn part_name(i: &I18n, p: Part) -> String {
    match p {
        Part::Head => i.tr("part-head"),
        Part::Body => i.tr("part-body"),
        Part::RightArm => i.tr("part-right-arm"),
        Part::LeftArm => i.tr("part-left-arm"),
        Part::RightLeg => i.tr("part-right-leg"),
        Part::LeftLeg => i.tr("part-left-leg"),
    }
}

/// Color de acento para la parte seleccionada; el mismo que usa el shader 3D.
const ACCENT: Color32 = Color32::from_rgb(51, 191, 179);

fn pose_name(i: &I18n, p: Preset) -> String {
    match p {
        Preset::Standing => i.tr("pose-standing"),
        Preset::Walking => i.tr("pose-walking"),
        Preset::Waving => i.tr("pose-waving"),
        Preset::ArmUp => i.tr("pose-arm-up"),
        Preset::Crouching => i.tr("pose-crouching"),
        Preset::Running => i.tr("pose-running"),
        Preset::Combat => i.tr("pose-combat"),
        Preset::Neutral => i.tr("pose-neutral"),
    }
}

fn theme_pref(t: Theme) -> egui::ThemePreference {
    match t {
        Theme::System => egui::ThemePreference::System,
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    }
}

/// Imagen para mostrar, con las partes ocultas en blanco (transparentes).
fn skin_image(skin: &Skin, visible: impl Fn(Part, Layer) -> bool) -> ColorImage {
    let mut px = skin.pixels.clone();
    for part in PARTS {
        for layer in LAYERS {
            if !visible(part, layer) {
                for face in FACES {
                    for (x, y) in rect(part, layer, face, skin.model).pixels() {
                        px.put_pixel(x, y, ERASE);
                    }
                }
            }
        }
    }
    ColorImage::from_rgba_unmultiplied([64, 64], px.as_raw())
}

fn png_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter("PNG", &["png"])
}

fn with_png_ext(mut p: PathBuf) -> PathBuf {
    if !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) {
        p.set_extension("png");
    }
    p
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
