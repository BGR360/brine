use bevy::prelude::*;

use brine_voxel::{Direction, Mesher, MeshingView, SimpleMesher, VoxelView};

mod common;

use common::{IntChunk, MeshViewerPlugin, CHUNK_SIDE};

struct BoolView<'a> {
    chunk: &'a IntChunk,
}

impl<'a> BoolView<'a> {
    fn is_empty(&self, x: u8, y: u8, z: u8) -> Option<bool> {
        self.chunk.get(x, y, z).map(|i| i == 0)
    }
}

impl<'a> VoxelView for BoolView<'a> {
    #[inline(always)]
    fn size_x(&self) -> u8 {
        CHUNK_SIDE
    }

    #[inline(always)]
    fn size_y(&self) -> u8 {
        CHUNK_SIDE
    }

    #[inline(always)]
    fn size_z(&self) -> u8 {
        CHUNK_SIDE
    }
}

impl<'a> MeshingView for BoolView<'a> {
    type Quads = Option<[[f32; 3]; 4]>;

    #[inline(always)]
    fn is_empty(&self, x: u8, y: u8, z: u8) -> bool {
        Self::is_empty(self, x, y, z).unwrap()
    }

    #[inline(always)]
    fn is_full_cube(&self, _x: u8, _y: u8, _z: u8) -> bool {
        true
    }

    #[inline]
    fn is_face_occluded(&self, x: u8, y: u8, z: u8, direction: Direction) -> bool {
        println!("pos: {:?}, direction: {:?}", [x, y, z], direction);
        direction
            .translate_pos([x, y, z], 1)
            .and_then(|[x, y, z]| {
                println!("neighbor: {:?}", [x, y, z]);
                Self::is_empty(self, x, y, z)
            })
            .map(|neighbor_empty| {
                println!("empty: {}", neighbor_empty);
                !neighbor_empty
            })
            .unwrap_or(false)
    }

    #[inline(always)]
    fn face_quads(&self, _x: u8, _y: u8, _z: u8, _face: Direction) -> Self::Quads {
        None
    }

    #[inline(always)]
    fn non_face_quads(&self, _x: u8, _y: u8, _z: u8) -> Self::Quads {
        None
    }
}

fn main() {
    let chunk = IntChunk::random(1);

    let bool_view = BoolView { chunk: &chunk };

    let mesh = SimpleMesher.generate_mesh(bool_view);

    println!("Chunk:");
    println!("{chunk}");

    // println!("Mesh: {mesh:#?}");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MeshViewerPlugin::new(mesh))
        .run();
}
