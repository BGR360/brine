use brine_data::{blocks::BlockStateId, MinecraftData};

/// Prints information about a given block.
#[derive(clap::Args)]
pub struct Args {
    /// Block state id.
    #[clap(short, long)]
    state_id: u16,
}

pub(crate) fn main(args: Args) {
    print_block(BlockStateId(args.state_id));
}

fn print_block(block_state_id: BlockStateId) {
    let data = MinecraftData::for_version("1.14.4");

    let block = data
        .blocks()
        .get_by_state_id(block_state_id)
        .expect("no such block");

    println!("{:#?}", block);
}
