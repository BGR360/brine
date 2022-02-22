mod print;
mod view;

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

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Print(args) => print::main(args),
        Subcommand::View(args) => view::main(args),
    }
}
