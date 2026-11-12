//! Imagen al frente: una imagen cualquiera, recortada y reducida, pintada sobre las caras
//! frontales de todo el cuerpo, como un póster sobre el personaje.

use crate::core::{Face, Layer, Model, Part, rect};
use image::{Rgba, RgbaImage, imageops};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Entra entera; sobran bordes transparentes.
    Contain,
    /// Cubre toda la silueta; se recorta lo que sobra.
    Cover,
}

/// Tamaño de la silueta frontal en píxeles: brazo + torso + brazo de ancho, 32 de alto.
pub fn silhouette(model: Model) -> (u32, u32) {
    let arm = if model == Model::Alex { 3 } else { 4 };
    (8 + 2 * arm, 32)
}

/// Reduce el recorte `(x, y, w, h)` de `src` al tamaño de la silueta.
pub fn fit(
    src: &RgbaImage,
    (x, y, w, h): (u32, u32, u32, u32),
    model: Model,
    mode: Fit,
) -> RgbaImage {
    let (cw, ch) = silhouette(model);
    let crop = imageops::crop_imm(src, x, y, w.max(1), h.max(1)).to_image();
    let (sw, sh) = (crop.width() as f32, crop.height() as f32);
    let scale = match mode {
        Fit::Contain => (cw as f32 / sw).min(ch as f32 / sh),
        Fit::Cover => (cw as f32 / sw).max(ch as f32 / sh),
    };
    let (tw, th) = (
        ((sw * scale).round() as u32).max(1),
        ((sh * scale).round() as u32).max(1),
    );
    // `thumbnail` promedia por áreas: al reducir mucho no se pierden detalles al azar.
    let small = imageops::thumbnail(&crop, tw, th);
    let mut out = RgbaImage::new(cw, ch);
    imageops::overlay(
        &mut out,
        &small,
        (cw as i64 - tw as i64) / 2,
        (ch as i64 - th as i64) / 2,
    );
    out
}

/// Lleva la silueta a una capa de 64×64 sobre las caras frontales. Los píxeles casi
/// transparentes se descartan y el resto queda opaco.
pub fn to_front(canvas: &RgbaImage, model: Model, layer: Layer) -> RgbaImage {
    let arm = (silhouette(model).0 - 8) / 2;
    // Origen de cada parte dentro de la silueta. El lado derecho del personaje queda a la
    // izquierda de quien lo mira.
    let places = [
        (Part::Head, arm, 0),
        (Part::RightArm, 0, 8),
        (Part::Body, arm, 8),
        (Part::LeftArm, arm + 8, 8),
        (Part::RightLeg, arm, 20),
        (Part::LeftLeg, arm + 4, 20),
    ];
    let mut out = RgbaImage::new(64, 64);
    for (part, ox, oy) in places {
        let r = rect(part, layer, Face::Front, model);
        for (x, y) in r.pixels() {
            let p = canvas.get_pixel(ox + x - r.x, oy + y - r.y);
            if p[3] >= 128 {
                out.put_pixel(x, y, Rgba([p[0], p[1], p[2], 255]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contain_keeps_aspect_and_cover_fills() {
        let red = RgbaImage::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
        let c = fit(&red, (0, 0, 100, 100), Model::Steve, Fit::Contain);
        assert_eq!(c.dimensions(), (16, 32));
        assert_eq!(c.get_pixel(8, 4)[3], 0);
        assert_eq!(c.get_pixel(8, 16)[3], 255);
        assert_eq!(c.get_pixel(8, 28)[3], 0);
        let f = fit(&red, (0, 0, 100, 100), Model::Alex, Fit::Cover);
        assert_eq!(f.dimensions(), (14, 32));
        assert!(f.pixels().all(|p| p[3] == 255));
    }

    #[test]
    fn crop_selects_the_region() {
        let mut img = RgbaImage::from_pixel(40, 40, Rgba([0, 0, 255, 255]));
        for (x, y, p) in img.enumerate_pixels_mut() {
            if x >= 20 && y >= 20 {
                *p = Rgba([0, 255, 0, 255]);
            }
        }
        let c = fit(&img, (20, 20, 20, 20), Model::Steve, Fit::Cover);
        assert!(c.pixels().all(|p| *p == Rgba([0, 255, 0, 255])));
    }

    #[test]
    fn front_mapping() {
        for model in [Model::Steve, Model::Alex] {
            let (w, h) = silhouette(model);
            // Cada píxel guarda su posición en la silueta.
            let canvas = RgbaImage::from_fn(w, h, |x, y| Rgba([x as u8, y as u8, 0, 255]));
            let out = to_front(&canvas, model, Layer::Base);
            let arm = if model == Model::Alex { 3 } else { 4 };
            let head = rect(Part::Head, Layer::Base, Face::Front, model);
            assert_eq!(
                out.get_pixel(head.x + 1, head.y + 2).0,
                [arm as u8 + 1, 2, 0, 255]
            );
            let left_arm = rect(Part::LeftArm, Layer::Base, Face::Front, model);
            assert_eq!(left_arm.w, arm);
            assert_eq!(
                out.get_pixel(left_arm.x + arm - 1, left_arm.y).0,
                [(w - 1) as u8, 8, 0, 255]
            );
            let leg = rect(Part::LeftLeg, Layer::Overlay, Face::Front, model);
            let over = to_front(&canvas, model, Layer::Overlay);
            assert_eq!(
                over.get_pixel(leg.x, leg.y + 11).0,
                [arm as u8 + 4, 31, 0, 255]
            );
            // Solo caras frontales: el resto queda transparente.
            // Solo caras frontales: cabeza 8×8, torso 8×12, dos brazos y dos piernas.
            let opaque = out.pixels().filter(|p| p[3] > 0).count() as u32;
            assert_eq!(opaque, 64 + 96 + 2 * arm * 12 + 96);
        }
    }
}
