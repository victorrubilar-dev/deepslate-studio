//! Poses: ángulos por articulación, en grados, alrededor de los ejes X, Y y Z.
//! Con X negativo la extremidad va hacia adelante; con Z negativo el brazo derecho se abre hacia afuera.

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
}
