use std::collections::{HashMap, HashSet};

use minecraft_assets::{
    api::{ModelIdentifier, ResourceIdentifier},
    schemas::models::{
        BlockFace, Element as McElement, ElementFace as McElementFace, Model as McModel,
    },
};
use tracing::{debug, warn};

use crate::{
    bakery_v2::models::{CuboidRotation, QuarterRotation},
    storage::{
        Cuboid, CuboidKey, CuboidTable, Model, ModelKey, ModelTable, Quad, QuadKey, QuadTable,
        TextureTable,
    },
};

pub(crate) mod resolved;
pub(crate) mod unresolved;

use resolved::ResolvedModelTable;

#[derive(Debug)]
pub(crate) struct ModelBuilder<'a> {
    pub(crate) model_table: ModelTable,
    pub(crate) cuboid_table: CuboidTable,
    pub(crate) quad_table: QuadTable,
    known_nonexistent_models: HashSet<String>,
    resolved_mc_models: &'a ResolvedModelTable,
    textures: &'a TextureTable,
}

impl<'a> ModelBuilder<'a> {
    pub fn new(resolved_mc_models: &'a ResolvedModelTable, textures: &'a TextureTable) -> Self {
        Self {
            resolved_mc_models,
            textures,
            model_table: Default::default(),
            cuboid_table: Default::default(),
            quad_table: Default::default(),
            known_nonexistent_models: Default::default(),
        }
    }

    // pub fn build(&mut self) -> Result<()> {
    //     for (model_name, mc_model) in self.resolved_mc_models.0.iter() {
    //         self.build_model(model_name, mc_model);
    //     }

    //     info!("Built {} models", self.model_table.count());

    //     Ok(())
    // }

    pub fn get_or_build_model(&mut self, name: &str) -> Option<ModelKey> {
        let name = ModelIdentifier::model_name(name);

        if let Some(key) = self.model_table.get_key(name) {
            return Some(key);
        }

        if self.known_nonexistent_models.contains(name) {
            return None;
        }

        let mc_model = self.resolved_mc_models.0.get(name).or_else(|| {
            let mut available_names = self.resolved_mc_models.0.keys().collect::<Vec<_>>();
            available_names.sort();

            warn!(
                "No model for name {}. Available: {:#?}",
                name, available_names
            );

            None
        })?;

        self.build_model(name, mc_model).or_else(|| {
            self.known_nonexistent_models.insert(String::from(name));
            None
        })
    }

    pub fn build_model(&mut self, name: &str, mc_model: &McModel) -> Option<ModelKey> {
        debug!("Building model {:?}", name);

        let ambient_occlusion = mc_model.ambient_occlusion.unwrap_or(true);

        let (first_cuboid, last_cuboid) = self.build_cuboids(mc_model)?;

        let model = Model {
            ambient_occlusion,
            first_cuboid,
            last_cuboid,
        };

        let key = self.model_table.insert(name, model);

        Some(key)
    }

    pub fn build_cuboids(&mut self, mc_model: &McModel) -> Option<(CuboidKey, CuboidKey)> {
        let mc_elements = &mc_model.elements.as_ref().or_else(|| {
            warn!("No elements in model: {:#?}", mc_model);
            None
        })?[..];

        assert!(!mc_elements.is_empty(), "Empty list of elements");

        let first = self.cuboid_table.next_key();
        let mut last = first;

        for mc_element in mc_elements {
            last = self.build_cuboid(mc_model, mc_element)?;
        }

        Some((first, last))
    }

    pub fn build_cuboid(
        &mut self,
        mc_model: &McModel,
        mc_element: &McElement,
    ) -> Option<CuboidKey> {
        let from = mc_element.from;
        let to = mc_element.to;
        let rotation = CuboidRotation::from(mc_element.rotation.clone());
        let shade = mc_element.shade;

        let (first_face, last_face) = self.build_quads(mc_model, &mc_element.faces)?;

        let cuboid = Cuboid {
            from,
            to,
            rotation,
            shade,
            first_face,
            last_face,
        };

        let key = self.cuboid_table.insert(cuboid);

        Some(key)
    }

    pub fn build_quads(
        &mut self,
        mc_model: &McModel,
        mc_faces: &HashMap<BlockFace, McElementFace>,
    ) -> Option<(QuadKey, QuadKey)> {
        assert!(!mc_faces.is_empty(), "Empty list of faces");

        let first = self.quad_table.next_key();
        let mut last = first;

        for (block_face, element_face) in mc_faces.iter() {
            last = self.build_quad(mc_model, *block_face, element_face)?;
        }

        Some((first, last))
    }

    pub fn build_quad(
        &mut self,
        mc_model: &McModel,
        block_face: BlockFace,
        element_face: &McElementFace,
    ) -> Option<QuadKey> {
        let face = block_face;
        let uv = element_face.uv.unwrap_or_else(|| [0.0, 0.0, 16.0, 16.0]);
        let cull_face = element_face.cull_face;
        let rotation = QuarterRotation::from(element_face.rotation);

        let tint_index = match element_face.tint_index {
            -1 => None,
            i @ 0.. => Some(i as u8),
            i => panic!("Invalid tintindex: {}", i),
        };

        let texture = &element_face.texture;
        let resolved_texture = if let Some(resolved) = texture.location() {
            resolved
        } else {
            texture
                .resolve(mc_model.textures.as_ref().unwrap())
                .or_else(|| {
                    warn!(
                        "Could not resolve texture variable: {:?}. Textures: {:#?}",
                        element_face.texture, &mc_model.textures
                    );
                    None
                })?
        };

        let texture = self
            .textures
            .get_key(&ResourceIdentifier::texture(resolved_texture))
            .or_else(|| {
                warn!(
                    "Could not resolve texture identifier to a known texture: {}",
                    resolved_texture
                );
                None
            })?;

        let quad = Quad {
            face,
            texture,
            uv,
            cull_face,
            rotation,
            tint_index,
        };

        let key = self.quad_table.insert(quad);

        Some(key)
    }
}
