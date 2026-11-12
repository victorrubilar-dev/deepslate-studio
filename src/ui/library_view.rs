//! Pantalla de biblioteca: tarjetas con fondo, personaje en su pose y datos.

use super::icons::{self, Icon};
use super::{App, Dialog, blank_project, file_name, png_dialog, style};
use crate::core::Model;
use crate::editor::templates::{self, Template};
use crate::i18n::I18n;
use crate::library::{self, backgrounds};
use crate::project::{self, Background, Card};
use crate::render3d::{Camera, Scene};
use crate::settings::{self, Settings, View};
use eframe::egui::{
    self, Color32, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
    UiBuilder, Vec2,
};
use image::Rgba;
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
    /// Cuándo se entró a la biblioteca (segundos de egui): las tarjetas aparecen desde ahí.
    pub shown_at: f64,
    /// Colección que se muestra; `None` muestra todas.
    collection: Option<String>,
    collections: Vec<String>,
    sort: library::Sort,
    new_collection: String,
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
            return_view: View::ThreeD,
            shown_at: 0.0,
            collection: None,
            collections: Vec::new(),
            sort: library::Sort::Favorites,
            new_collection: String::new(),
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

/// Formas de empezar una skin, en la bienvenida y en el menú «Nueva skin».
#[derive(Clone, Copy)]
pub(super) enum Start {
    Empty(Model),
    Base,
    Open,
    Import,
}

fn start_options(i: &I18n) -> [(Start, String, String); 5] {
    [
        (
            Start::Empty(Model::Steve),
            i.tr("start-empty-steve"),
            i.tr("start-steve-desc"),
        ),
        (
            Start::Empty(Model::Alex),
            i.tr("start-empty-alex"),
            i.tr("start-alex-desc"),
        ),
        (Start::Base, i.tr("start-base"), i.tr("start-base-desc")),
        (Start::Open, i.tr("start-open"), i.tr("start-open-desc")),
        (
            Start::Import,
            i.tr("start-import"),
            i.tr("start-import-desc"),
        ),
    ]
}

/// Botón grande de la bienvenida, con el mismo aspecto de bloque que las tarjetas.
fn tile(ui: &mut egui::Ui, title: &str, desc: &str, appear: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(250.0, 76.0), Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        resp.id,
        resp.hovered() || resp.has_focus(),
        ui.style().animation_time,
    );
    let mut p = ui.painter().clone();
    p.multiply_opacity(appear);
    let v = ui.visuals();
    p.rect_filled(
        rect.translate(Vec2::splat(5.0)),
        0.0,
        Color32::from_black_alpha(120),
    );
    // Se levanta sobre su sombra al pasar el cursor y entra desde abajo al aparecer.
    let rect = rect.translate(Vec2::new(-2.0 * hover, 10.0 * (1.0 - appear) - 2.0 * hover));
    p.rect_filled(
        rect,
        0.0,
        v.widgets
            .inactive
            .bg_fill
            .lerp_to_gamma(v.widgets.hovered.bg_fill, hover),
    );
    let edge = Color32::from_black_alpha(200).lerp_to_gamma(style::ACCENT, hover);
    p.rect_stroke(rect, 0.0, Stroke::new(2.0, edge), StrokeKind::Outside);
    let inner = rect.shrink2(Vec2::new(14.0, 12.0));
    let text = v.strong_text_color();
    p.text(
        inner.left_top(),
        egui::Align2::LEFT_TOP,
        title,
        style::pixel_font(style::SIZE_TITLE),
        text,
    );
    let desc = p.layout(
        desc.to_string(),
        egui::FontId::proportional(style::SIZE_BODY),
        v.weak_text_color(),
        inner.width(),
    );
    p.galley(inner.left_top() + Vec2::new(0.0, 26.0), desc, text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

enum CardAction {
    Open,
    Favorite,
    Edit,
    Duplicate,
    Delete,
    /// Mover a una colección, o a la raíz con `None`.
    Move(Option<String>),
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

fn sort_name(i: &I18n, s: library::Sort) -> String {
    match s {
        library::Sort::Favorites => i.tr("sort-favorites"),
        library::Sort::Recent => i.tr("sort-recent"),
        library::Sort::Name => i.tr("sort-name"),
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
    /// Carpeta donde entran las skins nuevas: la colección abierta o la raíz.
    pub fn current_dir(&self, root: &Path) -> PathBuf {
        self.collection
            .as_ref()
            .map_or_else(|| root.to_path_buf(), |c| root.join(c))
    }

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
            library::sort(&mut self.lib.entries, self.lib.sort);
            self.lib.collections = library::collections(&dir);
            if let Some(c) = &self.lib.collection
                && !self.lib.collections.contains(c)
            {
                self.lib.collection = None;
            }
            self.lib.stale = false;
        }
        self.load_thumbs(ui.ctx());

        let i = &self.i18n;
        let mut start = None;
        ui.horizontal_wrapped(|ui| {
            ui.add(icons::image(ui.ctx(), Icon::Search).tint(ui.visuals().weak_text_color()));
            ui.add(
                egui::TextEdit::singleline(&mut self.lib.query)
                    .hint_text(i.tr("library-search"))
                    .desired_width(220.0),
            );
            let star = if self.lib.only_favorites {
                Icon::StarOn
            } else {
                Icon::Star
            };
            ui.toggle_value(
                &mut self.lib.only_favorites,
                (icons::image(ui.ctx(), star), i.tr("library-favorites")),
            );
            let sort_before = self.lib.sort;
            egui::ComboBox::from_id_salt("library-sort")
                .selected_text(sort_name(i, self.lib.sort))
                .show_ui(ui, |ui| {
                    for s in [
                        library::Sort::Favorites,
                        library::Sort::Recent,
                        library::Sort::Name,
                    ] {
                        ui.selectable_value(&mut self.lib.sort, s, sort_name(i, s));
                    }
                })
                .response
                .on_hover_text(i.tr("library-sort"));
            if self.lib.sort != sort_before {
                library::sort(&mut self.lib.entries, self.lib.sort);
            }
            ui.separator();
            let new = egui::Button::image_and_text(
                icons::image(ui.ctx(), Icon::Plus),
                i.tr("library-new"),
            )
            .image_tint_follows_text_color(true);
            egui::containers::menu::MenuButton::from_button(new).ui(ui, |ui| {
                for (s, title, desc) in start_options(i) {
                    if ui.button(title).on_hover_text(desc).clicked() {
                        start = Some(s);
                    }
                }
            });
            if ui
                .add(style::menu_button(ui, Icon::Reset, i.tr("library-refresh")))
                .clicked()
            {
                self.lib.stale = true;
            }
        });
        ui.add(
            egui::Label::new(RichText::new(dir.display().to_string()).weak().small()).truncate(),
        );
        let mut created = None;
        ui.horizontal_wrapped(|ui| {
            let mut pick = |ui: &mut egui::Ui, c: Option<String>, label: &str| {
                let on = self.lib.collection == c;
                let resp = style::tab(ui, None, label, on);
                if on {
                    style::slide_marker(ui, egui::Id::unique("collection-marker"), resp.rect);
                }
                if resp.clicked() {
                    self.lib.collection = c;
                }
            };
            pick(ui, None, &i.tr("collection-all"));
            for c in self.lib.collections.clone() {
                pick(ui, Some(c.clone()), &c);
            }
            let new = style::menu_button(ui, Icon::Plus, i.tr("collection-new"))
                .frame_when_inactive(false);
            egui::containers::menu::MenuButton::from_button(new).ui(ui, |ui| {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut self.lib.new_collection)
                        .hint_text(i.tr("collection-name-hint")),
                );
                let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button(i.tr("collection-create")).clicked() || enter {
                    created = Some(std::mem::take(&mut self.lib.new_collection));
                    ui.close();
                }
            });
        });
        if let Some(name) = created {
            match library::collection_name(&name) {
                Some(name) => match std::fs::create_dir_all(dir.join(&name)) {
                    Ok(()) => {
                        self.lib.collection = Some(name);
                        self.lib.stale = true;
                    }
                    Err(e) => self.error(
                        self.i18n
                            .tr_args("error-io", &[("detail", e.to_string().into())]),
                    ),
                },
                None => self.error(self.i18n.tr("error-collection-name")),
            }
        }
        ui.separator();
        if self.lib.entries.is_empty() {
            start = start.or(self.welcome(ui));
        }
        if let Some(s) = start {
            let target = self.lib.current_dir(&dir);
            return self.start(s, &target);
        }

        let shown: Vec<usize> = (0..self.lib.entries.len())
            .filter(|&k| {
                let e = &self.lib.entries[k];
                (!self.lib.only_favorites || e.project.card.favorite)
                    && (self.lib.collection.is_none() || e.collection == self.lib.collection)
                    && e.matches(&self.lib.query)
            })
            .collect();
        if shown.is_empty() {
            if !self.lib.entries.is_empty() {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| ui.label(self.i18n.tr("library-no-match")));
            }
            return;
        }
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(16.0);
            ui.horizontal_wrapped(|ui| {
                for (n, k) in shown.into_iter().enumerate() {
                    // Escalonadas, pero sin que una biblioteca grande tarde en aparecer.
                    let delay = (n.min(12) as f64) * 0.035;
                    let appear = style::appear(ui, self.lib.shown_at, delay, 0.25);
                    if let Some(a) = self.card(ui, k, appear) {
                        action = Some((k, a));
                    }
                }
            });
        });
        if let Some((k, a)) = action {
            self.card_action(k, a, &dir);
        }
    }

    /// Bienvenida de la biblioteca vacía: opciones grandes para empezar.
    fn welcome(&self, ui: &mut egui::Ui) -> Option<Start> {
        let i = &self.i18n;
        let mut start = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    RichText::new(i.tr("home-title")).font(style::pixel_font(style::SIZE_HERO)),
                );
                ui.add_space(6.0);
                ui.label(i.tr("home-subtitle"));
                ui.add_space(24.0);
            });
            // Rejilla centrada: tantas columnas como entren.
            let cols = ((ui.available_width() + 20.0) / 270.0)
                .floor()
                .clamp(1.0, 3.0) as usize;
            let width = cols as f32 * 270.0 - 20.0;
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
                egui::Grid::new("welcome")
                    .spacing(Vec2::splat(20.0))
                    .show(ui, |ui| {
                        for (k, (s, title, desc)) in start_options(i).into_iter().enumerate() {
                            let appear = style::appear(ui, self.lib.shown_at, k as f64 * 0.05, 0.3);
                            if tile(ui, &title, &desc, appear).clicked() {
                                start = Some(s);
                            }
                            if (k + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                ui.weak(i.tr("home-drop-hint"));
            });
        });
        start
    }

    fn start(&mut self, s: Start, dir: &Path) {
        let to_editor = match s {
            Start::Empty(model) => {
                let mut p = blank_project();
                p.skin = crate::core::Skin::blank(model);
                self.new_tab(p, None);
                true
            }
            Start::Base => {
                let mut p = blank_project();
                p.skin.pixels = templates::draw(Template::Base, Model::Steve, Rgba([0; 4]));
                self.new_tab(p, None);
                true
            }
            Start::Open => png_dialog().pick_file().is_some_and(|p| self.open_path(p)),
            Start::Import => {
                if let Some(files) = png_dialog().pick_files() {
                    self.import_files(dir, files);
                }
                false
            }
        };
        if to_editor {
            self.settings.view = self.lib.return_view;
            self.settings_changed = true;
        }
    }

    fn card(&mut self, ui: &mut egui::Ui, k: usize, appear: f32) -> Option<CardAction> {
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
        let hover = ctx.animate_bool_with_time(
            resp.id,
            resp.hovered() || resp.has_focus(),
            ui.style().animation_time,
        );
        let mut painter = ui.painter_at(rect.expand(16.0));
        painter.multiply_opacity(appear);
        let old_opacity = ui.opacity();
        ui.multiply_opacity(appear);

        // Sombra dura y marco de bloque. La tarjeta se levanta sobre la sombra con el cursor
        // y entra desde abajo al aparecer; la zona que recibe el clic no se mueve.
        painter.rect_filled(
            rect.translate(Vec2::splat(5.0)),
            0.0,
            Color32::from_black_alpha(120),
        );
        let rect = rect.translate(Vec2::new(-3.0 * hover, 14.0 * (1.0 - appear) - 3.0 * hover));
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
        let edge = Color32::from_black_alpha(200).lerp_to_gamma(style::ACCENT, hover);
        painter.rect_stroke(rect, 0.0, Stroke::new(2.0, edge), StrokeKind::Outside);

        let mut action = None;
        ui.scope_builder(
            UiBuilder::new().max_rect(info.shrink2(Vec2::new(8.0, 5.0))),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 2.0);
                ui.add(
                    egui::Label::new(RichText::new(&name).font(style::pixel_font(style::SIZE_UI)))
                        .truncate(),
                );
                ui.add(egui::Label::new(RichText::new(&sub).weak().small()).truncate());
            },
        );
        let i = &self.i18n;
        let star = egui::Button::image(
            icons::image(&ctx, if favorite { Icon::StarOn } else { Icon::Star }).tint(
                if favorite {
                    style::ACCENT
                } else {
                    Color32::WHITE
                },
            ),
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
            .put(
                more_rect,
                egui::Button::image(icons::image(&ctx, Icon::More))
                    .image_tint_follows_text_color(true),
            )
            .on_hover_text(i.tr("card-more"));
        let collections = &self.lib.collections;
        let current = &self.lib.entries[k].collection;
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
            if !collections.is_empty() {
                ui.menu_button(i.tr("card-move"), |ui| {
                    let targets = std::iter::once((None, i.tr("collection-none")))
                        .chain(collections.iter().map(|c| (Some(c.clone()), c.clone())));
                    for (c, label) in targets {
                        if ui
                            .add_enabled(c != *current, egui::Button::new(label))
                            .clicked()
                        {
                            *action = Some(CardAction::Move(c));
                        }
                    }
                });
            }
        };
        egui::Popup::menu(&more).show(|ui| menu(ui, &mut action));
        resp.context_menu(|ui| menu(ui, &mut action));
        if resp.clicked() {
            action = Some(CardAction::Open);
        }
        ui.set_opacity(old_opacity);
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
                // La copia queda en la misma colección que el original.
                let folder = path.parent().unwrap_or(dir);
                let target = library::free_path(folder, &self.lib.entries[k].name());
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
            CardAction::Move(c) => match library::move_to(&path, dir, c.as_deref()) {
                Ok(new) => {
                    self.retarget_tab(&path, new);
                    self.lib.stale = true;
                }
                Err(e) => self.error(
                    self.i18n
                        .tr_args("error-io", &[("detail", e.to_string().into())]),
                ),
            },
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

    /// Copia a la biblioteca las skins válidas de `files`; las demás se saltan.
    /// Devuelve cuántas entraron.
    pub(super) fn import_files(&mut self, dir: &Path, files: Vec<PathBuf>) -> usize {
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
        count
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
