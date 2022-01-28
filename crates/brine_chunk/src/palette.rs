//! A palette maps numeric IDs to block states.
//!
//! The concept is more commonly used with colors in an image. In Minecraft,
//! palettes are used to reduce the amount of data needed to represent a chunk
//! of block states. This is done by leveraging the fact that a single chunk
//! section will most likely have only a few different block types in it. So we
//! can use only a few bits for each block's id and combine that with some
//! mapping from id to full block state.
//!
//! ## The Global Palette
//!
//! For a given version of Minecraft, there is the notion of the **Global
//! Palette**. This is mapping of IDs to every possible block state in the game.
//!
//! ## Section Palettes
//!
//! No single chunk section can contain blocks in all possible block states
//! (there are too many to fit in one section), and most chunk sections contain
//! only a few unique block states.
//!
//! For this reason, most chunk sections come with their own palette which
//! serves as a middle-man between the compacted IDs in the data blob and the
//! global palette.
//!
//! If a section has its own palette, then the compacted IDs are translated
//! **twice**. First, they are translated through the section palette to get
//! expanded IDs, then they are translated again through the global palette to
//! get block states. The [`SectionPalette`] struct does this double-translation
//! internally.
//!
//! See also
//! <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Global_and_section_palettes>.

use std::{fmt, io};

use tracing::trace;

use crate::{
    decode::{Result, VarIntRead},
    BlockState,
};

/// Trait representing a block state palette.
pub trait Palette {
    fn id_to_block_state(&self, id: u32) -> Option<BlockState>;
}

/// The palette of block states for a given [`ChunkSection`][crate::ChunkSection].
///
/// See <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Palettes>.
#[derive(Default)]
pub struct SectionPalette {
    id_to_block_state: Vec<BlockState>,
}

impl SectionPalette {
    /// The maximum value of the `bits_per_block` field for which a section
    /// palette is used rather than directly using the global palette.
    pub const MAX_BITS_PER_BLOCK: u8 = 8;

    /// Decodes a chunk section's palette from a data blob.
    ///
    /// See <https://wiki.vg/index.php?title=Chunk_Format&oldid=14901#Palettes>
    pub fn decode(global_palette: &impl Palette, data: &mut impl io::Read) -> Result<Self> {
        trace!("SectionPalette::decode");

        let palette_length: usize = data.read_var_i32()?.try_into()?;
        trace!("palette_length: {}", palette_length);

        let mut id_to_block_state = Vec::with_capacity(palette_length);
        for _ in 0..palette_length {
            let expanded_id: u32 = data.read_var_i32()?.try_into()?;
            let block_state = global_palette.id_to_block_state(expanded_id).unwrap();
            id_to_block_state.push(block_state);
        }

        Ok(Self { id_to_block_state })
    }
}

impl Palette for SectionPalette {
    fn id_to_block_state(&self, id: u32) -> Option<BlockState> {
        self.id_to_block_state.get(id as usize).copied()
    }
}

impl fmt::Debug for SectionPalette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(
                self.id_to_block_state
                    .iter()
                    .map(|block_state| block_state.0),
            )
            .finish()
    }
}
