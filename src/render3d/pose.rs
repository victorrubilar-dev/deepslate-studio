//! Poses: ángulos por articulación, en grados, alrededor de los ejes X, Y y Z.
//! Con X negativo la extremidad va hacia adelante; con Z negativo el brazo derecho se abre hacia afuera.

use crate::core::Part;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pose {
    pub head: [f32; 3],
    pub body: [f32; 3],
    pub right_arm: [f32; 3],
    pub left_arm: [f32; 3],
    pub right_leg: [f32; 3],
    pub left_leg: [f32; 3],
}

/// Límite de los ángulos que se pueden ajustar a mano.
pub const ANGLE_LIMIT: f32 = 180.0;

impl Pose {
    pub fn angles_mut(&mut self, part: Part) -> &mut [f32; 3] {
        match part {
            Part::Head => &mut self.head,
            Part::Body => &mut self.body,
            Part::RightArm => &mut self.right_arm,
            Part::LeftArm => &mut self.left_arm,
            Part::RightLeg => &mut self.right_leg,
            Part::LeftLeg => &mut self.left_leg,
        }
    }

    /// Mezcla lineal de ángulos: `t = 0` es `self`, `t = 1` es `other`.
    pub fn lerp(&self, other: &Pose, t: f32) -> Pose {
        let mix = |a: [f32; 3], b: [f32; 3]| [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t);
        Pose {
            head: mix(self.head, other.head),
            body: mix(self.body, other.body),
            right_arm: mix(self.right_arm, other.right_arm),
            left_arm: mix(self.left_arm, other.left_arm),
            right_leg: mix(self.right_leg, other.right_leg),
            left_leg: mix(self.left_leg, other.left_leg),
        }
    }

    /// La misma pose vista en un espejo: izquierda y derecha se intercambian.
    pub fn mirrored(&self) -> Pose {
        // Reflejar en el plano YZ deja igual el giro en X e invierte Y y Z.
        let m = |[x, y, z]: [f32; 3]| [x, -y, -z];
        Pose {
            head: m(self.head),
            body: m(self.body),
            right_arm: m(self.left_arm),
            left_arm: m(self.right_arm),
            right_leg: m(self.left_leg),
            left_leg: m(self.right_leg),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Standing,
    Walking,
    Waving,
    ArmUp,
    Crouching,
    Running,
    Combat,
    Neutral,
}

pub const PRESETS: [Preset; 8] = [
    Preset::Standing,
    Preset::Walking,
    Preset::Waving,
    Preset::ArmUp,
    Preset::Crouching,
    Preset::Running,
    Preset::Combat,
    Preset::Neutral,
];

impl Preset {
    pub fn pose(self) -> Pose {
        let p = Pose::default();
        match self {
            Preset::Standing => p,
            Preset::Walking => Pose {
                right_arm: [-30.0, 0.0, 0.0],
                left_arm: [30.0, 0.0, 0.0],
                right_leg: [25.0, 0.0, 0.0],
                left_leg: [-25.0, 0.0, 0.0],
                ..p
            },
            Preset::Waving => Pose {
                head: [0.0, -10.0, 0.0],
                right_arm: [0.0, 0.0, -150.0],
                left_arm: [0.0, 0.0, 5.0],
                ..p
            },
            Preset::ArmUp => Pose {
                right_arm: [-180.0, 0.0, 0.0],
                ..p
            },
            Preset::Crouching => Pose {
                head: [-20.0, 0.0, 0.0],
                body: [28.0, 0.0, 0.0],
                right_arm: [-10.0, 0.0, 0.0],
                left_arm: [-10.0, 0.0, 0.0],
                right_leg: [-10.0, 0.0, 0.0],
                left_leg: [-10.0, 0.0, 0.0],
            },
            Preset::Running => Pose {
                head: [-5.0, 0.0, 0.0],
                body: [10.0, 0.0, 0.0],
                right_arm: [-60.0, 0.0, -5.0],
                left_arm: [60.0, 0.0, 5.0],
                right_leg: [50.0, 0.0, 0.0],
                left_leg: [-50.0, 0.0, 0.0],
            },
            Preset::Combat => Pose {
                body: [0.0, 15.0, 0.0],
                right_arm: [-80.0, 0.0, 0.0],
                left_arm: [-35.0, 0.0, 10.0],
                right_leg: [-10.0, 0.0, 0.0],
                left_leg: [10.0, 0.0, 0.0],
                ..p
            },
            Preset::Neutral => Pose {
                head: [5.0, 8.0, 0.0],
                right_arm: [0.0, 0.0, -6.0],
                left_arm: [0.0, 0.0, 6.0],
                right_leg: [0.0, 0.0, -2.0],
                left_leg: [0.0, 0.0, 2.0],
                ..p
            },
        }
    }

    /// Preset que coincide exactamente con la pose, si hay alguno.
    pub fn matching(pose: &Pose) -> Option<Preset> {
        PRESETS.into_iter().find(|p| p.pose() == *pose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_distinct_and_recognized() {
        for a in PRESETS {
            assert_eq!(Preset::matching(&a.pose()), Some(a));
        }
        assert_eq!(Preset::Standing.pose(), Pose::default());
    }

    #[test]
    fn mirror_and_lerp() {
        let walk = Preset::Walking.pose();
        let m = walk.mirrored();
        assert_eq!((m.right_arm, m.left_leg), (walk.left_arm, walk.right_leg));
        let wave = Preset::Waving.pose();
        assert_eq!(wave.mirrored().left_arm, [0.0, 0.0, 150.0]);
        for p in crate::render3d::PRESETS {
            assert_eq!(p.pose().mirrored().mirrored(), p.pose());
        }
        assert_eq!(walk.lerp(&m, 0.0), walk);
        assert_eq!(walk.lerp(&m, 1.0), m);
        // A mitad de camino entre caminar y su espejo, brazos y piernas quedan rectos.
        assert_eq!(walk.lerp(&m, 0.5), Pose::default());
    }
}
