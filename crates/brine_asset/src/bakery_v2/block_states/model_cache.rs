use minecraft_assets::schemas::blockstates::ModelProperties;

use crate::bakery_v2::models::{BakedModel, ModelBakery};

pub struct BakedModelCache<'a, 'b> {
    model_bakery: &'a ModelBakery<'b>,
    models: Vec<(&'a ModelProperties, BakedModel)>,
}

impl<'a, 'b> BakedModelCache<'a, 'b> {
    pub fn new(model_bakery: &'a ModelBakery<'b>) -> Self {
        Self {
            model_bakery,
            models: Default::default(),
        }
    }

    pub fn get_or_bake_model(
        &mut self,
        model_properties: &'a ModelProperties,
    ) -> Option<&BakedModel> {
        if self.get_cached(model_properties).is_none() {
            if let Some(baked_model) = self
                .model_bakery
                .bake_model_from_properties(model_properties)
            {
                self.models.push((model_properties, baked_model));
            }
        }

        self.get_cached(model_properties)
    }

    pub fn get_cached(&self, model_properties: &'a ModelProperties) -> Option<&BakedModel> {
        self.models
            .iter()
            .find(|(properties, _)| {
                properties.model == model_properties.model
                    && properties.x == model_properties.x
                    && properties.y == model_properties.y
                    && properties.uv_lock == model_properties.uv_lock
            })
            .map(|(_id, cached_model)| cached_model)
    }
}
