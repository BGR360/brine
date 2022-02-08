//! API for accessing Minecraft asset data at runtime.

use std::{path::Path, sync::Arc};

use minecraft_assets::api::AssetPack;

pub use minecraft_assets::{api::Result, schemas::models::BlockFace};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData, Version,
};

use crate::{
    bakery,
    storage::{ModelTable, TextureTable},
};

mod blocks;
mod models;
mod textures;

pub(crate) use models::McModel;

pub use blocks::Blocks;
pub use models::Models;
pub use textures::Textures;

/// Provides access to Minecraft assets for a given assets directory.
///
/// This type is intended to be initialized once at program startup and accessed
/// by reference thereafter. Construction is **not** an inexpensive operation,
/// but access **is** an inexpensive operation.
#[derive(Clone)]
pub struct MinecraftAssets {
    inner: Arc<MinecraftAssetsInner>,
}

impl MinecraftAssets {
    pub fn new(path: impl AsRef<Path>, data: &MinecraftData) -> Result<Self> {
        let inner = MinecraftAssetsInner::build(path.as_ref(), data)?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn blocks(&self) -> &Blocks {
        &self.inner.blocks
    }

    pub fn models(&self) -> Models<'_> {
        Models::new(&self.inner.models)
    }

    pub fn textures(&self) -> &Textures {
        &self.inner.textures
    }
}

struct MinecraftAssetsInner {
    blocks: Blocks,
    models: ModelTable,
    textures: Textures,
    _texture_table: TextureTable,
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let blocks = Blocks::build(&assets, data)?;
        let models = bakery::models::build(&assets, data)?;
        let textures = Textures::build(&assets, data)?;
        let texture_table = bakery::textures::build(&assets)?;

        Ok(Self {
            blocks,
            models,
            textures,
            _texture_table: texture_table,
        })
    }
}
