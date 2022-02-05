mod print;

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
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Print(args) => print::main(args),
    }
}
