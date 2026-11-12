//! Pantalla de biblioteca: tarjetas con fondo, personaje en su pose y datos.

use super::{App, Dialog, blank_project, file_name, style};
use crate::i18n::I18n;
use crate::library::{self, backgrounds};
use crate::project::{self, Background, Card};
use crate::render3d::{Camera, Scene};
use crate::settings::{self, Settings, View};
use eframe::egui::{
    self, Color32, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
    UiBuilder, Vec2,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Lado de las miniaturas, en píxeles.
const THUMB: u32 = 192;
/// Capturas 3D por cuadro: más trabaría la interfaz al abrir una biblioteca grande.
const CAPTURES_PER_FRAME: usize = 2;

pub(super) struct LibraryState {
    entries: Vec<library::Entry>,
    /// Hay que volver a leer la carpeta.
    pub stale: bool,
    query: String,
    only_favorites: bool,
    thumbs: HashMap<u64, TextureHandle>,
    presets: Vec<TextureHandle>,
    /// Fondos propios por nombre de archivo; `None` si no se pudo leer.
    custom: HashMap<String, Option<TextureHandle>>,
    /// Vista del editor a la que se vuelve al abrir una skin.
    pub return_view: View,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            stale: true,
            query: String::new(),
            only_favorites: false,
            thumbs: HashMap::new(),
            presets: Vec::new(),
            custom: HashMap::new(),
            return_view: View::Flat,
        }
    }
}

/// Lo que se edita en el diálogo de tarjeta.
pub(super) struct CardDraft {
    pub name: String,
    /// Etiquetas separadas por comas.
    pub tags: String,
    pub favorite: bool,
    pub background: Background,
}

impl CardDraft {
    fn from(card: &Card) -> Self {
        Self {
            name: card.name.clone(),
            tags: card.tags.join(", "),
            favorite: card.favorite,
            background: card.background.clone(),
        }
    }

    fn apply(&self, card: &mut Card) {
        card.name = self.name.trim().to_string();
        card.tags = self
            .tags
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect();
        card.favorite = self.favorite;
        card.background = self.background.clone();
    }
}

enum CardAction {
    Open,
    Favorite,
    Edit,
    Duplicate,
    Delete,
}

pub(super) fn library_dir(s: &Settings) -> Option<PathBuf> {
    s.library_dir
        .clone()
        .or_else(|| settings::log::data_dir().map(|d| d.join("skins")))
}

fn backgrounds_dir() -> Option<PathBuf> {
    settings::log::data_dir().map(|d| d.join("fondos"))
}

fn thumbs_dir() -> Option<PathBuf> {
    settings::log::cache_dir().map(|d| d.join("miniaturas"))
}

/// Borra las miniaturas guardadas; se vuelven a generar al abrir la biblioteca.
pub(super) fn clear_thumb_cache() -> std::io::Result<()> {
    match thumbs_dir() {
        Some(d) if d.exists() => std::fs::remove_dir_all(d),
        _ => Ok(()),
    }
}

fn preset_name(i: &I18n, n: u8) -> String {
    match n % backgrounds::COUNT {
        0 => i.tr("bg-slate"),
        1 => i.tr("bg-sky"),
        2 => i.tr("bg-night"),
        3 => i.tr("bg-meadow"),
        4 => i.tr("bg-sunset"),
        _ => i.tr("bg-cave"),
    }
}

fn to_color_image(img: &image::RgbaImage) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    )
}

/// Recorte de la textura que cubre `rect` sin deformarse.
fn cover_uv(tex: &TextureHandle, rect: Rect) -> Rect {
    let [w, h] = tex.size().map(|v| v as f32);
    let (img, area) = (w / h, rect.width() / rect.height());
    if img > area {
        let k = area / img;
        Rect::from_min_max(
            egui::pos2((1.0 - k) / 2.0, 0.0),
            egui::pos2((1.0 + k) / 2.0, 1.0),
        )
    } else {
        let k = img / area;
        Rect::from_min_max(
            egui::pos2(0.0, (1.0 - k) / 2.0),
            egui::pos2(1.0, (1.0 + k) / 2.0),
        )
    }
}

impl LibraryState {
    pub fn clear_thumbs(&mut self) {
        self.thumbs.clear();
    }

    fn background(&mut self, ctx: &egui::Context, bg: &Background) -> Option<TextureHandle> {
        if self.presets.is_empty() {
            self.presets = (0..backgrounds::COUNT)
                .map(|n| {
                    let img = to_color_image(&backgrounds::preset(n));
                    ctx.load_texture(format!("fondo{n}"), img, TextureOptions::NEAREST)
                })
                .collect();
        }
        match bg {
            Background::Preset(n) => Some(self.presets[(n % backgrounds::COUNT) as usize].clone()),
            Background::Custom(file) => self
                .custom
                .entry(file.clone())
                .or_insert_with(|| {
                    let path = backgrounds_dir()?.join(file);
                    let img = image::open(&path)
                        .inspect_err(|e| log::warn!("fondo {}: {e}", path.display()))
                        .ok()?
                        .into_rgba8();
                    Some(ctx.load_texture(file, to_color_image(&img), TextureOptions::LINEAR))
                })
                .clone()
                // Si el fondo propio ya no existe, se usa el primero de la app.
                .or_else(|| self.presets.first().cloned()),
        }
    }
}

impl App {
    pub(super) fn library(&mut self, ui: &mut egui::Ui) {
        let Some(dir) = library_dir(&self.settings) else {
            ui.label(self.i18n.tr("library-no-dir"));
            return;
        };
        if self.lib.stale {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                log::error!("biblioteca {}: {e}", dir.display());
            }
            self.lib.entries = library::scan(&dir);
            self.lib.stale = false;
        }
        self.load_thumbs(ui.ctx());

        let i = &self.i18n;
        let mut new = false;
        let mut import = false;
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.lib.query)
                    .hint_text(i.tr("library-search"))
                    .desired_width(220.0),
            );
            ui.toggle_value(&mut self.lib.only_favorites, i.tr("library-favorites"));
            ui.separator();
            new = ui.button(i.tr("library-new")).clicked();
            import = ui.button(i.tr("library-import")).clicked();
            if ui.button(i.tr("library-refresh")).clicked() {
                self.lib.stale = true;
            }
        });
        ui.add(
            egui::Label::new(RichText::new(dir.display().to_string()).weak().small()).truncate(),
        );
        ui.separator();
        if new {
            self.new_tab(blank_project(), None);
            self.settings.view = self.lib.return_view;
            return;
        }
        if import {
            self.import_to_library(&dir);
        }

        let shown: Vec<usize> = (0..self.lib.entries.len())
            .filter(|&k| {
                let e = &self.lib.entries[k];
                (!self.lib.only_favorites || e.project.card.favorite) && e.matches(&self.lib.query)
            })
            .collect();
        if shown.is_empty() {
            let key = if self.lib.entries.is_empty() {
                "library-empty"
            } else {
                "library-no-match"
            };
            ui.add_space(24.0);
            ui.vertical_centered(|ui| ui.label(self.i18n.tr(key)));
            return;
        }
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(16.0);
            ui.horizontal_wrapped(|ui| {
                for k in shown {
                    if let Some(a) = self.card(ui, k) {
                        action = Some((k, a));
                    }
                }
            });
        });
        if let Some((k, a)) = action {
            self.card_action(k, a, &dir);
        }
    }

    fn card(&mut self, ui: &mut egui::Ui, k: usize) -> Option<CardAction> {
        let w = self.settings.card_size;
        let pic_h = (w * 1.25).round();
        let info_h = 44.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, pic_h + info_h), Sense::click());
        if !ui.is_rect_visible(rect.expand(8.0)) {
            return None;
        }
        let ctx = ui.ctx().clone();
        let e = &self.lib.entries[k];
        let (name, favorite, key) = (e.name(), e.project.card.favorite, e.key);
        let bg = e.project.card.background.clone();
        let sub = format!(
            "{:?} · {}",
            e.project.skin.model,
            e.modified.map(library::date).unwrap_or_default()
        );
        let v = ui.visuals().clone();
        let painter = ui.painter_at(rect.expand(8.0));

        // Sombra dura y marco de bloque.
        painter.rect_filled(
            rect.translate(Vec2::splat(5.0)),
            0.0,
            Color32::from_black_alpha(120),
        );
        let pic = Rect::from_min_size(rect.min, Vec2::new(w, pic_h));
        if let Some(tex) = self.lib.background(&ctx, &bg) {
            painter.image(tex.id(), pic, cover_uv(&tex, pic), Color32::WHITE);
        }
        if let Some(t) = self.lib.thumbs.get(&key) {
            let side = w.min(pic_h);
            let r = Rect::from_center_size(pic.center(), Vec2::splat(side));
            painter.image(
                t.id(),
                r,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let info = Rect::from_min_max(egui::pos2(rect.min.x, pic.max.y), rect.max);
        painter.rect_filled(info, 0.0, v.window_fill);
        painter.line_segment(
            [info.left_top(), info.right_top()],
            Stroke::new(2.0, Color32::from_black_alpha(160)),
        );
        let edge = if resp.hovered() {
            style::ACCENT
        } else {
            Color32::from_black_alpha(200)
        };
        painter.rect_stroke(rect, 0.0, Stroke::new(2.0, edge), StrokeKind::Outside);

        let mut action = None;
        ui.scope_builder(
            UiBuilder::new().max_rect(info.shrink2(Vec2::new(8.0, 5.0))),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 2.0);
                ui.add(
                    egui::Label::new(RichText::new(&name).font(style::pixel_font(15.0))).truncate(),
                );
                ui.add(egui::Label::new(RichText::new(&sub).weak().small()).truncate());
            },
        );
        let i = &self.i18n;
        let star = egui::Button::new(
            RichText::new(if favorite { "★" } else { "☆" })
                .size(18.0)
                .color(if favorite {
                    style::ACCENT
                } else {
                    Color32::WHITE
                }),
        )
        .frame(false);
        let star_rect = Rect::from_min_size(
            egui::pos2(pic.max.x - 28.0, pic.min.y + 4.0),
            Vec2::splat(24.0),
        );
        if ui
            .put(star_rect, star)
            .on_hover_text(i.tr("card-favorite"))
            .clicked()
        {
            action = Some(CardAction::Favorite);
        }
        let more_rect = Rect::from_min_size(pic.min + Vec2::splat(4.0), Vec2::new(28.0, 22.0));
        let more = ui
            .put(more_rect, egui::Button::new("⋯"))
            .on_hover_text(i.tr("card-more"));
        let menu = |ui: &mut egui::Ui, action: &mut Option<CardAction>| {
            for (a, label) in [
                (CardAction::Open, i.tr("card-open")),
                (CardAction::Edit, i.tr("card-edit")),
                (CardAction::Duplicate, i.tr("card-duplicate")),
                (CardAction::Delete, i.tr("card-delete")),
            ] {
                if ui.button(label).clicked() {
                    *action = Some(a);
                }
            }
        };
        egui::Popup::menu(&more).show(|ui| menu(ui, &mut action));
        resp.context_menu(|ui| menu(ui, &mut action));
        if resp.clicked() {
            action = Some(CardAction::Open);
        }
        resp.on_hover_text(&name);
        action
    }

    fn card_action(&mut self, k: usize, action: CardAction, dir: &Path) {
        let path = self.lib.entries[k].path.clone();
        match action {
            CardAction::Open => {
                self.open_path(path);
                self.settings.view = self.lib.return_view;
                self.settings_changed = true;
            }
            CardAction::Favorite => {
                self.change_card(&path, |c| c.favorite = !c.favorite);
            }
            CardAction::Edit => {
                let draft = CardDraft::from(&self.lib.entries[k].project.card);
                self.dialog = Some(Dialog::EditCard(path, draft));
            }
            CardAction::Duplicate => {
                let target = library::free_path(dir, &self.lib.entries[k].name());
                match std::fs::copy(&path, &target) {
                    Ok(_) => self.lib.stale = true,
                    Err(e) => self.error(
                        self.i18n
                            .tr_args("error-io", &[("detail", e.to_string().into())]),
                    ),
                }
            }
            CardAction::Delete => {
                let name = self.lib.entries[k].name();
                self.dialog = Some(Dialog::DeleteSkin(path, name));
            }
        }
    }

    /// Cambia la tarjeta en el archivo y en la pestaña que lo tenga abierto, para que
    /// guardar esa pestaña no la pise con la versión vieja.
    pub(super) fn change_card(&mut self, path: &Path, f: impl FnOnce(&mut Card)) {
        match library::update_card(path, f) {
            Ok(card) => {
                self.set_open_card(path, card);
                self.lib.stale = true;
            }
            Err(e) => self.error(self.i18n.tr_args("error-io", &[("detail", e.into())])),
        }
    }

    pub(super) fn save_card_draft(&mut self, path: &Path, draft: &CardDraft) {
        self.change_card(path, |c| draft.apply(c));
    }

    pub(super) fn delete_skin(&mut self, path: &Path) {
        match std::fs::remove_file(path) {
            Ok(()) => {
                log::info!("eliminado {}", path.display());
                self.lib.stale = true;
            }
            Err(e) => self.error(
                self.i18n
                    .tr_args("error-io", &[("detail", e.to_string().into())]),
            ),
        }
    }

    fn import_to_library(&mut self, dir: &Path) {
        let Some(files) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .pick_files()
        else {
            return;
        };
        let mut count = 0;
        for src in files {
            let ok = std::fs::read(&src)
                .map_err(|e| e.to_string())
                .and_then(|b| project::load(&b).map(|_| ()).map_err(|e| format!("{e:?}")))
                .and_then(|()| {
                    let name = src
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    std::fs::copy(&src, library::free_path(dir, &name)).map_err(|e| e.to_string())
                });
            match ok {
                Ok(_) => count += 1,
                Err(e) => log::warn!("no se importó {}: {e}", src.display()),
            }
        }
        self.status = self
            .i18n
            .tr_args("status-library-imported", &[("count", count.into())]);
        self.lib.stale = true;
    }

    /// Carga miniaturas de la caché en disco o las dibuja con la vista 3D (unas pocas por cuadro).
    fn load_thumbs(&mut self, ctx: &egui::Context) {
        let cache = thumbs_dir();
        let mut captures = 0;
        let mut pending = false;
        for e in &self.lib.entries {
            if self.lib.thumbs.contains_key(&e.key) {
                continue;
            }
            let file = cache
                .as_ref()
                .map(|d| d.join(format!("{:016x}.png", e.key)));
            let cached = file
                .as_ref()
                .and_then(|f| image::open(f).ok())
                .map(|i| i.into_rgba8());
            let img = match cached {
                Some(img) => img,
                None if captures < CAPTURES_PER_FRAME => {
                    let Some(r) = &self.renderer else { break };
                    captures += 1;
                    r.upload_skin(&e.project.skin.pixels);
                    let camera = Camera::thumbnail();
                    let scene = Scene {
                        model: e.project.skin.model,
                        pose: &e.project.pose,
                        camera: &camera,
                        selected: None,
                        visible: &|_, _| true,
                        cape: false,
                        lighting: self.settings.lighting,
                    };
                    let Some(img) = r.capture(THUMB, &scene) else {
                        continue;
                    };
                    if let Some(f) = &file {
                        let saved = f
                            .parent()
                            .map_or(Ok(()), std::fs::create_dir_all)
                            .map_err(|e| e.to_string())
                            .and_then(|()| img.save(f).map_err(|e| e.to_string()));
                        if let Err(err) = saved {
                            log::warn!("miniatura {}: {err}", f.display());
                        }
                    }
                    img
                }
                None => {
                    pending = true;
                    continue;
                }
            };
            let tex = ctx.load_texture(
                format!("mini{:x}", e.key),
                to_color_image(&img),
                TextureOptions::LINEAR,
            );
            self.lib.thumbs.insert(e.key, tex);
        }
        if captures > 0 {
            // La textura de la skin en la GPU cambió: la vista 3D tiene que volver a subirla.
            self.uploaded = None;
        }
        if pending {
            ctx.request_repaint();
        }
    }
}

/// Selector de fondo del diálogo de tarjeta: los de la app, los propios y «Agregar».
/// Devuelve un mensaje si falló agregar un fondo.
pub(super) fn background_picker(
    lib: &mut LibraryState,
    i: &I18n,
    ui: &mut egui::Ui,
    draft: &mut CardDraft,
) -> Option<String> {
    let ctx = ui.ctx().clone();
    let mut options: Vec<Background> = (0..backgrounds::COUNT).map(Background::Preset).collect();
    if let Some(dir) = backgrounds_dir()
        && let Ok(read) = std::fs::read_dir(dir)
    {
        let mut files: Vec<String> = read
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        files.sort();
        options.extend(files.into_iter().map(Background::Custom));
    }
    let mut error = None;
    ui.horizontal_wrapped(|ui| {
        for bg in options {
            let Some(tex) = lib.background(&ctx, &bg) else {
                continue;
            };
            let (r, resp) = ui.allocate_exact_size(Vec2::new(40.0, 50.0), Sense::click());
            ui.painter()
                .image(tex.id(), r, cover_uv(&tex, r), Color32::WHITE);
            let stroke = if draft.background == bg {
                Stroke::new(3.0, style::ACCENT)
            } else {
                Stroke::new(1.0, Color32::from_black_alpha(200))
            };
            ui.painter()
                .rect_stroke(r, 0.0, stroke, StrokeKind::Outside);
            let tip = match &bg {
                Background::Preset(n) => preset_name(i, *n),
                Background::Custom(f) => f.clone(),
            };
            if resp.on_hover_text(tip).clicked() {
                draft.background = bg;
            }
        }
        if ui.button(i.tr("card-add-background")).clicked() {
            match add_background(i) {
                Ok(Some(bg)) => draft.background = bg,
                Ok(None) => {}
                Err(e) => error = Some(e),
            }
        }
    });
    error
}

/// Copia una imagen elegida por el usuario a la carpeta de fondos.
fn add_background(i: &I18n) -> Result<Option<Background>, String> {
    let Some(src) = rfd::FileDialog::new()
        .add_filter("PNG", &["png"])
        .pick_file()
    else {
        return Ok(None);
    };
    if image::open(&src).is_err() {
        return Err(i.tr("error-not-png"));
    }
    let Some(dir) = backgrounds_dir() else {
        return Ok(None);
    };
    let name = src
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    std::fs::create_dir_all(&dir)
        .and_then(|()| {
            let target = library::free_path(&dir, &name);
            std::fs::copy(&src, &target).map(|_| target)
        })
        .map(|target| Some(Background::Custom(file_name(&target))))
        .map_err(|e| i.tr_args("error-io", &[("detail", e.to_string().into())]))
}

/// Muestra el nombre de la tarjeta en el diálogo de borrar.
pub(super) fn delete_question(i: &I18n, name: &str) -> String {
    i.tr_args("card-delete-question", &[("name", name.into())])
}
