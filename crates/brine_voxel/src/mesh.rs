use std::ops::Add;

use bevy::{
    ecs::component::Component,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
};
use brine_chunk::BlockState;

/// The six sides of a voxel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Axis {
    XPos = 0,
    XNeg = 1,
    YPos = 2,
    YNeg = 3,
    ZPos = 4,
    ZNeg = 5,
}

impl Default for Axis {
    fn default() -> Self {
        Self::XPos
    }
}

impl Axis {
    pub const fn normal(&self) -> [i8; 3] {
        match self {
            Axis::XPos => [1, 0, 0],
            Axis::XNeg => [-1, 0, 0],
            Axis::YPos => [0, 1, 0],
            Axis::YNeg => [0, -1, 0],
            Axis::ZPos => [0, 0, 1],
            Axis::ZNeg => [0, 0, -1],
        }
    }
}

/// A mesh made up of one or more voxels.
#[derive(Component, Debug, Default, Clone)]
pub struct VoxelMesh {
    /// A list of faces that make up the mesh.
    pub faces: Vec<VoxelFace>,

    pub voxel_values: Vec<BlockState>,
}

/// A single face in a [`VoxelMesh`].
#[derive(Debug, Default, Clone)]
pub struct VoxelFace {
    /// The [x, y, z] index of the voxel that contains this face.
    pub voxel: [u8; 3],

    /// The direction of this face's normal vector.
    pub axis: Axis,

    /// The positions of the face's vertices in 3D space.
    /// `[x, y, z] * 4`
    pub positions: [[f32; 3]; 4],

    /// The texture coordinates (UV's) of the face's vertices.
    pub tex_coords: [[f32; 2]; 4],

    /// Vertex indices.
    ///
    /// These describe how to draw the face using two triangles.
    /// Each entry is an index into the `positions` array.
    pub indices: [u8; 6],
}

impl VoxelMesh {
    pub fn to_render_mesh(&self) -> Mesh {
        let num_vertices = self.faces.len() * 4;
        let mut positions = Vec::with_capacity(num_vertices);
        let mut tex_coords = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);

        for face in self.faces.iter() {
            positions.extend_from_slice(&face.positions);
            tex_coords.extend_from_slice(&face.tex_coords);

            let normal = face.axis.normal().map(|elt| elt as f32);
            normals.extend_from_slice(&[normal; 4]);
        }

        let indices = if num_vertices > u16::MAX as usize {
            Indices::U32(self.get_indices::<u32>())
        } else {
            Indices::U16(self.get_indices::<u16>())
        };

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(indices));

        mesh
    }

    fn get_indices<T>(&self) -> Vec<T>
    where
        T: Copy + Clone + From<u8> + Add<Output = T>,
    {
        let num_indices = self.faces.len() * 6;
        let mut all_indices = Vec::with_capacity(num_indices);

        let mut offset = T::from(0u8);
        for face in self.faces.iter() {
            let indices = face.indices.map(|i| offset + T::from(i));
            all_indices.extend_from_slice(&indices);
            offset = offset + T::from(4u8);
        }

        all_indices
    }
}
