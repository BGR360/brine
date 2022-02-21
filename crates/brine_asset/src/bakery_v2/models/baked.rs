use smallvec::SmallVec;

use minecraft_assets::schemas::models::BlockFace;

use crate::storage::TextureKey;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BakedQuad {
    pub positions: [[f32; 3]; 4],

    pub normal: [f32; 3],

    /// Tex coords specified as `[u0, v0, u1, v1]`.
    pub tex_coords: [f32; 4],

    pub texture: TextureKey,

    pub cull_face: Option<BlockFace>,

    pub tinted: bool,

    pub shade: bool,
}

impl BakedQuad {
    pub const INDICES: [usize; 6] = [0, 1, 3, 1, 2, 3];
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BakedModel {
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
    baked_models: Vec<BakedModel>,
}

impl BakedModelTable {
    pub fn insert(&mut self, baked_model: BakedModel) -> BakedModelKey {
        let index = self.baked_models.len();

        self.baked_models.push(baked_model);

        BakedModelKey(index)
    }

    pub fn get_by_key(&self, key: BakedModelKey) -> Option<&BakedModel> {
        self.baked_models.get(key.0)
    }
}
