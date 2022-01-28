//! A library for decoding Minecraft chunk data from network packets.
//!
//! Currently only supports version 1.14.4.

use std::fmt;

pub mod decode;
pub mod palette;

pub use palette::{Palette, SectionPalette};

pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_WIDTH: usize = 16;
pub const SECTION_HEIGHT: usize = 16;
pub const SECTION_WIDTH: usize = CHUNK_WIDTH;
pub const SECTIONS_PER_CHUNK: usize = CHUNK_HEIGHT / SECTION_HEIGHT;
pub const BLOCKS_PER_SECTION: usize = SECTION_HEIGHT * SECTION_WIDTH * SECTION_WIDTH;

/// A [`Chunk`] is a 16x256x16 chunk of blocks. It is split vertically into 16 chunk
/// sections (see [`ChunkSection`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_x: i32,
    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_z: i32,
    /// Data for this chunk.
    pub data: ChunkData,
    // TODO: block entities
}

impl Chunk {
    pub fn empty(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            data: Default::default(),
        }
    }
}

/// The actual data for a chunk, in its fully decoded form.
///
/// Chunk data can either be full or partial. In the former case, this
/// represents a new chunk that the client should load. In the latter case, it
/// serves as a big multi-block delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkData {
    Full {
        /// List of non-empty sections in this chunk, in increasing Y order.
        sections: Vec<ChunkSection>,
        /// Grid of biome IDs dictating which biome a given vertical X,Z slice
        /// is part of.
        biomes: Box<Biomes>,
    },

    Delta {
        /// List of sections that have changed, in increasing Y order.
        sections: Vec<ChunkSection>,
    },
}

impl ChunkData {
    /// Returns whether or not this contains the full data for a chunk or if
    /// it's just a delta.
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full { .. })
    }

    pub fn sections(&self) -> &[ChunkSection] {
        match self {
            ChunkData::Full { sections, .. } => sections,
            ChunkData::Delta { sections } => sections,
        }
    }

    pub fn sections_mut(&mut self) -> &mut [ChunkSection] {
        match self {
            ChunkData::Full { sections, .. } => sections,
            ChunkData::Delta { sections } => sections,
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::Delta {
            sections: Default::default(),
        }
    }
}

/// A [`ChunkSection`] is a 16x16x16 cubic section of a [`Chunk`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSection {
    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_y: u8,
    /// Number of non-air blocks present in the chunk section, for lighting
    /// purposes. "Non-air" is defined as any block other than air, cave air,
    /// and void air (in particular, note that fluids such as water are still
    /// counted).
    pub block_count: u16,
    /// The block state for every block in the chunk section.
    pub block_states: BlockStates,
}

impl ChunkSection {
    pub fn empty(chunk_y: u8) -> Self {
        Self {
            chunk_y,
            block_count: 0,
            block_states: Default::default(),
        }
    }
}

/// The block state for every block in a [`ChunkSection`], stored in
/// Y-Z-X-major order. In other words, an array of flat Z-X slices in increasing
/// Y order.
#[derive(Clone, PartialEq, Eq)]
pub struct BlockStates(pub [BlockState; BLOCKS_PER_SECTION]);

impl BlockStates {
    #[inline]
    pub fn iter(&self) -> BlockIter<'_> {
        BlockIter::new(self)
    }
}

impl Default for BlockStates {
    fn default() -> Self {
        Self([BlockState::AIR; BLOCKS_PER_SECTION])
    }
}

impl fmt::Debug for BlockStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BlockStates").field(&"...").finish()
    }
}

/// Iterator through a [`ChunkSection`]'s block states.
pub struct BlockIter<'a> {
    block_states: &'a BlockStates,
    cur_index: usize,
}

impl<'a> BlockIter<'a> {
    fn new(block_states: &'a BlockStates) -> Self {
        Self {
            block_states,
            cur_index: 0,
        }
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = (u8, u8, u8, BlockState);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index >= SECTION_HEIGHT * SECTION_WIDTH * SECTION_WIDTH {
            return None;
        }

        // Y-Z-X-major order, 4 bits per axis.
        const Y_SHIFT: usize = 8;
        const Z_SHIFT: usize = 4;
        const X_SHIFT: usize = 0;
        const Y_MASK: usize = 0b1111 << Y_SHIFT;
        const Z_MASK: usize = 0b1111 << Z_SHIFT;
        const X_MASK: usize = 0b1111 << X_SHIFT;

        let x = (self.cur_index & X_MASK) >> X_SHIFT;
        let y = (self.cur_index & Y_MASK) >> Y_SHIFT;
        let z = (self.cur_index & Z_MASK) >> Z_SHIFT;

        let next = (
            x as u8,
            y as u8,
            z as u8,
            self.block_states.0[self.cur_index],
        );

        self.cur_index += 1;

        Some(next)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState(pub u32);

impl BlockState {
    pub const AIR: Self = Self(0);

    /// See <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Direct>.
    pub const MAX_BLOCK_STATES_LOG_2: usize = 14;
}

/// Grid of biome IDs dictating which biome a given vertical X,Z slice of a
/// [`Chunk`] is part of.
#[derive(Clone, PartialEq, Eq)]
pub struct Biomes([BiomeId; SECTION_WIDTH * SECTION_WIDTH]);

impl Default for Biomes {
    fn default() -> Self {
        Self([BiomeId::VOID; SECTION_WIDTH * SECTION_WIDTH])
    }
}

impl fmt::Debug for Biomes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Biomes").field(&"...").finish()
    }
}

/// Unique identifier for a biome.
///
/// See <https://minecraft.fandom.com/wiki/Biome/ID?oldid=1278248>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BiomeId(pub u16);

impl BiomeId {
    pub const VOID: Self = Self(127);
}
