//! Implementation of a chunk builder that just generates a cube for each block.

use bevy::prelude::*;

use brine_chunk::{BlockState, Chunk, ChunkSection};

use super::{
    component::{BuiltChunkBundle, BuiltChunkSectionBundle},
    AddToWorld, ChunkBuilder,
};

pub struct ChunkBlocks {
    chunk_x: i32,
    chunk_z: i32,
    block_mesh: Mesh,
    sections: Vec<SectionBlocks>,
}

pub struct SectionBlocks {
    section_y: u8,
    transforms: Vec<Transform>,
}

impl AddToWorld for ChunkBlocks {
    fn add_to_world(self, meshes: &mut Assets<Mesh>, commands: &mut Commands) -> Entity {
        let handle = meshes.add(self.block_mesh);

        commands
            .spawn()
            .insert_bundle(BuiltChunkBundle::<NaiveBlocksChunkBuilder>::new(
                self.chunk_x,
                self.chunk_z,
            ))
            .with_children(|parent| {
                for section in self.sections.into_iter() {
                    parent
                        .spawn()
                        .insert_bundle(BuiltChunkSectionBundle::<NaiveBlocksChunkBuilder>::new(
                            section.section_y,
                        ))
                        .with_children(|parent| {
                            for transform in section.transforms.into_iter() {
                                parent.spawn().insert_bundle(PbrBundle {
                                    mesh: handle.clone(),
                                    transform,
                                    ..Default::default()
                                });
                            }
                        });
                }
            })
            .id()
    }
}

/// A [`ChunkBuilder`] that just generates a cube mesh for each block.
#[derive(Default)]
pub struct NaiveBlocksChunkBuilder;

impl NaiveBlocksChunkBuilder {
    pub fn build_chunk(chunk: &Chunk) -> ChunkBlocks {
        ChunkBlocks {
            chunk_x: chunk.chunk_x,
            chunk_z: chunk.chunk_z,
            block_mesh: Mesh::from(shape::Cube { size: 1.0 }),
            sections: chunk
                .sections()
                .iter()
                .map(Self::build_chunk_section)
                .collect(),
        }
    }

    pub fn build_chunk_section(section: &ChunkSection) -> SectionBlocks {
        SectionBlocks {
            section_y: section.chunk_y,
            transforms: section
                .block_states
                .iter()
                .filter_map(|(x, y, z, block_state)| {
                    if block_state != BlockState::AIR {
                        Some(Transform::from_translation(Vec3::new(
                            x as f32 + 0.5,
                            y as f32 + 0.5,
                            z as f32 + 0.5,
                        )))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl ChunkBuilder for NaiveBlocksChunkBuilder {
    type Output = ChunkBlocks;

    fn build_chunk(&self, chunk: &Chunk) -> ChunkBlocks {
        Self::build_chunk(chunk)
    }
}
