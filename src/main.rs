pub mod cli;
pub mod gpg;
mod handle;

use clap::Parser;
use cli::Args;
use handle::{handle, Handler, HandlerResult, OnDiskPersister, StdinSecretReader};

fn main() -> anyhow::Result<()> {
    let handler = Handler::new(OnDiskPersister::new(), StdinSecretReader);
    match handle(handler, &Args::parse())? {
        HandlerResult::Insert(_) => println!("Secret saved."),
    }
    Ok(())
}
