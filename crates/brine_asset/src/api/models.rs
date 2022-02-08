pub(crate) use minecraft_assets::schemas::Model as McModel;

use crate::storage::{Model, ModelKey, ModelTable};

pub struct Models<'a> {
    model_table: &'a ModelTable,
}

impl<'a> Models<'a> {
    #[inline]
    pub(crate) fn new(model_table: &'a ModelTable) -> Self {
        Self { model_table }
    }

    #[inline]
    pub fn get_by_name(&self, name: &str) -> Option<&Model> {
        let key = self.model_table.get_key(name)?;
        self.model_table.get_by_key(key)
    }

    #[inline]
    pub fn get_by_key(&self, key: ModelKey) -> Option<&Model> {
        self.model_table.get_by_key(key)
    }

    #[inline]
    pub fn get_key(&self, name: &str) -> Option<ModelKey> {
        self.model_table.get_key(name)
    }
}
