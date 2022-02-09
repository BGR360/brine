//! API for accessing Minecraft asset data at runtime.

use std::{path::Path, sync::Arc};

use minecraft_assets::api::AssetPack;

pub use minecraft_assets::{api::Result, schemas::models::BlockFace};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData, Version,
};

use crate::{
    bakery::{self, block_states::BlockStateBuilder, models::ModelBuilder},
    storage::{BlockStateTable, CuboidTable, ModelTable, QuadTable, TextureTable},
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

    pub fn blocks(&self) -> Blocks<'_> {
        Blocks::new(&self.inner)
    }

    pub fn models(&self) -> Models<'_> {
        Models::new(&self.inner)
    }

    pub fn textures(&self) -> Textures<'_> {
        Textures::new(&self.inner)
    }
}

#[derive(Debug)]
pub(crate) struct MinecraftAssetsInner {
    pub(crate) block_state_table: BlockStateTable,
    pub(crate) cuboid_table: CuboidTable,
    pub(crate) model_table: ModelTable,
    pub(crate) quad_table: QuadTable,
    pub(crate) texture_table: TextureTable,
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let texture_table = bakery::textures::load_texture_paths(&assets)?;

        let mc_models = {
            let unresolved_models = bakery::models::unresolved::load_block_models(&assets)?;

            bakery::models::resolved::resolve_models(&unresolved_models)
        };

        let mc_block_states = bakery::block_states::load_block_states(&assets)?;

        let mut model_builder = ModelBuilder::new(&mc_models, &texture_table);

        let mut block_state_builder =
            BlockStateBuilder::new(data, &mc_block_states, &mut model_builder);

        block_state_builder.build()?;

        let BlockStateBuilder {
            block_state_table, ..
        } = block_state_builder;

        let ModelBuilder {
            model_table,
            cuboid_table,
            quad_table,
            ..
        } = model_builder;

        let new = Self {
            block_state_table,
            cuboid_table,
            model_table,
            quad_table,
            texture_table,
        };

        Ok(new)
    }
}
