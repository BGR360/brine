use std::path::PathBuf;

use minecraft_assets::{
    api::{AssetPack, ModelResolver, ResourceLocation, Result},
    schemas::{
        blockstates::BlockStates,
        models::{BlockFace, Model},
    },
};

use brine_data::{blocks::BlockStateId, MinecraftData};

pub struct Textures {
    data: MinecraftData,
    assets: AssetPack,
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

        let first_variant = match blockstates {
            BlockStates::Variants { ref variants } => variants.values().next().unwrap(),
            BlockStates::Multipart { ref cases } => &cases[0].apply,
        };

        let model_name = &first_variant.models()[0].model;

        // if model_name.contains("water")
        //     || model_name.contains("lava")
        //     || model_name.contains("fire")
        // {
        //     return None;
        // }

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
