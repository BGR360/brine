use std::fmt;

use bevy::{prelude::*, tasks::Task};

use crate::mesh::VoxelMesh;

use super::ChunkBuilderType;

#[derive(Component)]
pub struct PendingChunk {
    pub task: Task<(brine_chunk::Chunk, Vec<VoxelMesh>)>,
    pub builder: ChunkBuilderType,
}

/// Component that stores the original chunk data for a chunk section.
#[derive(Component)]
pub struct ChunkSection(pub brine_chunk::ChunkSection);

/// Component that signifies a built chunk.
///
/// Typically has one or more children with [`BuiltChunkSection`] components.
#[derive(Debug, Default, Component)]
pub struct BuiltChunk {
    pub builder: ChunkBuilderType,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl fmt::Display for BuiltChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk ({}, {})", self.chunk_x, self.chunk_z)
    }
}

/// Component that signifies a built chunk section.
#[derive(Debug, Default, Component)]
pub struct BuiltChunkSection {
    pub builder: ChunkBuilderType,
    pub section_y: u8,
}

impl fmt::Display for BuiltChunkSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Section {}", self.section_y)
    }
}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkBundle {
    pub built_chunk: BuiltChunk,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl BuiltChunkBundle {
    pub fn new(builder: ChunkBuilderType, chunk_x: i32, chunk_z: i32) -> Self {
        let built_chunk = BuiltChunk {
            builder,
            chunk_x,
            chunk_z,
        };

        let name = Name::new(built_chunk.to_string());

        Self {
            built_chunk,
            name,
            transform: Transform::from_translation(Vec3::new(
                (chunk_x * 16) as f32,
                0.0,
                (chunk_z * 16) as f32,
            )),
            global_transform: Default::default(),
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct BuiltChunkSectionBundle {
    pub built_chunk_section: BuiltChunkSection,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl BuiltChunkSectionBundle {
    pub fn new(builder: ChunkBuilderType, section_y: u8) -> Self {
        let built_chunk_section = BuiltChunkSection { builder, section_y };

        let name = Name::new(built_chunk_section.to_string());

        Self {
            built_chunk_section,
            name,
            transform: Transform::from_translation(Vec3::new(0.0, (section_y * 16) as f32, 0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
