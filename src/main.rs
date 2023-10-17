use std::process::Command;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: TCtrlCommand,
}

#[derive(Subcommand, Debug)]
enum TCtrlCommand {}

macro_rules! cmd {
    ($name:expr, $args:expr) => {
        Command::new($name).args($args)
    };
}

fn main() -> Result<()> {
    let out = cmd!("tmux", &["-V"]).get_stdout()?;
    println!("tmux version: {}", out);

    let _args = Args::parse();

    Ok(())
}

trait Asdf {
    fn get_stdout(&mut self) -> Result<String>;
}

impl Asdf for Command {
    fn get_stdout(&mut self) -> Result<String> {
        let out = self.output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}
