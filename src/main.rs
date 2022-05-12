pub mod cli;
pub mod gpg;
pub mod input;
mod output;
pub mod secrets;

use clap::Parser;
use cli::Args;
use input::{handle, Handler, OnDiskPersister};
use output::{write_result, TerminalOutput};
use secrets::StdinSecretReader;
use std::io;

fn main() -> anyhow::Result<()> {
    let output = TerminalOutput::new(io::stdout());
    let mut handler = Handler::new(OnDiskPersister::new(), StdinSecretReader);
    let result = handle(&mut handler, &Args::parse())?;
    write_result(result, output)
}
