//! Biblioteca: tarjetas de las skins guardadas en una carpeta. Cada PNG de la carpeta es una
//! tarjeta; sus datos (nombre, fondo, favorito, etiquetas) van dentro del propio PNG.
//! Las subcarpetas de primer nivel son colecciones.

pub mod backgrounds;

use crate::project::{self, Card, Project};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct Entry {
    pub path: PathBuf,
    pub project: Project,
    pub modified: Option<SystemTime>,
    /// Huella del contenido del archivo, para la caché de miniaturas.
    pub key: u64,
    /// Subcarpeta donde está, o `None` si está en la raíz de la biblioteca.
    pub collection: Option<String>,
}

/// Orden de las tarjetas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    /// Favoritas primero y luego las editadas más recientemente.
    Favorites,
    Recent,
    Name,
}

impl Entry {
    /// Nombre de la tarjeta, o el del archivo si no tiene.
    pub fn name(&self) -> String {
        let card = &self.project.card.name;
        if card.trim().is_empty() {
            stem(&self.path)
        } else {
            card.clone()
        }
    }

    /// Cada palabra de la búsqueda tiene que estar en el nombre o en alguna etiqueta.
    pub fn matches(&self, query: &str) -> bool {
        let name = self.name().to_lowercase();
        let tags: Vec<String> = self
            .project
            .card
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .collect();
        query
            .to_lowercase()
            .split_whitespace()
            .all(|w| name.contains(w) || tags.iter().any(|t| t.contains(w)))
    }
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn is_png(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
}

/// Nombres de las colecciones (subcarpetas de primer nivel), en orden alfabético.
pub fn collections(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !n.starts_with('.'))
        .collect();
    out.sort_by_key(|n| n.to_lowercase());
    out
}

/// Nombre de colección válido como nombre de carpeta, o `None`.
pub fn collection_name(name: &str) -> Option<String> {
    let name = name.trim();
    let bad = name.is_empty()
        || name.starts_with('.')
        || name.chars().any(|c| {
            matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()
        });
    (!bad).then(|| name.to_string())
}

/// Mueve un archivo de la biblioteca a una colección (o a la raíz con `None`). Devuelve la
/// ruta nueva; si ya hay uno con ese nombre, usa otro libre.
pub fn move_to(path: &Path, dir: &Path, collection: Option<&str>) -> std::io::Result<PathBuf> {
    let target_dir = collection.map_or_else(|| dir.to_path_buf(), |c| dir.join(c));
    if path.parent() == Some(target_dir.as_path()) {
        return Ok(path.to_path_buf());
    }
    std::fs::create_dir_all(&target_dir)?;
    let target = free_path(&target_dir, &stem(path));
    std::fs::rename(path, &target)?;
    Ok(target)
}

/// Lee todas las skins de la carpeta y de sus colecciones, favoritas primero y luego las
/// editadas más recientemente. Los archivos que no son skins válidas se saltan.
pub fn scan(dir: &Path) -> Vec<Entry> {
    let mut out = scan_one(dir, None);
    for c in collections(dir) {
        out.extend(scan_one(&dir.join(&c), Some(c)));
    }
    sort(&mut out, Sort::Favorites);
    out
}

pub fn sort(entries: &mut [Entry], by: Sort) {
    match by {
        Sort::Favorites => entries.sort_by(|a, b| {
            (b.project.card.favorite, b.modified).cmp(&(a.project.card.favorite, a.modified))
        }),
        Sort::Recent => entries.sort_by_key(|e| std::cmp::Reverse(e.modified)),
        Sort::Name => entries.sort_by_cached_key(|e| e.name().to_lowercase()),
    }
}

fn scan_one(dir: &Path, collection: Option<String>) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    read.flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_png(p))
        .filter_map(|path| {
            let bytes = std::fs::read(&path).ok()?;
            let project = project::load(&bytes)
                .inspect_err(|e| log::warn!("biblioteca: se salta {}: {e:?}", path.display()))
                .ok()?;
            let mut h = std::collections::hash_map::DefaultHasher::new();
            bytes.hash(&mut h);
            Some(Entry {
                modified: std::fs::metadata(&path).and_then(|m| m.modified()).ok(),
                key: h.finish(),
                path,
                project,
                collection: collection.clone(),
            })
        })
        .collect()
}

/// Cambia los datos de tarjeta de un archivo sin tocar sus píxeles ni su pose.
pub fn update_card(path: &Path, f: impl FnOnce(&mut Card)) -> Result<Card, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut p = project::load(&bytes).map_err(|e| format!("{e:?}"))?;
    f(&mut p.card);
    std::fs::write(path, project::save(&p.skin, &p.pose, &p.card)).map_err(|e| e.to_string())?;
    Ok(p.card)
}

/// Colección donde nacen las skins nuevas. El nombre en disco es fijo; la interfaz lo traduce.
pub const DRAFTS: &str = "drafts";

/// `true` si `path` es un borrador de la biblioteca `dir`.
pub fn is_draft(dir: &Path, path: &Path) -> bool {
    path.parent() == Some(dir.join(DRAFTS).as_path())
}

/// `dir/nombre.png`, o `dir/nombre-2.png`, `-3`… si ya existe.
pub fn free_path(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(format!("{name}.png"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{name}-{n}.png")))
        .find(|p| !p.exists())
        .expect("hay infinitos nombres")
}

/// Fecha (UTC) en formato AAAA-MM-DD.
pub fn date(t: SystemTime) -> String {
    let secs = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Días desde 1970-01-01 a fecha civil (algoritmo de Howard Hinnant).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Model, Skin};
    use crate::project::Background;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("deepslate-lib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write(dir: &Path, file: &str, card: Card) {
        let skin = Skin::blank(Model::Steve);
        std::fs::write(
            dir.join(file),
            project::save(&skin, &Default::default(), &card),
        )
        .unwrap();
    }

    #[test]
    fn dates() {
        assert_eq!(date(SystemTime::UNIX_EPOCH), "1970-01-01");
        let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_709_164_800);
        assert_eq!(date(t), "2024-02-29");
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn scan_sorts_skips_and_searches() {
        let dir = temp_dir("scan");
        write(&dir, "a.png", Card::default());
        write(
            &dir,
            "b.png",
            Card {
                name: "Pirata Azul".into(),
                favorite: true,
                tags: vec!["Mar".into()],
                ..Card::default()
            },
        );
        std::fs::write(dir.join("roto.png"), b"no es png").unwrap();
        std::fs::write(dir.join("nota.txt"), b"x").unwrap();
        let entries = scan(&dir);
        let names: Vec<String> = entries.iter().map(Entry::name).collect();
        assert_eq!(names, ["Pirata Azul", "a"]);
        let b = &entries[0];
        assert!(b.matches("pirata"));
        assert!(b.matches("azul mar"));
        assert!(!b.matches("pirata rojo"));
        assert!(b.matches(""));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn update_card_keeps_pixels() {
        let dir = temp_dir("update");
        write(&dir, "a.png", Card::default());
        let path = dir.join("a.png");
        let before = project::load(&std::fs::read(&path).unwrap()).unwrap();
        let card = update_card(&path, |c| {
            c.favorite = true;
            c.background = Background::Preset(3);
        })
        .unwrap();
        let after = project::load(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(after.card, card);
        assert!(card.favorite);
        assert_eq!(after.skin, before.skin);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn free_paths() {
        let dir = temp_dir("free");
        assert_eq!(free_path(&dir, "x"), dir.join("x.png"));
        write(&dir, "x.png", Card::default());
        write(&dir, "x-2.png", Card::default());
        assert_eq!(free_path(&dir, "x"), dir.join("x-3.png"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn collections_scan_move_and_sort() {
        let dir = temp_dir("collections");
        write(&dir, "zeta.png", Card::default());
        std::fs::create_dir(dir.join("Piratas")).unwrap();
        write(&dir.join("Piratas"), "alfa.png", Card::default());
        std::fs::create_dir(dir.join(".oculta")).unwrap();
        assert_eq!(collections(&dir), ["Piratas"]);
        let mut entries = scan(&dir);
        sort(&mut entries, Sort::Name);
        let got: Vec<_> = entries
            .iter()
            .map(|e| (e.name(), e.collection.clone()))
            .collect();
        assert_eq!(
            got,
            [
                ("alfa".into(), Some("Piratas".into())),
                ("zeta".into(), None)
            ]
        );
        // Mover a la colección; si el nombre está ocupado, usa otro.
        write(&dir.join("Piratas"), "zeta.png", Card::default());
        let moved = move_to(&dir.join("zeta.png"), &dir, Some("Piratas")).unwrap();
        assert_eq!(moved, dir.join("Piratas").join("zeta-2.png"));
        assert!(!dir.join("zeta.png").exists());
        let back = move_to(&moved, &dir, None).unwrap();
        assert_eq!(back, dir.join("zeta-2.png"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn drafts_are_only_direct_children() {
        let dir = Path::new("biblioteca");
        assert!(is_draft(dir, &dir.join(DRAFTS).join("a.png")));
        assert!(!is_draft(dir, &dir.join("a.png")));
        assert!(!is_draft(dir, &dir.join("Piratas").join("a.png")));
        assert!(!is_draft(dir, &dir.join(DRAFTS).join("x").join("a.png")));
    }

    #[test]
    fn collection_names_are_safe() {
        assert_eq!(collection_name("  Piratas "), Some("Piratas".into()));
        for bad in ["", "  ", "..", ".x", "a/b", "a\\b", "c:d"] {
            assert_eq!(collection_name(bad), None, "{bad}");
        }
    }
}
