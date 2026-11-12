//! Herramientas y pila de deshacer. La interfaz solo cambia la skin a través de `Editor`.
//! Cada paso de deshacer guarda la skin entera: son 16 KB y evita errores de diferencias.

pub mod colors;
pub mod image_import;
pub mod templates;

use crate::core::{
    FACES, LAYERS, Layer, PARTS, Part, Rect, Skin, TRANSPARENT, convert_model, face_at,
    mirror_pixel, rect,
};
use crate::render3d::Pose;
use image::{Rgba, RgbaImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tool {
    Pencil,
    Eraser,
    Fill,
    Picker,
    Gradient,
    Stamp,
    Select,
}

/// Pintar en espejo: nada, todo el personaje, o solo cuando se pinta en esa parte o en su pareja.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    Off,
    All,
    Part(Part),
}

/// Qué partes y capas entran al copiar o pegar, indexado por `[parte][capa]`.
pub type PartMask = [[bool; 2]; 6];

pub struct Editor {
    skin: Skin,
    undo: Vec<Skin>,
    redo: Vec<Skin>,
    limit: usize,
    stroke: Option<Skin>,
    revision: u64,
    saved_revision: u64,
    /// Visibilidad por parte y capa; lo oculto tampoco se puede pintar.
    visible: [[bool; 2]; 6],
    /// Se guarda con el proyecto; cambiarla deja cambios sin guardar, pero no entra en deshacer.
    pose: Pose,
    /// Si hay selección, solo se puede pintar dentro de ella.
    selection: Option<Rect>,
    mirror: Mirror,
}

impl Editor {
    pub fn new(skin: Skin, limit: usize) -> Self {
        Self {
            skin,
            undo: Vec::new(),
            redo: Vec::new(),
            limit,
            stroke: None,
            revision: 0,
            saved_revision: 0,
            visible: [[true; 2]; 6],
            pose: Pose::default(),
            selection: None,
            mirror: Mirror::Off,
        }
    }

    pub fn skin(&self) -> &Skin {
        &self.skin
    }

    /// Cambia con cada modificación; sirve para saber cuándo redibujar.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn pose(&self) -> &Pose {
        &self.pose
    }

    pub fn set_pose(&mut self, pose: Pose) {
        if pose != self.pose {
            self.pose = pose;
            self.revision += 1;
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
        self.trim();
    }

    pub fn is_visible(&self, part: Part, layer: Layer) -> bool {
        self.visible[part as usize][layer as usize]
    }

    pub fn set_visible(&mut self, part: Part, layer: Layer, on: bool) {
        self.visible[part as usize][layer as usize] = on;
        self.revision += 1;
        self.saved_revision += 1; // la visibilidad no es un cambio de la skin
    }

    pub fn editable(&self, x: u32, y: u32) -> bool {
        self.selection.is_none_or(|r| r.contains(x, y))
            && matches!(face_at(self.skin.model, x, y), Some((p, l, _)) if self.is_visible(p, l))
    }

    pub fn selection(&self) -> Option<Rect> {
        self.selection
    }

    /// No cambia la skin, así que no entra en deshacer.
    pub fn set_selection(&mut self, r: Option<Rect>) {
        self.selection = r.filter(|r| r.w > 0 && r.h > 0);
    }

    pub fn set_mirror(&mut self, m: Mirror) {
        self.mirror = m;
    }

    /// El píxel simétrico de (x, y) si el espejo está activo para ese punto.
    fn mirrored(&self, x: u32, y: u32) -> Option<(u32, u32)> {
        let model = self.skin.model;
        let applies = match self.mirror {
            Mirror::Off => false,
            Mirror::All => true,
            Mirror::Part(p) => {
                matches!(face_at(model, x, y), Some((q, _, _)) if q == p || q == p.mirrored())
            }
        };
        applies.then(|| mirror_pixel(model, x, y)).flatten()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn begin_stroke(&mut self) {
        self.stroke = Some(self.skin.clone());
    }

    /// Pinta una línea de píxeles dentro del trazo en curso (lápiz o borrador).
    pub fn paint_line(&mut self, from: (u32, u32), to: (u32, u32), color: Rgba<u8>) {
        debug_assert!(self.stroke.is_some(), "paint_line fuera de un trazo");
        for (x, y) in line(from, to) {
            self.put(x, y, color);
            if let Some((mx, my)) = self.mirrored(x, y) {
                self.put(mx, my, color);
            }
        }
    }

    fn put(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        if self.editable(x, y) && *self.skin.pixels.get_pixel(x, y) != color {
            self.skin.pixels.put_pixel(x, y, color);
            self.revision += 1;
        }
    }

    /// Ejecuta `f` y, si cambió la skin, lo registra como un solo paso de deshacer.
    fn step(&mut self, f: impl FnOnce(&mut Self)) {
        let before = self.skin.clone();
        f(self);
        if before != self.skin {
            self.revision += 1;
            self.push_undo(before);
        }
    }

    pub fn end_stroke(&mut self) {
        if let Some(before) = self.stroke.take()
            && before != self.skin
        {
            self.push_undo(before);
        }
    }

    /// Rellena el área del mismo color, sin salir de la cara donde se hizo clic
    /// (ni de la selección, si la hay).
    pub fn fill(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        self.step(|e| {
            e.flood(x, y, color);
            if let Some((mx, my)) = e.mirrored(x, y) {
                e.flood(mx, my, color);
            }
        });
    }

    fn flood(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        if !self.editable(x, y) {
            return;
        }
        let Some((part, layer, face)) = face_at(self.skin.model, x, y) else {
            return;
        };
        let bounds = rect(part, layer, face, self.skin.model);
        let target = *self.skin.pixels.get_pixel(x, y);
        if target == color {
            return;
        }
        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            if !bounds.contains(x, y)
                || !self.editable(x, y)
                || *self.skin.pixels.get_pixel(x, y) != target
            {
                continue;
            }
            self.skin.pixels.put_pixel(x, y, color);
            stack.extend([
                (x + 1, y),
                (x.wrapping_sub(1), y),
                (x, y + 1),
                (x, y.wrapping_sub(1)),
            ]);
        }
    }

    /// Degradado lineal de `c0` (en `from`) a `c1` (en `to`). Cubre la selección si la hay;
    /// si no, la cara donde empezó el arrastre.
    pub fn gradient(&mut self, from: (u32, u32), to: (u32, u32), c0: Rgba<u8>, c1: Rgba<u8>) {
        let area = match self.selection {
            Some(r) => r,
            None => match face_at(self.skin.model, from.0, from.1) {
                Some((p, l, f)) => rect(p, l, f, self.skin.model),
                None => return,
            },
        };
        let (fx, fy) = (from.0 as f32, from.1 as f32);
        let (dx, dy) = (to.0 as f32 - fx, to.1 as f32 - fy);
        let len2 = (dx * dx + dy * dy).max(1.0);
        self.step(|e| {
            for (x, y) in area.pixels() {
                let t = (((x as f32 - fx) * dx + (y as f32 - fy) * dy) / len2).clamp(0.0, 1.0);
                let c = Rgba(std::array::from_fn(|i| {
                    (c0[i] as f32 + (c1[i] as f32 - c0[i] as f32) * t).round() as u8
                }));
                e.put(x, y, c);
            }
        });
    }

    /// Copia los píxeles de la selección (para usarlos con el sello).
    pub fn copy_selection(&self) -> Option<RgbaImage> {
        let r = self.selection?;
        Some(image::imageops::crop_imm(&self.skin.pixels, r.x, r.y, r.w, r.h).to_image())
    }

    /// Estampa `img` con la esquina superior izquierda en `at`. Lo transparente no tapa.
    pub fn stamp(&mut self, at: (u32, u32), img: &RgbaImage) {
        self.step(|e| {
            for (sx, sy, px) in img.enumerate_pixels() {
                let (x, y) = (at.0 + sx, at.1 + sy);
                if px[3] > 0 && x < 64 && y < 64 {
                    e.put(x, y, *px);
                }
            }
        });
    }

    pub fn erase_selection(&mut self) {
        if let Some(r) = self.selection {
            self.step(|e| r.pixels().for_each(|(x, y)| e.put(x, y, ERASE)));
        }
    }

    /// Pega las partes marcadas de `source`. Si el modelo es distinto se convierten los brazos.
    /// Pega aunque la capa esté oculta: es una elección explícita.
    /// Pinta encima los píxeles no transparentes de `layer` (una plantilla), como un solo paso.
    pub fn apply_layer(&mut self, layer: &RgbaImage) {
        self.step(|e| {
            for (x, y, p) in layer.enumerate_pixels() {
                if p[3] > 0 {
                    e.put(x, y, *p);
                }
            }
        });
    }

    pub fn paste_parts(&mut self, source: &Skin, mask: &PartMask) {
        let model = self.skin.model;
        let source = convert_model(source, model);
        self.step(|e| {
            for part in PARTS {
                for layer in LAYERS {
                    if !mask[part as usize][layer as usize] {
                        continue;
                    }
                    for face in FACES {
                        for (x, y) in rect(part, layer, face, model).pixels() {
                            e.skin
                                .pixels
                                .put_pixel(x, y, *source.pixels.get_pixel(x, y));
                        }
                    }
                }
            }
        });
    }

    pub fn pick(&self, x: u32, y: u32) -> Rgba<u8> {
        *self.skin.pixels.get_pixel(x, y)
    }

    /// Reemplaza la skin completa como un solo paso de deshacer (p. ej. Steve ↔ Alex).
    pub fn replace(&mut self, skin: Skin) {
        let before = std::mem::replace(&mut self.skin, skin);
        self.revision += 1;
        self.push_undo(before);
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.skin, prev));
            self.revision += 1;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.skin, next));
            self.revision += 1;
        }
    }

    fn push_undo(&mut self, before: Skin) {
        self.undo.push(before);
        self.redo.clear();
        self.trim();
    }

    fn trim(&mut self) {
        let excess = self.undo.len().saturating_sub(self.limit);
        self.undo.drain(..excess);
    }
}

pub const ERASE: Rgba<u8> = TRANSPARENT;

/// Píxeles de una línea entre dos puntos (Bresenham), para que un arrastre rápido no deje huecos.
fn line((x0, y0): (u32, u32), (x1, y1): (u32, u32)) -> Vec<(u32, u32)> {
    let (mut x, mut y) = (x0 as i32, y0 as i32);
    let (x1, y1) = (x1 as i32, y1 as i32);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    let mut out = vec![(x as u32, y as u32)];
    while (x, y) != (x1, y1) {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        out.push((x as u32, y as u32));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Face, Model};

    const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);

    fn editor() -> Editor {
        Editor::new(Skin::blank(Model::Steve), 100)
    }

    #[test]
    fn stroke_is_one_undo_step() {
        let mut e = editor();
        e.begin_stroke();
        e.paint_line((8, 8), (15, 8), RED);
        e.end_stroke();
        assert!((8..16).all(|x| e.pick(x, 8) == RED));
        assert!(e.is_dirty());
        e.undo();
        assert_eq!(e.skin(), &Skin::blank(Model::Steve));
        e.redo();
        assert_eq!(e.pick(15, 8), RED);
    }

    #[test]
    fn empty_stroke_adds_nothing() {
        let mut e = editor();
        e.begin_stroke();
        e.paint_line((0, 0), (3, 0), RED); // zona sin uso de la textura
        e.end_stroke();
        assert!(!e.can_undo());
        assert_eq!(e.pick(0, 0), TRANSPARENT);
    }

    #[test]
    fn fill_stays_in_face() {
        let mut e = editor();
        e.fill(9, 9, RED);
        let front = rect(Part::Head, Layer::Base, Face::Front, Model::Steve);
        assert!(front.pixels().all(|(x, y)| e.pick(x, y) == RED));
        assert_eq!(e.pick(7, 9), TRANSPARENT); // cara derecha, vecina en la textura
        assert_eq!(e.undo.len(), 1);
    }

    #[test]
    fn hidden_layer_is_locked() {
        let mut e = editor();
        e.set_visible(Part::Head, Layer::Base, false);
        assert!(!e.is_dirty());
        e.fill(9, 9, RED);
        e.begin_stroke();
        e.paint_line((9, 9), (10, 9), RED);
        e.end_stroke();
        assert_eq!(e.pick(9, 9), TRANSPARENT);
        assert!(e.editable(41, 9)); // el overlay sigue visible
    }

    #[test]
    fn new_action_clears_redo_and_limit_applies() {
        let mut e = Editor::new(Skin::blank(Model::Steve), 2);
        for c in 1..=3u8 {
            e.fill(9, 9, Rgba([c, 0, 0, 255]));
        }
        assert_eq!(e.undo.len(), 2);
        e.undo();
        assert!(e.can_redo());
        e.fill(9, 9, RED);
        assert!(!e.can_redo());
    }

    #[test]
    fn line_has_no_gaps() {
        let pts = line((0, 0), (5, 2));
        assert_eq!(pts.first(), Some(&(0, 0)));
        assert_eq!(pts.last(), Some(&(5, 2)));
        assert!(
            pts.windows(2)
                .all(|w| w[0].0.abs_diff(w[1].0) <= 1 && w[0].1.abs_diff(w[1].1) <= 1)
        );
    }

    #[test]
    fn mirror_paints_both_sides() {
        let mut e = editor();
        e.set_mirror(Mirror::All);
        e.begin_stroke();
        e.paint_line((8, 9), (8, 9), RED);
        e.end_stroke();
        assert_eq!(e.pick(15, 9), RED);
        // Solo la parte elegida: pintar en la cabeza ya no se refleja.
        e.set_mirror(Mirror::Part(Part::RightArm));
        e.begin_stroke();
        e.paint_line((9, 12), (9, 12), RED);
        e.end_stroke();
        assert_eq!(e.pick(14, 12), TRANSPARENT);
        e.undo();
        e.undo();
        assert_eq!(e.skin(), &Skin::blank(Model::Steve));
    }

    #[test]
    fn selection_limits_painting() {
        let mut e = editor();
        e.set_selection(Some(Rect {
            x: 8,
            y: 8,
            w: 2,
            h: 2,
        }));
        e.fill(8, 8, RED);
        assert_eq!(e.pick(9, 9), RED);
        assert_eq!(e.pick(10, 8), TRANSPARENT);
        e.erase_selection();
        assert_eq!(e.pick(8, 8), TRANSPARENT);
        assert_eq!(e.undo.len(), 2);
    }

    #[test]
    fn gradient_goes_from_one_color_to_other() {
        let mut e = editor();
        let blue = Rgba([0, 0, 255, 255]);
        e.gradient((8, 8), (15, 8), RED, blue);
        assert_eq!(e.pick(8, 12), RED);
        assert_eq!(e.pick(15, 12), blue);
        let mid = e.pick(11, 12);
        assert!(mid[0] > 0 && mid[2] > 0);
        assert_eq!(e.pick(7, 12), TRANSPARENT); // fuera de la cara
        assert_eq!(e.undo.len(), 1);
    }

    #[test]
    fn copy_and_stamp() {
        let mut e = editor();
        e.fill(9, 9, RED);
        e.set_selection(Some(Rect {
            x: 8,
            y: 8,
            w: 2,
            h: 1,
        }));
        let img = e.copy_selection().unwrap();
        e.set_selection(None);
        e.stamp((40, 8), &img);
        assert_eq!(
            (e.pick(40, 8), e.pick(41, 8), e.pick(42, 8)),
            (RED, RED, TRANSPARENT)
        );
    }

    #[test]
    fn paste_parts_only_marked() {
        let mut src = editor();
        src.fill(9, 9, RED); // cabeza
        src.fill(21, 21, RED); // torso
        let mut mask = [[false; 2]; 6];
        mask[Part::Head as usize][Layer::Base as usize] = true;
        let mut e = Editor::new(Skin::blank(Model::Alex), 100);
        e.paste_parts(src.skin(), &mask);
        assert_eq!(e.pick(9, 9), RED);
        assert_eq!(e.pick(21, 21), TRANSPARENT);
        assert_eq!(e.skin().model, Model::Alex);
    }

    #[test]
    fn apply_layer_is_one_step_and_keeps_transparent() {
        let mut skin = Skin::blank(Model::Steve);
        skin.pixels.put_pixel(8, 8, Rgba([1, 2, 3, 255]));
        let mut e = Editor::new(skin.clone(), 10);
        let mut layer = RgbaImage::new(64, 64);
        layer.put_pixel(20, 20, Rgba([9, 9, 9, 255]));
        e.apply_layer(&layer);
        assert_eq!(*e.skin().pixels.get_pixel(20, 20), Rgba([9, 9, 9, 255]));
        assert_eq!(*e.skin().pixels.get_pixel(8, 8), Rgba([1, 2, 3, 255]));
        assert!(e.can_undo());
        e.undo();
        assert_eq!(*e.skin(), skin);
        assert!(!e.can_undo());
    }
}
