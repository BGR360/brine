use bevy::render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};

use brine_asset::MinecraftAssets;
use brine_chunk::ChunkSection;
use brine_data::MinecraftData;
use brine_voxel::{Mesh as VoxelMesh, Mesher, SimpleMesher};

use super::meshing_view::ChunkView;

#[derive(Debug)]
pub struct BakedChunk {
    pub mesh: Mesh,
}

impl Default for BakedChunk {
    fn default() -> Self {
        Self {
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
        }
    }
}

pub struct ChunkBakery<'a> {
    mc_data: &'a MinecraftData,
    // mc_assets: &'a MinecraftAssets,
}

impl<'a> ChunkBakery<'a> {
    pub fn new(mc_data: &'a MinecraftData) -> Self {
        Self { mc_data }
    }

    pub fn bake_chunk(&self, chunk: &ChunkSection) -> BakedChunk {
        let view = ChunkView::new(self.mc_data, chunk);

        let voxel_mesh = SimpleMesher.generate_mesh(view);

        let mesh = build_bevy_mesh(&voxel_mesh);

        BakedChunk { mesh }
    }
}

pub fn build_bevy_mesh(voxel_mesh: &VoxelMesh) -> Mesh {
    let num_vertices = voxel_mesh.quads.len() * 4;
    let num_indices = voxel_mesh.quads.len() * 6;
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for quad in voxel_mesh.quads.iter() {
        indices.extend_from_slice(
            &quad
                .get_indices()
                .map(|i| positions.len() as u32 + i as u32),
        );

        positions.extend_from_slice(&quad.positions);
        normals.extend_from_slice(&quad.get_normals());
        tex_coords.extend_from_slice(&quad.get_tex_coords());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
