//! Two implementations of chunk builders using algorithms from the `block-mesh` crate.

use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use block_mesh::{
    ndshape::{ConstShape3u32, Shape},
    GreedyQuadsBuffer, MergeVoxel, OrientedBlockFace, UnitQuadBuffer, UnorientedQuad, Voxel,
    RIGHT_HANDED_Y_UP_CONFIG,
};

use brine_chunk::{Chunk, ChunkSection, SECTION_WIDTH};

use crate::chunk_builder::AddToWorld;

use super::{
    component::{BuiltChunkBundle, BuiltChunkSectionBundle},
    ChunkBuilder,
};

/// The output of [`VisibleFacesChunkBuilder`] and [`GreedyQuadsChunkBuilder`].
pub struct ChunkMeshes<Builder> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub sections: Vec<SectionMesh>,

    _phantom: PhantomData<Builder>,
}

pub struct SectionMesh {
    pub section_y: u8,
    pub mesh: Mesh,
}

impl<Builder> AddToWorld for ChunkMeshes<Builder>
where
    Builder: 'static,
{
    fn add_to_world<'w, 's>(self, meshes: &mut Assets<Mesh>, commands: &mut Commands) -> Entity {
        commands
            .spawn()
            .insert_bundle(BuiltChunkBundle::<Builder>::new(self.chunk_x, self.chunk_z))
            .with_children(move |parent| {
                for section in self.sections.into_iter() {
                    parent
                        .spawn()
                        .insert_bundle(BuiltChunkSectionBundle::<Builder>::new(section.section_y))
                        .with_children(|parent| {
                            parent.spawn().insert_bundle(PbrBundle {
                                mesh: meshes.add(section.mesh),
                                // Mesh needs to be offset by [-1, -1, -1] to be
                                // properly aligned.
                                transform: Transform::from_translation(Vec3::new(-1.0, -1.0, -1.0)),
                                ..Default::default()
                            });
                        });
                }
            })
            .id()
    }
}

/// A [`ChunkBuilder`] that uses the [`visible_block_faces`] algorithm from the
/// [`block_mesh`] crate to build chunks.
///
/// [`visible_block_faces`]: block_mesh::visible_block_faces
#[derive(Default)]
pub struct VisibleFacesChunkBuilder;

impl VisibleFacesChunkBuilder {
    pub fn build_chunk(chunk: &Chunk) -> ChunkMeshes<Self> {
        ChunkMeshes {
            chunk_x: chunk.chunk_x,
            chunk_z: chunk.chunk_z,
            sections: chunk
                .data
                .sections()
                .iter()
                .map(Self::build_chunk_section)
                .collect(),

            _phantom: PhantomData,
        }
    }

    pub fn build_chunk_section(chunk_section: &ChunkSection) -> SectionMesh {
        BlockMeshBuilder::build_with(chunk_section, |params| {
            let mut buffer = UnitQuadBuffer::new();
            block_mesh::visible_block_faces(
                params.voxels,
                params.shape,
                params.min,
                params.max,
                params.faces,
                &mut buffer,
            );
            BlockMeshOutput::VisibleFaces(buffer)
        })
    }
}

impl ChunkBuilder for VisibleFacesChunkBuilder {
    type Output = ChunkMeshes<Self>;

    fn build_chunk(&self, chunk: &Chunk) -> Self::Output {
        Self::build_chunk(chunk)
    }
}

/// A [`ChunkBuilder`] that uses the [`greedy_quads`] algorithm from the
/// [`block_mesh`] crate to build chunks.
///
/// [`greedy_quads`]: block_mesh::greedy_quads
#[derive(Default)]
pub struct GreedyQuadsChunkBuilder;

impl GreedyQuadsChunkBuilder {
    pub fn build_chunk(chunk: &Chunk) -> ChunkMeshes<Self> {
        ChunkMeshes {
            chunk_x: chunk.chunk_x,
            chunk_z: chunk.chunk_z,
            sections: chunk
                .data
                .sections()
                .iter()
                .map(Self::build_chunk_section)
                .collect(),

            _phantom: PhantomData,
        }
    }

    pub fn build_chunk_section(chunk_section: &ChunkSection) -> SectionMesh {
        BlockMeshBuilder::build_with(chunk_section, |params| {
            let mut buffer = GreedyQuadsBuffer::new(params.voxels.len());
            block_mesh::greedy_quads(
                params.voxels,
                params.shape,
                params.min,
                params.max,
                params.faces,
                &mut buffer,
            );
            BlockMeshOutput::GreedyQuads(buffer)
        })
    }
}

impl ChunkBuilder for GreedyQuadsChunkBuilder {
    type Output = ChunkMeshes<Self>;

    fn build_chunk(&self, chunk: &Chunk) -> Self::Output {
        Self::build_chunk(chunk)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct BlockState(brine_chunk::BlockState);

impl BlockState {
    const EMPTY: Self = Self(brine_chunk::BlockState::AIR);
}

impl Voxel for BlockState {
    #[inline]
    fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    #[inline]
    fn is_opaque(&self) -> bool {
        true
    }
}

impl MergeVoxel for BlockState {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

const SHAPE_SIDE: u32 = (SECTION_WIDTH as u32) + 2;
type ChunkShape = ConstShape3u32<SHAPE_SIDE, SHAPE_SIDE, SHAPE_SIDE>;

struct BlockMeshBuilderParams<'a> {
    voxels: &'a [BlockState],
    shape: &'a ChunkShape,
    min: [u32; 3],
    max: [u32; 3],
    faces: &'a [OrientedBlockFace; 6],
}

struct BlockMeshBuilder;

impl BlockMeshBuilder {
    fn build_with<F>(chunk_section: &ChunkSection, func: F) -> SectionMesh
    where
        F: FnOnce(BlockMeshBuilderParams) -> BlockMeshOutput,
    {
        let shape = ChunkShape {};
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

        let mut block_states = [BlockState::EMPTY; (SHAPE_SIDE * SHAPE_SIDE * SHAPE_SIDE) as usize];
        for (x, y, z, block_state) in chunk_section.block_states.iter() {
            let index = shape.linearize([x as u32 + 1, y as u32 + 1, z as u32 + 1]);
            block_states[index as usize] = BlockState(block_state);
        }

        let output = func(BlockMeshBuilderParams {
            voxels: &block_states[..],
            shape: &shape,
            min: [0; 3],
            max: [SHAPE_SIDE - 1; 3],
            faces: &faces,
        });

        let render_mesh = Self::generate_mesh(&faces, output);

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

    fn generate_mesh(faces: &[OrientedBlockFace; 6], output: BlockMeshOutput) -> Mesh {
        let num_indices = output.num_quads() * 6;
        let num_vertices = output.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);

        output.for_each_quad_and_face(faces, |quad, face| {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        });

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
        render_mesh.set_indices(Some(Indices::U32(indices)));

        render_mesh
    }
}

enum BlockMeshOutput {
    VisibleFaces(UnitQuadBuffer),
    GreedyQuads(GreedyQuadsBuffer),
}

impl BlockMeshOutput {
    #[inline]
    fn num_quads(&self) -> usize {
        match self {
            Self::VisibleFaces(buffer) => buffer.num_quads(),
            Self::GreedyQuads(buffer) => buffer.quads.num_quads(),
        }
    }

    #[inline]
    fn for_each_quad_and_face(
        self,
        faces: &[OrientedBlockFace; 6],
        mut func: impl FnMut(UnorientedQuad, &OrientedBlockFace),
    ) {
        match self {
            Self::VisibleFaces(buffer) => {
                for (group, face) in buffer.groups.into_iter().zip(faces.iter()) {
                    for quad in group.into_iter() {
                        func(quad.into(), face);
                    }
                }
            }
            Self::GreedyQuads(buffer) => {
                for (group, face) in buffer.quads.groups.into_iter().zip(faces.iter()) {
                    for quad in group.into_iter() {
                        func(quad, face)
                    }
                }
            }
        }
    }
}
