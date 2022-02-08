use indexmap::IndexMap;

use crate::storage::CuboidKey;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Model {
    pub ambient_occlusion: bool,
    pub first_cuboid: CuboidKey,
    pub last_cuboid: CuboidKey,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModelTable {
    models: IndexMap<String, Model>,
}

impl ModelTable {
    pub fn count(&self) -> usize {
        self.models.len()
    }

    pub fn insert(&mut self, name: &str, model: Model) -> ModelKey {
        let (index, _) = self.models.insert_full(name.to_string(), model);

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
