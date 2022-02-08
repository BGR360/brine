//! API for accessing Minecraft asset data at runtime.

use std::{path::Path, sync::Arc};

use minecraft_assets::api::AssetPack;

pub use minecraft_assets::{api::Result, schemas::models::BlockFace};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData, Version,
};
use tracing::trace;

use crate::{
    bakery::{self, models::ModelBuilder},
    storage::{CuboidTable, ModelTable, QuadTable, TextureTable},
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
        Models::new(&self.inner)
    }

    pub fn textures(&self) -> &Textures {
        &self.inner.textures
    }
}

#[derive(Debug)]
pub(crate) struct MinecraftAssetsInner {
    pub(crate) blocks: Blocks,
    pub(crate) cuboid_table: CuboidTable,
    pub(crate) model_table: ModelTable,
    pub(crate) quad_table: QuadTable,
    pub(crate) textures: Textures,
    pub(crate) texture_table: TextureTable,
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let texture_table = bakery::textures::build(&assets)?;

        let ModelBuilder {
            model_table,
            cuboid_table,
            quad_table,
            ..
        } = bakery::models::ModelBuilder::build(&assets, data, &texture_table)?;

        let blocks = Blocks::build(&assets, data)?;
        let textures = Textures::build(&assets, data)?;

        trace!("Quad table: {:#?}", &quad_table);
        trace!("Cuboid table: {:#?}", &cuboid_table);
        trace!("Model table: {:#?}", &model_table);

        let new = Self {
            blocks,
            cuboid_table,
            model_table,
            quad_table,
            textures,
            texture_table,
        };

        Ok(new)
    }
}
