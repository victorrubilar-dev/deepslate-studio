//! Modelo de skin: disposición de la textura 64×64, carga de PNG y conversiones.

mod convert;
mod layout;

pub use convert::{convert_model, from_legacy};
pub use layout::{
    FACES, Face, LAYERS, Layer, Model, PARTS, Part, Rect, face_at, mirror_pixel, rect,
};

use image::{Rgba, RgbaImage};

pub const SIZE: u32 = 64;

#[derive(Clone, Debug, PartialEq)]
pub struct Skin {
    pub pixels: RgbaImage,
    pub model: Model,
}

#[derive(Debug, PartialEq)]
pub enum LoadError {
    Decode,
    /// Tamaño recibido (ancho, alto).
    BadSize(u32, u32),
}

impl Skin {
    pub fn blank(model: Model) -> Self {
        Self {
            pixels: RgbaImage::new(SIZE, SIZE),
            model,
        }
    }

    /// Maniquí neutro para el previsualizador: cada parte de un gris distinto, sin capa exterior.
    pub fn mannequin(model: Model) -> Self {
        let mut s = Self::blank(model);
        for (k, part) in PARTS.into_iter().enumerate() {
            for face in FACES {
                // Las caras de arriba y abajo un poco más oscuras, para que se lea el volumen.
                let shade = if matches!(face, Face::Top | Face::Bottom) {
                    20
                } else {
                    0
                };
                let g = 170 + (k as u8 % 2) * 20 - shade;
                for (x, y) in rect(part, Layer::Base, face, model).pixels() {
                    s.pixels
                        .put_pixel(x, y, Rgba([g, g, g.saturating_add(8), 255]));
                }
            }
        }
        s
    }

    /// Decodifica un PNG de 64×64 o 64×32 (este último se convierte).
    /// El modelo se deduce de la textura; quien llama puede cambiarlo después.
    pub fn from_png(bytes: &[u8]) -> Result<Self, LoadError> {
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .map_err(|_| LoadError::Decode)?
            .to_rgba8();
        match img.dimensions() {
            (64, 64) => {
                let model = guess_model(&img);
                Ok(Self { pixels: img, model })
            }
            (64, 32) => Ok(Self {
                pixels: from_legacy(&img),
                model: Model::Steve,
            }),
            (w, h) => Err(LoadError::BadSize(w, h)),
        }
    }

    pub fn to_png(&self) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        self.pixels
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("codificar PNG en memoria no falla");
        out.into_inner()
    }

    /// Píxeles de la capa base que no son totalmente opacos. El juego los muestra opacos.
    pub fn translucent_base_pixels(&self) -> usize {
        let mut n = 0;
        for part in PARTS {
            for face in FACES {
                let r = rect(part, Layer::Base, face, self.model);
                n += r
                    .pixels()
                    .filter(|&(x, y)| self.pixels.get_pixel(x, y)[3] < 255)
                    .count();
            }
        }
        n
    }
}

/// En Alex la cara trasera del brazo derecho mide 3 px, así que la columna x=55 queda vacía.
fn guess_model(img: &RgbaImage) -> Model {
    let empty = (20..32).all(|y| img.get_pixel(55, y)[3] == 0);
    let used = (20..32).any(|y| img.get_pixel(51, y)[3] != 0);
    if empty && used {
        Model::Alex
    } else {
        Model::Steve
    }
}

pub const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Lleva una imagen de capa de la espalda a 64×32. Acepta 64×32 y sus múltiplos
/// (se reducen) y el formato antiguo de 22×17 (se coloca arriba a la izquierda).
pub fn cape_texture(img: &RgbaImage) -> Option<RgbaImage> {
    let (w, h) = img.dimensions();
    if (w, h) == (22, 17) {
        let mut out = RgbaImage::new(64, 32);
        image::imageops::replace(&mut out, img, 0, 0);
        return Some(out);
    }
    (w == 2 * h && w % 64 == 0)
        .then(|| image::imageops::resize(img, 64, 32, image::imageops::FilterType::Nearest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cape_sizes() {
        let px = Rgba([9, 9, 9, 255]);
        let big = RgbaImage::from_pixel(128, 64, px);
        assert_eq!(cape_texture(&big).unwrap().get_pixel(63, 31), &px);
        let old = RgbaImage::from_pixel(22, 17, px);
        let c = cape_texture(&old).unwrap();
        assert_eq!(
            (c.dimensions(), c.get_pixel(21, 16), c.get_pixel(22, 0)[3]),
            ((64, 32), &px, 0)
        );
        assert!(cape_texture(&RgbaImage::new(64, 64)).is_none());
        assert!(cape_texture(&RgbaImage::new(100, 50)).is_none());
    }

    #[test]
    fn mannequin_is_opaque_base_only() {
        for model in [Model::Steve, Model::Alex] {
            let m = Skin::mannequin(model);
            assert_eq!(m.translucent_base_pixels(), 0);
            let overlay = rect(Part::Head, Layer::Overlay, Face::Front, model);
            assert!(
                overlay
                    .pixels()
                    .all(|(x, y)| m.pixels.get_pixel(x, y)[3] == 0)
            );
        }
    }

    #[test]
    fn png_roundtrip() {
        let mut s = Skin::blank(Model::Steve);
        s.pixels.put_pixel(8, 8, Rgba([1, 2, 3, 255]));
        let back = Skin::from_png(&s.to_png()).unwrap();
        assert_eq!(back.pixels, s.pixels);
    }

    #[test]
    fn rejects_bad_size() {
        let img = RgbaImage::new(32, 32);
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        assert_eq!(
            Skin::from_png(buf.get_ref()),
            Err(LoadError::BadSize(32, 32))
        );
        assert_eq!(Skin::from_png(b"no es png"), Err(LoadError::Decode));
    }

    #[test]
    fn guesses_alex() {
        let fill = |model| {
            let mut s = Skin::blank(model);
            for face in FACES {
                for (x, y) in rect(Part::RightArm, Layer::Base, face, model).pixels() {
                    s.pixels.put_pixel(x, y, Rgba([9, 9, 9, 255]));
                }
            }
            Skin::from_png(&s.to_png()).unwrap().model
        };
        assert_eq!(fill(Model::Alex), Model::Alex);
        assert_eq!(fill(Model::Steve), Model::Steve);
    }

    #[test]
    fn counts_translucent_base() {
        let mut s = Skin::blank(Model::Steve);
        let total = s.translucent_base_pixels();
        // Cabeza 384 + torso 352 + 4 extremidades de 224.
        assert_eq!(total, 384 + 352 + 4 * 224);
        for (x, y) in rect(Part::Head, Layer::Base, Face::Front, Model::Steve).pixels() {
            s.pixels.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
        assert_eq!(s.translucent_base_pixels(), total - 64);
    }
}
