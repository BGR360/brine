//! API for accessing Minecraft asset data at runtime.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use minecraft_assets::api::{AssetPack, ResourcePath};
use tracing::*;

pub(crate) use minecraft_assets::schemas::Model as McModel;

pub use minecraft_assets::{api::Result, schemas::models::BlockFace};

pub use brine_data::{
    blocks::{BlockId, BlockStateId},
    MinecraftData, Version,
};

use crate::{
    bakery::{self, block_states::BlockStateBuilder, models::ModelBuilder},
    storage::{
        BlockStateTable, CuboidTable, ModelTable, Quad, QuadTable, TextureKey, TextureTable,
    },
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
    pub fn block_states(&self) -> &BlockStateTable {
        &self.inner.block_state_table
    }

    #[inline]
    pub fn models(&self) -> &ModelTable {
        &self.inner.model_table
    }

    #[inline]
    pub fn cuboids(&self) -> &CuboidTable {
        &self.inner.cuboid_table
    }

    #[inline]
    pub fn quads(&self) -> &QuadTable {
        &self.inner.quad_table
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

        let block_state = self.block_states().get_by_key(block_state_id).or_else(|| {
            warn!("No block for {:?}", block_state_id);
            None
        })?;

        trace!("{:#?}", block_state);

        let grab_bag = block_state.models.first().or_else(|| {
            warn!("{:?} has no models!", block_state_id);
            None
        })?;

        // TODO: pick random model from grab bag.
        let model_key = grab_bag
            .choices
            .get(0)
            .or_else(|| {
                warn!("{:?} has no models!", block_state_id);
                None
            })?
            .model;

        if block_state.models.len() > 1 {
            debug!(
                "{:?} is composed of multiple models, using the first one",
                block_state_id
            );
        }

        let model = self.models().get_by_key(model_key).unwrap();

        trace!("{:#?}", model);

        if model.num_cuboids() > 1 {
            debug!(
                "{:?} for {:?} has more than one cuboid element, using the first one",
                model_key, block_state_id
            );
        }

        let cuboid = self.cuboids().get_by_key(model.first_cuboid).unwrap();

        trace!("{:#?}", cuboid);

        let quads: Vec<&Quad> = cuboid
            .quads()
            .map(|quad_key| self.quads().get_by_key(quad_key).unwrap())
            .collect();

        let quad = quads
            .iter()
            .find(|quad| {
                trace!("{:#?}", quad);
                quad.face == face
            })
            .or_else(|| {
                debug!(
                    "Cuboid of {:?} for {:?} has no quad on block face {:?}",
                    model_key, block_state_id, face
                );

                quads.first()
            })?;

        let texture_key = quad.texture;

        let texture_path = self.get_texture_path(texture_key)?;

        trace!("{}", texture_path.to_string_lossy());

        Some(texture_path)
    }
}

#[derive(Debug)]
pub(crate) struct MinecraftAssetsInner {
    pub(crate) root: PathBuf,
    pub(crate) block_state_table: BlockStateTable,
    pub(crate) cuboid_table: CuboidTable,
    pub(crate) model_table: ModelTable,
    pub(crate) quad_table: QuadTable,
    pub(crate) texture_table: TextureTable,
}

impl MinecraftAssetsInner {
    fn build(root: &Path, data: &MinecraftData) -> Result<Self> {
        let assets = AssetPack::at_path(root);

        let texture_table = bakery::textures::load_texture_ids(&assets)?;

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
            root: PathBuf::from(root),
            block_state_table,
            cuboid_table,
            model_table,
            quad_table,
            texture_table,
        };

        Ok(new)
    }
}
