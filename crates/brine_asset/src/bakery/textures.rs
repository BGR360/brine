use minecraft_assets::api::{AssetPack, ResourceKind, Result};

use crate::storage::TextureTable;

pub fn load_texture_ids(assets: &AssetPack) -> Result<TextureTable> {
    let mut table = TextureTable::default();

    for texture_id in assets
        .enumerate_resources("minecraft", ResourceKind::Texture)?
        .into_iter()
    {
        table.insert(texture_id);
    }

    Ok(table)
}
