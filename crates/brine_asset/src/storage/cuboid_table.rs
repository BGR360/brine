pub use minecraft_assets::schemas::models::Axis;

use crate::{bakery_v2::models::CuboidRotation, storage::QuadKey};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cuboid {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: CuboidRotation,
    pub shade: bool,
    pub first_face: QuadKey,
    pub last_face: QuadKey,
}

impl Cuboid {
    pub fn quads(&self) -> impl Iterator<Item = QuadKey> + '_ {
        let begin = self.first_face.0;
        let end = self.last_face.0 + 1;

        (begin..end).map(QuadKey)
    }
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
