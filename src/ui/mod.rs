//! Ventana principal: menús, panel de herramientas, lienzo 2D y diálogos.

use crate::core::{FACES, LAYERS, Layer, LoadError, Model, PARTS, Part, Skin, convert_model, rect};
use crate::editor::{ERASE, Editor, Tool};
use crate::i18n::I18n;
use crate::project::{self, Project, ProjectError};
use crate::render3d::{Camera, PRESETS, Preset, Renderer3d, Scene, pick};
use crate::settings::{self, Settings, Theme, View};
use eframe::egui::{
    self, Color32, ColorImage, Key, KeyboardShortcut, Modifiers, Pos2, Rect, Sense, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Vec2, color_picker, ecolor::Hsva,
};
use image::Rgba;
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
    Export(usize),
    Error(String),
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
    uploaded_rev: Option<u64>,
    camera: Camera,
    selected: Option<Part>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings_path = Settings::path();
        let settings = settings_path
            .as_deref()
            .map(Settings::load_from)
            .unwrap_or_default();
        cc.egui_ctx.set_theme(theme_pref(settings.theme));
        Self {
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
            uploaded_rev: None,
            camera: Camera::default(),
            selected: None,
        }
    }

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.editor = Editor::new(project.skin, self.settings.undo_levels);
        self.editor.set_pose(project.pose);
        self.editor.mark_saved();
        self.path = path;
        self.texture_rev = None;
        self.uploaded_rev = None;
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
        let Some(path) = png_dialog().pick_file() else {
            return;
        };
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

    fn export(&mut self) {
        let Some(path) = png_dialog().set_file_name("skin.png").save_file() else {
            return;
        };
        let path = with_png_ext(path);
        match std::fs::write(&path, self.editor.skin().to_png()) {
            Ok(()) => {
                self.status = self
                    .i18n
                    .tr_args("status-exported", &[("file", file_name(&path).into())])
            }
            Err(e) => self.error(
                self.i18n
                    .tr_args("error-io", &[("detail", e.to_string().into())]),
            ),
        }
    }

    fn request_export(&mut self) {
        match self.editor.skin().translucent_base_pixels() {
            0 => self.export(),
            n => self.dialog = Some(Dialog::Export(n)),
        }
    }

    fn error(&mut self, msg: String) {
        self.status = self
            .i18n
            .tr_args("status-error", &[("detail", msg.clone().into())]);
        self.dialog = Some(Dialog::Error(msg));
    }

    // ---- Entrada global ----

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.painting || self.dialog.is_some() {
            return;
        }
        let key = |m, k| ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)));
        // Ctrl+Mayús+Z primero: Ctrl+Z también coincidiría con él.
        if key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) || key(Modifiers::COMMAND, Key::Y) {
            self.editor.redo();
        } else if key(Modifiers::COMMAND, Key::Z) {
            self.editor.undo();
        } else if key(Modifiers::COMMAND, Key::S) {
            self.save(false);
        } else if key(Modifiers::COMMAND, Key::O) {
            self.request(ctx, After::Open);
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
        let ctx = ui.ctx().clone();
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(self.i18n.tr("menu-file"), |ui| {
                if ui.button(self.i18n.tr("file-new")).clicked() {
                    self.request(&ctx, After::New);
                }
                if ui.button(self.i18n.tr("file-open")).clicked() {
                    self.request(&ctx, After::Open);
                }
                ui.separator();
                if ui.button(self.i18n.tr("file-save")).clicked() {
                    self.save(false);
                }
                if ui.button(self.i18n.tr("file-save-as")).clicked() {
                    self.save(true);
                }
                if ui.button(self.i18n.tr("file-export")).clicked() {
                    self.request_export();
                }
            });
            ui.menu_button(self.i18n.tr("menu-edit"), |ui| {
                if ui
                    .add_enabled(
                        self.editor.can_undo(),
                        egui::Button::new(self.i18n.tr("edit-undo")),
                    )
                    .clicked()
                {
                    self.editor.undo();
                }
                if ui
                    .add_enabled(
                        self.editor.can_redo(),
                        egui::Button::new(self.i18n.tr("edit-redo")),
                    )
                    .clicked()
                {
                    self.editor.redo();
                }
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
        let i = &self.i18n;
        ui.heading(i.tr("tools"));
        for (tool, name) in [
            (Tool::Pencil, i.tr("tool-pencil")),
            (Tool::Eraser, i.tr("tool-eraser")),
            (Tool::Fill, i.tr("tool-fill")),
            (Tool::Picker, i.tr("tool-picker")),
        ] {
            ui.selectable_value(&mut self.tool, tool, name);
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(i.tr("color"));
            color_picker::color_edit_button_hsva(
                ui,
                &mut self.color,
                color_picker::Alpha::OnlyBlend,
            );
        });
        ui.separator();

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
        let names = [
            (Part::Head, i.tr("part-head")),
            (Part::Body, i.tr("part-body")),
            (Part::RightArm, i.tr("part-right-arm")),
            (Part::LeftArm, i.tr("part-left-arm")),
            (Part::RightLeg, i.tr("part-right-leg")),
            (Part::LeftLeg, i.tr("part-left-leg")),
        ];
        let layer_names = [i.tr("layer-base"), i.tr("layer-overlay")];
        egui::Grid::new("visibility").show(ui, |ui| {
            ui.label("");
            for n in &layer_names {
                ui.label(n);
            }
            ui.end_row();
            for (part, name) in names {
                if self.selected == Some(part) {
                    ui.label(egui::RichText::new(&name).strong().color(ACCENT));
                } else {
                    ui.label(&name);
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
        ui.horizontal(|ui| {
            let i = &self.i18n;
            ui.selectable_value(&mut self.settings.view, View::Flat, i.tr("view-2d"));
            ui.selectable_value(&mut self.settings.view, View::ThreeD, i.tr("view-3d"));
            if self.settings.view == View::ThreeD {
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
        if self.settings.view != before {
            self.settings_changed = true;
        }
        ui.separator();
        match self.settings.view {
            View::Flat => self.canvas(ui),
            View::ThreeD => self.view3d(ui),
        }
    }

    fn view3d(&mut self, ui: &mut egui::Ui) {
        let Some(renderer) = &mut self.renderer else {
            ui.label(self.i18n.tr("view3d-unavailable"));
            return;
        };
        let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        if resp.dragged() {
            let d = resp.drag_delta();
            self.camera.yaw += d.x * 0.5;
            self.camera.pitch = (self.camera.pitch + d.y * 0.5).clamp(-89.0, 89.0);
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.camera.distance = (self.camera.distance - scroll * 0.1).clamp(30.0, 160.0);
        }
        let editor = &self.editor;
        let visible = |p, l| editor.is_visible(p, l);
        let aspect = area.width() / area.height().max(1.0);
        if resp.clicked()
            && let Some(pos) = resp.interact_pointer_pos()
        {
            let nx = (pos.x - area.min.x) / area.width() * 2.0 - 1.0;
            let ny = 1.0 - (pos.y - area.min.y) / area.height() * 2.0;
            let model = editor.skin().model;
            self.selected = pick(
                model,
                editor.pose(),
                &self.camera,
                &visible,
                aspect,
                (nx, ny),
            );
        }
        if self.uploaded_rev != Some(editor.revision()) {
            renderer.upload_skin(&editor.skin().pixels);
            self.uploaded_rev = Some(editor.revision());
        }
        let scene = Scene {
            model: editor.skin().model,
            pose: editor.pose(),
            camera: &self.camera,
            selected: self.selected,
            visible: &visible,
        };
        let ppp = ui.ctx().pixels_per_point();
        let size = [area.width(), area.height()].map(|v| (v * ppp).round() as u32);
        let tex = renderer.render(size, &scene);
        ui.painter_at(area).image(
            tex,
            area,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        resp.on_hover_text(self.i18n.tr("view3d-hint"));
    }

    fn capture(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        let Some(path) = png_dialog().set_file_name("captura.png").save_file() else {
            return;
        };
        let path = with_png_ext(path);
        let editor = &self.editor;
        if self.uploaded_rev != Some(editor.revision()) {
            renderer.upload_skin(&editor.skin().pixels);
            self.uploaded_rev = Some(editor.revision());
        }
        let visible = |p, l| editor.is_visible(p, l);
        let scene = Scene {
            model: editor.skin().model,
            pose: editor.pose(),
            camera: &self.camera,
            selected: None,
            visible: &visible,
        };
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

            let hover = resp
                .interact_pointer_pos()
                .or(resp.hover_pos())
                .and_then(|p| {
                    let t = (p - area.min) / px;
                    (t.x >= 0.0 && t.y >= 0.0 && t.x < 64.0 && t.y < 64.0)
                        .then_some((t.x as u32, t.y as u32))
                });
            self.handle_canvas_input(&resp, hover);
        });
    }

    fn handle_canvas_input(&mut self, resp: &egui::Response, at: Option<(u32, u32)>) {
        match self.tool {
            Tool::Pencil | Tool::Eraser => {
                let down = resp.is_pointer_button_down_on()
                    && resp.ctx.input(|i| i.pointer.primary_down());
                if down && !self.painting {
                    self.editor.begin_stroke();
                    self.painting = true;
                    self.last_px = None;
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
                    Rgba(self.color.to_srgba_unmultiplied())
                };
                if let Some(p) = at {
                    self.editor.paint_line(self.last_px.unwrap_or(p), p, color);
                }
                self.last_px = at;
            }
            Tool::Fill => {
                if let (true, Some((x, y))) = (resp.clicked(), at) {
                    self.editor
                        .fill(x, y, Rgba(self.color.to_srgba_unmultiplied()));
                }
            }
            Tool::Picker => {
                if let (true, Some((x, y))) = (resp.clicked(), at) {
                    self.color = Hsva::from_srgba_unmultiplied(self.editor.pick(x, y).0);
                }
            }
        }
    }

    // ---- Ventanas ----

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        let before = self.settings.clone();
        let i = &self.i18n;
        let themes = [
            (Theme::System, i.tr("theme-system")),
            (Theme::Dark, i.tr("theme-dark")),
            (Theme::Light, i.tr("theme-light")),
        ];
        egui::Window::new(i.tr("settings-title"))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                let s = &mut self.settings;
                egui::Grid::new("settings").num_columns(2).show(ui, |ui| {
                    ui.label(i.tr("settings-theme"));
                    let current = themes
                        .iter()
                        .find(|(t, _)| *t == s.theme)
                        .map(|(_, n)| n.clone())
                        .unwrap_or_default();
                    egui::ComboBox::from_id_salt("theme")
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (t, n) in &themes {
                                ui.selectable_value(&mut s.theme, *t, n);
                            }
                        });
                    ui.end_row();
                    ui.label(i.tr("settings-pixel-size"));
                    ui.add(egui::Slider::new(&mut s.pixel_size, settings::PIXEL_SIZE));
                    ui.end_row();
                    ui.label(i.tr("settings-grid"));
                    ui.checkbox(&mut s.show_grid, "");
                    ui.end_row();
                    ui.label(i.tr("settings-undo"));
                    ui.add(egui::DragValue::new(&mut s.undo_levels).range(settings::UNDO_LEVELS));
                    ui.end_row();
                });
            });
        self.show_settings = open;

        if self.settings != before {
            ctx.set_theme(theme_pref(self.settings.theme));
            self.editor.set_limit(self.settings.undo_levels);
            self.settings_changed = true;
        }
        // Se escribe al soltar el ratón, no en cada cuadro mientras se arrastra un control.
        if self.settings_changed && !ctx.input(|i| i.pointer.any_down()) {
            self.settings_changed = false;
            if let Some(path) = &self.settings_path
                && let Err(e) = self.settings.save_to(path)
            {
                self.status = self
                    .i18n
                    .tr_args("error-io", &[("detail", e.to_string().into())]);
            }
        }
    }

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
        let resp = egui::Modal::new(id).show(ctx, |ui| match &dialog {
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
            Dialog::Export(n) => {
                ui.label(i.tr_args("export-translucent", &[("count", (*n).into())]));
                buttons(
                    ui,
                    &[
                        (Choice::Accept, i.tr("export-anyway")),
                        (Choice::Cancel, i.tr("cancel")),
                    ],
                )
            }
            Dialog::Error(msg) => {
                ui.label(msg);
                buttons(ui, &[(Choice::Cancel, i.tr("ok"))])
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
            (Dialog::Export(_), Choice::Accept) => self.export(),
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_close(&ctx);
        self.shortcuts(&ctx);
        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("tools")
            .default_size(260.0)
            .show(ui, |ui| self.side(ui));
        egui::CentralPanel::default_margins().show(ui, |ui| self.central(ui));
        self.settings_window(&ctx);
        self.dialogs(&ctx);
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
