//! Geometría del personaje, transformada en la CPU. Son unos 430 vértices, así que
//! recalcularlos en cada cuadro es más simple que pasar matrices por parte a la GPU.
//! Espacio del modelo: 1 unidad = 1 píxel, Y hacia arriba, el frente mira a +Z
//! y el lado derecho del personaje está en -X.

use super::pose::Pose;
use crate::core::{Face, LAYERS, Layer, Model, PARTS, Part, rect};

pub type V3 = [f32; 3];
type M3 = [[f32; 3]; 3];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: V3,
    pub uv: [f32; 2],
    pub normal: V3,
    pub part: f32,
}

/// Ángulos de la cámara, en grados, y distancia al centro del personaje.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: -25.0,
            pitch: 22.0,
            distance: 70.0,
        }
    }
}

pub const FOV_DEG: f32 = 35.0;
const CENTER: V3 = [0.0, 16.0, 0.0];

// ---- Álgebra mínima ----

fn mul(a: M3, b: M3) -> M3 {
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn apply(m: M3, v: V3) -> V3 {
    [0, 1, 2].map(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

fn transpose(m: M3) -> M3 {
    [0, 1, 2].map(|i| [m[0][i], m[1][i], m[2][i]])
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Rotación en grados: primero X, luego Y, luego Z.
fn rot(deg: [f32; 3]) -> M3 {
    let [x, y, z] = deg.map(f32::to_radians);
    let rx = [
        [1.0, 0.0, 0.0],
        [0.0, x.cos(), -x.sin()],
        [0.0, x.sin(), x.cos()],
    ];
    let ry = [
        [y.cos(), 0.0, y.sin()],
        [0.0, 1.0, 0.0],
        [-y.sin(), 0.0, y.cos()],
    ];
    let rz = [
        [z.cos(), -z.sin(), 0.0],
        [z.sin(), z.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ];
    mul(rz, mul(ry, rx))
}

/// Transformación rígida: p ↦ r·p + t.
#[derive(Clone, Copy, Debug)]
struct Xf {
    r: M3,
    t: V3,
}

impl Xf {
    fn about(pivot: V3, deg: [f32; 3]) -> Self {
        let r = rot(deg);
        Self {
            r,
            t: sub(pivot, apply(r, pivot)),
        }
    }
    fn then(self, outer: Xf) -> Xf {
        Xf {
            r: mul(outer.r, self.r),
            t: add(apply(outer.r, self.t), outer.t),
        }
    }
    fn point(&self, p: V3) -> V3 {
        add(apply(self.r, p), self.t)
    }
}

// ---- Partes ----

/// Caja (mín, máx) y punto de giro de cada parte.
fn part_box(part: Part, model: Model) -> (V3, V3, V3) {
    let w = if model == Model::Alex { 3.0 } else { 4.0 };
    match part {
        Part::Head => ([-4.0, 24.0, -4.0], [4.0, 32.0, 4.0], [0.0, 24.0, 0.0]),
        Part::Body => ([-4.0, 12.0, -2.0], [4.0, 24.0, 2.0], [0.0, 12.0, 0.0]),
        Part::RightArm => (
            [-4.0 - w, 12.0, -2.0],
            [-4.0, 24.0, 2.0],
            [-4.0 - w / 2.0, 22.0, 0.0],
        ),
        Part::LeftArm => (
            [4.0, 12.0, -2.0],
            [4.0 + w, 24.0, 2.0],
            [4.0 + w / 2.0, 22.0, 0.0],
        ),
        Part::RightLeg => ([-4.0, 0.0, -2.0], [0.0, 12.0, 2.0], [-2.0, 12.0, 0.0]),
        Part::LeftLeg => ([0.0, 0.0, -2.0], [4.0, 12.0, 2.0], [2.0, 12.0, 0.0]),
    }
}

fn angles(pose: &Pose, part: Part) -> [f32; 3] {
    match part {
        Part::Head => pose.head,
        Part::Body => pose.body,
        Part::RightArm => pose.right_arm,
        Part::LeftArm => pose.left_arm,
        Part::RightLeg => pose.right_leg,
        Part::LeftLeg => pose.left_leg,
    }
}

/// Cabeza y brazos cuelgan del torso; las piernas son independientes.
fn part_xf(part: Part, pose: &Pose, model: Model) -> Xf {
    let own = Xf::about(part_box(part, model).2, angles(pose, part));
    match part {
        Part::Head | Part::RightArm | Part::LeftArm => own.then(part_xf(Part::Body, pose, model)),
        _ => own,
    }
}

fn view_xf(cam: &Camera) -> Xf {
    let r = mul(rot([cam.pitch, 0.0, 0.0]), rot([0.0, cam.yaw, 0.0]));
    Xf {
        r,
        t: apply(r, sub([0.0; 3], CENTER)),
    }
}

fn inflation(part: Part, layer: Layer) -> f32 {
    match (layer, part) {
        (Layer::Base, _) => 0.0,
        (Layer::Overlay, Part::Head) => 0.5,
        (Layer::Overlay, _) => 0.25,
    }
}

struct PlacedBox {
    part: Part,
    layer: Layer,
    min: V3,
    max: V3,
    xf: Xf,
}

fn boxes(
    model: Model,
    pose: &Pose,
    cam: &Camera,
    visible: &dyn Fn(Part, Layer) -> bool,
) -> Vec<PlacedBox> {
    let view = view_xf(cam);
    let mut out = Vec::new();
    for part in PARTS {
        let (min, max, _) = part_box(part, model);
        let xf = part_xf(part, pose, model).then(view);
        for layer in LAYERS {
            if visible(part, layer) {
                let g = inflation(part, layer);
                out.push(PlacedBox {
                    part,
                    layer,
                    min: min.map(|v| v - g),
                    max: max.map(|v| v + g),
                    xf,
                });
            }
        }
    }
    out
}

/// Vértices ya en espacio de cámara: (capa base, overlay), como listas de triángulos.
pub fn build(
    model: Model,
    pose: &Pose,
    cam: &Camera,
    visible: &dyn Fn(Part, Layer) -> bool,
) -> (Vec<Vertex>, Vec<Vertex>) {
    let (mut base, mut overlay) = (Vec::new(), Vec::new());
    for b in boxes(model, pose, cam, visible) {
        let out = if b.layer == Layer::Base {
            &mut base
        } else {
            &mut overlay
        };
        let uv = |face| {
            let r = rect(b.part, b.layer, face, model);
            [r.x, r.y, r.x + r.w, r.y + r.h].map(|v| v as f32 / 64.0)
        };
        cuboid(out, b.min, b.max, &b.xf, b.part as usize as f32, uv);
    }
    (base, overlay)
}

/// Índice de parte para la capa de la espalda; nunca coincide con la selección.
const CAPE_PART: f32 = 6.0;

/// Capa de la espalda: caja de 10×16×1 colgada de los hombros, un poco inclinada hacia atrás.
/// Usa la disposición habitual de una textura de 64×32. Es una aproximación.
pub fn cape(pose: &Pose, cam: &Camera) -> Vec<Vertex> {
    let xf = Xf::about([0.0, 24.0, -2.0], [10.0, 0.0, 0.0])
        .then(part_xf(Part::Body, pose, Model::Steve))
        .then(view_xf(cam));
    // La capa se ve desde atrás: su cara «frontal» en la textura es la exterior (-Z).
    let uv = |face| {
        let (x, y, w, h) = match face {
            Face::Top => (1, 0, 10, 1),
            Face::Bottom => (11, 0, 10, 1),
            Face::Left => (0, 1, 1, 16),
            Face::Back => (1, 1, 10, 16),
            Face::Right => (11, 1, 1, 16),
            Face::Front => (12, 1, 10, 16),
        };
        [
            x as f32 / 64.0,
            y as f32 / 32.0,
            (x + w) as f32 / 64.0,
            (y + h) as f32 / 32.0,
        ]
    };
    let mut out = Vec::new();
    cuboid(
        &mut out,
        [-5.0, 8.0, -3.0],
        [5.0, 24.0, -2.0],
        &xf,
        CAPE_PART,
        uv,
    );
    out
}

/// Agrega las 6 caras de una caja. `uv` da (u0, v0, u1, v1) de cada cara.
fn cuboid(
    out: &mut Vec<Vertex>,
    [x0, y0, z0]: V3,
    [x1, y1, z1]: V3,
    xf: &Xf,
    part: f32,
    uv: impl Fn(Face) -> [f32; 4],
) {
    for face in crate::core::FACES {
        // Esquinas vistas desde fuera: arriba-izq, arriba-der, abajo-der, abajo-izq.
        let (corners, normal): ([V3; 4], V3) = match face {
            Face::Front => (
                [[x0, y1, z1], [x1, y1, z1], [x1, y0, z1], [x0, y0, z1]],
                [0.0, 0.0, 1.0],
            ),
            Face::Back => (
                [[x1, y1, z0], [x0, y1, z0], [x0, y0, z0], [x1, y0, z0]],
                [0.0, 0.0, -1.0],
            ),
            Face::Right => (
                [[x0, y1, z0], [x0, y1, z1], [x0, y0, z1], [x0, y0, z0]],
                [-1.0, 0.0, 0.0],
            ),
            Face::Left => (
                [[x1, y1, z1], [x1, y1, z0], [x1, y0, z0], [x1, y0, z1]],
                [1.0, 0.0, 0.0],
            ),
            Face::Top => (
                [[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]],
                [0.0, 1.0, 0.0],
            ),
            // Igual que en el juego: el borde de abajo de la textura toca el frente.
            Face::Bottom => (
                [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
                [0.0, -1.0, 0.0],
            ),
        };
        let [u0, v0, u1, v1] = uv(face);
        let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
        let normal = apply(xf.r, normal);
        for i in [0, 3, 2, 0, 2, 1] {
            out.push(Vertex {
                pos: xf.point(corners[i]),
                uv: uvs[i],
                normal,
                part,
            });
        }
    }
}

/// Posición en pantalla (-1..1, Y hacia arriba) de un punto del modelo, o `None` si queda
/// detrás de la cámara.
pub fn project(cam: &Camera, aspect: f32, p: V3) -> Option<[f32; 2]> {
    let [x, y, z] = view_xf(cam).point(p);
    let depth = cam.distance - z;
    if depth <= 1.0 {
        return None;
    }
    let f = 1.0 / (FOV_DEG.to_radians() / 2.0).tan();
    Some([x * f / aspect / depth, y * f / depth])
}

/// Matriz de proyección por la traslación de la cámara, por columnas, para WGSL.
pub fn projection(aspect: f32, cam: &Camera) -> [[f32; 4]; 4] {
    let f = 1.0 / (FOV_DEG.to_radians() / 2.0).tan();
    let (n, far) = (1.0, 500.0);
    let a = far / (n - far);
    let b = n * far / (n - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, a, -1.0],
        [0.0, 0.0, b - a * cam.distance, cam.distance],
    ]
}

/// Parte bajo el punto (nx, ny) en coordenadas normalizadas (-1..1, Y hacia arriba).
pub fn pick(
    model: Model,
    pose: &Pose,
    cam: &Camera,
    visible: &dyn Fn(Part, Layer) -> bool,
    aspect: f32,
    (nx, ny): (f32, f32),
) -> Option<Part> {
    let h = (FOV_DEG.to_radians() / 2.0).tan();
    let origin = [0.0, 0.0, cam.distance];
    let dir = [nx * h * aspect, ny * h, -1.0];
    let mut best: Option<(f32, Part)> = None;
    for b in boxes(model, pose, cam, visible) {
        let inv = transpose(b.xf.r);
        let o = apply(inv, sub(origin, b.xf.t));
        let d = apply(inv, dir);
        let (mut t0, mut t1) = (f32::NEG_INFINITY, f32::INFINITY);
        for k in 0..3 {
            if d[k].abs() < 1e-9 {
                if o[k] < b.min[k] || o[k] > b.max[k] {
                    t1 = f32::NEG_INFINITY;
                }
                continue;
            }
            let (a, c) = ((b.min[k] - o[k]) / d[k], (b.max[k] - o[k]) / d[k]);
            t0 = t0.max(a.min(c));
            t1 = t1.min(a.max(c));
        }
        if t0 <= t1 && t1 > 0.0 && best.is_none_or(|(t, _)| t0 < t) {
            best = Some((t0, b.part));
        }
    }
    best.map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render3d::pose::Preset;

    fn all(_: Part, _: Layer) -> bool {
        true
    }

    #[test]
    fn vertex_counts() {
        let (base, overlay) = build(Model::Steve, &Pose::default(), &Camera::default(), &all);
        assert_eq!((base.len(), overlay.len()), (6 * 6 * 6, 6 * 6 * 6));
        let (base, overlay) = build(
            Model::Steve,
            &Pose::default(),
            &Camera::default(),
            &|_, l| l == Layer::Base,
        );
        assert_eq!((base.len(), overlay.len()), (216, 0));
    }

    #[test]
    fn head_front_uses_its_texture() {
        let cam = Camera {
            yaw: 0.0,
            pitch: 0.0,
            distance: 70.0,
        };
        let (base, _) = build(Model::Steve, &Pose::default(), &cam, &all);
        // La cara frontal de la cabeza es la que tiene normal +Z y UV dentro de (8,8)-(16,16).
        let front: Vec<_> = base
            .iter()
            .filter(|v| v.part == 0.0 && v.normal[2] > 0.9)
            .collect();
        assert_eq!(front.len(), 6);
        assert!(
            front
                .iter()
                .all(|v| (8.0..=16.0).contains(&(v.uv[0] * 64.0))
                    && (8.0..=16.0).contains(&(v.uv[1] * 64.0)))
        );
        // Arriba a la izquierda (lado derecho del personaje, -X) es la esquina (8,8).
        let tl = front
            .iter()
            .find(|v| v.pos[0] < 0.0 && v.pos[1] > 0.0)
            .unwrap();
        assert_eq!((tl.uv[0] * 64.0, tl.uv[1] * 64.0), (8.0, 8.0));
    }

    #[test]
    fn picking() {
        let cam = Camera {
            yaw: 0.0,
            pitch: 0.0,
            distance: 70.0,
        };
        let pick_at = |p, vis: &dyn Fn(Part, Layer) -> bool| {
            pick(Model::Steve, &Pose::default(), &cam, vis, 1.0, p)
        };
        assert_eq!(pick_at((0.0, 0.0), &all), Some(Part::Body));
        assert_eq!(pick_at((0.0, 0.9), &all), None);
        // La cabeza está arriba: ~12 unidades sobre el centro a 70 de distancia.
        let ny = 12.0 / 66.0 / (FOV_DEG.to_radians() / 2.0).tan();
        assert_eq!(pick_at((0.0, ny), &all), Some(Part::Head));
        // Lo oculto no se puede seleccionar.
        assert_eq!(pick_at((0.0, 0.0), &|p, _| p != Part::Body), None);
    }

    #[test]
    fn arm_up_points_up() {
        let (base, _) = build(
            Model::Steve,
            &Preset::ArmUp.pose(),
            &Camera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 70.0,
            },
            &all,
        );
        let top = base
            .iter()
            .filter(|v| v.part == 2.0)
            .map(|v| v.pos[1])
            .fold(f32::MIN, f32::max);
        assert!(
            top > 12.0,
            "el brazo derecho debería apuntar hacia arriba: {top}"
        );
    }

    #[test]
    fn cape_hangs_behind() {
        let cam = Camera {
            yaw: 0.0,
            pitch: 0.0,
            distance: 70.0,
        };
        let v = cape(&Pose::default(), &cam);
        assert_eq!(v.len(), 36);
        assert!(
            v.iter().all(|v| v.pos[2] < -1.9),
            "la capa debe quedar detrás del torso"
        );
        // El centro del personaje cae en el centro de la pantalla.
        let c = project(&cam, 1.0, CENTER).unwrap();
        assert!(c[0].abs() < 1e-5 && c[1].abs() < 1e-5);
    }
}
