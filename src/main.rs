mod core;
mod config;

use std::{fs, io::Cursor, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use home::home_dir;
use skim::prelude::*;

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
    Open {
        path: Option<PathBuf>,
        #[arg(short, long, help = "The client to open the project in.")]
        client: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    args.command.run()?;

    Ok(())
}

impl TCtrlCommand {
    fn run(&self) -> Result<()> {
        let config = Config::load()?;

        match self {
            TCtrlCommand::InTmux => {
                let in_tmux = core::in_tmux();
                println!("{}", in_tmux);
                Ok(())
            }
            TCtrlCommand::Open { path, client } => {
                match path {
                    Some(path) => core::open(path, client.as_deref(), &config)?,
                    None => {
                        let path = prompt_for_path()?;
                        core::open(&path, client.as_deref(), &config)?
                    }
                };

                Ok(())
            }
        }
    }
}

fn prompt_for_path() -> Result<PathBuf> {
    let options = SkimOptionsBuilder::default()
        .build()
        .expect("Failed to create SkimOptionsBuilder");

    let search_folder = home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home dir"))?
        .join("Projects");
    let suggestions = fs::read_dir(search_folder)?
        .map(|f| fs::read_dir(f.unwrap().path()))
        .filter_map(|f| f.ok())
        .flatten()
        .filter_map(|f| f.ok())
        .map(|f| f.path().canonicalize())
        .filter_map(|f| f.ok())
        .map(|f| f.to_string_lossy().to_string());
    let suggestions = suggestions.collect::<Vec<_>>().join("\n");

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(suggestions));

    let path =
        Skim::run_with(&options, Some(items)).ok_or_else(|| anyhow::anyhow!("No path selected"))?;
    if path.is_abort {
        return Err(anyhow::anyhow!("Aborted"));
    }
    let path = path.selected_items.first().expect("Empty vec from Skim");

    let buf = PathBuf::from(path.output().to_string());
    Ok(buf)
}
