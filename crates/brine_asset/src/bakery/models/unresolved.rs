use std::{collections::HashMap, path::Path};

use minecraft_assets::api::{AssetPack, Result};

use crate::api::McModel;

pub type UnresolvedModelTable = HashMap<String, McModel>;

pub fn load_block_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = Default::default();

    assets.for_each_block_model(|path| load_model(assets, path, &mut table))?;

    Ok(table)
}

pub fn _load_item_models(assets: &AssetPack) -> Result<UnresolvedModelTable> {
    let mut table = Default::default();

    assets.for_each_item_model(|path| load_model(assets, path, &mut table))?;

    Ok(table)
}

fn load_model(assets: &AssetPack, path: &Path, table: &mut UnresolvedModelTable) -> Result<()> {
    let model: McModel = assets.load_resource_at_path(path)?;

    let model_name = path.file_stem().unwrap().to_string_lossy().to_string();

    table.insert(model_name, model);

    Ok(())
}
