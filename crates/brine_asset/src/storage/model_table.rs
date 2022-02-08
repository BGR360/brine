use std::borrow::Borrow;

use indexmap::IndexMap;

use crate::api::Model;

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct ModelName(pub String);

impl Borrow<str> for ModelName {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

pub struct ModelKey(pub usize);

#[derive(Debug, Default, Clone)]
pub struct ModelTable {
    pub(crate) models: IndexMap<ModelName, Model>,
}

impl ModelTable {
    pub fn insert(&mut self, name: String, model: Model) -> ModelKey {
        let (index, _) = self.models.insert_full(ModelName(name), model);

        ModelKey(index)
    }

    #[inline]
    pub fn get_key(&self, name: &str) -> Option<ModelKey> {
        self.models.get_index_of(name).map(ModelKey)
    }

    #[inline]
    pub fn get_by_key(&self, key: ModelKey) -> Option<&Model> {
        self.models.get_index(key.0).map(|(_name, model)| model)
    }
}
