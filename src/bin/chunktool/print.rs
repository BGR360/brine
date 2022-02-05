use std::{collections::HashMap, path::PathBuf};

use brine::chunk::{load_chunk, Result};
use brine_chunk::{BlockState, ChunkSection};
use brine_data::{blocks::BlockStateId, MinecraftData};

/// Prints a summary of a chunk loaded from disk.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,
}

pub(crate) fn main(args: Args) {
    match print_chunk(&args) {
        Ok(()) => {}
        Err(e) => println!("ERROR: {}", e),
    }
}

fn print_chunk(args: &Args) -> Result<()> {
    let data = MinecraftData::for_version("1.14.4");

    let chunk = load_chunk(&args.file)?;

    let section_ys = chunk
        .sections
        .iter()
        .map(|section| section.chunk_y)
        .collect::<Vec<_>>();

    println!("================= CHUNK =================");
    println!();
    println!("Position: x = {}, z = {}", chunk.chunk_x, chunk.chunk_z);
    println!();
    println!("{} Sections:", section_ys.len());
    println!("{:?}", section_ys);

    for section in chunk.sections.iter().rev() {
        println!();
        println!("=============== Section ===============");
        println!();
        println!("Position: y = {}", section.chunk_y);
        println!();
        println!("{} Blocks:", section.block_count);
        println!();

        let mut entries = block_counts(section).into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(_, count)| *count);

        for (block_state, count) in entries.into_iter().rev() {
            let name = data
                .blocks()
                .get_by_state_id(BlockStateId(block_state.0 as u16))
                .map(|block| &block.display_name[..])
                .unwrap_or_else(|| "");

            println!("{block_state:5?} ({name:10}): {count}");
        }
    }

    Ok(())
}

fn block_counts(section: &ChunkSection) -> HashMap<BlockState, usize> {
    let mut counts = HashMap::new();

    for (_x, _y, _z, block_state) in section.block_states.iter() {
        *counts.entry(block_state).or_insert(0) += 1;
    }

    counts
}
