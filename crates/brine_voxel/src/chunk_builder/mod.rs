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

use bevy_asset::Assets;
use bevy_ecs::{entity::Entity, system::Commands};
use bevy_render::mesh::Mesh;

use brine_chunk::Chunk;

mod block_mesh;
pub mod component;
mod naive_blocks;
mod plugin;

pub use self::block_mesh::{GreedyQuadsChunkBuilder, VisibleFacesChunkBuilder};
pub use naive_blocks::NaiveBlocksChunkBuilder;
pub use plugin::ChunkBuilderPlugin;

pub trait AddToWorld {
    fn add_to_world(self, meshes: &mut Assets<Mesh>, commands: &mut Commands) -> Entity;
}

/// A trait for types that can turn a [`Chunk`] into a renderable representation
/// that can be added to a Bevy world.
///
/// See the [module documentation][self] for more information.
pub trait ChunkBuilder {
    /// The output type must implement [`AddToWorld`] so that it can be added to
    /// the world once the build process completes.
    ///
    /// Implementations should insert the [`BuiltChunk`] and [`BuiltChunkSection`]
    /// components onto the appropriate entities when `add_to_world` is called.
    ///
    /// [`BuiltChunk`]: component::BuiltChunk
    /// [`BuiltChunkSection`]: component::BuiltChunkSection
    type Output: AddToWorld;

    /// Generates the output data from the provided chunk data.
    fn build_chunk(&self, chunk: &Chunk) -> Self::Output;
}
