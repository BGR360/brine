use minecraft_assets::schemas::blockstates::{multipart::Case, Variant};
use smallvec::SmallVec;
use tracing::warn;

use brine_data::{blocks::Block, BlockStateId, MinecraftData};

use crate::{
    bakery_v2::{
        block_states::{BakedBlockState, BlockStateGrabBag, UnbakedBlockStatesTable},
        models::BakedModelKey,
        BakedModel, ModelBakery,
    },
    storage::ModelKey,
};

pub struct BlockStatesBakery<'a, F> {
    mc_data: &'a MinecraftData,
    unbaked_block_states: &'a UnbakedBlockStatesTable,
    model_bakery: ModelBakery<'a>,
    register_baked_model: F,
}

impl<'a, F> BlockStatesBakery<'a, F>
where
    F: 'a + FnMut(BakedModel) -> BakedModelKey,
{
    pub fn new(
        mc_data: &'a MinecraftData,
        unbaked_block_states: &'a UnbakedBlockStatesTable,
        model_bakery: ModelBakery<'a>,
        register_baked_model: F,
    ) -> Self {
        Self {
            mc_data,
            unbaked_block_states,
            model_bakery,
            register_baked_model,
        }
    }

    pub fn bake_block_states_for_block(
        &self,
        block_id: &str,
    ) -> Vec<(BlockStateId, BakedBlockState)> {
        if let Some(block_states) = self.unbaked_block_states.get(block_id) {
            Default::default()
        } else {
            warn!("No blockstates definition found for block {}", block_id);
            Default::default()
        }
    }

    pub fn bake_multipart_for_state(
        &self,
        multipart_cases: &[Case],
        block_with_state: Block<'_>,
    ) -> BlockStateGrabBag {
        Default::default()
    }

    pub fn bake_model_for_block_variant(&self, variant: &Variant) -> BakedModelKey {
        Default::default()
    }
}
