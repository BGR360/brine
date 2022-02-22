#![allow(clippy::too_many_arguments)]

mod print;
mod view;

use brine_data::{BlockId, BlockStateId, MinecraftData};
use clap::Parser;

/// Utility application for looking at blocks.
#[derive(Parser)]
#[clap(name = "blocktool")]
struct Args {
    #[clap(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Print(print::Args),
    View(view::Args),
}

pub fn parse_block_reference(block_reference: &str, mc_data: &MinecraftData) -> Vec<BlockStateId> {
    if let Ok(id) = block_reference.parse::<u16>() {
        vec![BlockStateId(id)]
    } else if let Some(colon_position) = block_reference.chars().position(|c| c == ':') {
        let min_state = (&block_reference[..colon_position])
            .parse::<u16>()
            .expect("Invalid min state id");
        let max_state = (&block_reference[colon_position + 1..])
            .parse::<u16>()
            .expect("Invalid max state id");

        (min_state..max_state + 1).map(BlockStateId).collect()
    } else {
        let block = mc_data
            .blocks()
            .get_by_name(block_reference)
            .expect("No block with the provided name");
        let block_id = BlockId(block.id);

        mc_data
            .blocks()
            .iter_states_for_block(block_id)
            .unwrap()
            .map(|(block_state_id, _)| block_state_id)
            .collect()
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Print(args) => print::main(args),
        Subcommand::View(args) => view::main(args),
    }
}
