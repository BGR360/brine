use minecraft_assets::{
    api::{AssetPack, ModelResolver, Result},
    schemas::models::Model as McModel,
};

use brine_data::MinecraftData;

use crate::storage::{Model, ModelTable};

mod unresolved;

use unresolved::UnresolvedModelTable;

pub(crate) fn build(assets: &AssetPack, _data: &MinecraftData) -> Result<ModelTable> {
    let unresolved = unresolved::load_block_models(assets)?;

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
