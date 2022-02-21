use crate::{bakery_v2::QuarterRotation, storage::TextureKey, BlockFace};

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

    pub rotation: QuarterRotation,

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
