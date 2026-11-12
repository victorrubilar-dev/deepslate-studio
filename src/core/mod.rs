//! Modelo de skin: disposición de la textura 64×64, carga de PNG y conversiones.

mod convert;
mod layout;

pub use convert::{convert_model, from_legacy};
#[cfg_attr(not(test), allow(unused_imports))] // Face solo lo usan las pruebas por ahora
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

#[cfg(test)]
mod tests {
    use super::*;

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
