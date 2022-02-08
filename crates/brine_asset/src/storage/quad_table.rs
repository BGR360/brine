use std::fmt;

use crate::{storage::TextureKey, BlockFace};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QuadRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Default for QuadRotation {
    fn default() -> Self {
        Self::Deg0
    }
}

impl From<u32> for QuadRotation {
    fn from(deg: u32) -> Self {
        match deg {
            0 => Self::Deg0,
            90 => Self::Deg90,
            180 => Self::Deg180,
            270 => Self::Deg270,
            _ => panic!("Invalid quad rotation: {}", deg),
        }
    }
}

impl fmt::Debug for QuadRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deg0 => write!(f, "0"),
            Self::Deg90 => write!(f, "90"),
            Self::Deg180 => write!(f, "180"),
            Self::Deg270 => write!(f, "270"),
        }
    }
}

/// A single quad (face) of a cuboid model element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// Which face of the cuboid this quad belongs to.
    pub face: BlockFace,

    /// The texture that this quad should be rendered with.
    pub texture: TextureKey,

    /// The texture coordinates of the four corners of the quad.
    ///
    /// Specified as [u0, v0, u1, v1], each in the range [0.0, 16.0].
    pub uv: [f32; 4],

    pub cull_face: Option<BlockFace>,

    pub rotation: QuadRotation,

    pub tint_index: Option<u8>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QuadTable {
    quads: Vec<Quad>,
}

impl QuadTable {
    pub fn insert(&mut self, quad: Quad) -> QuadKey {
        let key = self.next_key();
        self.quads.push(quad);

        key
    }

    pub fn get_by_key(&self, key: QuadKey) -> Option<&Quad> {
        self.quads.get(key.0)
    }

    pub fn next_key(&self) -> QuadKey {
        QuadKey(self.quads.len())
    }
}
