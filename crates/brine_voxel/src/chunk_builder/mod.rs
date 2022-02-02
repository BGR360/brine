//! Generating renderable content from chunk data.
//!
//! Chunk building is the process of taking a chunk's data and turning it into
//! meshes and materials and such.
//!
//! This process is designed to happen asynchronously outside of the main game
//! loop. This makes things slightly awkward in that the chunk builders don't
//! have access to the game world in order to create, access, or register
//! various assets. See the [`ChunkBuilder`] docs for details on how this is
//! dealt with.

use std::{fmt, marker::PhantomData};

use brine_chunk::Chunk;

mod block_mesh;
pub mod component;
mod naive_blocks;
mod plugin;

use crate::mesh::VoxelMesh;

pub use self::block_mesh::{GreedyQuadsChunkBuilder, VisibleFacesChunkBuilder};
pub use naive_blocks::NaiveBlocksChunkBuilder;
pub use plugin::ChunkBuilderPlugin;

/// A trait for types that can turn a [`Chunk`] into a renderable representation
/// that can be added to a Bevy world.
///
/// See the [module documentation][self] for more information.
pub trait ChunkBuilder: Sized {
    const TYPE: ChunkBuilderType;

    /// Generates the output data from the provided chunk data.
    fn build_chunk(&self, chunk: &Chunk) -> ChunkMeshes<Self>;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkBuilderType(pub &'static str);

impl ChunkBuilderType {
    pub const UNKNOWN: Self = Self("UNKNOWN_CHUNK_BUILDER");
    pub const GREEDY_QUADS: Self = Self("GreedyQuadsChunkBuilder");
    pub const VISIBLE_FACES: Self = Self("VisibleFacesChunkBuilder");
    pub const NAIVE_BLOCKS: Self = Self("NaiveBlocksChunkBuilder");
}

impl fmt::Debug for ChunkBuilderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.0).finish()
    }
}

/// The output of a chunk builder.
pub struct ChunkMeshes<Builder> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub sections: Vec<SectionMesh>,

    _phantom: PhantomData<Builder>,
}

pub struct SectionMesh {
    pub section_y: u8,
    pub mesh: VoxelMesh,
}
