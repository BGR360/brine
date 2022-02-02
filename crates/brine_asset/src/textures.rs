use std::path::PathBuf;

use minecraft_assets::{
    api::{AssetPack, ModelResolver, ResourceLocation, Result},
    schemas::BlockStates,
};

use brine_data::{blocks::BlockStateId, MinecraftData};

pub struct Textures {
    data: MinecraftData,
    assets: AssetPack,
}

impl Textures {
    pub fn get_texture_path(&self, block_state_id: BlockStateId) -> Option<PathBuf> {
        let block = self.data.blocks().get_by_state_id(block_state_id)?;
        let name = &block.name;
        let blockstates = self.assets.load_blockstates(name).ok()?;

        let first_variant = match blockstates {
            BlockStates::Variants { ref variants } => variants.values().next().unwrap(),
            BlockStates::Multipart { ref cases } => &cases[0].apply,
        };

        let model_name = &first_variant.models()[0].model;

        if model_name.contains("water")
            || model_name.contains("lava")
            || model_name.contains("fire")
        {
            return None;
        }

        let models = self.assets.load_block_model_recursive(model_name).ok()?;

        let model = ModelResolver::resolve_model(models.iter());

        let texture = &model.textures.as_ref()?.values().next()?.0;

        let path = self
            .assets
            .get_resource_path(&ResourceLocation::Texture(texture.into()));

        Some(path.strip_prefix("assets").unwrap().into())
    }

    pub(crate) fn build(assets: &AssetPack, data: &MinecraftData) -> Result<Self> {
        Ok(Self {
            assets: assets.clone(),
            data: data.clone(),
        })
    }
}
