//! Conversiones: 64×32 → 64×64 y Steve ↔ Alex.

use super::layout::{FACES, Face, LAYERS, Layer, Model, Part, rect};
use super::{SIZE, Skin, TRANSPARENT};
use image::RgbaImage;

/// Convierte una skin antigua de 64×32. Las extremidades izquierdas no existían: el juego
/// dibujaba las derechas espejadas, así que se copian volteadas y con exterior/interior intercambiados.
pub fn from_legacy(old: &RgbaImage) -> RgbaImage {
    let mut out = RgbaImage::new(SIZE, SIZE);
    for (x, y, px) in old.enumerate_pixels() {
        out.put_pixel(x, y, *px);
    }
    for (src_part, dst_part) in [
        (Part::RightLeg, Part::LeftLeg),
        (Part::RightArm, Part::LeftArm),
    ] {
        for face in FACES {
            let dst_face = match face {
                Face::Right => Face::Left,
                Face::Left => Face::Right,
                f => f,
            };
            let src = rect(src_part, Layer::Base, face, Model::Steve);
            let dst = rect(dst_part, Layer::Base, dst_face, Model::Steve);
            for (i, j) in (0..src.w).flat_map(|i| (0..src.h).map(move |j| (i, j))) {
                let px = *old.get_pixel(src.x + i, src.y + j);
                out.put_pixel(dst.x + src.w - 1 - i, dst.y + j, px);
            }
        }
    }
    // Muchas skins antiguas no tenían canal alfa y el sombrero venía relleno. El juego trata
    // esa zona como transparente si no hay ningún píxel transparente en ella.
    let hat_area_opaque = (0..32).all(|y| (32..64).all(|x| old.get_pixel(x, y)[3] >= 128));
    if hat_area_opaque {
        for face in FACES {
            for (x, y) in rect(Part::Head, Layer::Overlay, face, Model::Steve).pixels() {
                out.put_pixel(x, y, TRANSPARENT);
            }
        }
    }
    out
}

/// Columna de origen para la columna `dx` de destino al cambiar el ancho del brazo.
/// 4→3 quita la columna 2; 3→4 duplica la 1. Así Alex → Steve → Alex no pierde nada.
fn source_column(dx: u32, from_w: u32, to_w: u32) -> u32 {
    match (from_w, to_w) {
        (4, 3) if dx >= 2 => dx + 1,
        (3, 4) if dx >= 2 => dx - 1,
        _ => dx,
    }
}

pub fn convert_model(skin: &Skin, to: Model) -> Skin {
    let from = skin.model;
    let mut out = skin.pixels.clone();
    let arms = [Part::RightArm, Part::LeftArm];
    for part in arms {
        for layer in LAYERS {
            for face in FACES {
                for (x, y) in rect(part, layer, face, from).pixels() {
                    out.put_pixel(x, y, TRANSPARENT);
                }
            }
        }
    }
    for part in arms {
        for layer in LAYERS {
            for face in FACES {
                let src = rect(part, layer, face, from);
                let dst = rect(part, layer, face, to);
                for dx in 0..dst.w {
                    let sx = source_column(dx, src.w, dst.w);
                    for dy in 0..dst.h {
                        out.put_pixel(
                            dst.x + dx,
                            dst.y + dy,
                            *skin.pixels.get_pixel(src.x + sx, src.y + dy),
                        );
                    }
                }
            }
        }
    }
    Skin {
        pixels: out,
        model: to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PARTS;
    use image::Rgba;

    /// Color distinto por coordenada, para detectar cualquier píxel mal ubicado.
    fn mark(x: u32, y: u32) -> Rgba<u8> {
        Rgba([x as u8, y as u8, 7, 255])
    }

    #[test]
    fn legacy_mirrors_limbs() {
        let old = RgbaImage::from_fn(64, 32, mark);
        let out = from_legacy(&old);
        for (y, row) in old.rows().enumerate() {
            for (x, px) in row.enumerate() {
                if y >= 16 || x < 32 {
                    assert_eq!(out.get_pixel(x as u32, y as u32), px);
                }
            }
        }
        for (src_part, dst_part) in [
            (Part::RightLeg, Part::LeftLeg),
            (Part::RightArm, Part::LeftArm),
        ] {
            for (sf, df) in [
                (Face::Right, Face::Left),
                (Face::Left, Face::Right),
                (Face::Front, Face::Front),
                (Face::Back, Face::Back),
                (Face::Top, Face::Top),
                (Face::Bottom, Face::Bottom),
            ] {
                let s = rect(src_part, Layer::Base, sf, Model::Steve);
                let d = rect(dst_part, Layer::Base, df, Model::Steve);
                for i in 0..s.w {
                    for j in 0..s.h {
                        assert_eq!(
                            out.get_pixel(d.x + s.w - 1 - i, d.y + j),
                            &mark(s.x + i, s.y + j)
                        );
                    }
                }
            }
        }
        // Overlays de extremidades y torso vacíos.
        for part in [
            Part::Body,
            Part::RightArm,
            Part::LeftArm,
            Part::RightLeg,
            Part::LeftLeg,
        ] {
            for face in FACES {
                for (x, y) in rect(part, Layer::Overlay, face, Model::Steve).pixels() {
                    assert_eq!(out.get_pixel(x, y)[3], 0);
                }
            }
        }
    }

    #[test]
    fn legacy_hat_hack() {
        let opaque = RgbaImage::from_pixel(64, 32, Rgba([1, 1, 1, 255]));
        let out = from_legacy(&opaque);
        assert_eq!(out.get_pixel(40, 8)[3], 0);
        assert_eq!(out.get_pixel(8, 8)[3], 255);

        let mut with_alpha = opaque.clone();
        with_alpha.put_pixel(60, 30, TRANSPARENT);
        assert_eq!(from_legacy(&with_alpha).get_pixel(40, 8)[3], 255);
    }

    fn painted(model: Model) -> Skin {
        let mut s = Skin::blank(model);
        for part in PARTS {
            for layer in LAYERS {
                for face in FACES {
                    for (x, y) in rect(part, layer, face, model).pixels() {
                        s.pixels.put_pixel(x, y, mark(x, y));
                    }
                }
            }
        }
        s
    }

    #[test]
    fn alex_steve_alex_is_lossless() {
        let alex = painted(Model::Alex);
        let steve = convert_model(&alex, Model::Steve);
        assert_eq!(steve.model, Model::Steve);
        assert_eq!(convert_model(&steve, Model::Alex), alex);
    }

    #[test]
    fn steve_to_alex_keeps_rest() {
        let steve = painted(Model::Steve);
        let alex = convert_model(&steve, Model::Alex);
        for part in [Part::Head, Part::Body, Part::RightLeg, Part::LeftLeg] {
            for layer in LAYERS {
                for face in FACES {
                    for (x, y) in rect(part, layer, face, Model::Steve).pixels() {
                        assert_eq!(alex.pixels.get_pixel(x, y), steve.pixels.get_pixel(x, y));
                    }
                }
            }
        }
        // Las caras laterales no cambian de ancho: se copian enteras.
        let s = rect(Part::RightArm, Layer::Base, Face::Left, Model::Steve);
        let a = rect(Part::RightArm, Layer::Base, Face::Left, Model::Alex);
        assert_eq!(alex.pixels.get_pixel(a.x, a.y), &mark(s.x, s.y));
        // La columna 55 (sobrante en Alex) queda vacía.
        assert!((20..32).all(|y| alex.pixels.get_pixel(55, y)[3] == 0));
    }
}
