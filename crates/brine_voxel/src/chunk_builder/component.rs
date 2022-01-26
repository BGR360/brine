use std::{fmt, marker::PhantomData};

use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

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

impl<T> fmt::Debug for BuiltChunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltChunk").finish()
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

impl<T> fmt::Debug for BuiltChunkSection<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltChunkSection").finish()
    }
}

/// SAFETY: BuiltChunkSection is not inhabited.
unsafe impl<T> Send for BuiltChunkSection<T> {}
unsafe impl<T> Sync for BuiltChunkSection<T> {}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkBundle<Builder: 'static> {
    pub built_chunk: BuiltChunk<Builder>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl<Builder: 'static> BuiltChunkBundle<Builder> {
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            transform: Transform::from_translation(Vec3::new(
                (chunk_x * 16) as f32,
                0.0,
                (chunk_z * 16) as f32,
            )),
            global_transform: GlobalTransform::default(),
            built_chunk: BuiltChunk::<Builder>::default(),
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkSectionBundle<Builder: 'static> {
    pub built_chunk_section: BuiltChunkSection<Builder>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl<Builder: 'static> BuiltChunkSectionBundle<Builder> {
    pub fn new(section_y: u8) -> Self {
        Self {
            transform: Transform::from_translation(Vec3::new(0.0, (section_y * 16) as f32, 0.0)),
            global_transform: GlobalTransform::default(),
            built_chunk_section: BuiltChunkSection::<Builder>::default(),
        }
    }
}
