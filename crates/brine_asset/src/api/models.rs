pub(crate) use minecraft_assets::schemas::Model as McModel;

use crate::{
    api::{BlockStateId, MinecraftAssetsInner},
    storage::ModelTable,
};

pub struct Models<'a> {
    _model_table: &'a ModelTable,
}

impl<'a> Models<'a> {
    #[inline]
    pub(crate) fn new(parent: &'a MinecraftAssetsInner) -> Self {
        Self {
            _model_table: &parent.model_table,
        }
    }

    pub fn is_cube(&self, _block_state_id: BlockStateId) -> Option<bool> {
        todo!()
    }
}
