//! Estilo visual propio: esquinas rectas y sombras duras (aspecto de bloque), tonos pizarra
//! y el menta del ícono como acento. Los títulos usan una fuente pixel.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke,
    TextStyle, Theme,
};
use std::sync::Arc;

/// Menta del ícono. `render3d/skin.wgsl` usa el mismo para la parte seleccionada.
pub const ACCENT: Color32 = Color32::from_rgb(64, 214, 180);
/// Versión más oscura para el tema claro, legible con texto blanco encima.
const ACCENT_DARK: Color32 = Color32::from_rgb(26, 125, 103);
/// Texto sobre el acento en el tema oscuro.
const ON_ACCENT: Color32 = Color32::from_rgb(15, 26, 23);
const PIXEL: &str = "pixel";

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
        s.text_styles.insert(
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Name(PIXEL.into())),
        );
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
