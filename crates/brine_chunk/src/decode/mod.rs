use std::{io, num::TryFromIntError};

use byteorder::{BigEndian, ReadBytesExt};
use tracing::trace;

use crate::{
    palette::{Palette, SectionPalette},
    Biomes, BlockState, BlockStates, Chunk, ChunkSection, BLOCKS_PER_SECTION, SECTIONS_PER_CHUNK,
};

mod packed_vec;
mod varint;

pub use packed_vec::PackedIntVec;
pub use varint::VarIntRead;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    InvalidInt(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Chunk {
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

        let biomes = if full_chunk {
            Some(Box::new(Biomes::decode(data)?))
        } else {
            None
        };

        Ok(Self {
            chunk_x,
            chunk_z,
            sections,
            biomes,
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

impl ChunkSection {
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

        // Protocol spec says any value below 4 should be treated as 4.
        let bits_per_block = if bits_per_block < 4 {
            4
        } else {
            bits_per_block
        };

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

impl BlockStates {
    /// See <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Compacted_data_array>.
    pub fn decode(
        bits_per_block: u8,
        palette: &impl Palette,
        data: &mut impl io::Read,
    ) -> Result<Self> {
        trace!("BlockStates::decode");

        let array_length = data.read_var_i32()?;
        trace!("array_length: {}", array_length);

        let mut longs = Vec::<u64>::with_capacity(array_length.try_into()?);
        for _ in 0..array_length {
            longs.push(data.read_u64::<BigEndian>()?);
        }

        let packed_vec_length = BLOCKS_PER_SECTION;
        let packed_vec =
            PackedIntVec::from_parts(longs, packed_vec_length, bits_per_block).unwrap();

        let block_states: Vec<BlockState> = packed_vec
            .iter()
            .map(|block_state_id| palette.id_to_block_state(block_state_id).unwrap())
            .collect();

        Ok(Self(block_states.try_into().unwrap()))
    }
}

impl Biomes {
    pub fn decode(_data: &mut impl io::Read) -> Result<Self> {
        // TODO
        Ok(Default::default())
    }
}
