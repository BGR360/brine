use std::fmt;

use minecraft_assets::{
    api::{AssetPack, Result},
    schemas::BlockStates,
};

use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData,
};

pub struct Blocks {
    data: MinecraftData,
    block_states: Vec<BlockStates>,
}

impl fmt::Debug for Blocks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Blocks")
            .field("block_states", &self.block_states)
            .finish()
    }
}

impl Blocks {
    pub fn get_block_states_by_id(&self, block_id: BlockId) -> Option<&BlockStates> {
        self.block_states.get(block_id.0 as usize)
    }

    pub fn get_block_states_by_state_id(
        &self,
        block_state_id: BlockStateId,
    ) -> Option<&BlockStates> {
        let index = self
            .data
            .blocks()
            .state_id_to_block
            .get(block_state_id.0 as usize)?;

        self.block_states.get(*index as usize)
    }

    pub(crate) fn build(assets: &AssetPack, data: &MinecraftData) -> Result<Self> {
        let num_blocks = data.blocks().count();
        let mut block_states: Vec<BlockStates> = vec![Default::default(); num_blocks];

        assets.for_each_blockstates(|block_name, path| -> Result<_> {
            //println!("Loading `{}` at {}", block_name, path.to_string_lossy());

            if let Some(block_index) = data.blocks().name_to_block.get(block_name.as_str()) {
                let blockstates: BlockStates = assets.load_resource_at_path(path)?;

                block_states[*block_index as usize] = blockstates;
            } else {
                // TODO: check items
                //println!("**** NOT FOUND: {} ****", block_name);
            }

            Ok(())
        })?;

        Ok(Self {
            data: data.clone(),
            block_states,
        })
    }
}
