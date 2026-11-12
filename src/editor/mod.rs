//! Herramientas y pila de deshacer. La interfaz solo cambia la skin a través de `Editor`.
//! Cada paso de deshacer guarda la skin entera: son 16 KB y evita errores de diferencias.

use crate::core::{Layer, Part, Skin, TRANSPARENT, face_at, rect};
use crate::render3d::Pose;
use image::Rgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Fill,
    Picker,
}

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
        matches!(face_at(self.skin.model, x, y), Some((p, l, _)) if self.is_visible(p, l))
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
            if self.editable(x, y) && *self.skin.pixels.get_pixel(x, y) != color {
                self.skin.pixels.put_pixel(x, y, color);
                self.revision += 1;
            }
        }
    }

    pub fn end_stroke(&mut self) {
        if let Some(before) = self.stroke.take()
            && before != self.skin
        {
            self.push_undo(before);
        }
    }

    /// Rellena el área del mismo color, sin salir de la cara donde se hizo clic.
    pub fn fill(&mut self, x: u32, y: u32, color: Rgba<u8>) {
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
        let before = self.skin.clone();
        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            if !bounds.contains(x, y) || *self.skin.pixels.get_pixel(x, y) != target {
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
        self.revision += 1;
        self.push_undo(before);
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
}
