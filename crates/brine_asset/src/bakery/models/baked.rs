use smallvec::SmallVec;

use minecraft_assets::schemas::models::BlockFace;

use crate::bakery::{models::Cuboid, textures::TextureKey};

#[derive(Debug, Clone, PartialEq)]
pub struct BakedQuad {
    pub positions: [[f32; 3]; 4],

    pub normal: [f32; 3],

    pub tex_coords: [[f32; 2]; 4],

    pub texture: TextureKey,

    pub face: BlockFace,

    pub cull_face: Option<BlockFace>,

    pub tinted: bool,

    pub shade: bool,
}

impl BakedQuad {
    #[inline(always)]
    pub fn indices(&self) -> [u8; 6] {
        Cuboid::get_indices(self.face)
    }
}

pub struct BakedCuboid {
    pub is_full_cube: bool,
    pub quads: SmallVec<[BakedQuad; 6]>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BakedModel {
    pub is_full_cube: bool,
    pub quads: SmallVec<[BakedQuad; 6]>,
    /*
    TODO:
        - ambient_occlusion
        - display_transforms
        - gui_light_mode
    */
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BakedModelKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BakedModelTable {
    pub models: Vec<BakedModel>,
}

impl BakedModelTable {
    pub fn insert(&mut self, baked_model: BakedModel) -> BakedModelKey {
        let index = self.models.len();

        self.models.push(baked_model);

        BakedModelKey(index)
    }

    pub fn get_by_key(&self, key: BakedModelKey) -> Option<&BakedModel> {
        self.models.get(key.0)
    }
}
