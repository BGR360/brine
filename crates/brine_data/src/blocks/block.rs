use std::collections::HashMap;

use minecraft_data_rs::models::block::BoundingBox;
pub use minecraft_data_rs::models::block::{Block as McBlock, State as McState};

use crate::Api;

use super::{state::McBlockExt, BlockState};

pub(crate) type IndexType = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub IndexType);

impl<T> From<T> for BlockId
where
    T: Into<IndexType>,
{
    #[inline]
    fn from(source: T) -> Self {
        Self(source.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockStateId(pub IndexType);

impl<T> From<T> for BlockStateId
where
    T: Into<IndexType>,
{
    #[inline]
    fn from(source: T) -> Self {
        Self(source.into())
    }
}

/// A reference to a block in the [`Blocks`] data provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    pub id: IndexType,
    pub display_name: &'a str,
    pub name: &'a str,
    pub transparent: bool,
    pub empty: bool,
    pub state: BlockState<'a>,
}

/// Provides access to Minecraft block data for a specific version.
///
/// See the [module documentation][self] for more information.
pub struct Blocks {
    /// List of blocks by increasing [`BlockId`].
    ///
    /// Block ids are not contiguous (though they are monotonic). Therefore,
    /// there is no meaningful mapping one can make using an index into this
    /// list.
    blocks: Vec<McBlock>,

    /// Mapping from [`BlockStateId`] to block index.
    ///
    /// Use the `BlockStateId` as an index into this list, and use the
    /// corresponding entry as an index into the `blocks` array.
    pub state_id_to_block: Vec<IndexType>,

    /// Mapping from block name to block index.
    // TODO: faster hashmap?
    pub name_to_block: HashMap<String, IndexType>,
}

impl Blocks {
    /// Returns the number of unique blocks in this version of Minecraft.
    #[inline]
    pub fn count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns the [`Block`] with the given block id in its default state, or
    /// `None` if no such block exists.
    #[inline]
    pub fn get_by_id(&self, block_id: BlockId) -> Option<Block<'_>> {
        // A block's id should always equal its minimum state id.
        self.get_by_state_id(BlockStateId(block_id.0))
    }

    /// Returns the [`Block`] with the given name in its default state, or
    /// `None` if no such block exists.
    #[inline]
    pub fn get_by_name(&self, name: &str) -> Option<Block<'_>> {
        let index = self.name_to_block.get(name)?;

        Some(self.get_by_index_and_state_id(*index, None))
    }

    /// Returns the [`Block`] associated with the given block state id, or
    /// `None` if no such block exists.
    #[inline]
    pub fn get_by_state_id(&self, block_state_id: BlockStateId) -> Option<Block<'_>> {
        let state_id = block_state_id.0;
        let block_index = self.state_id_to_block.get(state_id as usize)?;

        Some(self.get_by_index_and_state_id(*block_index, Some(block_state_id)))
    }

    pub(crate) fn get_by_index_and_state_id(
        &self,
        index: IndexType,
        state_id: Option<BlockStateId>,
    ) -> Block<'_> {
        let mc_block = &self.blocks[index as usize];

        let state_id =
            state_id.unwrap_or_else(|| BlockStateId(mc_block.default_state.unwrap() as IndexType));

        let state_offset = state_id.0 - (mc_block.min_state_id.unwrap() as IndexType);

        let possible_block_states = mc_block.possible_block_states();

        let state = possible_block_states.get_nth(state_offset);

        Block {
            id: mc_block.id as IndexType,
            display_name: &mc_block.display_name,
            name: &mc_block.name,
            transparent: mc_block.transparent,
            empty: matches!(mc_block.bounding_box, BoundingBox::Empty),
            state,
        }
    }

    pub(crate) fn from_api(api: &Api) -> Self {
        let blocks = api.blocks.blocks_array().unwrap();

        let max_state_id = blocks.last().unwrap().max_state_id.unwrap();
        let mut state_id_to_block = vec![0; (max_state_id + 1) as usize];

        let mut name_to_block = HashMap::default();

        for (block_index, block) in blocks.iter().enumerate() {
            let block_index = block_index as IndexType;

            let min_state_id = block.min_state_id.unwrap();
            let max_state_id = block.max_state_id.unwrap();
            for state_id in min_state_id..max_state_id + 1 {
                state_id_to_block[state_id as usize] = block_index;
            }

            let name = block.name.clone();
            name_to_block.insert(name, block_index);
        }

        Self {
            blocks,
            state_id_to_block,
            name_to_block,
        }
    }
}
