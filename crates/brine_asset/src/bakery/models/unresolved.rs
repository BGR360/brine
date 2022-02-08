use std::{collections::HashMap, path::Path};

use minecraft_assets::api::{AssetPack, ModelIdentifier, ResourceIdentifier, Result};
use tracing::debug;

use crate::api::McModel;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct UnresolvedModelTable(pub HashMap<String, McModel>);

pub fn load_block_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = Default::default();

    assets.for_each_block_model(|name, path| load_model(assets, name, path, &mut table))?;

    Ok(table)
}

pub fn _load_item_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = Default::default();

    assets.for_each_item_model(|name, path| load_model(assets, name, path, &mut table))?;

    Ok(table)
}

fn load_model(
    assets: &AssetPack,
    name: &ResourceIdentifier,
    path: &Path,
    table: &mut UnresolvedModelTable,
) -> Result<()> {
    debug!("Loading model {:?}", name);

    let model: McModel = assets.load_resource_at_path(path)?;

    let model_id = ModelIdentifier::from(name.into_owned());

    table.0.insert(model_id.model_name().to_string(), model);

    Ok(())
}
