pub mod cli;
pub mod fs;
pub mod gpg;
pub mod input;
mod output;
pub mod secrets;
pub mod store;

use clap::Parser;
use cli::Args;
use fs::FileSystemOperations;
use input::{handle, Handler, PASSWORD_STORE_DIRECTORY};
use output::{write_result, TerminalOutput};
use secrets::StdinSecretReader;
use std::io;
use store::OnDiskStore;

fn main() -> anyhow::Result<()> {
    let store_dir = format!("{}/{}", "/home/chaospie", PASSWORD_STORE_DIRECTORY);
    let output = TerminalOutput::new(io::stdout());
    let handler = Handler::new(
        OnDiskStore::new(store_dir),
        StdinSecretReader,
        FileSystemOperations,
    );
    let result = handle(&handler, &Args::parse())?;
    write_result(result, output)
}
