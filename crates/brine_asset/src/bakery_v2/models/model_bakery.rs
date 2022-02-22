use minecraft_assets::{
    api::{ModelResolver, ResourceIdentifier},
    schemas::{
        blockstates::ModelProperties,
        models::{BlockFace, Textures},
    },
};
use smallvec::SmallVec;
use tracing::*;

use crate::{
    bakery_v2::models::{
        cuboid_math::QuadRotation, BakedModel, BakedQuad, Cuboid, CuboidRotation, UnbakedCuboid,
        UnbakedModel, UnbakedModels, UnbakedQuad,
    },
    storage::TextureTable,
};

pub struct ModelBakery<'a> {
    unbaked_models: &'a UnbakedModels,
    texture_table: &'a TextureTable,
}

impl<'a> ModelBakery<'a> {
    pub fn new(unbaked_models: &'a UnbakedModels, texture_table: &'a TextureTable) -> Self {
        Self {
            unbaked_models,
            texture_table,
        }
    }

    pub fn bake_model_from_properties(
        &self,
        model_properties: &ModelProperties,
    ) -> Option<BakedModel> {
        let mut baked_model = self.bake_model(&model_properties.model, model_properties.uv_lock)?;

        let rotation = QuadRotation::new(model_properties.x, model_properties.y);

        for quad in baked_model.quads.iter_mut() {
            rotation.rotate_quad(quad);
        }

        Some(baked_model)
    }

    pub fn bake_model(&self, model_name: &str, uv_lock: bool) -> Option<BakedModel> {
        debug!("Baking model: {}", model_name);

        let mut baked_quads = SmallVec::new();

        let model = self
            .unbaked_models
            .get(&ResourceIdentifier::block_model(model_name))?;
        let parent_chain = self.get_parent_chain(model);

        let resolved_textures = ModelResolver::resolve_textures(parent_chain.iter().copied());

        if let Some(cuboid_elements) = ModelResolver::resolve_elements(parent_chain.iter().copied())
        {
            for cuboid in cuboid_elements {
                let mut cuboid_quads = self.bake_cuboid(&cuboid, &resolved_textures, uv_lock);
                baked_quads.append(&mut cuboid_quads);
            }
        }

        Some(BakedModel { quads: baked_quads })
    }

    pub fn bake_cuboid(
        &self,
        cuboid: &'a UnbakedCuboid,
        resolved_textures: &Textures,
        uv_lock: bool,
    ) -> SmallVec<[BakedQuad; 6]> {
        let rotation = CuboidRotation::from(cuboid.rotation.clone());
        let rotated_cuboid = rotation.rotate_cuboid(Cuboid::new(cuboid.from, cuboid.to));
        let rotated_and_scaled_cuboid = rotated_cuboid.scaled(1.0 / 16.0);

        let shade_faces = cuboid.shade;

        [
            BlockFace::Down,
            BlockFace::Up,
            BlockFace::North,
            BlockFace::South,
            BlockFace::West,
            BlockFace::East,
        ]
        .into_iter()
        .filter_map(|face| {
            let unbaked_quad = cuboid.faces.get(&face)?;
            self.bake_quad(
                unbaked_quad,
                &rotation,
                &rotated_and_scaled_cuboid,
                face,
                resolved_textures,
                shade_faces,
                uv_lock,
            )
        })
        .collect()
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn bake_quad(
        &self,
        quad: &UnbakedQuad,
        rotation: &CuboidRotation,
        rotated_cuboid: &Cuboid,
        face: BlockFace,
        resolved_textures: &Textures,
        shade: bool,
        uv_lock: bool,
    ) -> Option<BakedQuad> {
        let positions = rotated_cuboid.get_face(face).map(|vec3a| vec3a.into());
        let normal = rotation.rotate_vector(Cuboid::get_normal(face)).into();
        let tex_coords = Self::get_quad_tex_coords(quad, uv_lock)?;

        let resolved_texture = quad.texture.resolve(resolved_textures).or_else(|| {
            warn!(
                "No resolution for texture {:?} in {:?}",
                quad.texture, resolved_textures
            );
            None
        })?;

        let texture_key = self
            .texture_table
            .get_key(&ResourceIdentifier::texture(resolved_texture))
            .or_else(|| {
                warn!("Texture not in texture table: {}", resolved_texture);
                None
            })?;

        Some(BakedQuad {
            positions,
            normal,
            tex_coords,
            shade,
            cull_face: quad.cull_face,
            tinted: quad.tint_index >= 0,
            texture: texture_key,
        })
    }

    #[inline(always)]
    pub fn get_quad_tex_coords(quad: &UnbakedQuad, uv_lock: bool) -> Option<[f32; 4]> {
        /*
            (u0, v0)    (u1, v0)


            (u0, v1)    (u1, v1)
        */
        let [u0, v0, u1, v1] = quad.uv.unwrap_or([0.0, 0.0, 16.0, 16.0]);

        let uvs = match (uv_lock, quad.rotation) {
            /*
            (u0, v0)
                     \
                      \>
                        (u1, v1)
            */
            (true, _) | (false, 0) => Some([u0, v0, u1, v1]),

            /*
                        (u1, v0)
                      /
                    </
            (u0, v1)
            */
            (false, 90) => Some([u1, v0, u0, v1]),

            /*
            (u0, v0)
                    <\
                      \
                        (u1, v1)
            */
            (false, 180) => Some([u1, v1, u0, v0]),

            /*
                        (u1, v0)
                      />
                     /
            (u0, v1)
            */
            (false, 270) => Some([u0, v1, u1, v0]),

            (false, x) => {
                warn!("Invalid face rotation: {}", x);
                None
            }
        };

        uvs.map(|uvs| uvs.map(|x| x / 16.0))
    }

    pub fn get_parent_chain(&self, mut child: &'a UnbakedModel) -> SmallVec<[&'a UnbakedModel; 4]> {
        let mut chain = SmallVec::new();
        chain.push(child);

        while let Some(parent) = child.parent.as_ref() {
            trace!("Parent: {}", parent);
            child = self
                .unbaked_models
                .get(&ResourceIdentifier::block_model(parent))
                .unwrap();
            chain.push(child);
        }

        chain
    }
}
