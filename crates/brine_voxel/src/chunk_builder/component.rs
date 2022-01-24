use std::marker::PhantomData;

use bevy_ecs::component::Component;

/// Component that stores the original chunk data for a chunk.
#[derive(Component)]
pub struct Chunk(pub brine_chunk::Chunk);

/// Component that signifies a built chunk.
///
/// Typically has one or more children with [`BuiltChunkSection`] components.
#[derive(Component)]
pub struct BuiltChunk<T>(PhantomData<T>);

impl<T> Default for BuiltChunk<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// SAFETY: BuiltChunk is not inhabited.
unsafe impl<T> Send for BuiltChunk<T> {}
unsafe impl<T> Sync for BuiltChunk<T> {}

/// Component that signifies a built chunk section.
#[derive(Component)]
pub struct BuiltChunkSection<T>(PhantomData<T>);

impl<T> Default for BuiltChunkSection<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// SAFETY: BuiltChunkSection is not inhabited.
unsafe impl<T> Send for BuiltChunkSection<T> {}
unsafe impl<T> Sync for BuiltChunkSection<T> {}
