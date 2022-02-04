//! Implementation of a chunk builder that just generates a cube for each block.

use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

use brine_chunk::{BlockState, Chunk, ChunkSection};

use crate::mesh::{Axis, VoxelFace, VoxelMesh};

use super::{ChunkBuilder, ChunkBuilderType};

/// A [`ChunkBuilder`] that just generates a cube mesh for each block.
#[derive(Default)]
pub struct NaiveBlocksChunkBuilder;

impl NaiveBlocksChunkBuilder {
    pub fn build_chunk(chunk: &Chunk) -> Vec<VoxelMesh> {
        chunk
            .sections
            .iter()
            .map(Self::build_chunk_section)
            .collect()
    }

    pub fn build_chunk_section(section: &ChunkSection) -> VoxelMesh {
        let num_blocks = section.block_count as usize;
        let num_faces = num_blocks * 6;
        let mut faces = Vec::with_capacity(num_faces);

        for (x, y, z, block_state) in section.block_states.iter() {
            if block_state != BlockState::AIR {
                Self::build_voxel(x, y, z, &mut faces);
            }
        }

        VoxelMesh { faces }
    }

    fn build_voxel(x: u8, y: u8, z: u8, faces: &mut Vec<VoxelFace>) {
        let cube_mesh = Self::get_cube_mesh(x, y, z);

        let positions = cube_mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let tex_coords = cube_mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap();
        let normals = cube_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
        let indices = cube_mesh.indices().unwrap();

        if let (
            VertexAttributeValues::Float32x3(positions),
            VertexAttributeValues::Float32x3(normals),
            VertexAttributeValues::Float32x2(tex_coords),
            Indices::U32(indices),
        ) = (positions, normals, tex_coords, indices)
        {
            for face_index in 0..6 {
                let vertex_index = face_index * 4;
                let index_index = face_index * 6;

                let voxel = [x, y, z];
                let axis = Self::get_axis_from_normal(normals[vertex_index]);
                let positions = positions[vertex_index..vertex_index + 4]
                    .try_into()
                    .unwrap();
                let tex_coords = tex_coords[vertex_index..vertex_index + 4]
                    .try_into()
                    .unwrap();
                let indices: [u32; 6] = indices[index_index..index_index + 6].try_into().unwrap();

                faces.push(VoxelFace {
                    voxel,
                    axis,
                    positions,
                    tex_coords,
                    indices: indices.map(|i| (i as usize - vertex_index) as u8),
                });
            }
        } else {
            unreachable!();
        };
    }

    fn get_cube_mesh(x: u8, y: u8, z: u8) -> Mesh {
        let x = x as f32;
        let y = y as f32;
        let z = z as f32;
        let cube = bevy::prelude::shape::Box {
            min_x: x,
            max_x: x + 1.0,
            min_y: y,
            max_y: y + 1.0,
            min_z: z,
            max_z: z + 1.0,
        };

        Mesh::from(cube)
    }

    fn get_axis_from_normal(normal: [f32; 3]) -> Axis {
        match normal {
            [x, _, _] if x > 0.0 => Axis::XPos,
            [x, _, _] if x < 0.0 => Axis::XNeg,
            [_, y, _] if y > 0.0 => Axis::YPos,
            [_, y, _] if y < 0.0 => Axis::YNeg,
            [_, _, z] if z > 0.0 => Axis::ZPos,
            [_, _, z] if z < 0.0 => Axis::ZNeg,
            _ => unreachable!(),
        }
    }
}

impl ChunkBuilder for NaiveBlocksChunkBuilder {
    const TYPE: ChunkBuilderType = ChunkBuilderType::NAIVE_BLOCKS;

    fn build_chunk(&self, chunk: &Chunk) -> Vec<VoxelMesh> {
        Self::build_chunk(chunk)
    }
}
