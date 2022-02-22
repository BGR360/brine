use std::collections::HashMap;

use minecraft_assets::schemas::blockstates::{
    multipart::{Case, StateValue as McStateValue},
    Variant,
};
use tracing::*;

use brine_data::{blocks::StateValue, BlockId, BlockState, BlockStateId, MinecraftData};

use crate::bakery_v2::{
    block_states::{
        half_baked::{HalfBakedBlockStateGrabBag, HalfBakedGrabBagChoice},
        model_cache::BakedModelCache,
        HalfBakedBlockState, UnbakedBlockStatesTable,
    },
    models::ModelBakery,
};

pub struct BlockStatesBakery<'a> {
    mc_data: &'a MinecraftData,
    unbaked_block_states: &'a UnbakedBlockStatesTable,
    model_bakery: ModelBakery<'a>,
}

impl<'a> BlockStatesBakery<'a> {
    pub fn new(
        mc_data: &'a MinecraftData,
        unbaked_block_states: &'a UnbakedBlockStatesTable,
        model_bakery: ModelBakery<'a>,
    ) -> Self {
        Self {
            mc_data,
            unbaked_block_states,
            model_bakery,
        }
    }

    pub fn bake_block_states_for_block(
        &self,
        block_name: &str,
    ) -> Vec<(BlockStateId, HalfBakedBlockState)> {
        debug!("Baking block states for block: {}", block_name);

        self.bake_block_states_for_block_inner(block_name)
            .unwrap_or_default()
    }

    fn bake_block_states_for_block_inner(
        &self,
        block_name: &str,
    ) -> Option<Vec<(BlockStateId, HalfBakedBlockState)>> {
        let mut model_cache = BakedModelCache::new(&self.model_bakery);

        let block_states_definition = self.unbaked_block_states.get(block_name).or_else(|| {
            warn!("No blockstates definition found for block {}", block_name);
            None
        })?;

        let multipart_cases = block_states_definition.clone().into_multipart();

        let block = self.mc_data.blocks().get_by_name(block_name).or_else(|| {
            warn!("No block data for block {}", block_name);
            None
        })?;
        trace!("Block: {:?}", block);

        let block_state_ids: Vec<BlockStateId> = self
            .mc_data
            .blocks()
            .iter_states_for_block(BlockId(block.id))
            .unwrap()
            .map(|(block_state_id, _)| block_state_id)
            .collect();
        trace!("Block state ids: {:?}", block_state_ids);

        Some(
            self.mc_data
                .blocks()
                .iter_states_for_block(BlockId(block.id))
                .unwrap()
                .map(|(block_state_id, block_with_state)| {
                    let block_state = block_with_state.state;
                    let baked =
                        self.bake_block_state(&multipart_cases[..], block_state, &mut model_cache);
                    (block_state_id, baked)
                })
                .collect(),
        )
    }

    pub fn bake_block_state(
        &self,
        multipart_cases: &'a [Case],
        block_state_properties: BlockState<'a>,
        model_cache: &mut BakedModelCache<'_, 'a>,
    ) -> HalfBakedBlockState {
        // Convert to `minecraft_assets` types.
        let block_state_properties: HashMap<&str, McStateValue> = block_state_properties
            .iter()
            .map(|(state, value)| {
                let mc_state_value = match value {
                    StateValue::Bool(b) => McStateValue::Bool(*b),
                    StateValue::Int(i) => McStateValue::String(i.to_string()),
                    StateValue::Enum(value) => McStateValue::String(value.to_string()),
                };

                (*state, mc_state_value)
            })
            .collect();

        let variants_that_apply = multipart_cases
            .iter()
            .filter(|case| {
                case.applies(
                    block_state_properties
                        .iter()
                        .map(|(property, value)| (*property, value)),
                )
            })
            .map(|case| &case.apply);

        let grab_bags = variants_that_apply
            .map(|variant| self.bake_grab_bag_for_block_variant(variant, model_cache))
            .collect();

        HalfBakedBlockState { models: grab_bags }
    }

    pub fn bake_grab_bag_for_block_variant(
        &self,
        variant: &'a Variant,
        model_cache: &mut BakedModelCache<'_, 'a>,
    ) -> HalfBakedBlockStateGrabBag {
        let choices = variant
            .models()
            .iter()
            .filter_map(|model_properties| {
                let baked_model = model_cache.get_or_bake_model(&model_properties.model)?;
                let weight = model_properties.weight;

                Some(HalfBakedGrabBagChoice {
                    model: baked_model.clone(),
                    weight,
                })
            })
            .collect();

        HalfBakedBlockStateGrabBag { choices }
    }
}
