use std::path::Path;

use minecraft_assets::api::{AssetPack, ResourceKind, ResourcePath, Result};

use crate::storage::{Texture, TextureTable};

pub fn load_texture_paths(root_dir: impl AsRef<Path>, assets: &AssetPack) -> Result<TextureTable> {
    let mut table = TextureTable::default();

    for texture_location in assets
        .enumerate_resources("minecraft", ResourceKind::Texture)?
        .into_iter()
    {
        let path = ResourcePath::for_resource(root_dir.as_ref(), &texture_location);
        let texture = Texture {
            path: path.into_inner(),
        };

        table.insert(&texture_location, texture);
    }

    Ok(table)
}
