pub(crate) use minecraft_assets::schemas::Model as McModel;

mod model;
mod model_table;
mod unresolved;

pub use model::Model;
pub use model_table::{ModelKey, ModelTable};

use minecraft_assets::api::{AssetPack, ModelResolver, Result};

use brine_data::MinecraftData;

use unresolved::{UnresolvedModelLoader, UnresolvedModelTable};

pub struct Models<'a> {
    models: &'a ModelTable,
}

impl<'a> Models<'a> {
    #[inline]
    pub(crate) fn new(models: &'a ModelTable) -> Self {
        Self { models }
    }

    #[inline]
    pub fn get_by_name(&self, name: &str) -> Option<&Model> {
        let key = self.models.get_key(name)?;
        self.models.get_by_key(key)
    }

    #[inline]
    pub fn get_by_key(&self, key: ModelKey) -> Option<&Model> {
        self.models.get_by_key(key)
    }

    #[inline]
    pub fn get_key(&self, name: &str) -> Option<ModelKey> {
        self.models.get_key(name)
    }
}

pub(crate) fn build(assets: &AssetPack, _data: &MinecraftData) -> Result<ModelTable> {
    let unresolved = UnresolvedModelLoader::load_block_models(assets)?;

    let mut resolved = ModelTable::default();

    for (name, unresolved_model) in unresolved.iter() {
        let resolved_model = resolve_model(&unresolved, unresolved_model);

        resolved.insert(
            name.clone(),
            Model {
                resolved: resolved_model,
            },
        );
    }

    Ok(resolved)
}

fn resolve_model(unresolved: &UnresolvedModelTable, model: &McModel) -> McModel {
    let mut parents = vec![model];

    while let Some(parent_name) = model.parent.as_ref() {
        if let Some(parent) = unresolved.get(parent_name) {
            parents.push(parent);
        } else {
            break;
        }
    }

    ModelResolver::resolve_model(parents.into_iter())
}
