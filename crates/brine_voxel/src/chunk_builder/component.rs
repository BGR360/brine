use std::{fmt, marker::PhantomData};

use bevy::prelude::*;

use super::{ChunkBuilder, ChunkBuilderType};

/// Component that marks the parent entity of a chunk column.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ChunkMarker;

/// Component that stores the original chunk data for a chunk section.
#[derive(Component)]
pub struct ChunkSection(pub brine_chunk::ChunkSection);

/// Component that signifies a built chunk.
///
/// Typically has one or more children with [`BuiltChunkSection`] components.
#[derive(Component)]
pub struct BuiltChunk<T> {
    pub builder: ChunkBuilderType,
    pub chunk_x: i32,
    pub chunk_z: i32,
    _phantom: PhantomData<T>,
}

impl<T> Default for BuiltChunk<T> {
    fn default() -> Self {
        Self {
            builder: ChunkBuilderType::UNKNOWN,
            chunk_x: 0,
            chunk_z: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for BuiltChunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BuiltChunk")
            .field(&self.builder)
            .field(&self.chunk_x)
            .field(&self.chunk_z)
            .finish()
    }
}

impl<T> fmt::Display for BuiltChunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk ({}, {})", self.chunk_x, self.chunk_z)
    }
}

/// SAFETY: BuiltChunk is not inhabited.
unsafe impl<T> Send for BuiltChunk<T> {}
unsafe impl<T> Sync for BuiltChunk<T> {}

/// Component that signifies a built chunk section.
#[derive(Component)]
pub struct BuiltChunkSection {
    pub builder: ChunkBuilderType,
    pub section_y: u8,
}

impl Default for BuiltChunkSection {
    fn default() -> Self {
        Self {
            builder: ChunkBuilderType::UNKNOWN,
            section_y: 0,
        }
    }
}

impl fmt::Debug for BuiltChunkSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BuiltChunkSection")
            .field(&self.builder)
            .field(&self.section_y)
            .finish()
    }
}

impl fmt::Display for BuiltChunkSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Section {}", self.section_y)
    }
}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkBundle<Builder: 'static> {
    pub built_chunk: BuiltChunk<Builder>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub chunk: ChunkMarker,
}

impl<Builder: 'static> BuiltChunkBundle<Builder>
where
    Builder: ChunkBuilder,
{
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            built_chunk: BuiltChunk::<Builder> {
                builder: Builder::TYPE,
                chunk_x,
                chunk_z,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                (chunk_x * 16) as f32,
                0.0,
                (chunk_z * 16) as f32,
            )),
            global_transform: Default::default(),
            chunk: Default::default(),
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkSectionBundle {
    pub built_chunk_section: BuiltChunkSection,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl BuiltChunkSectionBundle {
    pub fn new(builder: ChunkBuilderType, section_y: u8) -> Self {
        Self {
            built_chunk_section: BuiltChunkSection { builder, section_y },
            transform: Transform::from_translation(Vec3::new(0.0, (section_y * 16) as f32, 0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
