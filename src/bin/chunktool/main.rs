mod save;

use clap::Parser;

/// Utility application for debugging chunk building / rendering.
#[derive(Parser)]
#[clap(name = "chunktool")]
struct Args {
    #[clap(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Save(save::Args),
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Save(args) => save::main(args),
    }
}
