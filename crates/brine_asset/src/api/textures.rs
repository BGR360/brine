use std::path::PathBuf;

use minecraft_assets::{api::ResourcePath, schemas::models::BlockFace};

use brine_data::blocks::BlockStateId;
use tracing::{debug, trace, warn};

use crate::{
    api::MinecraftAssetsInner,
    storage::{Quad, TextureKey, TextureTable},
};

pub struct Textures<'a> {
    parent: &'a MinecraftAssetsInner,
}

impl<'a> Textures<'a> {
    pub(crate) fn new(parent: &'a MinecraftAssetsInner) -> Self {
        Self { parent }
    }

    pub fn get_texture_path(&self, texture_key: TextureKey) -> Option<PathBuf> {
        let texture_id = self.parent.texture_table.get_by_key(texture_key)?;

        let texture_path = ResourcePath::for_resource(&self.parent.root, texture_id);

        Some(texture_path.strip_prefix("assets").unwrap().into())
    }

    pub fn table(&self) -> &'a TextureTable {
        &self.parent.texture_table
    }

    // TODO: deprecate
    pub fn get_texture_path_for_block_state_and_face(
        &self,
        block_state_id: BlockStateId,
        face: BlockFace,
    ) -> Option<PathBuf> {
        trace!("Querying texture for {:?}:{:?}", block_state_id, face);

        let block_state = self
            .parent
            .block_state_table
            .get_by_key(block_state_id)
            .or_else(|| {
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

        let model = self.parent.model_table.get_by_key(model_key).unwrap();

        trace!("{:#?}", model);

        if model.num_cuboids() > 1 {
            debug!(
                "{:?} for {:?} has more than one cuboid element, using the first one",
                model_key, block_state_id
            );
        }

        let cuboid = self
            .parent
            .cuboid_table
            .get_by_key(model.first_cuboid)
            .unwrap();

        trace!("{:#?}", cuboid);

        let quads: Vec<&Quad> = cuboid
            .quads()
            .map(|quad_key| self.parent.quad_table.get_by_key(quad_key).unwrap())
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
