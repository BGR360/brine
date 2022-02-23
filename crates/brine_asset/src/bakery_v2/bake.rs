use minecraft_assets::api::{AssetPack, Result};

use rayon::prelude::*;
use smallvec::SmallVec;
use tracing::*;

use brine_data::{BlockStateId, MinecraftData};

use crate::bakery_v2::{
    self,
    block_states::{
        BakedBlockState, BakedBlockStateTable, BlockStateGrabBag, BlockStatesBakery,
        HalfBakedBlockState, HalfBakedGrabBagChoice,
    },
    models::{BakedModelTable, ModelBakery},
    textures::TextureTable,
};

#[derive(Debug, Default)]
pub struct BakedAssets {
    pub block_states: BakedBlockStateTable,
    pub models: BakedModelTable,
    pub textures: TextureTable,
}

pub fn bake_all(mc_data: &MinecraftData, asset_pack: &AssetPack) -> Result<BakedAssets> {
    let texture_table = bakery_v2::textures::load_texture_table(asset_pack)?;

    let unbaked_models = bakery_v2::models::load_unbaked_block_models(asset_pack)?;
    let model_bakery = ModelBakery::new(&unbaked_models, &texture_table);

    let unbaked_block_states = bakery_v2::block_states::load_unbaked_block_states(asset_pack)?;
    let block_states_bakery = BlockStatesBakery::new(mc_data, &unbaked_block_states, model_bakery);

    // (Half-)Bake block states in parallel.
    let half_baked_block_states: Vec<(BlockStateId, HalfBakedBlockState)> = unbaked_block_states
        .par_iter()
        .map(|(key, _)| key)
        .flat_map(|block_name| block_states_bakery.bake_block_states_for_block(block_name))
        .collect();

    debug!("Finished half-baking block states");
    // trace!(
    //     "Half-baked block states: {:#?}",
    //     &half_baked_block_states[0..100]
    // );

    let max_block_state_id = half_baked_block_states
        .iter()
        .map(|(block_state_id, _)| *block_state_id)
        .max()
        .unwrap();

    let num_models: usize = half_baked_block_states
        .iter()
        .flat_map(|(_id, block_state)| {
            block_state
                .models
                .iter()
                .map(|grab_bag| grab_bag.choices.len())
        })
        .sum();

    let mut baked_block_states =
        vec![BakedBlockState::default(); max_block_state_id.0 as usize + 1];

    let mut baked_models = BakedModelTable {
        models: Vec::with_capacity(num_models),
    };

    // Turn half-baked block states into fully-baked block states.
    for (block_state_id, half_baked_block_state) in half_baked_block_states.into_iter() {
        trace!("{:?}", block_state_id);
        let baked_grab_bags = half_baked_block_state
            .models
            .into_iter()
            .map(|half_baked_grab_bag| {
                let mut choices = SmallVec::new();

                for HalfBakedGrabBagChoice { model, weight } in
                    half_baked_grab_bag.choices.into_iter()
                {
                    let model_key = baked_models.insert(model);
                    for _ in 0..weight {
                        choices.push(model_key);
                    }
                }

                BlockStateGrabBag { choices }
            })
            .collect();

        let baked_block_state = BakedBlockState {
            models: baked_grab_bags,
        };

        baked_block_states[block_state_id.0 as usize] = baked_block_state;
    }

    debug!("Finished fully baking block states");

    // trace!(
    //     "Fully baked: {:#?}",
    //     baked_block_states
    //         .iter()
    //         .enumerate()
    //         .filter(|(_index, baked_block_state)| !baked_block_state.models.is_empty())
    //         .collect::<Vec<_>>()
    // );

    Ok(BakedAssets {
        block_states: BakedBlockStateTable {
            block_states: baked_block_states,
        },
        models: baked_models,
        textures: texture_table,
    })
}
