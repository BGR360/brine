use minecraft_assets::{
    api::{ModelResolver, ResourceIdentifier},
    schemas::{blockstates::ModelProperties, models::Textures},
};
use smallvec::SmallVec;
use tracing::*;

use crate::{
    bakery_v2::models::{
        cuboid_math::QuadRotation, BakedModel, BakedQuad, CuboidBakery, UnbakedCuboid,
        UnbakedModel, UnbakedModels,
    },
    storage::TextureTable,
};

pub struct ModelBakery<'a> {
    unbaked_models: &'a UnbakedModels,
    texture_table: &'a TextureTable,
}

impl<'a> ModelBakery<'a> {
    pub fn new(unbaked_models: &'a UnbakedModels, texture_table: &'a TextureTable) -> Self {
        Self {
            unbaked_models,
            texture_table,
        }
    }

    pub fn bake_model_from_properties(
        &self,
        model_properties: &ModelProperties,
    ) -> Option<BakedModel> {
        let mut baked_model = self.bake_model(&model_properties.model, model_properties.uv_lock)?;

        let rotation = QuadRotation::new(model_properties.x, model_properties.y);

        for quad in baked_model.quads.iter_mut() {
            rotation.rotate_quad(quad);
        }

        Some(baked_model)
    }

    pub fn bake_model(&self, model_name: &str, uv_lock: bool) -> Option<BakedModel> {
        debug!("Baking model: {}", model_name);

        let mut baked_quads = SmallVec::new();

        let model = self
            .unbaked_models
            .get(&ResourceIdentifier::block_model(model_name))?;
        let parent_chain = self.get_parent_chain(model);

        let resolved_textures = ModelResolver::resolve_textures(parent_chain.iter().copied());

        if let Some(cuboid_elements) = ModelResolver::resolve_elements(parent_chain.iter().copied())
        {
            for cuboid in cuboid_elements {
                let mut cuboid_quads = self.bake_cuboid(&cuboid, &resolved_textures, uv_lock);
                baked_quads.append(&mut cuboid_quads);
            }
        }

        Some(BakedModel { quads: baked_quads })
    }

    pub fn bake_cuboid(
        &self,
        cuboid: &'a UnbakedCuboid,
        resolved_textures: &Textures,
        uv_lock: bool,
    ) -> SmallVec<[BakedQuad; 6]> {
        let cuboid_bakery =
            CuboidBakery::new(cuboid, resolved_textures, self.texture_table, uv_lock);

        cuboid_bakery.bake()
    }

    pub fn get_parent_chain(&self, mut child: &'a UnbakedModel) -> SmallVec<[&'a UnbakedModel; 4]> {
        let mut chain = SmallVec::new();
        chain.push(child);

        while let Some(parent) = child.parent.as_ref() {
            trace!("Parent: {}", parent);
            child = self
                .unbaked_models
                .get(&ResourceIdentifier::block_model(parent))
                .unwrap();
            chain.push(child);
        }

        chain
    }
}
