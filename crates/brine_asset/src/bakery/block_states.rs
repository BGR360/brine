use std::collections::HashMap;

use minecraft_assets::{
    api::AssetPack,
    schemas::blockstates::{
        multipart::StateValue as McStateValue, BlockStates as McBlockStates,
        ModelProperties as McModelProperties, Variant as McBlockVariant,
    },
};

use brine_data::{
    blocks::{Block, StateValue},
    MinecraftData,
};

use crate::{
    api::{BlockStateId, Result},
    storage::{BlockState, BlockStateGrabBag, BlockStateModel, BlockStateTable, QuarterRotation},
};

use super::models::ModelBuilder;

pub(crate) struct BlockStateBuilder<'a, 'b> {
    pub(crate) block_state_table: BlockStateTable,
    mc_data: &'a MinecraftData,
    mc_block_states: &'a McBlockStatesTable,
    model_builder: &'a mut ModelBuilder<'b>,
}

impl<'a, 'b> BlockStateBuilder<'a, 'b> {
    pub fn new(
        mc_data: &'a MinecraftData,
        mc_block_states: &'a McBlockStatesTable,
        model_builder: &'a mut ModelBuilder<'b>,
    ) -> Self {
        Self {
            mc_data,
            mc_block_states,
            model_builder,
            block_state_table: Default::default(),
        }
    }

    pub fn build(&mut self) -> Result<()> {
        for block_with_state in self.mc_data.blocks().iter_states() {
            let block_name = block_with_state.name;
            let mc_block_states = self.mc_block_states.0.get(block_name).unwrap();

            self.build_block_state(block_with_state, mc_block_states);
        }

        Ok(())
    }

    fn build_block_state(
        &mut self,
        block: Block<'_>,
        mc_block_states: &McBlockStates,
    ) -> BlockStateId {
        // let states_as_strings = block
        //     .state
        //     .iter()
        //     .map(|(state, value)| [state.to_string(), value.to_string()].join("="))
        //     .collect::<Vec<_>>();
        // trace!(
        //     "Building block {:?}[{}]",
        //     block.name,
        //     states_as_strings.join(",")
        // );

        let state_values: HashMap<&str, McStateValue> = block
            .state
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

        let cases = mc_block_states.clone().into_multipart();

        let variants_to_apply = cases
            .into_iter()
            .filter(|case| case.applies(state_values.iter().map(|(state, value)| (*state, value))))
            .map(|case| case.apply);

        let models = variants_to_apply
            .map(|variant| self.build_grab_bag_from_variant(variant))
            .collect();

        let block_state = BlockState { models };

        self.block_state_table.insert(block_state)
    }

    fn build_grab_bag_from_variant(&mut self, mc_variant: McBlockVariant) -> BlockStateGrabBag {
        let choices = mc_variant
            .models()
            .iter()
            .filter_map(|model_properties| self.build_model_from_properties(model_properties))
            .collect();

        BlockStateGrabBag { choices }
    }

    fn build_model_from_properties(
        &mut self,
        mc_model_properties: &McModelProperties,
    ) -> Option<BlockStateModel> {
        let model_name = &mc_model_properties.model;

        let model_key = self.model_builder.get_or_build_model(model_name)?;

        let rot_x = QuarterRotation::from(mc_model_properties.x as u32);
        let rot_y = QuarterRotation::from(mc_model_properties.y as u32);
        let uv_lock = mc_model_properties.uv_lock;
        let weight = mc_model_properties.weight as u8;

        Some(BlockStateModel {
            model: model_key,
            rot_x,
            rot_y,
            uv_lock,
            weight,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct McBlockStatesTable(pub HashMap<String, McBlockStates>);

pub fn load_block_states(assets: &AssetPack) -> Result<McBlockStatesTable> {
    let mut table = McBlockStatesTable::default();

    assets.for_each_blockstates(|name, path| -> Result<()> {
        let block_states: McBlockStates = assets.load_resource_at_path(path)?;

        table.0.insert(name.as_str().to_string(), block_states);

        Ok(())
    })?;

    Ok(table)
}
