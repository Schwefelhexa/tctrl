mod config;
mod core;

use std::{io::Cursor, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use skim::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: TCtrlCommand,

    #[arg(short, long, global = true, help = "A custom configuration file to use. Takes priority over all other config files.")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum TCtrlCommand {
    #[command(about = "Prints true or false to stdout.")]
    InTmux,
    #[command(about = "Opens a project, using a provided path.")]
    Open {
        path: Option<PathBuf>,
        #[arg(short = 't', long, help = "The client to open the project in.")]
        client: Option<String>,
        #[arg(short, long, help = "The name of the project. If not provided, will call lua config.")]
        name: Option<String>,
    },
    #[command(about = "Print the default configuration.")]
    PrintDefaultConfig,
}

fn main() -> Result<()> {
    let args = Args::parse();

    args.command.run(&args)?;

    Ok(())
}

impl TCtrlCommand {
    fn run(&self, global: &Args) -> Result<()> {
        let config = Config::load(global.config.as_ref())?;

        match self {
            TCtrlCommand::InTmux => {
                let in_tmux = core::in_tmux();
                println!("{}", in_tmux);
                Ok(())
            }
            TCtrlCommand::Open { path, client, name } => {
                match path {
                    Some(path) => core::open(path, client.as_deref(), name.as_deref(), &config)?,
                    None => {
                        let path = prompt_for_path(&config)?;
                        core::open(&path, client.as_deref(), name.as_deref(), &config)?
                    }
                };

                Ok(())
            }
            TCtrlCommand::PrintDefaultConfig => {
                println!("{}", config::get_default_config());
                Ok(())
            }
        }
    }
}

fn prompt_for_path(config: &Config) -> Result<PathBuf> {
    let options = SkimOptionsBuilder::default()
        .build()
        .expect("Failed to create SkimOptionsBuilder");

    let suggestions = config.list_projects()?;
    let suggestions = suggestions
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n");

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
