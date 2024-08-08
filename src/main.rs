pub mod commands;
use anyhow::Error;
use clap::{Args, Parser, Subcommand};
use commands::{build, init};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Build
    // Deploy, TODO: Add deployment commands
}

#[derive(Args)]
struct InitArgs {
    name: Option<String>,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => init(args.name.clone()),
        Commands::Build => build()
    }
}