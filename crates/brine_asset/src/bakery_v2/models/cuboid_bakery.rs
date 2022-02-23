use minecraft_assets::{
    api::ResourceIdentifier,
    schemas::models::{BlockFace, Textures},
};
use smallvec::SmallVec;
use tracing::*;

use crate::bakery_v2::{
    models::{BakedQuad, Cuboid, CuboidRotation, UnbakedCuboid, UnbakedQuad},
    textures::TextureTable,
};

/// Bakes a single cuboid for a model.
pub struct CuboidBakery<'a> {
    unbaked_cuboid: &'a UnbakedCuboid,
    resolved_textures: &'a Textures,
    texture_table: &'a TextureTable,

    original_cuboid: Cuboid,
    rotation: CuboidRotation,
    rotated_and_scaled_cuboid: Cuboid,
    uv_lock: bool,
}

impl<'a> CuboidBakery<'a> {
    pub fn new(
        unbaked_cuboid: &'a UnbakedCuboid,
        resolved_textures: &'a Textures,
        texture_table: &'a TextureTable,
        uv_lock: bool,
    ) -> Self {
        let original_cuboid = Cuboid::new(unbaked_cuboid.from, unbaked_cuboid.to);
        let rotation = CuboidRotation::from(unbaked_cuboid.rotation.clone());
        let rotated_cuboid = rotation.rotate_cuboid(original_cuboid.clone());
        let rotated_and_scaled_cuboid = rotated_cuboid.scaled(1.0 / 16.0);

        Self {
            unbaked_cuboid,
            resolved_textures,
            texture_table,
            original_cuboid,
            rotation,
            rotated_and_scaled_cuboid,
            uv_lock,
        }
    }

    pub fn bake(&self) -> SmallVec<[BakedQuad; 6]> {
        let mut quads = SmallVec::new();

        for face in [
            BlockFace::Down,
            BlockFace::Up,
            BlockFace::North,
            BlockFace::South,
            BlockFace::West,
            BlockFace::East,
        ] {
            if let Some(baked_quad) = self
                .unbaked_cuboid
                .faces
                .get(&face)
                .and_then(|unbaked_quad| self.bake_quad(unbaked_quad, face))
            {
                quads.push(baked_quad);
            }
        }

        quads
    }

    #[inline]
    pub fn bake_quad(&self, quad: &UnbakedQuad, face: BlockFace) -> Option<BakedQuad> {
        let positions = self
            .rotated_and_scaled_cuboid
            .get_face(face)
            .map(|vec3a| vec3a.into());

        let normal = self.rotation.rotate_vector(Cuboid::get_normal(face)).into();
        let tex_coords = self.get_quad_tex_coords(quad, face)?;

        let resolved_texture = quad.texture.resolve(self.resolved_textures).or_else(|| {
            warn!(
                "No resolution for texture {:?} in {:?}",
                quad.texture, self.resolved_textures
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
            shade: self.unbaked_cuboid.shade,
            face,
            cull_face: quad.cull_face,
            tinted: quad.tint_index >= 0,
            texture: texture_key,
        })
    }

    #[inline(always)]
    pub fn get_quad_tex_coords(
        &self,
        quad: &UnbakedQuad,
        face: BlockFace,
    ) -> Option<[[f32; 2]; 4]> {
        /*
                a           b
            (u0, v0)    (u1, v0)

                c           d
            (u0, v1)    (u1, v1)
        */
        let [c, d, a, b] = quad
            .uv
            .map(|[u0, v0, u1, v1]| {
                let a = [u0, v0];
                let b = [u1, v0];
                let c = [u0, v1];
                let d = [u1, v1];
                [c, d, a, b]
            })
            .unwrap_or_else(|| self.infer_quad_tex_coords_from_cuboid(face));

        let uvs = match (self.uv_lock, quad.rotation) {
            /*
                a --- b
                  \
                   \
                    \
                c --- d
            */
            (true, _) | (false, 0) => Some([c, d, a, b]),

            /*
                c --- a
                  \
                   \
                    \
                d --- b
            */
            (false, 90) => Some([d, b, c, a]),

            /*
                d --- c
                  \
                   \
                    \
                b --- a
            */
            (false, 180) => Some([b, a, d, c]),

            /*
                b --- d
                  \
                   \
                    \
                a --- c
            */
            (false, 270) => Some([a, c, b, d]),

            (false, x) => {
                warn!("Invalid face rotation: {}", x);
                None
            }
        };

        uvs.map(|uvs| uvs.map(|[u, v]| [u / 16.0, v / 16.0]))
    }

    #[inline(always)]
    pub fn infer_quad_tex_coords_from_cuboid(&self, face: BlockFace) -> [[f32; 2]; 4] {
        let face_verts: [[f32; 3]; 4] = self.original_cuboid.get_face(face).map(Into::into);

        face_verts.map(|[x, y, z]| match face {
            BlockFace::Down => [x, 16.0 - z],
            BlockFace::Up => [x, z],
            BlockFace::North => [16.0 - x, 16.0 - y],
            BlockFace::South => [x, 16.0 - y],
            BlockFace::West => [z, 16.0 - y],
            BlockFace::East => [16.0 - z, 16.0 - y],
        })
    }
}
