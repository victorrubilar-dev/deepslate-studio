//! Ventana principal: menús, panel de herramientas, lienzo 2D y diálogos.

mod icons;
mod library_view;
mod settings_window;
mod style;

use crate::core::{
    FACES, Face, LAYERS, Layer, LoadError, Model, PARTS, Part, Skin, convert_model, rect,
};
use crate::editor::colors::{palette_from, parse_hex, to_hex};
use crate::editor::image_import::{Fit, fit, to_front};
use crate::editor::templates::{self, Template};
use crate::editor::{ERASE, Editor, Mirror, PartMask, Tool};
use crate::i18n::I18n;
use crate::project::{self, Card, Project, ProjectError};
use crate::render3d::{Camera, Hit, PRESETS, Pose, Preset, Renderer3d, Scene, pick, project};
use crate::settings::{Action, NamedColor, Settings, Shortcut, StartView, Theme, View};
use eframe::egui::{
    self, Color32, ColorImage, Key, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2, color_picker, ecolor::Hsva,
};
use icons::Icon;
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};
use style::ACCENT;

#[derive(Clone, Copy)]
enum After {
    /// Cerrar la pestaña activa.
    CloseTab,
    /// Cerrar la ventana: se pregunta por cada pestaña con cambios.
    Close,
}

/// Una skin abierta en una pestaña. La activa vive en los campos de `App`
/// (`editor`, `path`, `camera`, `selected`); `tabs[active]` es solo un hueco de intercambio.
struct Doc {
    editor: Editor,
    path: Option<PathBuf>,
    camera: Camera,
    selected: Option<Part>,
    card: Card,
}

impl Doc {
    fn empty() -> Self {
        Self {
            editor: Editor::new(Skin::blank(Model::Steve), 1),
            path: None,
            camera: Camera::default(),
            selected: None,
            card: Card::default(),
        }
    }
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
    EditCard(PathBuf, library_view::CardDraft),
    ImportImage(Box<ImportState>),
    /// Ruta y nombre de la tarjeta.
    DeleteSkin(PathBuf, String),
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

/// Diálogo de «Imagen al frente». El recorte está en píxeles de la imagen original.
struct ImportState {
    src: RgbaImage,
    crop: (u32, u32, u32, u32),
    fit: Fit,
    layer: Layer,
    drag_from: Option<(u32, u32)>,
    /// Vista previa reducida de la imagen (una foto grande no entra como textura).
    preview: Option<TextureHandle>,
}

impl ImportState {
    fn new(src: RgbaImage) -> Self {
        Self {
            crop: (0, 0, src.width(), src.height()),
            src,
            fit: Fit::Contain,
            layer: Layer::Base,
            drag_from: None,
            preview: None,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, i: &I18n, model: Model) {
        let ctx = ui.ctx().clone();
        let src = &self.src;
        let tex = self.preview.get_or_insert_with(|| {
            let k = (512.0 / src.width().max(src.height()) as f32).min(1.0);
            let small = image::imageops::thumbnail(
                src,
                ((src.width() as f32 * k) as u32).max(1),
                ((src.height() as f32 * k) as u32).max(1),
            );
            ctx.load_texture(
                "import-src",
                ColorImage::from_rgba_unmultiplied(
                    [small.width() as usize, small.height() as usize],
                    small.as_raw(),
                ),
                TextureOptions::LINEAR,
            )
        });
        ui.horizontal_top(|ui| {
            // Imagen con el recorte; arrastrar marca uno nuevo.
            let (w, h) = (self.src.width() as f32, self.src.height() as f32);
            let k = (320.0 / w).min(320.0 / h);
            let (area, resp) = ui.allocate_exact_size(Vec2::new(w * k, h * k), Sense::drag());
            ui.painter().image(
                tex.id(),
                area,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            let to_img = |p: Pos2| {
                let v = (p - area.min) / k;
                (
                    v.x.clamp(0.0, w - 1.0) as u32,
                    v.y.clamp(0.0, h - 1.0) as u32,
                )
            };
            if resp.drag_started() {
                self.drag_from = resp.interact_pointer_pos().map(to_img);
            }
            if let (Some(a), Some(p)) = (self.drag_from, resp.interact_pointer_pos())
                && resp.dragged()
            {
                let b = to_img(p);
                self.crop = (
                    a.0.min(b.0),
                    a.1.min(b.1),
                    a.0.abs_diff(b.0) + 1,
                    a.1.abs_diff(b.1) + 1,
                );
            }
            if resp.drag_stopped() {
                self.drag_from = None;
            }
            let (x, y, cw, ch) = self.crop;
            let crop = Rect::from_min_size(
                area.min + Vec2::new(x as f32, y as f32) * k,
                Vec2::new(cw as f32, ch as f32) * k,
            );
            ui.painter()
                .rect_stroke(crop, 0.0, Stroke::new(2.0, ACCENT), StrokeKind::Outside);

            // Resultado sobre la silueta frontal.
            let canvas = fit(&self.src, self.crop, model, self.fit);
            let result = ctx.load_texture(
                "import-result",
                ColorImage::from_rgba_unmultiplied(
                    [canvas.width() as usize, canvas.height() as usize],
                    canvas.as_raw(),
                ),
                TextureOptions::NEAREST,
            );
            let size = Vec2::new(canvas.width() as f32, canvas.height() as f32) * 8.0;
            let (r, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter()
                .rect_filled(r, 0.0, ui.visuals().extreme_bg_color);
            ui.painter().image(
                result.id(),
                r,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.fit, Fit::Contain, i.tr("import-fit-contain"));
            ui.radio_value(&mut self.fit, Fit::Cover, i.tr("import-fit-cover"));
            ui.separator();
            ui.radio_value(&mut self.layer, Layer::Base, i.tr("layer-base"));
            ui.radio_value(&mut self.layer, Layer::Overlay, i.tr("layer-overlay"));
            ui.separator();
            if ui.button(i.tr("import-whole")).clicked() {
                self.crop = (0, 0, self.src.width(), self.src.height());
            }
        });
    }
}

/// Animación de la vista 3D: va y vuelve entre la pose de la skin y `target`.
/// Solo se ve en pantalla; no cambia la pose guardada.
struct Anim {
    playing: bool,
    /// `None`: el espejo de la pose (sirve para caminar y correr).
    target: Option<Preset>,
    /// Ciclos por segundo.
    speed: f32,
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
    /// Tarjeta de biblioteca de la pestaña activa; se guarda con el proyecto.
    card: Card,
    tabs: Vec<Doc>,
    active: usize,
    lib: library_view::LibraryState,
    anim: Anim,
    tool: Tool,
    color: Hsva,
    texture: Option<TextureHandle>,
    texture_rev: Option<u64>,
    /// Diálogo de conversión: el personaje en 3D antes y después, y cuál se muestra.
    convert_views: Option<[TextureHandle; 2]>,
    convert_after: bool,
    painting: bool,
    last_px: Option<(u32, u32)>,
    /// Trazo en la vista 3D: último píxel pintado, última posición del cursor y la skin
    /// al empezar (decide qué es transparente, para no atravesar lo que se va borrando).
    last_hit: Option<Hit>,
    last_pos3d: Option<Pos2>,
    stroke_start: Option<RgbaImage>,
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
        style::apply(&cc.egui_ctx);
        cc.egui_ctx.set_theme(theme_pref(settings.theme));
        let mut settings = settings;
        match settings.start_view {
            StartView::Last => {}
            StartView::Library => settings.view = View::Library,
            StartView::Flat => settings.view = View::Flat,
            StartView::ThreeD => settings.view = View::ThreeD,
        }
        if cc.wgpu_render_state.is_none() {
            log::warn!("sin wgpu: la vista 3D no estará disponible");
        }
        // Un archivo pasado al abrir (desde «Abrir con» o la terminal) gana sobre la última skin.
        let last = std::env::args_os().nth(1).map(PathBuf::from).or_else(|| {
            settings
                .open_last
                .then(|| settings.last_file.clone())
                .flatten()
        });
        let start_tool = settings.start_tool;
        let lang = settings.resolved_language();
        let mut app = Self {
            i18n: I18n::new(lang),
            editor: Editor::new(Skin::blank(Model::Steve), settings.undo_levels),
            settings,
            settings_path,
            settings_changed: false,
            show_settings: false,
            path: None,
            card: Card::default(),
            tabs: vec![Doc::empty()],
            active: 0,
            lib: Default::default(),
            anim: Anim {
                playing: false,
                target: None,
                speed: 1.0,
            },
            tool: Tool::Pencil,
            color: Hsva::from_srgba_unmultiplied([196, 92, 52, 255]),
            texture: None,
            texture_rev: None,
            convert_views: None,
            convert_after: true,
            painting: false,
            last_px: None,
            last_hit: None,
            last_pos3d: None,
            stroke_start: None,
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
        let from_args = std::env::args_os().nth(1).is_some();
        if let Some(path) = last.filter(|p| p.exists())
            && app.open_path(path)
            && from_args
            && app.settings.view == View::Library
        {
            // Abrir un archivo desde el explorador lleva directo al editor.
            app.settings.view = View::ThreeD;
        }
        app
    }

    fn apply_undo_limit(&mut self) {
        let n = self.settings.undo_levels;
        self.editor.set_limit(n);
        for d in &mut self.tabs {
            d.editor.set_limit(n);
        }
    }

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.editor = Editor::new(project.skin, self.settings.undo_levels);
        self.editor.set_pose(project.pose);
        self.editor.mark_saved();
        self.card = project.card;
        self.path = path;
        self.texture_rev = None;
        self.uploaded = None;
    }

    // ---- Archivo ----

    fn request(&mut self, ctx: &egui::Context, after: After) {
        if let After::Close = after
            && let Some(i) = (0..self.tabs.len()).find(|&i| self.tab_dirty(i))
        {
            self.switch_tab(i);
        }
        if self.editor.is_dirty() {
            self.dialog = Some(Dialog::Discard(after));
        } else {
            self.run(ctx, after);
        }
    }

    fn run(&mut self, ctx: &egui::Context, after: After) {
        match after {
            After::CloseTab => self.close_tab(),
            After::Close if (0..self.tabs.len()).any(|i| self.tab_dirty(i)) => {
                // Esta pestaña ya se guardó o se descartó; se pasa a la siguiente con cambios.
                self.close_tab();
                self.request(ctx, After::Close);
            }
            After::Close => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    // ---- Pestañas ----

    /// Intercambia la pestaña activa entre los campos de `App` y su hueco en `tabs`.
    fn swap_active(&mut self) {
        let d = &mut self.tabs[self.active];
        std::mem::swap(&mut self.editor, &mut d.editor);
        std::mem::swap(&mut self.path, &mut d.path);
        std::mem::swap(&mut self.camera, &mut d.camera);
        std::mem::swap(&mut self.selected, &mut d.selected);
        std::mem::swap(&mut self.card, &mut d.card);
    }

    fn tab_dirty(&self, i: usize) -> bool {
        if i == self.active {
            self.editor.is_dirty()
        } else {
            self.tabs[i].editor.is_dirty()
        }
    }

    fn tab_path(&self, i: usize) -> Option<&Path> {
        if i == self.active {
            self.path.as_deref()
        } else {
            self.tabs[i].path.as_deref()
        }
    }

    fn switch_tab(&mut self, i: usize) {
        if i == self.active || i >= self.tabs.len() {
            return;
        }
        if self.painting {
            self.editor.end_stroke();
            self.painting = false;
        }
        self.swap_active();
        self.active = i;
        self.swap_active();
        self.texture_rev = None;
        self.uploaded = None;
    }

    fn new_tab(&mut self, project: Project, path: Option<PathBuf>) {
        self.tabs.push(Doc::empty());
        self.switch_tab(self.tabs.len() - 1);
        self.set_project(project, path);
    }

    /// Cierra la pestaña activa sin preguntar. La última se reemplaza por una vacía.
    fn close_tab(&mut self) {
        if self.tabs.len() == 1 {
            self.set_project(blank_project(), None);
            return;
        }
        self.tabs.remove(self.active);
        self.active = self.active.min(self.tabs.len() - 1);
        self.swap_active();
        // El hueco quedó con la pestaña cerrada; se libera su historial.
        self.tabs[self.active] = Doc::empty();
        self.texture_rev = None;
        self.uploaded = None;
    }

    /// Actualiza la tarjeta de la pestaña que tenga abierto `path`, si hay alguna.
    fn set_open_card(&mut self, path: &Path, card: Card) {
        if let Some(i) = (0..self.tabs.len()).find(|&i| self.tab_path(i) == Some(path)) {
            if i == self.active {
                self.card = card;
            } else {
                self.tabs[i].card = card;
            }
        }
    }

    /// Una pestaña sin archivo, sin cambios y sin historial: se puede reutilizar al abrir.
    fn active_is_untouched(&self) -> bool {
        self.path.is_none() && !self.editor.is_dirty() && !self.editor.can_undo()
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut switch = None;
        let mut close = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for i in 0..self.tabs.len() {
                let name = self
                    .tab_path(i)
                    .map_or_else(|| self.i18n.tr("untitled"), file_name);
                let label = if self.tab_dirty(i) {
                    format!("{name} •")
                } else {
                    name
                };
                let tab = style::tab(ui, None, &label, i == self.active);
                if i == self.active {
                    style::slide_marker(ui, egui::Id::unique("tab-marker"), tab.rect);
                }
                if tab.clicked() {
                    switch = Some(i);
                }
                if tab.middle_clicked() {
                    close = Some(i);
                }
                let x = egui::Button::image(
                    icons::image(ctx, Icon::Close).fit_to_exact_size(Vec2::splat(12.0)),
                )
                .image_tint_follows_text_color(true)
                .frame_when_inactive(false);
                if ui.add(x).on_hover_text(self.i18n.tr("tab-close")).clicked() {
                    close = Some(i);
                }
                ui.add_space(10.0);
            }
            let plus = egui::Button::image(icons::image(ctx, Icon::Plus))
                .image_tint_follows_text_color(true)
                .frame_when_inactive(false);
            if ui
                .add(plus)
                .on_hover_text(self.i18n.tr("file-new"))
                .clicked()
            {
                self.new_tab(blank_project(), None);
            }
        });
        if let Some(i) = switch {
            self.switch_tab(i);
        }
        if let Some(i) = close {
            self.switch_tab(i);
            self.request(ctx, After::CloseTab);
        }
    }

    fn open(&mut self) {
        if let Some(path) = png_dialog().pick_file() {
            self.open_path(path);
        }
    }

    /// Devuelve `true` si la skin quedó abierta en una pestaña.
    fn open_path(&mut self, path: PathBuf) -> bool {
        if let Some(i) = (0..self.tabs.len()).find(|&i| self.tab_path(i) == Some(path.as_path())) {
            self.switch_tab(i);
            return true;
        }
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.error(
                    self.i18n
                        .tr_args("error-io", &[("detail", e.to_string().into())]),
                );
                return false;
            }
        };
        match project::load(&bytes) {
            Ok(project) => {
                // Un PNG suelto se importa sin ruta, para que "Guardar" no lo sobrescriba sin
                // preguntar. Los de la biblioteca sí la conservan: ya son copias de la app.
                let in_library = library_view::library_dir(&self.settings)
                    .is_some_and(|d| path.parent() == Some(d.as_path()));
                let is_project = project::is_project(&bytes) || in_library;
                let key = if is_project {
                    "status-opened"
                } else {
                    "status-imported"
                };
                self.status = self.i18n.tr_args(key, &[("file", file_name(&path).into())]);
                log::info!("abierto {}", path.display());
                self.remember_file(&path);
                let path = is_project.then_some(path);
                if self.active_is_untouched() {
                    self.set_project(project, path);
                } else {
                    self.new_tab(project, path);
                }
                true
            }
            Err(e) => {
                self.error(self.load_error(e));
                false
            }
        }
    }

    /// Archivos soltados desde el explorador. En la biblioteca, las skins se copian ahí;
    /// en el editor se abren, y una imagen que no es skin va a «Imagen al frente».
    fn handle_drops(&mut self, ctx: &egui::Context) {
        let (hovering, dropped) = ctx.input(|i| {
            let paths: Vec<PathBuf> = i
                .raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect();
            (!i.raw.hovered_files.is_empty(), paths)
        });
        let library = self.settings.view == View::Library;
        let shown = ctx.animate_bool_with_time(
            egui::Id::unique("drop-fade"),
            hovering,
            ctx.global_style().animation_time,
        );
        if shown > 0.0 {
            let key = if library { "drop-library" } else { "drop-open" };
            let rect = ctx.content_rect();
            let mut painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("drop"),
            ));
            painter.multiply_opacity(shown);
            painter.rect_filled(rect, 0.0, Color32::from_black_alpha(170));
            // El marco se acerca desde afuera mientras aparece.
            painter.rect_stroke(
                rect.shrink(12.0 + 12.0 * (1.0 - shown)),
                0.0,
                Stroke::new(3.0, ACCENT),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.i18n.tr(key),
                style::pixel_font(style::SIZE_HERO),
                Color32::WHITE,
            );
        }
        if dropped.is_empty() || self.dialog.is_some() {
            return;
        }
        let (skins, others): (Vec<PathBuf>, Vec<PathBuf>) = dropped
            .into_iter()
            .partition(|p| std::fs::read(p).is_ok_and(|b| project::load(&b).is_ok()));
        if library {
            if let Some(dir) = library_view::library_dir(&self.settings) {
                if !skins.is_empty() {
                    self.import_files(&dir, skins);
                }
                if !others.is_empty() {
                    self.error(self.i18n.tr("error-drop-library"));
                }
            }
            return;
        }
        for p in skins {
            self.open_path(p);
        }
        if let Some(p) = others.into_iter().next() {
            match image::open(&p) {
                Ok(img) => {
                    self.dialog = Some(Dialog::ImportImage(Box::new(ImportState::new(
                        img.into_rgba8(),
                    ))))
                }
                Err(e) => {
                    log::warn!("soltado {}: {e}", p.display());
                    self.error(self.i18n.tr("error-image"));
                }
            }
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
            _ => match library_view::library_dir(&self.settings)
                .filter(|d| d.is_dir())
                .map_or_else(png_dialog, |d| png_dialog().set_directory(d))
                .set_file_name("skin.png")
                .save_file()
            {
                Some(p) => with_png_ext(p),
                None => return false,
            },
        };
        match std::fs::write(
            &path,
            project::save(self.editor.skin(), self.editor.pose(), &self.card),
        ) {
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
        if self.dialog.is_none() && !self.painting {
            for d in &mut self.tabs {
                if let (true, Some(path)) = (d.editor.is_dirty(), &d.path) {
                    match std::fs::write(
                        path,
                        project::save(d.editor.skin(), d.editor.pose(), &d.card),
                    ) {
                        Ok(()) => d.editor.mark_saved(),
                        Err(e) => log::error!("autoguardado de {}: {e}", path.display()),
                    }
                }
            }
            if self.editor.is_dirty() && self.path.is_some() && self.save(false) {
                log::info!("autoguardado");
            }
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
            Action::New => self.new_tab(blank_project(), None),
            Action::Open => self.open(),
            Action::CloseTab => self.request(ctx, After::CloseTab),
            Action::NextTab => self.switch_tab((self.active + 1) % self.tabs.len()),
            Action::PrevTab => {
                self.switch_tab((self.active + self.tabs.len() - 1) % self.tabs.len())
            }
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
                    View::ThreeD | View::Preview | View::Library => View::Flat,
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
        let name = action_name(&self.i18n, action);
        let mut button = match action_icon(action) {
            Some(icon) => style::menu_button(ui, icon, name),
            None => egui::Button::new(name),
        };
        if let Some(sc) = self.settings.shortcut(action) {
            button = button.shortcut_text(shortcut_label(&self.i18n, &sc));
        }
        if ui.add_enabled(self.can(action), button).clicked() {
            let ctx = ui.ctx().clone();
            self.act(&ctx, action);
        }
    }

    fn handle_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested())
            && !self.closing
            && (0..self.tabs.len()).any(|i| self.tab_dirty(i))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request(ctx, After::Close);
        }
    }

    // ---- Paneles ----

    fn menu(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(self.i18n.tr("menu-file"), |ui| {
                self.menu_item(ui, Action::New);
                if ui
                    .add(style::menu_button(
                        ui,
                        Icon::New,
                        self.i18n.tr("file-new-base"),
                    ))
                    .clicked()
                {
                    let skin = Skin {
                        pixels: templates::draw(Template::Base, Model::Steve, self.primary()),
                        model: Model::Steve,
                    };
                    self.new_tab(
                        Project {
                            skin,
                            ..blank_project()
                        },
                        None,
                    );
                }
                self.menu_item(ui, Action::Open);
                self.menu_item(ui, Action::CloseTab);
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
                ui.separator();
                if ui
                    .add(style::menu_button(
                        ui,
                        Icon::Open,
                        self.i18n.tr("edit-image-front"),
                    ))
                    .clicked()
                {
                    self.import_image();
                }
                ui.menu_button(self.i18n.tr("edit-apply-template"), |ui| {
                    ui.weak(self.i18n.tr("template-color-note"));
                    for (t, name) in [
                        (Template::Shirt, self.i18n.tr("template-shirt")),
                        (Template::Pants, self.i18n.tr("template-pants")),
                        (Template::Hair, self.i18n.tr("template-hair")),
                        (Template::Armor, self.i18n.tr("template-armor")),
                    ] {
                        if ui.button(name).clicked() {
                            let layer =
                                templates::draw(t, self.editor.skin().model, self.primary());
                            self.editor.apply_layer(&layer);
                        }
                    }
                });
            });
            let settings = style::menu_button(ui, Icon::Settings, self.i18n.tr("menu-settings"))
                .frame_when_inactive(false);
            if ui.add(settings).clicked() {
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
            let id = egui::Id::unique("status-since");
            let now = ui.input(|i| i.time);
            let (text, since) = ui
                .data(|d| d.get_temp::<(String, f64)>(id))
                .unwrap_or_default();
            let since = if text == self.status {
                since
            } else {
                ui.data_mut(|d| d.insert_temp(id, (self.status.clone(), now)));
                now
            };
            let t = style::appear(ui, since, 0.0, 1.2);
            let color = ACCENT.lerp_to_gamma(ui.visuals().text_color(), t);
            ui.label(egui::RichText::new(&self.status).color(color));
        });
    }

    fn side(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.side_tools(ui);
            ui.separator();
            self.side_colors(ui);
            ui.separator();
            self.side_parts(ui);
            if self.settings.view == View::ThreeD {
                ui.separator();
                self.side_joint(ui);
            }
        });
    }

    /// Ajuste fino de la articulación de la parte seleccionada.
    fn side_joint(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        style::section(ui, Icon::Cube, &i.tr("joint-title"));
        let Some(part) = self.selected else {
            ui.weak(i.tr("joint-hint"));
            return;
        };
        ui.label(part_name(i, part));
        let mut pose = *self.editor.pose();
        let angles = pose.angles_mut(part);
        let lim = crate::render3d::ANGLE_LIMIT;
        egui::Grid::new("joint").num_columns(2).show(ui, |ui| {
            for (k, axis) in ["X", "Y", "Z"].into_iter().enumerate() {
                ui.label(axis);
                ui.add(
                    egui::Slider::new(&mut angles[k], -lim..=lim)
                        .suffix("°")
                        .step_by(1.0),
                );
                ui.end_row();
            }
        });
        if ui.button(i.tr("joint-reset")).clicked() {
            *angles = [0.0; 3];
        }
        ui.weak(i.tr("joint-axes"));
        self.editor.set_pose(pose);
    }

    fn side_tools(&mut self, ui: &mut egui::Ui) {
        let i = &self.i18n;
        style::section(ui, Icon::Pencil, &i.tr("tools"));
        let mut current = String::new();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for (tool, action) in [
                (Tool::Pencil, Action::Pencil),
                (Tool::Eraser, Action::Eraser),
                (Tool::Fill, Action::Fill),
                (Tool::Picker, Action::Picker),
                (Tool::Gradient, Action::Gradient),
                (Tool::Stamp, Action::Stamp),
                (Tool::Select, Action::Select),
            ] {
                let name = action_name(i, action);
                let tip = match self.settings.shortcut(action) {
                    Some(sc) => format!("{name} ({})", shortcut_label(i, &sc)),
                    None => name.clone(),
                };
                let icon = action_icon(action).unwrap_or(Icon::Pencil);
                let resp = style::icon_button(ui, icon, self.tool == tool, &tip);
                if resp.clicked() {
                    self.tool = tool;
                }
                if tool == Tool::Stamp && self.stamp.is_none() {
                    resp.on_hover_text(i.tr("stamp-empty"));
                }
                if self.tool == tool {
                    current = name;
                }
            }
        });
        ui.label(egui::RichText::new(current).font(style::pixel_font(style::SIZE_UI)));
        ui.horizontal_wrapped(|ui| {
            ui.add(icons::image(ui.ctx(), Icon::Mirror).tint(ui.visuals().text_color()));
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
        style::section(ui, Icon::Fill, &i.tr("color"));
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
            if ui
                .add(style::menu_button(
                    ui,
                    Icon::Swap,
                    action_name(i, Action::SwapColors),
                ))
                .clicked()
            {
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
            self.convert_views = None;
            self.convert_after = true;
            self.dialog = Some(Dialog::Convert(convert_model(self.editor.skin(), other)));
        }
        ui.separator();

        style::section(ui, Icon::Library, &i.tr("layers"));
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
        if before != View::Library {
            self.lib.return_view = before;
        }
        let view3d = matches!(before, View::ThreeD | View::Preview);
        ui.horizontal_wrapped(|ui| {
            let i = &self.i18n;
            for (view, icon, key) in [
                (View::Library, Icon::Library, "view-library"),
                (View::ThreeD, Icon::Cube, "view-3d"),
                (View::Flat, Icon::Flat, "view-2d"),
                (View::Preview, Icon::Preview, "view-preview"),
            ] {
                let on = self.settings.view == view;
                let resp = style::tab(ui, Some(icon), &i.tr(key), on);
                if on {
                    style::slide_marker(ui, egui::Id::unique("view-marker"), resp.rect);
                }
                if resp.clicked() {
                    self.settings.view = view;
                }
                if view == View::Library {
                    ui.separator();
                }
            }
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
                ui.toggle_value(
                    &mut self.anim.playing,
                    (icons::image(ui.ctx(), Icon::Play), i.tr("anim-play")),
                );
                if self.anim.playing {
                    let shown = self
                        .anim
                        .target
                        .map_or_else(|| i.tr("anim-mirror"), |p| pose_name(i, p));
                    egui::ComboBox::from_id_salt("anim")
                        .selected_text(shown)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.anim.target, None, i.tr("anim-mirror"));
                            for p in PRESETS {
                                ui.selectable_value(
                                    &mut self.anim.target,
                                    Some(p),
                                    pose_name(i, p),
                                );
                            }
                        })
                        .response
                        .on_hover_text(i.tr("anim-target"));
                    ui.add(
                        egui::Slider::new(&mut self.anim.speed, 0.2..=3.0).text(i.tr("anim-speed")),
                    );
                }
                if ui
                    .add(style::menu_button(ui, Icon::Reset, i.tr("view-reset")))
                    .clicked()
                {
                    self.camera = Camera::default();
                }
                if ui
                    .add(style::menu_button(ui, Icon::Camera, i.tr("capture")))
                    .clicked()
                {
                    self.capture();
                }
            }
        });
        if self.settings.view == View::Preview {
            self.preview_bar(ui);
        }
        if self.settings.view != before {
            self.settings_changed = true;
            // Al entrar a la biblioteca se vuelve a leer la carpeta: puede haber cambiado afuera.
            if self.settings.view == View::Library {
                self.lib.stale = true;
                self.lib.shown_at = ui.input(|i| i.time);
            }
        }
        ui.separator();
        match self.settings.view {
            View::Library => self.library(ui),
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

    /// El personaje actual y `after`, de tres cuartos y de pie, para comparar en un diálogo.
    /// Sin vista 3D se muestran las texturas planas.
    fn model_views(&mut self, ctx: &egui::Context, after: &Skin) -> [TextureHandle; 2] {
        let camera = Camera::thumbnail();
        let pose = Pose::default();
        let lighting = self.settings.lighting;
        let renderer = self.renderer.as_ref();
        let shot = |skin: &Skin, name: &str| {
            let img = renderer.and_then(|r| {
                r.upload_skin(&skin.pixels);
                r.capture(
                    320,
                    &Scene {
                        model: skin.model,
                        pose: &pose,
                        camera: &camera,
                        selected: None,
                        visible: &|_, _| true,
                        cape: false,
                        lighting,
                    },
                )
            });
            match img {
                Some(img) => ctx.load_texture(
                    name,
                    ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        img.as_raw(),
                    ),
                    TextureOptions::LINEAR,
                ),
                None => {
                    ctx.load_texture(name, skin_image(skin, |_, _| true), TextureOptions::NEAREST)
                }
            }
        };
        let views = [
            shot(self.editor.skin(), "convert-before"),
            shot(after, "convert-after"),
        ];
        // La skin en la GPU cambió: la vista 3D tiene que volver a subirla.
        self.uploaded = None;
        views
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

    /// Pose que se dibuja: la de la skin o, con la animación en marcha, la del momento.
    fn shown_pose(&self, ctx: &egui::Context) -> Pose {
        let pose = *self.editor.pose();
        if !self.anim.playing {
            return pose;
        }
        let target = self
            .anim
            .target
            .map_or_else(|| pose.mirrored(), Preset::pose);
        let phase = ctx.input(|i| i.time) as f32 * self.anim.speed * std::f32::consts::TAU;
        ctx.request_repaint();
        pose.lerp(&target, (1.0 - phase.cos()) / 2.0)
    }

    fn view3d(&mut self, ui: &mut egui::Ui, preview: bool) {
        if self.renderer.is_none() {
            ui.label(self.i18n.tr("view3d-unavailable"));
            return;
        }
        self.sync_3d(preview);
        let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        // En el editor el botón izquierdo pinta; se gira con el derecho o la rueda presionada.
        let rotating = resp.dragged_by(PointerButton::Secondary)
            || resp.dragged_by(PointerButton::Middle)
            || (preview && resp.dragged_by(PointerButton::Primary));
        if rotating {
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
        // Con la animación en marcha no se pinta: el modelo no está en la pose de la skin.
        let hover = if preview || self.anim.playing {
            None
        } else {
            self.paint_3d(ui, &resp, area)
        };
        let model = self.model_3d(preview);
        let pose = self.shown_pose(ui.ctx());
        let editor = &self.editor;
        let visible = |p, l| preview || editor.is_visible(p, l);
        let scene = Scene {
            model,
            pose: &pose,
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
        if let Some(h) = hover {
            let pts: Vec<Pos2> = h
                .quad
                .iter()
                .map(|&[nx, ny]| {
                    area.center() + Vec2::new(nx * area.width(), -ny * area.height()) / 2.0
                })
                .collect();
            painter.add(egui::Shape::closed_line(
                pts.clone(),
                Stroke::new(3.0, Color32::BLACK),
            ));
            painter.add(egui::Shape::closed_line(
                pts,
                Stroke::new(1.5, Color32::WHITE),
            ));
        }
        if !preview && !self.painting {
            resp.on_hover_text(self.i18n.tr("view3d-hint"));
        }
    }

    /// Píxel de la skin bajo `pos` en la vista 3D del editor. Durante un trazo, lo transparente
    /// se decide con la skin de cuando empezó.
    fn hit_3d(&self, area: Rect, pos: Pos2) -> Option<Hit> {
        let editor = &self.editor;
        let pixels = match (&self.stroke_start, self.painting) {
            (Some(start), true) => start,
            _ => &editor.skin().pixels,
        };
        // La capa exterior transparente se atraviesa, salvo con la base oculta: así se puede
        // pintar la capa exterior ocultando la base.
        let solid = |p, l, x, y| {
            l == Layer::Base || pixels.get_pixel(x, y)[3] > 0 || !editor.is_visible(p, Layer::Base)
        };
        pick(
            editor.skin().model,
            editor.pose(),
            &self.camera,
            &|p, l| editor.is_visible(p, l),
            &solid,
            area.width() / area.height().max(1.0),
            (
                (pos.x - area.min.x) / area.width() * 2.0 - 1.0,
                1.0 - (pos.y - area.min.y) / area.height() * 2.0,
            ),
        )
    }

    /// Clic izquierdo en la vista 3D: selecciona la parte y usa la herramienta activa.
    /// Devuelve el píxel a resaltar bajo el cursor.
    fn paint_3d(&mut self, ui: &egui::Ui, resp: &egui::Response, area: Rect) -> Option<Hit> {
        let hover = resp.hover_pos().and_then(|p| self.hit_3d(area, p));
        let pressed = resp.hovered() && ui.input(|i| i.pointer.primary_pressed());
        if pressed {
            self.selected = hover.map(|h| h.part);
        }
        self.sync_mirror();
        match self.tool {
            Tool::Pencil | Tool::Eraser => {
                let down =
                    resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
                if down && !self.painting {
                    self.editor.begin_stroke();
                    self.painting = true;
                    self.stroke_start = Some(self.editor.skin().pixels.clone());
                    self.last_hit = None;
                    self.last_pos3d = None;
                    if self.tool == Tool::Pencil {
                        self.remember(self.primary());
                    }
                }
                if self.painting && !down {
                    self.editor.end_stroke();
                    self.painting = false;
                    self.stroke_start = None;
                }
                if !self.painting {
                    return hover;
                }
                let color = if self.tool == Tool::Eraser {
                    ERASE
                } else {
                    self.primary()
                };
                let Some(pos) = resp.interact_pointer_pos() else {
                    return hover;
                };
                // Se recorre el tramo en pantalla, no en la textura: un trazo puede pasar de una
                // cara a otra que en la textura está lejos.
                let from = self.last_pos3d.unwrap_or(pos);
                let steps = ((pos - from).length() / 2.0).ceil().max(1.0) as usize;
                for k in 1..=steps {
                    let hit = self.hit_3d(area, from.lerp(pos, k as f32 / steps as f32));
                    if let Some(h) = hit {
                        let start = self
                            .last_hit
                            .filter(|l| (l.part, l.layer, l.face) == (h.part, h.layer, h.face))
                            .map_or((h.x, h.y), |l| (l.x, l.y));
                        self.editor.paint_line(start, (h.x, h.y), color);
                    }
                    self.last_hit = hit;
                }
                self.last_pos3d = Some(pos);
            }
            Tool::Fill => {
                if let (true, Some(h)) = (resp.clicked(), hover) {
                    let c = self.primary();
                    self.editor.fill(h.x, h.y, c);
                    self.remember(c);
                }
            }
            Tool::Picker => {
                if let (true, Some(h)) = (resp.clicked(), hover) {
                    self.color = Hsva::from_srgba_unmultiplied(self.editor.pick(h.x, h.y).0);
                }
            }
            // Degradado, sello y selección solo tienen sentido en 2D.
            Tool::Gradient | Tool::Stamp | Tool::Select => return None,
        }
        hover
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
            if self.settings.show_labels && px >= 6.0 {
                // Cada parte tiene un rincón sin uso arriba a la izquierda: ahí va su nombre.
                let font = egui::FontId::proportional((px * 1.1).clamp(9.0, 13.0));
                for part in PARTS {
                    for layer in LAYERS
                        .into_iter()
                        .filter(|&l| self.editor.is_visible(part, l))
                    {
                        let x = rect(part, layer, Face::Right, model).x;
                        let y = rect(part, layer, Face::Top, model).y;
                        let pos = area.min + Vec2::new(x as f32, y as f32) * px + Vec2::splat(2.0);
                        let name = part_name(&self.i18n, part);
                        let text = match layer {
                            Layer::Base => name,
                            Layer::Overlay => self
                                .i18n
                                .tr_args("part-label-overlay", &[("part", name.into())]),
                        };
                        let galley = painter.layout_no_wrap(text, font.clone(), v.text_color());
                        let bg = Rect::from_min_size(pos, galley.size()).expand(1.0);
                        painter.rect_filled(bg, 2.0, v.extreme_bg_color.gamma_multiply(0.6));
                        painter.galley(pos, galley, v.text_color());
                    }
                }
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

    fn import_image(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(self.i18n.tr("import-images"), &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return;
        };
        match image::open(&path) {
            Ok(img) => {
                self.dialog = Some(Dialog::ImportImage(Box::new(ImportState::new(
                    img.into_rgba8(),
                ))))
            }
            Err(e) => {
                log::warn!("imagen {}: {e}", path.display());
                self.error(self.i18n.tr("error-image"));
            }
        }
    }

    fn sync_mirror(&mut self) {
        self.editor.set_mirror(match self.mirror {
            MirrorMode::Off => Mirror::Off,
            MirrorMode::All => Mirror::All,
            MirrorMode::Selected => self.selected.map_or(Mirror::Off, Mirror::Part),
        });
    }

    fn handle_canvas_input(
        &mut self,
        resp: &egui::Response,
        at: Option<(u32, u32)>,
        clamped: Option<(u32, u32)>,
    ) {
        self.sync_mirror();
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
        if let Dialog::Convert(skin) = &dialog
            && self.convert_views.is_none()
        {
            self.convert_views = Some(self.model_views(ctx, skin));
        }
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
        let mut picker_error = None;
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
                // Un solo personaje; el selector cambia entre antes y después en el mismo lugar.
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.convert_after, false, i.tr("convert-before"));
                    ui.selectable_value(&mut self.convert_after, true, i.tr("convert-after"));
                });
                if let Some(views) = &self.convert_views {
                    ui.vertical_centered(|ui| {
                        ui.image((
                            views[usize::from(self.convert_after)].id(),
                            Vec2::splat(300.0),
                        ));
                    });
                }
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
            Dialog::EditCard(_, draft) => {
                ui.heading(i.tr("card-title"));
                egui::Grid::new("card").num_columns(2).show(ui, |ui| {
                    ui.label(i.tr("card-name"));
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.name)
                            .hint_text(i.tr("card-name-hint")),
                    );
                    ui.end_row();
                    ui.label(i.tr("card-tags"));
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.tags)
                            .hint_text(i.tr("card-tags-hint")),
                    );
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut draft.favorite, i.tr("card-favorite-check"));
                    ui.end_row();
                });
                ui.label(i.tr("card-background"));
                picker_error = library_view::background_picker(&mut self.lib, i, ui, draft);
                ui.add_space(6.0);
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("card-save")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::ImportImage(state) => {
                ui.heading(i.tr("import-title"));
                ui.label(i.tr("import-note"));
                state.ui(ui, i, self.editor.skin().model);
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("convert-apply")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::DeleteSkin(_, name) => {
                ui.label(library_view::delete_question(i, name));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("card-delete-confirm")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
        });
        if let Some(e) = picker_error {
            self.error(e);
        }
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
                self.apply_undo_limit();
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
            (Dialog::EditCard(path, draft), Choice::Accept) => self.save_card_draft(&path, &draft),
            (Dialog::DeleteSkin(path, _), Choice::Accept) => self.delete_skin(&path),
            (Dialog::ImportImage(st), Choice::Accept) => {
                let model = self.editor.skin().model;
                let canvas = fit(&st.src, st.crop, model, st.fit);
                self.editor.apply_layer(&to_front(&canvas, model, st.layer));
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // El idioma se cambia en caliente desde Ajustes.
        let lang = self.settings.resolved_language();
        if lang != self.i18n.lang() {
            self.i18n = I18n::new(lang);
        }
        style::set_motion(&ctx, self.settings.reduce_motion);
        self.handle_close(&ctx);
        self.handle_drops(&ctx);
        self.shortcuts(&ctx);
        self.autosave(&ctx);
        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::top("tabs").show(ui, |ui| self.tab_bar(ui, &ctx));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        if self.settings.view != View::Library {
            egui::Panel::left("tools")
                .default_size(260.0)
                .show(ui, |ui| self.side(ui));
        }
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

fn blank_project() -> Project {
    Project {
        skin: Skin::blank(Model::Steve),
        pose: Default::default(),
        card: Card::default(),
    }
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
        Action::CloseTab => i.tr("tab-close"),
        Action::NextTab => i.tr("tab-next"),
        Action::PrevTab => i.tr("tab-prev"),
    }
}

fn action_icon(a: Action) -> Option<Icon> {
    Some(match a {
        Action::New => Icon::New,
        Action::Open => Icon::Open,
        Action::Save | Action::SaveAs => Icon::Save,
        Action::Export | Action::QuickExport => Icon::Export,
        Action::Undo => Icon::Undo,
        Action::Redo => Icon::Redo,
        Action::CopySelection => Icon::Select,
        Action::EraseSelection | Action::Eraser => Icon::Eraser,
        Action::Deselect | Action::CloseTab => Icon::Close,
        Action::Pencil => Icon::Pencil,
        Action::Fill => Icon::Fill,
        Action::Picker => Icon::Picker,
        Action::Gradient => Icon::Gradient,
        Action::Stamp => Icon::Stamp,
        Action::Select => Icon::Select,
        Action::ToggleMirror => Icon::Mirror,
        Action::SwapColors => Icon::Swap,
        Action::ToggleView => Icon::Cube,
        Action::CopyParts | Action::PasteParts | Action::NextTab | Action::PrevTab => {
            return None;
        }
    })
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
