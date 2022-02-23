//! API for accessing Minecraft asset data at runtime.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use minecraft_assets::api::{AssetPack, ResourcePath};
use tracing::*;

pub use minecraft_assets::{api::Result, schemas::models::BlockFace};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData, Version,
};

use crate::bakery::{
    self,
    block_states::BakedBlockStateTable,
    models::BakedModelTable,
    textures::{TextureKey, TextureTable},
    BakedAssets,
};

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

    #[inline]
    pub fn root(&self) -> &Path {
        &self.inner.root
    }

    #[inline]
    pub fn block_states(&self) -> &BakedBlockStateTable {
        &self.inner.block_state_table
    }

    #[inline]
    pub fn models(&self) -> &BakedModelTable {
        &self.inner.model_table
    }

    #[inline]
    pub fn textures(&self) -> &TextureTable {
        &self.inner.texture_table
    }

    #[inline]
    pub fn get_texture_path(&self, texture_key: TextureKey) -> Option<PathBuf> {
        let texture_id = self.textures().get_by_key(texture_key)?;

        let texture_path = ResourcePath::for_resource(&self.root(), texture_id);

        Some(texture_path.strip_prefix("assets").unwrap().into())
    }

    // TODO: deprecate
    pub fn get_texture_path_for_block_state_and_face(
        &self,
        block_state_id: BlockStateId,
        face: BlockFace,
    ) -> Option<PathBuf> {
        trace!("Querying texture for {:?}:{:?}", block_state_id, face);

        let baked_block_state = self.block_states().get_by_key(block_state_id).or_else(|| {
            warn!("No block for {:?}", block_state_id);
            None
        })?;

        if baked_block_state.models.len() > 1 {
            debug!(
                "{:?} is composed of multiple models, using the first one",
                block_state_id
            );
        }

        // TODO: pick random model from grab bag.
        let model_key = baked_block_state.get_first_model().or_else(|| {
            warn!("{:?} has no models!", block_state_id);
            None
        })?;

        let model = self.models().get_by_key(model_key).or_else(|| {
            warn!("No model with key {:?}", model_key);
            None
        })?;

        let quad = model.quads.iter().find(|quad| {
            quad.cull_face
                .map(|cull_face| cull_face == face)
                .unwrap_or(false)
        })?;

        let texture_key = quad.texture;

        let texture_path = self.get_texture_path(texture_key).unwrap();

        Some(texture_path)
    }
}

#[derive(Debug)]
pub(crate) struct MinecraftAssetsInner {
    pub(crate) root: PathBuf,
    pub(crate) block_state_table: BakedBlockStateTable,
    pub(crate) model_table: BakedModelTable,
    pub(crate) texture_table: TextureTable,
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let BakedAssets {
            block_states,
            models,
            textures,
        } = bakery::bake_all(data, &assets)?;

        let new = Self {
            root: PathBuf::from(root),
            block_state_table: block_states,
            model_table: models,
            texture_table: textures,
        };

        Ok(new)
    }
}
