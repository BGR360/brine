use std::borrow::Borrow;

use indexmap::IndexMap;

use crate::api::McModel;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub resolved: McModel,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ModelName(pub String);

impl Borrow<str> for ModelName {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModelTable {
    models: IndexMap<ModelName, Model>,
}

impl ModelTable {
    pub fn insert(&mut self, name: String, model: Model) -> ModelKey {
        let (index, _) = self.models.insert_full(ModelName(name), model);

        ModelKey(index)
    }

    #[inline]
    pub fn get_by_key(&self, key: ModelKey) -> Option<&Model> {
        self.models.get_index(key.0).map(|(_name, model)| model)
    }

    #[inline]
    pub fn get_key(&self, name: &str) -> Option<ModelKey> {
        self.models.get_index_of(name).map(ModelKey)
    }
}
