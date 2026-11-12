//! Estilo visual propio: esquinas rectas y sombras duras (aspecto de bloque), tonos pizarra
//! y el menta del ícono como acento. Los títulos usan una fuente pixel.

use super::icons::{self, Icon};
use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Rect, Response,
    RichText, Sense, Shadow, Stroke, StrokeKind, TextStyle, Theme, Vec2,
};
use std::sync::Arc;

/// Menta del ícono. `render3d/skin.wgsl` usa el mismo para la parte seleccionada.
pub const ACCENT: Color32 = Color32::from_rgb(64, 214, 180);
/// Versión más oscura para el tema claro, legible con texto blanco encima.
const ACCENT_DARK: Color32 = Color32::from_rgb(26, 125, 103);
/// Texto sobre el acento en el tema oscuro.
const ON_ACCENT: Color32 = Color32::from_rgb(15, 26, 23);
const PIXEL: &str = "pixel";

/// Escala de tamaños de texto. Todo texto destacado usa uno de estos.
pub const SIZE_SMALL: f32 = 11.0;
pub const SIZE_BODY: f32 = 13.0;
/// Botones, pestañas y menús, en fuente pixel.
pub const SIZE_UI: f32 = 15.0;
pub const SIZE_TITLE: f32 = 18.0;
pub const SIZE_HERO: f32 = 32.0;

/// Duración de las transiciones, en segundos. Con «Reducir animaciones» es 0.
const MOTION: f32 = 0.18;

pub fn apply(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        PIXEL.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/PixelifySans.ttf"
        ))),
    );
    // Si a la fuente pixel le falta un carácter, se usa la normal.
    let mut family = vec![PIXEL.to_string()];
    family.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts
        .families
        .insert(FontFamily::Name(PIXEL.into()), family);
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|s| {
        s.text_styles
            .insert(TextStyle::Heading, pixel_font(SIZE_TITLE));
        s.text_styles.insert(TextStyle::Button, pixel_font(SIZE_UI));
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(SIZE_BODY));
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(SIZE_SMALL));
        s.animation_time = MOTION;
        let v = &mut s.visuals;
        v.window_corner_radius = CornerRadius::ZERO;
        v.menu_corner_radius = CornerRadius::ZERO;
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = CornerRadius::ZERO;
        }
        // Sombra dura desplazada, sin desenfoque: como la cara oscura de un bloque.
        let shadow = |offset, alpha| Shadow {
            offset: [offset, offset],
            blur: 0,
            spread: 0,
            color: Color32::from_black_alpha(alpha),
        };
        v.window_shadow = shadow(5, if v.dark_mode { 150 } else { 60 });
        v.popup_shadow = shadow(3, if v.dark_mode { 150 } else { 60 });
    });

    ctx.style_mut_of(Theme::Dark, |s| {
        let v = &mut s.visuals;
        let edge = Color32::from_rgb(14, 16, 20);
        v.panel_fill = Color32::from_rgb(28, 31, 38);
        v.window_fill = Color32::from_rgb(34, 38, 47);
        v.faint_bg_color = Color32::from_rgb(35, 39, 48);
        v.extreme_bg_color = Color32::from_rgb(19, 21, 26);
        v.window_stroke = Stroke::new(1.0, edge);
        v.hyperlink_color = ACCENT;
        // Texto secundario con contraste 4.5:1 sobre el panel.
        v.weak_text_color = Some(Color32::from_rgb(148, 155, 168));
        v.selection.bg_fill = ACCENT;
        v.selection.stroke = Stroke::new(1.0, ON_ACCENT);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = v.panel_fill;
        w.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(46, 51, 61));
        w.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(201, 206, 216));
        for (wv, fill) in [
            (&mut w.inactive, Color32::from_rgb(45, 50, 61)),
            (&mut w.open, Color32::from_rgb(45, 50, 61)),
            (&mut w.hovered, Color32::from_rgb(56, 62, 75)),
            (&mut w.active, Color32::from_rgb(67, 74, 89)),
        ] {
            wv.bg_fill = fill;
            wv.weak_bg_fill = fill;
            wv.bg_stroke = Stroke::new(1.0, edge);
        }
        w.hovered.bg_stroke = Stroke::new(1.0, ACCENT.gamma_multiply(0.7));
        w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    });

    ctx.style_mut_of(Theme::Light, |s| {
        let v = &mut s.visuals;
        v.hyperlink_color = ACCENT_DARK;
        v.selection.bg_fill = ACCENT_DARK;
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        let w = &mut v.widgets;
        let edge = Stroke::new(1.0, Color32::from_rgb(169, 175, 186));
        w.inactive.bg_stroke = edge;
        w.open.bg_stroke = edge;
        w.hovered.bg_stroke = Stroke::new(1.0, ACCENT_DARK);
        w.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK);
    });
}

/// Fuente pixel de los títulos, para usarla en otros textos destacados.
pub fn pixel_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(PIXEL.into()))
}

/// Activa o apaga las transiciones según «Reducir animaciones».
pub fn set_motion(ctx: &egui::Context, reduce: bool) {
    let t = if reduce { 0.0 } else { MOTION };
    if ctx.global_style().animation_time != t {
        ctx.all_styles_mut(|s| s.animation_time = t);
    }
}

/// Progreso de 0 a 1 de una transición, con salida suave.
fn animate(ui: &egui::Ui, id: egui::Id, on: bool) -> f32 {
    let t = ui
        .ctx()
        .animate_bool_with_time(id, on, ui.style().animation_time);
    1.0 - (1.0 - t).powi(3)
}

/// Botón cuadrado con ícono, con aspecto de bloque: se levanta al pasar el cursor y se
/// hunde al pulsarlo. `selected` lo pinta con el acento.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, selected: bool, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, tip));
    if ui.is_rect_visible(rect) {
        let v = ui.visuals();
        let hover = animate(ui, resp.id, resp.hovered() || resp.has_focus());
        let lift = if resp.is_pointer_button_down_on() {
            -1.0
        } else {
            2.0 * hover
        };
        let face = rect.shrink(2.0).translate(Vec2::splat(-lift));
        let p = ui.painter();
        p.rect_filled(
            rect.shrink(2.0).translate(Vec2::splat(2.0)),
            0.0,
            Color32::from_black_alpha(if v.dark_mode { 150 } else { 60 }),
        );
        let (fill, ink) = if selected {
            (v.selection.bg_fill, v.selection.stroke.color)
        } else {
            (
                v.widgets
                    .inactive
                    .bg_fill
                    .lerp_to_gamma(v.widgets.hovered.bg_fill, hover),
                v.text_color(),
            )
        };
        p.rect_filled(face, 0.0, fill);
        let edge = v
            .widgets
            .inactive
            .bg_stroke
            .color
            .lerp_to_gamma(ACCENT, hover);
        p.rect_stroke(face, 0.0, Stroke::new(1.0, edge), StrokeKind::Inside);
        let tex = icons::texture(ui.ctx(), icon);
        p.image(
            tex.id(),
            Rect::from_center_size(face.center(), Vec2::splat(16.0)),
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            ink,
        );
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip)
}

/// Pestaña o segmento sin marco: ícono opcional y texto pixel, resaltado si está activo.
pub fn tab(ui: &mut egui::Ui, icon: Option<Icon>, text: &str, active: bool) -> Response {
    let v = ui.visuals();
    let color = if active {
        v.strong_text_color()
    } else {
        v.weak_text_color()
    };
    let text = RichText::new(text).color(color);
    let button = match icon {
        Some(i) => egui::Button::image_and_text(icons::image(ui.ctx(), i).tint(color), text),
        None => egui::Button::new(text),
    };
    ui.add(button.frame_when_inactive(false).selected(false))
}

/// Barra de acento bajo `target` que se desliza hasta él cuando cambia.
pub fn slide_marker(ui: &egui::Ui, id: egui::Id, target: Rect) {
    let ctx = ui.ctx();
    let t = ui.style().animation_time;
    let x0 = ctx.animate_value_with_time(id.with(0), target.left(), t);
    let x1 = ctx.animate_value_with_time(id.with(1), target.right(), t);
    let y = target.bottom() + 1.0;
    ui.painter().rect_filled(
        Rect::from_min_max(egui::pos2(x0, y), egui::pos2(x1, y + 3.0)),
        0.0,
        ui.visuals().selection.bg_fill,
    );
}

/// Título de sección del panel lateral: ícono en acento y texto pixel.
pub fn section(ui: &mut egui::Ui, icon: Icon, title: &str) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let accent = ui.visuals().selection.bg_fill;
        ui.add(icons::image(ui.ctx(), icon).tint(accent));
        ui.heading(title);
    });
    ui.add_space(2.0);
}

/// Opción de menú con ícono a la izquierda.
pub fn menu_button(ui: &egui::Ui, icon: Icon, text: String) -> egui::Button<'static> {
    egui::Button::image_and_text(icons::image(ui.ctx(), icon), text)
        .image_tint_follows_text_color(true)
}

/// Progreso 0..1 de una aparición que empezó en `start` (segundos de egui) y dura `dur`,
/// con salida suave. Pide repintar mientras no termina.
pub fn appear(ui: &egui::Ui, start: f64, delay: f64, dur: f64) -> f32 {
    if ui.style().animation_time == 0.0 {
        return 1.0;
    }
    let now = ui.input(|i| i.time);
    let t = ((now - start - delay) / dur).clamp(0.0, 1.0) as f32;
    if t < 1.0 {
        ui.ctx().request_repaint();
    }
    1.0 - (1.0 - t).powi(3)
}
