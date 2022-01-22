//! A library for decoding Minecraft chunk data from network packets.
//!
//! Currently only supports version 1.14.4.

use std::{fmt, io, num::TryFromIntError};

use byteorder::{BigEndian, ReadBytesExt};
use tracing::trace;

pub mod packed_vec;
pub mod palette;
mod varint;

pub use palette::{Palette, SectionPalette};
use varint::VarIntRead;

pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_WIDTH: usize = 16;
pub const SECTION_HEIGHT: usize = 16;
pub const SECTION_WIDTH: usize = CHUNK_WIDTH;
pub const SECTIONS_PER_CHUNK: usize = CHUNK_HEIGHT / SECTION_HEIGHT;
pub const BLOCKS_PER_SECTION: usize = SECTION_HEIGHT * SECTION_WIDTH * SECTION_WIDTH;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    InvalidInt(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;

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

    /// Decodes a chunk from data provided by a Minecraft protocol packet.
    ///
    /// The `primary_bit_mask` indicates which chunk sections are included in
    /// the data blob. A `1` bit indicates that the chunk section is included;
    /// the least significant bit is for the lowest section (i.e., Y=0).
    ///
    /// The `full_chunk` boolean indicates whether the data blob includes the
    /// full data of a chunk.
    ///
    /// * If `full_chunk` is true, then the blob includes biome data and all
    ///   (non-empty) sections in the chunk. Sections not specified in the
    ///   primary bit mask are empty sections.
    ///
    /// * If `full_chunk` is false, then the blob contains only chunk sections
    ///   that have changed (basically a big multi-block delta). Sections not
    ///   specified in the primary bit mask are not changed and should be left
    ///   as-is. Biome data is *not* included.
    ///
    /// The `global_palette` is needed in order to perform translations from
    /// compacted block state IDs to full block states. See the [`palette`]
    /// module for more information on palettes.
    ///
    /// See:
    /// * <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Packet_structure>
    /// * <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Data_structure>
    pub fn decode(
        chunk_x: i32,
        chunk_z: i32,
        full_chunk: bool,
        primary_bit_mask: u16,
        global_palette: &impl Palette,
        data: &mut impl io::Read,
    ) -> Result<Self> {
        trace!("Chunk::decode");

        // Blob will always contain chunk sections.
        let sections = Self::decode_chunk_sections(primary_bit_mask, global_palette, data)?;

        let data = if full_chunk {
            let biomes = Box::new(Biomes::decode(data)?);

            ChunkData::Full { sections, biomes }
        } else {
            ChunkData::Delta { sections }
        };

        Ok(Self {
            chunk_x,
            chunk_z,
            data,
        })
    }

    /// Decodes a list of [`ChunkSection`]s from a data blob.
    pub fn decode_chunk_sections(
        primary_bit_mask: u16,
        global_palette: &impl Palette,
        data: &mut impl io::Read,
    ) -> Result<Vec<ChunkSection>> {
        trace!("ChunkSection::decode_chunk_sections");

        let section_ys = Self::bitmask_to_section_y_coordinates(primary_bit_mask);
        trace!("section_ys: {:?}", &section_ys);

        let mut sections = Vec::new();
        for section_y in section_ys {
            sections.push(ChunkSection::decode(section_y, global_palette, data)?);
        }

        Ok(sections)
    }

    /// Given a bitmask, returns which chunk section y-coordinates correspond to
    /// the chunk sections in the data blob.
    ///
    /// See also
    /// <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Empty_sections_and_the_primary_bit_mask>
    pub fn bitmask_to_section_y_coordinates(bitmask: u16) -> Vec<u8> {
        let mut y_coords = Vec::new();
        for i in 0..SECTIONS_PER_CHUNK {
            if (bitmask & (1 << i)) != 0 {
                y_coords.push(i as u8);
            }
        }
        y_coords
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

    /// Decodes a chunk section from a data blob.
    ///
    /// The `global_palette` is needed in order to perform translations from
    /// compacted block state IDs to full block states. See the [`palette`]
    /// module for more information on palettes.
    ///
    /// See also
    /// <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Chunk_Section_structure>
    pub fn decode(
        chunk_y: u8,
        global_palette: &impl Palette,
        data: &mut impl io::Read,
    ) -> Result<Self> {
        trace!("ChunkSection::decode");
        let block_count = data.read_i16::<BigEndian>()?.try_into()?;

        let bits_per_block = data.read_u8()?;
        trace!("bits_per_block: {}", bits_per_block);

        let block_states = if bits_per_block <= SectionPalette::MAX_BITS_PER_BLOCK {
            let palette = SectionPalette::decode(global_palette, data)?;

            trace!("palette: {:?}", &palette);

            BlockStates::decode(bits_per_block, &palette, data)?
        } else {
            BlockStates::decode(bits_per_block, global_palette, data)?
        };

        Ok(Self {
            chunk_y,
            block_count,
            block_states,
        })
    }
}

/// The block state for every block in a [`ChunkSection`], stored in
/// Y-Z-X-major order. In other words, an array of flat Z-X slices in increasing
/// Y order.
#[derive(Clone, PartialEq, Eq)]
pub struct BlockStates(pub [BlockState; BLOCKS_PER_SECTION]);

impl BlockStates {
    /// See <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Compacted_data_array>.
    pub fn decode(
        _bits_per_block: u8,
        _palette: &impl Palette,
        data: &mut impl io::Read,
    ) -> Result<Self> {
        trace!("BlockStates::decode");

        let array_length = data.read_var_i32()?;
        trace!("array_length: {}", array_length);

        let mut longs = Vec::<u64>::with_capacity(array_length.try_into()?);
        for _ in 0..array_length {
            longs.push(data.read_u64::<BigEndian>()?);
        }

        // TODO: read actual block states.

        Ok(Default::default())
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

impl Biomes {
    pub fn decode(_data: &mut impl io::Read) -> Result<Self> {
        // TODO
        Ok(Default::default())
    }
}

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
