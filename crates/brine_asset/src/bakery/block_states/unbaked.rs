use std::collections::HashMap;

use minecraft_assets::api::{AssetPack, ResourceKind, Result};

pub type UnbakedBlockStates = minecraft_assets::schemas::blockstates::BlockStates;

pub type UnbakedBlockStatesTable = HashMap<String, UnbakedBlockStates>;

pub fn load_unbaked_block_states(mc_assets: &AssetPack) -> Result<UnbakedBlockStatesTable> {
    let block_ids = mc_assets.enumerate_resources("minecraft", ResourceKind::BlockStates)?;

    let unbaked_block_states = block_ids
        .into_iter()
        .map(|block_id| {
            let model = mc_assets.load_blockstates(block_id.as_str())?;
            Ok((block_id.as_str().to_string(), model))
        })
        .collect::<Result<_>>()?;

    Ok(unbaked_block_states)
}
