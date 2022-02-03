use minecraft_assets::{
    api::{AssetPack, Result},
    schemas::BlockStates,
};

use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData,
};

pub use minecraft_assets::schemas::models::BlockFace;

pub struct Blocks {
    data: MinecraftData,
    block_states: Vec<BlockStates>,
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
        let num_blocks = data.blocks().blocks.len();
        let mut block_states: Vec<BlockStates> = vec![Default::default(); num_blocks];

        assets.for_each_blockstates(|path| -> Result<_> {
            let block_name = path.file_stem().unwrap().to_str().unwrap();

            println!("Loading `{}` at {}", block_name, path.to_string_lossy());

            if let Some(block_index) = data.blocks().name_to_block.get(block_name) {
                let blockstates: BlockStates = assets.load_resource_at_path(path)?;

                block_states[*block_index as usize] = blockstates;
            } else {
                // TODO: check items
                println!("**** NOT FOUND: {} ****", block_name);
            }

            Ok(())
        })?;

        Ok(Self {
            data: data.clone(),
            block_states,
        })
    }
}
