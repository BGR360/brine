use std::path::PathBuf;

use minecraft_assets::api::{AssetPack, Result};

use crate::storage::{Texture, TextureTable};

pub fn build(assets: &AssetPack) -> Result<TextureTable> {
    let mut table = TextureTable::default();

    assets.for_each_texture(|name, path| -> Result<()> {
        let texture = Texture {
            path: PathBuf::from(path),
        };

        table.insert(name, texture);

        Ok(())
    })?;

    Ok(table)
}
