use std::{fmt, path::PathBuf};

use minecraft_assets::{
    api::{AssetPack, ModelResolver, ResourceLocation, Result},
    schemas::{
        blockstates::multipart::StateValue as McStateValue,
        models::{BlockFace, Model},
    },
};

use brine_data::{
    blocks::{BlockStateId, StateValue},
    MinecraftData,
};

pub struct Textures {
    data: MinecraftData,
    assets: AssetPack,
}

impl fmt::Debug for Textures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Textures").finish()
    }
}

impl Textures {
    pub fn get_texture_path(
        &self,
        block_state_id: BlockStateId,
        face: BlockFace,
    ) -> Option<PathBuf> {
        let model = self.get_model(block_state_id)?;
        let model_textures = model.textures.as_ref()?;

        let first_element = &model.elements?[0];

        let element_face = first_element
            .faces
            .get(&face)
            .unwrap_or_else(|| first_element.faces.values().next().as_ref().unwrap());

        let texture = &element_face.texture;

        let texture_path = texture
            .resolve(model_textures)
            .or_else(|| texture.location())?;

        let path = self
            .assets
            .get_resource_path(&ResourceLocation::Texture(texture_path.into()));

        Some(path.strip_prefix("assets").unwrap().into())
    }

    fn get_model(&self, block_state_id: BlockStateId) -> Option<Model> {
        let block = self.data.blocks().get_by_state_id(block_state_id)?;

        let name = &block.name;
        let blockstates = self.assets.load_blockstates(name).ok()?;

        let cases = blockstates.into_multipart();

        let state_values: Vec<(&str, McStateValue)> = block
            .state
            .iter()
            .map(|(state, value)| (*state, state_value_to_mc_state_value(value)))
            .collect();

        let first_case_that_applies = cases
            .iter()
            .find(|case| case.applies(state_values.iter().map(|(state, value)| (*state, value))))?;

        let model_name = &first_case_that_applies.apply.models()[0].model;

        let models = self.assets.load_block_model_recursive(model_name).ok()?;

        let model = ModelResolver::resolve_model(models.iter());

        Some(model)
    }

    pub(crate) fn build(assets: &AssetPack, data: &MinecraftData) -> Result<Self> {
        Ok(Self {
            assets: assets.clone(),
            data: data.clone(),
        })
    }
}

pub(crate) fn state_value_to_mc_state_value(state_value: &StateValue) -> McStateValue {
    match state_value {
        StateValue::Bool(b) => McStateValue::from(*b),
        StateValue::Enum(value) => McStateValue::from(*value),
        StateValue::Int(i) => McStateValue::from(i.to_string()),
    }
}
