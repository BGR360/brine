use std::collections::HashMap;

use minecraft_assets::api::{AssetPack, ModelIdentifier, ResourceKind, Result};
use tracing::debug;

use crate::api::McModel;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct UnresolvedModelTable(pub HashMap<String, McModel>);

pub fn load_block_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = UnresolvedModelTable::default();

    for loc in assets
        .enumerate_resources("minecraft", ResourceKind::BlockModel)?
        .into_iter()
    {
        debug!("Loading model {:?}", loc.as_str());

        let model_name = ModelIdentifier::model_name(loc.as_str());
        let model = assets.load_block_model(model_name)?;
        table.0.insert(model_name.to_string(), model);
    }

    Ok(table)
}

pub fn _load_item_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = UnresolvedModelTable::default();

    for loc in assets
        .enumerate_resources("minecraft", ResourceKind::ItemModel)?
        .into_iter()
    {
        debug!("Loading model {:?}", loc.as_str());

        let model_name = ModelIdentifier::model_name(loc.as_str());
        let model = assets.load_item_model(model_name)?;
        table.0.insert(model_name.to_string(), model);
    }

    Ok(table)
}
