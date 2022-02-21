use std::collections::HashMap;

use minecraft_assets::api::{AssetPack, ResourceIdentifier, ResourceKind, Result};

pub type UnbakedQuad = minecraft_assets::schemas::models::ElementFace;

pub type UnbakedCuboid = minecraft_assets::schemas::models::Element;

pub type UnbakedModel = minecraft_assets::schemas::models::Model;

pub type UnbakedModels = HashMap<ResourceIdentifier<'static>, UnbakedModel>;

pub fn load_unbaked_block_models(mc_assets: &AssetPack) -> Result<UnbakedModels> {
    let model_ids = mc_assets.enumerate_resources("minecraft", ResourceKind::BlockModel)?;

    let unbaked_models = model_ids
        .into_iter()
        .map(|model_id| {
            let model = mc_assets.load_block_model(model_id.as_str())?;
            Ok((model_id, model))
        })
        .collect::<Result<_>>()?;

    Ok(unbaked_models)
}
