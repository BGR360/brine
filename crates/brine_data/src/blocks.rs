//! Minecraft block data.
//!
//! TODO: about block ids and block states.

use std::collections::HashMap;

use crate::{Api, Version};

pub use minecraft_data_rs::models::block::{Block, State as BlockState};

type BlockIndexType = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub BlockIndexType);

impl<T> From<T> for BlockId
where
    T: Into<BlockIndexType>,
{
    #[inline]
    fn from(source: T) -> Self {
        Self(source.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockStateId(pub BlockIndexType);

impl<T> From<T> for BlockStateId
where
    T: Into<BlockIndexType>,
{
    #[inline]
    fn from(source: T) -> Self {
        Self(source.into())
    }
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
    pub blocks: Vec<Block>,

    /// Mapping from [`BlockStateId`] to block index.
    ///
    /// Use the `BlockStateId` as an index into this list, and use the
    /// corresponding entry as an index into the `blocks` array.
    pub state_id_to_block: Vec<BlockIndexType>,

    /// Mapping from block name to block index.
    // TODO: faster hashmap?
    pub name_to_block: HashMap<String, BlockIndexType>,
}

impl Blocks {
    pub(crate) fn from_api(api: &Api) -> Self {
        let blocks = api.blocks.blocks_array().unwrap();

        let max_state_id = blocks.last().unwrap().max_state_id.unwrap();
        let mut state_id_to_block = vec![0; (max_state_id + 1) as usize];

        let mut name_to_block = HashMap::default();

        for (block_index, block) in blocks.iter().enumerate() {
            let block_index = block_index as BlockIndexType;

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

    pub fn for_version(version: impl Into<Version>) -> Self {
        Self::from_api(&Api::new(version.into().0))
    }

    /// Returns the [`Block`] with id `block_id`, or `None` if no such block exists.
    #[inline]
    pub fn get_by_id(&self, block_id: BlockId) -> Option<&Block> {
        // A block's id should always equal its minimum state id.
        self.get_by_state_id(BlockStateId(block_id.0))
    }

    /// Returns the [`Block`] associated with block state `block_state_id`, or
    /// `None` if no such block exists.
    #[inline]
    pub fn get_by_state_id(&self, block_state_id: BlockStateId) -> Option<&Block> {
        let state_id = block_state_id.0;
        let block_index = self.state_id_to_block.get(state_id as usize)?;

        Some(&self.blocks[*block_index as usize])
    }
}
