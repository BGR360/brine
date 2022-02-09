use std::collections::HashMap;

use minecraft_assets::api::{ModelIdentifier, ModelResolver};
use tracing::{debug, trace};

use crate::api::McModel;

use super::unresolved::UnresolvedModelTable;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ResolvedModelTable(pub HashMap<String, McModel>);

pub fn resolve_models(unresolved: &UnresolvedModelTable) -> ResolvedModelTable {
    let mut table = ResolvedModelTable::default();

    for (name, unresolved_model) in unresolved.0.iter() {
        debug!("Resolving {:?}", name);

        // if unresolved_model.parent.is_none() {
        //     trace!("No parent, skipping");
        //     continue;
        // }

        let resolved_model = resolve_model(unresolved, unresolved_model);

        let model_id = ModelIdentifier::from(name);

        table
            .0
            .insert(model_id.model_name().to_string(), resolved_model);
    }

    table
}

fn resolve_model<'a>(unresolved: &'a UnresolvedModelTable, mut model: &'a McModel) -> McModel {
    let mut parents = vec![model];

    while let Some(parent_name) = model.parent.as_ref() {
        trace!("parent: {:?}", parent_name);

        let parent_id = ModelIdentifier::from(parent_name);

        if let Some(parent) = unresolved.0.get(parent_id.model_name()) {
            parents.push(parent);

            model = parent;
        } else {
            trace!("not found");
            break;
        }
    }

    ModelResolver::resolve_model(parents.into_iter())
}
