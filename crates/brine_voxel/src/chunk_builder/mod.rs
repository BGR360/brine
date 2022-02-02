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

use std::marker::PhantomData;

use bevy::prelude::*;

use brine_chunk::Chunk;

mod block_mesh;
pub mod component;
mod naive_blocks;
mod plugin;

use crate::mesh::VoxelMesh;

pub use self::block_mesh::{GreedyQuadsChunkBuilder, VisibleFacesChunkBuilder};
pub use naive_blocks::NaiveBlocksChunkBuilder;
pub use plugin::ChunkBuilderPlugin;

use component::{BuiltChunkBundle, BuiltChunkSectionBundle};

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
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(section.mesh.to_render_mesh()),
                            ..Default::default()
                        });
                }
            })
            .id()
    }
}
