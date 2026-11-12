//! Plantillas dibujadas con código sobre el mapa de partes, así sirven para Steve y Alex.
//! «Base» es una skin completa; las prendas son capas con transparencia que se aplican encima.

use crate::core::{FACES, Face, Layer, Model, PARTS, Part, Rect, rect};
use image::{Rgba, RgbaImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    Base,
    Shirt,
    Pants,
    Hair,
    Armor,
}

const SIDES: [Face; 4] = [Face::Right, Face::Front, Face::Left, Face::Back];
const SKIN: Rgba<u8> = Rgba([198, 156, 123, 255]);
const SKIN_DARK: Rgba<u8> = Rgba([160, 116, 88, 255]);
const SHOES: Rgba<u8> = Rgba([58, 58, 62, 255]);
const IRON: Rgba<u8> = Rgba([200, 205, 211, 255]);
const IRON_DARK: Rgba<u8> = Rgba([142, 148, 156, 255]);

/// Mismo color, más oscuro (para bordes y detalles).
fn shade(c: Rgba<u8>, k: f32) -> Rgba<u8> {
    let [r, g, b, a] = c.0;
    let f = |v: u8| (v as f32 * k) as u8;
    Rgba([f(r), f(g), f(b), a])
}

struct Painter<'a> {
    img: &'a mut RgbaImage,
    model: Model,
}

impl Painter<'_> {
    fn face(&self, part: Part, layer: Layer, face: Face) -> Rect {
        rect(part, layer, face, self.model)
    }

    /// Pinta las filas `rows` (relativas a la cara) de una cara.
    fn rows(
        &mut self,
        part: Part,
        layer: Layer,
        face: Face,
        rows: std::ops::Range<u32>,
        c: Rgba<u8>,
    ) {
        let r = self.face(part, layer, face);
        for (x, y) in r.pixels() {
            if rows.contains(&(y - r.y)) {
                self.img.put_pixel(x, y, c);
            }
        }
    }

    fn all(&mut self, part: Part, layer: Layer, face: Face, c: Rgba<u8>) {
        self.rows(part, layer, face, 0..u32::MAX, c);
    }

    /// Píxel en coordenadas relativas a la cara.
    fn at(&mut self, part: Part, layer: Layer, face: Face, x: u32, y: u32, c: Rgba<u8>) {
        let r = self.face(part, layer, face);
        self.img.put_pixel(r.x + x, r.y + y, c);
    }
}

/// Dibuja la plantilla para el modelo dado. Las prendas usan `color`; base y armadura no.
pub fn draw(t: Template, model: Model, color: Rgba<u8>) -> RgbaImage {
    let mut img = RgbaImage::new(64, 64);
    let mut p = Painter {
        img: &mut img,
        model,
    };
    let dark = shade(color, 0.72);
    use Layer::{Base, Overlay};
    match t {
        Template::Base => {
            for part in PARTS {
                for face in FACES {
                    p.all(part, Base, face, SKIN);
                }
            }
            let eye_white = Rgba([240, 240, 240, 255]);
            let eye = Rgba([60, 70, 110, 255]);
            for (x, c) in [(1, eye_white), (2, eye), (5, eye), (6, eye_white)] {
                p.at(Part::Head, Base, Face::Front, x, 4, c);
            }
            for x in 3..5 {
                p.at(Part::Head, Base, Face::Front, x, 6, SKIN_DARK);
            }
        }
        Template::Shirt => {
            for face in FACES {
                p.all(Part::Body, Base, face, color);
            }
            for face in SIDES {
                p.rows(Part::Body, Base, face, 11..12, dark);
            }
            p.rows(Part::Body, Base, Face::Front, 0..1, dark);
            // Mangas cortas.
            for arm in [Part::RightArm, Part::LeftArm] {
                p.all(arm, Base, Face::Top, color);
                for face in SIDES {
                    p.rows(arm, Base, face, 0..4, color);
                    p.rows(arm, Base, face, 3..4, dark);
                }
            }
        }
        Template::Pants => {
            for leg in [Part::RightLeg, Part::LeftLeg] {
                p.all(leg, Base, Face::Top, color);
                p.all(leg, Base, Face::Bottom, SHOES);
                for face in SIDES {
                    p.rows(leg, Base, face, 0..10, color);
                    p.rows(leg, Base, face, 10..12, SHOES);
                }
            }
            // Cinturón.
            for face in SIDES {
                p.rows(Part::Body, Base, face, 10..12, dark);
            }
            p.all(Part::Body, Base, Face::Bottom, dark);
        }
        Template::Hair => {
            p.all(Part::Head, Base, Face::Top, color);
            p.rows(Part::Head, Base, Face::Back, 0..6, color);
            p.rows(Part::Head, Base, Face::Back, 5..6, dark);
            for side in [Face::Right, Face::Left] {
                p.rows(Part::Head, Base, side, 0..3, color);
                p.rows(Part::Head, Base, side, 2..3, dark);
            }
            // Flequillo.
            p.rows(Part::Head, Base, Face::Front, 0..1, color);
            for x in [0, 1, 6, 7] {
                p.at(Part::Head, Base, Face::Front, x, 1, dark);
            }
        }
        Template::Armor => {
            // Casco: arriba, costados y marco de la cara, en la capa exterior.
            p.all(Part::Head, Overlay, Face::Top, IRON);
            for face in SIDES {
                p.rows(Part::Head, Overlay, face, 0..2, IRON);
                p.rows(Part::Head, Overlay, face, 2..3, IRON_DARK);
            }
            for side in [Face::Right, Face::Left, Face::Back] {
                p.rows(Part::Head, Overlay, side, 3..6, IRON);
            }
            for y in 2..6 {
                p.at(Part::Head, Overlay, Face::Front, 0, y, IRON_DARK);
                p.at(Part::Head, Overlay, Face::Front, 7, y, IRON_DARK);
            }
            // Peto con borde oscuro.
            for face in FACES {
                p.all(Part::Body, Overlay, face, IRON);
            }
            for face in SIDES {
                let r = p.face(Part::Body, Overlay, face);
                p.rows(Part::Body, Overlay, face, r.h - 1..r.h, IRON_DARK);
            }
            for y in 1..10 {
                p.at(Part::Body, Overlay, Face::Front, 0, y, IRON_DARK);
                let r = p.face(Part::Body, Overlay, Face::Front);
                p.at(Part::Body, Overlay, Face::Front, r.w - 1, y, IRON_DARK);
            }
            // Hombreras y botas.
            for arm in [Part::RightArm, Part::LeftArm] {
                p.all(arm, Overlay, Face::Top, IRON);
                for face in SIDES {
                    p.rows(arm, Overlay, face, 0..3, IRON);
                    p.rows(arm, Overlay, face, 3..4, IRON_DARK);
                }
            }
            for leg in [Part::RightLeg, Part::LeftLeg] {
                p.all(leg, Overlay, Face::Bottom, IRON_DARK);
                for face in SIDES {
                    p.rows(leg, Overlay, face, 8..12, IRON);
                    p.rows(leg, Overlay, face, 8..9, IRON_DARK);
                }
            }
        }
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::LAYERS;

    const TEMPLATES: [Template; 5] = [
        Template::Base,
        Template::Shirt,
        Template::Pants,
        Template::Hair,
        Template::Armor,
    ];

    fn in_layout(model: Model, layer: Option<Layer>, x: u32, y: u32) -> bool {
        PARTS.iter().any(|&part| {
            LAYERS
                .iter()
                .filter(|&&l| layer.is_none_or(|w| w == l))
                .any(|&l| {
                    FACES
                        .iter()
                        .any(|&f| rect(part, l, f, model).contains(x, y))
                })
        })
    }

    /// Nada se pinta fuera de las caras del modelo, para Steve y para Alex.
    #[test]
    fn stays_inside_the_layout() {
        for model in [Model::Steve, Model::Alex] {
            for t in TEMPLATES {
                let img = draw(t, model, Rgba([200, 30, 30, 255]));
                let layer = (t == Template::Armor).then_some(Layer::Overlay);
                let painted: Vec<_> = img
                    .enumerate_pixels()
                    .filter(|(_, _, p)| p[3] > 0)
                    .collect();
                assert!(!painted.is_empty(), "{t:?} vacía");
                for (x, y, _) in painted {
                    assert!(
                        in_layout(model, layer, x, y),
                        "{t:?} {model:?} pinta fuera en ({x}, {y})"
                    );
                }
            }
        }
    }

    #[test]
    fn base_covers_every_base_face() {
        let img = draw(Template::Base, Model::Alex, Rgba([0; 4]));
        for part in PARTS {
            for face in FACES {
                for (x, y) in rect(part, Layer::Base, face, Model::Alex).pixels() {
                    assert_eq!(img.get_pixel(x, y)[3], 255, "{part:?} {face:?}");
                }
            }
        }
    }

    #[test]
    fn clothes_use_the_color() {
        let c = Rgba([10, 200, 90, 255]);
        let img = draw(Template::Shirt, Model::Steve, c);
        let r = rect(Part::Body, Layer::Base, Face::Front, Model::Steve);
        assert_eq!(*img.get_pixel(r.x + 3, r.y + 5), c);
        assert_eq!(shade(c, 0.5), Rgba([5, 100, 45, 255]));
    }

    /// Guarda base + todas las prendas, ampliado, para revisarlo a ojo.
    #[test]
    #[ignore = "solo para revisar a mano; correr con --ignored"]
    fn write_templates() {
        let dir = std::env::var("DEEPSLATE_RENDER_OUT").expect("falta DEEPSLATE_RENDER_OUT");
        let mut img = draw(Template::Base, Model::Steve, Rgba([0; 4]));
        let colors = [[60, 110, 200], [70, 60, 50], [120, 70, 30]];
        for (t, [r, g, b]) in [Template::Shirt, Template::Pants, Template::Hair]
            .into_iter()
            .zip(colors)
        {
            image::imageops::overlay(&mut img, &draw(t, Model::Steve, Rgba([r, g, b, 255])), 0, 0);
        }
        image::imageops::overlay(
            &mut img,
            &draw(Template::Armor, Model::Steve, Rgba([0; 4])),
            0,
            0,
        );
        img.save(std::path::Path::new(&dir).join("plantillas-skin.png"))
            .unwrap();
        let big = image::imageops::resize(&img, 512, 512, image::imageops::Nearest);
        big.save(std::path::Path::new(&dir).join("plantillas.png"))
            .unwrap();
    }
}
