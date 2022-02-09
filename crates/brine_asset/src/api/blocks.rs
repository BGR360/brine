use crate::{api::MinecraftAssetsInner, storage::BlockStateTable};

pub struct Blocks<'a> {
    _block_state_table: &'a BlockStateTable,
}

impl<'a> Blocks<'a> {
    pub(crate) fn new(parent: &'a MinecraftAssetsInner) -> Self {
        Self {
            _block_state_table: &parent.block_state_table,
        }
    }
}
