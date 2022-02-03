//! Generating renderable content from chunk data.
//!
//! Chunk building is the process of taking a chunk's data and turning it into
//! meshes and materials and such. This process is designed to happen
//! asynchronously outside of the main game loop.

use std::fmt;

use brine_chunk::Chunk;

mod block_mesh;
pub mod component;
mod naive_blocks;
mod plugin;

use crate::mesh::VoxelMesh;

pub use self::block_mesh::{GreedyQuadsChunkBuilder, VisibleFacesChunkBuilder};
pub use naive_blocks::NaiveBlocksChunkBuilder;
pub use plugin::ChunkBuilderPlugin;

/// A trait for types that can turn a [`Chunk`] into [`VoxelMesh`]es.
pub trait ChunkBuilder: Sized {
    const TYPE: ChunkBuilderType;

    fn build_chunk(&self, chunk: &Chunk) -> Vec<VoxelMesh>;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkBuilderType(pub &'static str);

impl ChunkBuilderType {
    pub const UNKNOWN: Self = Self("UNKNOWN_CHUNK_BUILDER");
    pub const GREEDY_QUADS: Self = Self("GreedyQuadsChunkBuilder");
    pub const VISIBLE_FACES: Self = Self("VisibleFacesChunkBuilder");
    pub const NAIVE_BLOCKS: Self = Self("NaiveBlocksChunkBuilder");
}

impl Default for ChunkBuilderType {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

impl fmt::Debug for ChunkBuilderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.0).finish()
    }
}
