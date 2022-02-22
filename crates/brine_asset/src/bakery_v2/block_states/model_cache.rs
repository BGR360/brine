use crate::bakery_v2::models::{BakedModel, ModelBakery};

pub struct BakedModelCache<'a, 'b> {
    model_bakery: &'a ModelBakery<'b>,
    models: Vec<(&'a str, BakedModel)>,
}

impl<'a, 'b> BakedModelCache<'a, 'b> {
    pub fn new(model_bakery: &'a ModelBakery<'b>) -> Self {
        Self {
            model_bakery,
            models: Default::default(),
        }
    }

    pub fn get_or_bake_model(&mut self, model_id: &'a str) -> Option<&BakedModel> {
        if !self.models.iter().any(|(id, _)| *id == model_id) {
            if let Some(baked_model) = self.model_bakery.bake_model(model_id) {
                self.models.push((model_id, baked_model));
            }
        }

        self.models
            .iter()
            .find(|(id, _)| *id == model_id)
            .map(|(_id, cached_model)| cached_model)
    }
}
