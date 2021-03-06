use smallvec::SmallVec;

use brine_data::BlockStateId;

use crate::bakery::models::BakedModelKey;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BakedBlockState {
    pub is_full_cube: bool,
    pub models: SmallVec<[BlockStateGrabBag; 1]>,
}

impl BakedBlockState {
    pub fn get_first_model(&self) -> Option<BakedModelKey> {
        self.models
            .first()
            .map(|grab_bag| *grab_bag.choices.first().unwrap())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockStateGrabBag {
    pub choices: SmallVec<[BakedModelKey; 1]>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BakedBlockStateTable {
    /// Indexed by [`BlockStateId`].
    pub block_states: Vec<BakedBlockState>,
}

impl BakedBlockStateTable {
    pub fn insert(&mut self, block_state: BakedBlockState) -> BlockStateId {
        let index = self.block_states.len();

        self.block_states.push(block_state);

        BlockStateId(index as u16)
    }

    pub fn get_by_key(&self, key: BlockStateId) -> Option<&BakedBlockState> {
        self.block_states.get(key.0 as usize)
    }
}
