use smallvec::SmallVec;

use crate::{
    api::BlockStateId,
    storage::{ModelKey, QuarterRotation},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub models: SmallVec<[BlockStateGrabBag; 1]>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockStateGrabBag {
    pub choices: SmallVec<[BlockStateModel; 1]>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    pub model: ModelKey,
    pub rot_x: QuarterRotation,
    pub rot_y: QuarterRotation,
    pub uv_lock: bool,
    pub weight: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockStateTable {
    /// Indexed by [`BlockStateId`].
    pub block_states: Vec<BlockState>,
}

impl BlockStateTable {
    pub fn insert(&mut self, block_state: BlockState) -> BlockStateId {
        let index = self.block_states.len();

        self.block_states.push(block_state);

        BlockStateId(index as u16)
    }

    pub fn get_by_key(&self, key: BlockStateId) -> Option<&BlockState> {
        self.block_states.get(key.0 as usize)
    }
}
