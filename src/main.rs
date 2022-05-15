pub mod cli;
pub mod gpg;
pub mod input;
mod output;
pub mod secrets;
pub mod store;

use clap::Parser;
use cli::Args;
use input::{handle, Handler};
use output::{write_result, TerminalOutput};
use secrets::StdinSecretReader;
use std::io;
use store::OnDiskStore;

fn main() -> anyhow::Result<()> {
    let output = TerminalOutput::new(io::stdout());
    let mut handler = Handler::new(OnDiskStore::new("~/.sec_store"), StdinSecretReader);
    let result = handle(&mut handler, &Args::parse())?;
    write_result(result, output)
}
