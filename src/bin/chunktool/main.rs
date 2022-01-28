mod print;
mod save;
mod view;

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
    Print(print::Args),
    Save(save::Args),
    View(view::Args),
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Print(args) => print::main(args),
        Subcommand::Save(args) => save::main(args),
        Subcommand::View(args) => view::main(args),
    }
}
