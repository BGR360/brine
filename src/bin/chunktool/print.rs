use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use brine::chunk::{load_chunk, Result};
use brine_chunk::{BlockState, ChunkSection};

/// Prints a summary of a chunk loaded from disk.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,
}

pub(crate) fn main(args: Args) {
    match print_chunk(&args.file) {
        Ok(()) => {}
        Err(e) => println!("ERROR: {}", e),
    }
}

fn print_chunk(path: &Path) -> Result<()> {
    let chunk = load_chunk(path)?;

    let section_ys = chunk
        .data
        .sections()
        .iter()
        .map(|section| section.chunk_y)
        .collect::<Vec<_>>();

    println!("================= CHUNK =================");
    println!();
    println!("Position: x = {}, z = {}", chunk.chunk_x, chunk.chunk_z);
    println!();
    println!("{} Sections:", section_ys.len());
    println!("{:?}", section_ys);

    for section in chunk.data.sections().iter().rev() {
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
            println!("{:?}: {}", block_state, count);
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
