//! Two implementations of chunk builders using algorithms from the `block-mesh` crate.

use std::marker::PhantomData;

use bevy::prelude::*;
use block_mesh::{
    ndshape::{ConstShape3u32, Shape},
    GreedyQuadsBuffer, MergeVoxel, OrientedBlockFace, UnitQuadBuffer, UnorientedQuad, Voxel,
    RIGHT_HANDED_Y_UP_CONFIG,
};

use brine_chunk::{Chunk, ChunkSection, SECTION_WIDTH};

use crate::{
    chunk_builder::ChunkBuilderType,
    mesh::{Axis, VoxelFace, VoxelMesh},
};

use super::{ChunkBuilder, ChunkMeshes, SectionMesh};

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
                .sections
                .iter()
                .map(Self::build_chunk_section)
                .collect(),

            _phantom: PhantomData,
        }
    }

    pub fn build_chunk_section(chunk_section: &ChunkSection) -> SectionMesh {
        BlockMeshBuilder::new().build_with(chunk_section, |builder| {
            let mut buffer = UnitQuadBuffer::new();
            block_mesh::visible_block_faces(
                &builder.voxels[..],
                &builder.shape,
                builder.min,
                builder.max,
                &builder.faces,
                &mut buffer,
            );
            BlockMeshOutput::VisibleFaces(buffer)
        })
    }
}

impl ChunkBuilder for VisibleFacesChunkBuilder {
    const TYPE: ChunkBuilderType = ChunkBuilderType::VISIBLE_FACES;

    fn build_chunk(&self, chunk: &Chunk) -> ChunkMeshes<Self> {
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
                .sections
                .iter()
                .map(Self::build_chunk_section)
                .collect(),

            _phantom: PhantomData,
        }
    }

    pub fn build_chunk_section(chunk_section: &ChunkSection) -> SectionMesh {
        BlockMeshBuilder::new().build_with(chunk_section, |builder| {
            let mut buffer = GreedyQuadsBuffer::new(builder.voxels.len());
            block_mesh::greedy_quads(
                &builder.voxels[..],
                &builder.shape,
                builder.min,
                builder.max,
                &builder.faces,
                &mut buffer,
            );
            BlockMeshOutput::GreedyQuads(buffer)
        })
    }
}

impl ChunkBuilder for GreedyQuadsChunkBuilder {
    const TYPE: ChunkBuilderType = ChunkBuilderType::GREEDY_QUADS;

    fn build_chunk(&self, chunk: &Chunk) -> ChunkMeshes<Self> {
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

struct BlockMeshBuilder {
    voxels: [BlockState; Self::BUFFER_SIZE],
    shape: ChunkShape,
    min: [u32; 3],
    max: [u32; 3],
    faces: [OrientedBlockFace; 6],
}

impl BlockMeshBuilder {
    const BUFFER_SIZE: usize = (SHAPE_SIDE * SHAPE_SIDE * SHAPE_SIDE) as usize;

    fn new() -> Self {
        Self {
            voxels: [BlockState::EMPTY; Self::BUFFER_SIZE],
            shape: ChunkShape {},
            min: [0; 3],
            max: [SHAPE_SIDE - 1; 3],
            faces: RIGHT_HANDED_Y_UP_CONFIG.faces,
        }
    }

    fn build_with<F>(&mut self, chunk_section: &ChunkSection, func: F) -> SectionMesh
    where
        F: FnOnce(&BlockMeshBuilder) -> BlockMeshOutput,
    {
        for (x, y, z, block_state) in chunk_section.block_states.iter() {
            let index = self
                .shape
                .linearize([x as u32 + 1, y as u32 + 1, z as u32 + 1]);
            self.voxels[index as usize] = BlockState(block_state);
        }

        let output = func(self);

        let voxel_mesh = self.generate_voxel_mesh(output);

        let section = SectionMesh {
            section_y: chunk_section.chunk_y,
            mesh: voxel_mesh,
        };

        debug!("built chunk");

        section
    }

    fn generate_voxel_mesh(&self, output: BlockMeshOutput) -> VoxelMesh {
        let num_faces = output.num_quads();
        let mut faces = Vec::with_capacity(num_faces);

        let mut block_states = Vec::new();

        output.for_each_quad_and_face(&self.faces, |quad, face| {
            let voxel = quad.minimum.map(|elt| elt as u8);
            let axis = Self::get_axis(face);
            // Mesh needs to be offset by [-1, -1, -1] to be properly aligned.
            let positions = face
                .quad_mesh_positions(&quad, 1.0)
                .map(|[x, y, z]| [x - 1.0, y - 1.0, z - 1.0]);
            let tex_coords = face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad);
            let indices = face.quad_mesh_indices(0).map(|i| i as u8);

            faces.push(VoxelFace {
                voxel,
                axis,
                positions,
                tex_coords,
                indices,
            });

            let block_state = self.voxels[self.shape.linearize(quad.minimum) as usize];
            block_states.push(block_state.0);
        });

        VoxelMesh {
            faces,
            voxel_values: block_states,
        }
    }

    fn get_axis(face: &OrientedBlockFace) -> Axis {
        match face.signed_normal().to_array() {
            [1, 0, 0] => Axis::XPos,
            [-1, 0, 0] => Axis::XNeg,
            [0, 1, 0] => Axis::YPos,
            [0, -1, 0] => Axis::YNeg,
            [0, 0, 1] => Axis::ZPos,
            [0, 0, -1] => Axis::ZNeg,
            _ => unreachable!(),
        }
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
