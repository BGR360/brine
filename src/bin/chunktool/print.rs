use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use brine::chunk::{load_chunk, Result};
use brine_chunk::{Chunk, ChunkSection};
use brine_data::{
    blocks::{BlockStateId, StateValue},
    MinecraftData,
};

/// Prints a summary of a chunk loaded from disk.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,

    /// Show detailed information for a specific chunk section.
    #[clap(short, long)]
    section: Option<usize>,
}

pub(crate) fn main(args: Args) {
    match print_chunk_from_file(&args.file, args.section) {
        Ok(()) => {}
        Err(e) => println!("ERROR: {}", e),
    }
}

fn print_chunk_from_file(path: &Path, section: Option<usize>) -> Result<()> {
    let data = MinecraftData::for_version("1.14.4");
    let chunk = load_chunk(path)?;

    let printer = ChunkPrinter { data, chunk };

    printer.print_chunk(section);

    Ok(())
}

pub struct ChunkPrinter {
    data: MinecraftData,
    chunk: Chunk,
}

impl ChunkPrinter {
    fn print_chunk(&self, section: Option<usize>) {
        let section_ys = self
            .chunk
            .sections
            .iter()
            .map(|section| section.chunk_y)
            .collect::<Vec<_>>();

        println!();
        println!("================= CHUNK =================");
        println!();
        println!(
            "Position: x = {}, z = {}",
            self.chunk.chunk_x, self.chunk.chunk_z
        );
        println!();
        println!("{} Sections:", section_ys.len());
        println!("{:?}", section_ys);

        if let Some(section_y) = section {
            let section = self
                .chunk
                .sections
                .iter()
                .find(|section| section.chunk_y as usize == section_y)
                .expect("Chunk has no section at that y-height");

            self.print_section(section, true);
        } else {
            for section in self.chunk.sections.iter().rev() {
                self.print_section(section, false);
            }
        }
    }

    fn print_section(&self, section: &ChunkSection, detailed: bool) {
        println!();
        println!("=============== Section ===============");
        println!();
        println!("Position: y = {}", section.chunk_y);
        println!();
        println!("{} Blocks:", section.block_count);
        println!();

        let mut entries = self.block_counts(section).into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(_, count)| *count);

        for (block_state, count) in entries.into_iter().rev() {
            let block = self
                .data
                .blocks()
                .get_by_state_id(BlockStateId(block_state.0 as u16));

            let name = block
                .as_ref()
                .map(|block| block.display_name)
                .unwrap_or_else(|| "");

            println!("{block_state:5?} ({name:10}): {count}");

            if detailed {
                if let Some(block) = block.as_ref() {
                    let mut states: Vec<(&str, StateValue)> = block
                        .state
                        .iter()
                        .map(|(name, state)| (*name, *state))
                        .collect();

                    states.sort_by_key(|(name, _)| *name);

                    for (state, value) in states.iter() {
                        println!("{:?} = {:?}", state, value);
                    }
                    println!();
                }
            }
        }
    }

    fn block_counts(&self, section: &ChunkSection) -> HashMap<BlockStateId, usize> {
        let mut counts = HashMap::new();

        for (_x, _y, _z, block_state) in section.block_states.iter() {
            let block_state_id = BlockStateId(block_state.0 as u16);
            *counts.entry(block_state_id).or_insert(0) += 1;
        }

        counts
    }
}
