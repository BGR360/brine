#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]

use std::{path::Path, sync::Arc};

use brine_data::MinecraftData;
use minecraft_assets::api::{AssetPack, Result};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    Version,
};

mod hash_slab;

pub mod blocks;
pub mod models;
pub mod textures;

pub use blocks::{BlockFace, Blocks};
pub use hash_slab::HashSlab;
pub use models::Models;
pub use textures::Textures;

use models::ModelTable;

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
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let blocks = Blocks::build(&assets, data)?;
        let models = models::build(&assets, data)?;
        let textures = Textures::build(&assets, data)?;

        Ok(Self {
            blocks,
            models,
            textures,
        })
    }
}
