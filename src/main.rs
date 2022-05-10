mod gpg;
use anyhow::anyhow;
use clap::Parser;
use gpg::Gpg;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Insert a value at the given key.
    Insert { key: String },
}

fn main() -> anyhow::Result<()> {
    initialize(&Args::parse())
}

pub fn initialize(args: &Args) -> anyhow::Result<()> {
    match &args.action {
        _ => println!("nada"),
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{initialize, Args};
}
