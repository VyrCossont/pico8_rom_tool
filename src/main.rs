mod music;
mod rom;
mod sfx;

use anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use packed_struct::prelude::*;
use std::path::PathBuf;

/// Tool for working with resources in PICO-8 ROMs.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Read a PICO-8 ROM and dump the output in human-readable form.
    Dump {
        #[clap(value_enum)]
        section: Section,
        #[clap(value_parser)]
        path: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum Section {
    Music,
    Sfx,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Dump {
            section: Section::Music,
            path,
        } => music::dump(path.as_path())?,
        Commands::Dump {
            section: Section::Sfx,
            path,
        } => sfx::dump(path.as_path())?,
    }
    Ok(())
}
