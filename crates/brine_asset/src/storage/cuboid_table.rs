use std::fmt;

pub use minecraft_assets::schemas::models::Axis;
use minecraft_assets::schemas::models::ElementRotation;

use crate::storage::QuadKey;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum DiscreteAngle {
    Neg45,
    Neg22_5,
    Zero,
    Pos22_5,
    Pos45,
}

impl From<f32> for DiscreteAngle {
    fn from(f: f32) -> Self {
        match f {
            f if f == -45.0 => Self::Neg45,
            f if f == -22.5 => Self::Neg22_5,
            f if f == 0.0 => Self::Zero,
            f if f == 22.5 => Self::Pos22_5,
            f if f == 45.0 => Self::Pos45,
            _ => panic!("Invalid model element rotation value: {}", f),
        }
    }
}

impl From<DiscreteAngle> for f32 {
    fn from(a: DiscreteAngle) -> Self {
        match a {
            DiscreteAngle::Neg45 => -45.0,
            DiscreteAngle::Neg22_5 => -22.5,
            DiscreteAngle::Zero => 0.0,
            DiscreteAngle::Pos22_5 => 22.5,
            DiscreteAngle::Pos45 => 45.0,
        }
    }
}

impl Default for DiscreteAngle {
    fn default() -> Self {
        Self::Zero
    }
}

impl fmt::Debug for DiscreteAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", f32::from(*self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CuboidRotation {
    pub origin: [f32; 3],
    pub axis: Axis,
    pub angle: DiscreteAngle,
    pub rescale: bool,
}

impl Default for CuboidRotation {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            axis: Axis::X,
            angle: Default::default(),
            rescale: false,
        }
    }
}

impl From<ElementRotation> for CuboidRotation {
    fn from(
        ElementRotation {
            origin,
            axis,
            angle,
            rescale,
        }: ElementRotation,
    ) -> Self {
        let angle = angle.into();

        Self {
            origin,
            axis,
            angle,
            rescale,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cuboid {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: CuboidRotation,
    pub shade: bool,
    pub first_face: QuadKey,
    pub last_face: QuadKey,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CuboidKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CuboidTable {
    cuboids: Vec<Cuboid>,
}

impl CuboidTable {
    pub fn insert(&mut self, cuboid: Cuboid) -> CuboidKey {
        let key = self.next_key();
        self.cuboids.push(cuboid);

        key
    }

    pub fn get_by_key(&self, key: CuboidKey) -> Option<&Cuboid> {
        self.cuboids.get(key.0)
    }

    pub fn next_key(&self) -> CuboidKey {
        CuboidKey(self.cuboids.len())
    }
}
