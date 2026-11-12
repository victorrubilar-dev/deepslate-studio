//! Formato de proyecto: un PNG de skin normal con los datos del proyecto en un bloque iTXt.
//! Cualquier visor lo abre como imagen, y un PNG sin ese bloque se importa como skin nueva.

use crate::core::{LoadError, Model, Skin};
use crate::render3d::Pose;
use serde::{Deserialize, Serialize};

const KEYWORD: &str = "deepslate-studio";
/// v1: modelo. v2: agrega la pose. v3: agrega los datos de la tarjeta de biblioteca.
pub const VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
struct Meta {
    version: u32,
    model: Model,
    #[serde(default)]
    pose: Pose,
    #[serde(default)]
    card: Card,
}

/// Datos de la tarjeta en la biblioteca. Viajan dentro del PNG, así que se conservan
/// al mover o compartir el archivo.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Card {
    /// Vacío: se muestra el nombre del archivo.
    pub name: String,
    pub background: Background,
    pub favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Background {
    /// Uno de los fondos dibujados por la app (índice).
    Preset(u8),
    /// Imagen del usuario, por nombre de archivo dentro de la carpeta de fondos.
    Custom(String),
}

impl Default for Background {
    fn default() -> Self {
        Self::Preset(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub skin: Skin,
    pub pose: Pose,
    pub card: Card,
}

#[derive(Debug, PartialEq)]
pub enum ProjectError {
    Skin(LoadError),
    /// El archivo es de una versión más nueva de la app.
    NewerVersion(u32),
    Corrupt,
}

pub fn save(skin: &Skin, pose: &Pose, card: &Card) -> Vec<u8> {
    let meta = Meta {
        version: VERSION,
        model: skin.model,
        pose: *pose,
        card: card.clone(),
    };
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, skin.pixels.width(), skin.pixels.height());
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.add_itxt_chunk(
        KEYWORD.into(),
        ron::to_string(&meta).expect("Meta siempre se serializa"),
    )
    .expect("añadir iTXt no falla");
    let mut w = enc.write_header().expect("escribir en memoria no falla");
    w.write_image_data(skin.pixels.as_raw())
        .expect("escribir en memoria no falla");
    w.finish().expect("escribir en memoria no falla");
    out
}

pub fn load(bytes: &[u8]) -> Result<Project, ProjectError> {
    let mut skin = Skin::from_png(bytes).map_err(ProjectError::Skin)?;
    let mut pose = Pose::default();
    let mut card = Card::default();
    if let Some(text) = read_meta(bytes) {
        #[derive(Deserialize)]
        struct VersionOnly {
            version: u32,
        }
        let v: VersionOnly = ron::from_str(&text).map_err(|_| ProjectError::Corrupt)?;
        if v.version > VERSION {
            return Err(ProjectError::NewerVersion(v.version));
        }
        let meta: Meta = ron::from_str(&text).map_err(|_| ProjectError::Corrupt)?;
        skin.model = meta.model;
        pose = meta.pose;
        card = meta.card;
    }
    Ok(Project { skin, pose, card })
}

/// Indica si el PNG trae datos de proyecto (y no es una skin suelta).
pub fn is_project(bytes: &[u8]) -> bool {
    read_meta(bytes).is_some()
}

fn read_meta(bytes: &[u8]) -> Option<String> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    reader
        .info()
        .utf8_text
        .iter()
        .find(|c| c.keyword == KEYWORD)?
        .get_text()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render3d::Preset;
    use image::Rgba;

    fn sample() -> Skin {
        let mut s = Skin::blank(Model::Alex);
        s.pixels.put_pixel(8, 8, Rgba([10, 20, 30, 255]));
        s.pixels.put_pixel(40, 8, Rgba([10, 20, 30, 100]));
        s
    }

    #[test]
    fn roundtrip_keeps_model_and_pixels() {
        // Sin metadatos, esta skin vacía se detectaría como Steve: el modelo viene del bloque.
        let pose = Preset::Waving.pose();
        let card = Card {
            name: "Mi skin".into(),
            background: Background::Custom("cielo.png".into()),
            favorite: true,
            tags: vec!["azul".into(), "pirata".into()],
        };
        let p = load(&save(&sample(), &pose, &card)).unwrap();
        assert_eq!(
            p,
            Project {
                skin: sample(),
                pose,
                card
            }
        );
    }

    #[test]
    fn plain_png_imports() {
        let s = load(&sample().to_png()).unwrap().skin;
        assert_eq!(s.pixels, sample().pixels);
        assert_eq!(s.model, Model::Steve);
    }

    fn with_meta(text: &str) -> Vec<u8> {
        let s = sample();
        let mut out = Vec::new();
        let mut enc = png::Encoder::new(&mut out, 64, 64);
        enc.set_color(png::ColorType::Rgba);
        enc.add_itxt_chunk(KEYWORD.into(), text.into()).unwrap();
        let mut w = enc.write_header().unwrap();
        w.write_image_data(s.pixels.as_raw()).unwrap();
        w.finish().unwrap();
        out
    }

    #[test]
    fn rejects_newer_version() {
        let bytes = with_meta("(version: 99, model: Steve, campo_futuro: 1)");
        assert_eq!(load(&bytes), Err(ProjectError::NewerVersion(99)));
    }

    #[test]
    fn rejects_corrupt_meta() {
        assert_eq!(load(&with_meta("basura")), Err(ProjectError::Corrupt));
    }

    /// Compatibilidad: un proyecto v1 escrito a mano debe seguir abriéndose igual.
    #[test]
    fn reads_v1() {
        let p = load(&with_meta("(version: 1, model: Alex)")).unwrap();
        assert_eq!((p.skin.model, p.pose), (Model::Alex, Pose::default()));
    }

    /// Compatibilidad: v2 con pose parcial (articulaciones faltantes quedan en cero).
    #[test]
    fn reads_v2() {
        let p = load(&with_meta(
            "(version: 2, model: Steve, pose: (right_arm: (-180.0, 0.0, 0.0)))",
        ))
        .unwrap();
        assert_eq!(p.pose, Preset::ArmUp.pose());
        assert_eq!(p.card, Card::default());
    }

    /// v3 con tarjeta parcial: lo que falta queda por defecto.
    #[test]
    fn reads_partial_card() {
        let p = load(&with_meta(
            "(version: 3, model: Steve, card: (favorite: true, background: Preset(4)))",
        ))
        .unwrap();
        assert_eq!(
            p.card,
            Card {
                favorite: true,
                background: Background::Preset(4),
                ..Card::default()
            }
        );
    }
}
