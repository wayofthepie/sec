pub mod cli;
pub mod gpg;
pub mod input;
mod output;

use std::io;

use clap::Parser;
use cli::Args;
use input::{handle, Handler, OnDiskPersister, StdinSecretReader};
use output::{write_result, TerminalOutput};

fn main() -> anyhow::Result<()> {
    let output = TerminalOutput::new(io::stdout());
    let mut handler = Handler::new(OnDiskPersister::new(), StdinSecretReader);
    let result = handle(&mut handler, &Args::parse())?;
    write_result(result, output)
}
