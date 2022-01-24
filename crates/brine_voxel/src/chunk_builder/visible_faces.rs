//! Implementation of a chunk builder using the `block-mesh` visible_faces algorithm.

use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_pbr::PbrBundle;
use bevy_render::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    render_resource::PrimitiveTopology,
};
use bevy_transform::prelude::*;
use block_mesh::{
    ndshape::{ConstShape3u32, Shape},
    UnitQuadBuffer, Voxel, RIGHT_HANDED_Y_UP_CONFIG,
};

use brine_chunk::{Chunk, ChunkSection};

use crate::chunk_builder::AddToWorld;

use super::{
    component::{BuiltChunk, BuiltChunkSection},
    ChunkBuilder,
};

/// The output of [`VisibleFacesChunkBuilder`].
pub struct ChunkMeshes {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub sections: Vec<SectionMesh>,
}

impl AddToWorld for ChunkMeshes {
    fn add_to_world<'w, 's>(self, meshes: &mut Assets<Mesh>, commands: &mut Commands) -> Entity {
        commands
            .spawn()
            .insert(Transform::from_translation(Vec3::new(
                (self.chunk_x * 16) as f32,
                0.0,
                (self.chunk_z * 16) as f32,
            )))
            .insert(GlobalTransform::default())
            .insert(BuiltChunk::<VisibleFacesChunkBuilder>::default())
            .with_children(move |parent| {
                for section in self.sections.into_iter() {
                    parent
                        .spawn()
                        .insert(BuiltChunkSection::<VisibleFacesChunkBuilder>::default())
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(section.mesh),
                            transform: Transform::from_translation(Vec3::new(
                                0.0,
                                (section.section_y * 16) as f32,
                                0.0,
                            )),
                            ..Default::default()
                        });
                }
            })
            .id()
    }
}

pub struct SectionMesh {
    pub section_y: u8,
    pub mesh: Mesh,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

impl BoolVoxel {
    const EMPTY: Self = Self(false);
    const NONEMPTY: Self = Self(true);
}

impl Voxel for BoolVoxel {
    #[inline]
    fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    #[inline]
    fn is_opaque(&self) -> bool {
        true
    }
}

/// A [`ChunkBuilder`] that uses the [`visible_block_faces`] algorithm from the
/// [`block_mesh`] crate to build chunks.
///
/// [`visible_block_faces`]: block_mesh::visible_block_faces
#[derive(Default)]
pub struct VisibleFacesChunkBuilder;

impl VisibleFacesChunkBuilder {
    pub fn build_chunk(chunk: &Chunk) -> ChunkMeshes {
        ChunkMeshes {
            chunk_x: chunk.chunk_x,
            chunk_z: chunk.chunk_z,
            sections: chunk
                .data
                .sections()
                .iter()
                .map(Self::build_chunk_section)
                .collect(),
        }
    }

    pub fn build_chunk_section(chunk_section: &ChunkSection) -> SectionMesh {
        type ChunkShape = ConstShape3u32<18, 18, 18>;

        let shape = &ChunkShape {};

        let mut buffer = UnitQuadBuffer::new();

        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

        let mut block_states = [BoolVoxel::EMPTY; 18 * 18 * 18];
        for (x, y, z, block_state) in chunk_section.block_states.iter() {
            if block_state != brine_chunk::BlockState::AIR {
                let index = shape.linearize([x as u32 + 1, y as u32 + 1, z as u32 + 1]);
                block_states[index as usize] = BoolVoxel::NONEMPTY;
            }
        }

        block_mesh::visible_block_faces(
            &block_states[..],
            &ChunkShape {},
            [0; 3],
            [17; 3],
            &faces,
            &mut buffer,
        );

        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        render_mesh.set_attribute(
            "Vertex_Position",
            VertexAttributeValues::Float32x3(positions),
        );
        render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));
        render_mesh.set_attribute(
            "Vertex_Uv",
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        render_mesh.set_indices(Some(Indices::U32(indices.clone())));

        debug!("built chunk");
        trace!(
            "mesh vertices: {:?}",
            render_mesh.attribute("Vertex_Position").unwrap()
        );

        SectionMesh {
            section_y: chunk_section.chunk_y,
            mesh: render_mesh,
        }
    }
}

impl ChunkBuilder for VisibleFacesChunkBuilder {
    type Output = ChunkMeshes;

    fn build_chunk(&self, chunk: &Chunk) -> ChunkMeshes {
        Self::build_chunk(chunk)
    }
}
