//! Dónde está cada cara de cada parte dentro de la textura 64×64.
//! "Derecha" e "izquierda" son las del personaje: la cara Right del brazo derecho es la exterior.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
    Steve,
    Alex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Head,
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    Base,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Top,
    Bottom,
    Right,
    Front,
    Left,
    Back,
}

pub const PARTS: [Part; 6] = [
    Part::Head,
    Part::Body,
    Part::RightArm,
    Part::LeftArm,
    Part::RightLeg,
    Part::LeftLeg,
];
pub const LAYERS: [Layer; 2] = [Layer::Base, Layer::Overlay];
pub const FACES: [Face; 6] = [
    Face::Top,
    Face::Bottom,
    Face::Right,
    Face::Front,
    Face::Left,
    Face::Back,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn pixels(self) -> impl Iterator<Item = (u32, u32)> {
        (self.y..self.y + self.h).flat_map(move |y| (self.x..self.x + self.w).map(move |x| (x, y)))
    }
}

/// Origen (u, v) de la caja en la textura y medidas (ancho, alto, fondo).
fn cube(part: Part, layer: Layer, model: Model) -> (u32, u32, u32, u32, u32) {
    let arm_w = if model == Model::Alex { 3 } else { 4 };
    let overlay = layer == Layer::Overlay;
    let ((u, v), (w, h, d)) = match part {
        Part::Head => (if overlay { (32, 0) } else { (0, 0) }, (8, 8, 8)),
        Part::Body => (if overlay { (16, 32) } else { (16, 16) }, (8, 12, 4)),
        Part::RightArm => (if overlay { (40, 32) } else { (40, 16) }, (arm_w, 12, 4)),
        Part::LeftArm => (if overlay { (48, 48) } else { (32, 48) }, (arm_w, 12, 4)),
        Part::RightLeg => (if overlay { (0, 32) } else { (0, 16) }, (4, 12, 4)),
        Part::LeftLeg => (if overlay { (0, 48) } else { (16, 48) }, (4, 12, 4)),
    };
    (u, v, w, h, d)
}

pub fn rect(part: Part, layer: Layer, face: Face, model: Model) -> Rect {
    let (u, v, w, h, d) = cube(part, layer, model);
    let (x, y, fw, fh) = match face {
        Face::Top => (u + d, v, w, d),
        Face::Bottom => (u + d + w, v, w, d),
        Face::Right => (u, v + d, d, h),
        Face::Front => (u + d, v + d, w, h),
        Face::Left => (u + d + w, v + d, d, h),
        Face::Back => (u + d + w + d, v + d, w, h),
    };
    Rect { x, y, w: fw, h: fh }
}

/// Cara que ocupa el píxel (x, y), si hay alguna.
pub fn face_at(model: Model, x: u32, y: u32) -> Option<(Part, Layer, Face)> {
    for part in PARTS {
        for layer in LAYERS {
            for face in FACES {
                if rect(part, layer, face, model).contains(x, y) {
                    return Some((part, layer, face));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(model: Model) -> Vec<Rect> {
        let mut v = Vec::new();
        for p in PARTS {
            for l in LAYERS {
                for f in FACES {
                    v.push(rect(p, l, f, model));
                }
            }
        }
        v
    }

    #[test]
    fn in_bounds_and_no_overlap() {
        for model in [Model::Steve, Model::Alex] {
            let mut used = [[false; 64]; 64];
            for r in all(model) {
                for (x, y) in r.pixels() {
                    assert!(x < 64 && y < 64, "{r:?} fuera de la textura");
                    assert!(
                        !used[y as usize][x as usize],
                        "{model:?}: solape en {x},{y}"
                    );
                    used[y as usize][x as usize] = true;
                }
            }
        }
    }

    #[test]
    fn known_positions() {
        // Valores de referencia de la disposición estándar.
        let s = Model::Steve;
        assert_eq!(
            rect(Part::Head, Layer::Base, Face::Front, s),
            Rect {
                x: 8,
                y: 8,
                w: 8,
                h: 8
            }
        );
        assert_eq!(
            rect(Part::Head, Layer::Overlay, Face::Front, s),
            Rect {
                x: 40,
                y: 8,
                w: 8,
                h: 8
            }
        );
        assert_eq!(
            rect(Part::Body, Layer::Base, Face::Front, s),
            Rect {
                x: 20,
                y: 20,
                w: 8,
                h: 12
            }
        );
        assert_eq!(
            rect(Part::RightArm, Layer::Base, Face::Front, s),
            Rect {
                x: 44,
                y: 20,
                w: 4,
                h: 12
            }
        );
        assert_eq!(
            rect(Part::LeftArm, Layer::Base, Face::Front, s),
            Rect {
                x: 36,
                y: 52,
                w: 4,
                h: 12
            }
        );
        assert_eq!(
            rect(Part::RightLeg, Layer::Base, Face::Front, s),
            Rect {
                x: 4,
                y: 20,
                w: 4,
                h: 12
            }
        );
        assert_eq!(
            rect(Part::LeftLeg, Layer::Base, Face::Front, s),
            Rect {
                x: 20,
                y: 52,
                w: 4,
                h: 12
            }
        );
        assert_eq!(
            rect(Part::RightArm, Layer::Base, Face::Back, Model::Alex),
            Rect {
                x: 51,
                y: 20,
                w: 3,
                h: 12
            }
        );
    }

    #[test]
    fn face_lookup() {
        assert_eq!(
            face_at(Model::Steve, 9, 9),
            Some((Part::Head, Layer::Base, Face::Front))
        );
        assert_eq!(face_at(Model::Steve, 0, 0), None);
        assert_eq!(face_at(Model::Alex, 55, 20), None);
        assert_eq!(
            face_at(Model::Steve, 55, 20),
            Some((Part::RightArm, Layer::Base, Face::Back))
        );
    }
}
