mod core;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: TCtrlCommand,
}

#[derive(Subcommand, Debug)]
enum TCtrlCommand {
    #[command(about = "Prints true or false to stdout.")]
    InTmux,
    #[command(about = "Opens a project, using a provided path.")]
    Open { path: PathBuf },
}

fn main() -> Result<()> {
    let args = Args::parse();

    args.command.run()?;

    Ok(())
}

impl TCtrlCommand {
    fn run(&self) -> Result<()> {
        match self {
            TCtrlCommand::InTmux => {
                let in_tmux = core::in_tmux();
                println!("{}", in_tmux);
                Ok(())
            },
            TCtrlCommand::Open { path } => {
                core::open(path)?;
                Ok(())
            },
        }
    }
}
