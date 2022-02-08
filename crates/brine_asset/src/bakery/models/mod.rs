use std::collections::HashMap;

use brine_data::MinecraftData;
use minecraft_assets::{
    api::{AssetPack, ResourceIdentifier, Result},
    schemas::models::{
        BlockFace, Element as McElement, ElementFace as McElementFace, Model as McModel,
    },
};
use tracing::{debug, info, trace};

use crate::storage::{
    Cuboid, CuboidKey, CuboidRotation, CuboidTable, Model, ModelTable, Quad, QuadKey, QuadRotation,
    QuadTable, TextureTable,
};

mod resolved;
mod unresolved;

#[derive(Debug)]
pub(crate) struct ModelBuilder<'a> {
    pub(crate) model_table: ModelTable,
    pub(crate) cuboid_table: CuboidTable,
    pub(crate) quad_table: QuadTable,
    texture_table: &'a TextureTable,
}

impl<'a> ModelBuilder<'a> {
    pub fn build(
        assets: &AssetPack,
        data: &MinecraftData,
        texture_table: &'a TextureTable,
    ) -> Result<Self> {
        let mut builder = Self {
            texture_table,
            model_table: Default::default(),
            cuboid_table: Default::default(),
            quad_table: Default::default(),
        };

        let resolved_models = {
            let unresolved_models = unresolved::load_block_models(assets)?;

            resolved::resolve_models(&unresolved_models)
        };

        for (name, model) in resolved_models.0.iter() {
            debug!("Building model {:?}", name);

            if let Some(model) = builder.build_model(model) {
                builder.model_table.insert(name, model);
            }
        }

        info!("Built {} models", builder.model_table.count());

        Ok(builder)
    }

    fn build_model(&mut self, mc_model: &McModel) -> Option<Model> {
        let ambient_occlusion = mc_model.ambient_occlusion.unwrap_or(true);

        let (first_cuboid, last_cuboid) = self.build_cuboids(mc_model)?;

        Some(Model {
            ambient_occlusion,
            first_cuboid,
            last_cuboid,
        })
    }

    fn build_cuboids(&mut self, mc_model: &McModel) -> Option<(CuboidKey, CuboidKey)> {
        let mc_elements = &mc_model.elements.as_ref().or_else(|| {
            trace!("No elements in model: {:#?}", mc_model);
            None
        })?[..];

        assert!(!mc_elements.is_empty(), "Empty list of elements");

        let first = self.cuboid_table.next_key();
        let mut last = first;

        for mc_element in mc_elements {
            let cuboid = self.build_cuboid(mc_model, mc_element)?;
            last = self.cuboid_table.insert(cuboid);
        }

        Some((first, last))
    }

    fn build_cuboid(&mut self, mc_model: &McModel, mc_element: &McElement) -> Option<Cuboid> {
        let from = mc_element.from;
        let to = mc_element.to;
        let rotation = CuboidRotation::from(mc_element.rotation.clone());
        let shade = mc_element.shade;

        let (first_face, last_face) = self.build_quads(mc_model, &mc_element.faces)?;

        Some(Cuboid {
            from,
            to,
            rotation,
            shade,
            first_face,
            last_face,
        })
    }

    fn build_quads(
        &mut self,
        mc_model: &McModel,
        mc_faces: &HashMap<BlockFace, McElementFace>,
    ) -> Option<(QuadKey, QuadKey)> {
        assert!(!mc_faces.is_empty(), "Empty list of faces");

        let first = self.quad_table.next_key();
        let mut last = first;

        for (block_face, element_face) in mc_faces.iter() {
            let quad = self.build_quad(mc_model, *block_face, element_face)?;
            last = self.quad_table.insert(quad);
        }

        Some((first, last))
    }

    fn build_quad(
        &self,
        mc_model: &McModel,
        block_face: BlockFace,
        element_face: &McElementFace,
    ) -> Option<Quad> {
        let face = block_face;
        let uv = element_face.uv.unwrap_or_else(|| [0.0, 0.0, 16.0, 16.0]);
        let cull_face = element_face.cull_face;
        let rotation = QuadRotation::from(element_face.rotation);

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
                    trace!(
                        "Could not resolve texture variable: {:?}. Textures: {:#?}",
                        element_face.texture,
                        &mc_model.textures
                    );
                    None
                })?
        };

        let texture = self
            .texture_table
            .get_key(&ResourceIdentifier::from(resolved_texture))
            .or_else(|| {
                trace!(
                    "Could not resolve texture identifier to a known texture: {}",
                    resolved_texture
                );
                None
            })?;

        Some(Quad {
            face,
            texture,
            uv,
            cull_face,
            rotation,
            tint_index,
        })
    }
}
