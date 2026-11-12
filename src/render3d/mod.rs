//! Vista 3D: geometría del personaje, poses y dibujo con wgpu.

mod gpu;
mod mesh;
mod pose;

pub use gpu::{Renderer3d, Scene};
pub use mesh::{Camera, Hit, pick, project};
pub use pose::{ANGLE_LIMIT, PRESETS, Pose, Preset};
