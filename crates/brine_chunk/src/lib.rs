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
///
/// This structure can either represent the full data of a chunk (i.e., when it
/// is first loaded into the game), or it can represent a delta, in which case
/// some information may be missing as noted in the fields' documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_x: i32,

    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_z: i32,

    /// List of non-empty sections in this chunk, in increasing Y order.
    ///
    /// If this is not the full data of a chunk, this may not include all
    /// non-empty sections in the chunk.
    pub sections: Vec<ChunkSection>,

    /// Grid of biome IDs indicating which biome each vertical slice is part of.
    ///
    /// If this is not the full data of a chunk, this is not included.
    pub biomes: Option<Box<Biomes>>,
    // TODO: block entities
}

impl Chunk {
    pub fn empty(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            sections: Vec::new(),
            biomes: Some(Box::new(Biomes::default())),
        }
    }

    pub fn empty_delta(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            biomes: None,
            ..Self::empty(chunk_x, chunk_z)
        }
    }

    /// Returns whether or not this contains the full data for a chunk or if
    /// it's just a delta.
    pub fn is_full(&self) -> bool {
        self.biomes.is_some()
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

    #[inline]
    pub fn get_block<K>(&self, key: K) -> Result<BlockState, <K as TryInto<SectionKey>>::Error>
    where
        K: TryInto<SectionKey>,
    {
        let key = key.try_into()?;

        let SectionKey { x, y, z } = key;
        Ok(self.block_states.get_block(x, y, z))
    }
}

/// A [`SectionKey`] is used to index a single block in a [`ChunkSection`]
pub struct SectionKey {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl<T> TryFrom<[T; 3]> for SectionKey
where
    T: Copy,
    u8: TryFrom<T>,
{
    type Error = <u8 as TryFrom<T>>::Error;

    fn try_from(value: [T; 3]) -> Result<Self, Self::Error> {
        Ok(Self {
            x: TryFrom::try_from(value[0])?,
            y: TryFrom::try_from(value[1])?,
            z: TryFrom::try_from(value[2])?,
        })
    }
}

impl<T> TryFrom<(T, T, T)> for SectionKey
where
    u8: TryFrom<T>,
{
    type Error = <u8 as TryFrom<T>>::Error;

    fn try_from((x, y, z): (T, T, T)) -> Result<Self, Self::Error> {
        Ok(Self {
            x: TryFrom::try_from(x)?,
            y: TryFrom::try_from(y)?,
            z: TryFrom::try_from(z)?,
        })
    }
}

/// The block state for every block in a [`ChunkSection`], stored in
/// Y-Z-X-major order. In other words, an array of flat Z-X slices in increasing
/// Y order.
#[derive(Clone, PartialEq, Eq)]
pub struct BlockStates(pub [BlockState; BLOCKS_PER_SECTION]);

impl BlockStates {
    // Y-Z-X-major order, 4 bits per axis.
    const Y_SHIFT: usize = 8;
    const Z_SHIFT: usize = 4;
    const X_SHIFT: usize = 0;
    const Y_MASK: usize = 0b1111 << Self::Y_SHIFT;
    const Z_MASK: usize = 0b1111 << Self::Z_SHIFT;
    const X_MASK: usize = 0b1111 << Self::X_SHIFT;

    #[inline]
    pub fn iter(&self) -> BlockIter<'_> {
        BlockIter::new(self)
    }

    #[inline]
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockState {
        self.0[Self::xyz_to_index(x, y, z)]
    }

    #[inline]
    pub fn xyz_to_index(x: u8, y: u8, z: u8) -> usize {
        ((x as usize) << Self::X_SHIFT)
            + ((y as usize) << Self::Y_SHIFT)
            + ((z as usize) << Self::Z_SHIFT)
    }

    #[inline]
    pub fn index_to_xyz(index: usize) -> (u8, u8, u8) {
        let x = (index & Self::X_MASK) >> Self::X_SHIFT;
        let y = (index & Self::Y_MASK) >> Self::Y_SHIFT;
        let z = (index & Self::Z_MASK) >> Self::Z_SHIFT;
        (x as u8, y as u8, z as u8)
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

        let (x, y, z) = BlockStates::index_to_xyz(self.cur_index);

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
