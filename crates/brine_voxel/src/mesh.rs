use bevy::ecs::component::Component;
use brine_chunk::BlockState;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VoxelFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

/// Describes a mesh made up of one or more voxels.
#[derive(Component)]
pub struct VoxelMesh {
    /// Vertex indices.
    ///
    /// These describe how to draw the mesh using triangles. Each entry is an
    /// index into the vectors below.
    pub indices: Vec<u16>,

    /// The position of each vertex in the mesh.
    pub positions: Vec<[f32; 3]>,

    /// The normal vector of each vertex in the mesh.
    pub normals: Vec<[f32; 3]>,

    /// The <u, v> texture coordinate of each vertex in the mesh.
    ///
    /// The texture coordinates of vertices on different faces are independent
    /// of each other; interpret these as though you were drawing only one face
    /// at a time with a separate texture per face.
    pub tex_coords: Vec<[f32; 2]>,

    /// The "value" of the voxel that each vertex belongs to.
    pub voxel_values: Vec<BlockState>,
}
