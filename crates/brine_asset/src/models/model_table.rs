use std::borrow::Borrow;

use crate::hash_slab::{HashSlab, UsizeKey};

use super::Model;

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct ModelName(pub String);

impl Borrow<str> for ModelName {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

pub type ModelKey = UsizeKey<ModelName>;

#[derive(Debug, Default, Clone)]
pub struct ModelTable {
    pub(crate) names: HashSlab<ModelName, ModelKey>,
    pub(crate) models: Vec<Model>,
}

impl ModelTable {
    pub fn insert(&mut self, name: String, model: Model) -> ModelKey {
        let key = self.names.insert(ModelName(name));

        self.models.push(model);

        assert_eq!(self.models.len() - 1, usize::from(key));

        key
    }

    #[inline]
    pub fn get_key(&self, name: &str) -> Option<ModelKey> {
        self.names.get_key(name)
    }

    #[inline]
    pub fn get_by_key(&self, key: ModelKey) -> Option<&Model> {
        self.models.get(usize::from(key))
    }
}
