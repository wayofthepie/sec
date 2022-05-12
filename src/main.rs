pub mod cli;
pub mod gpg;
mod handle;

use clap::Parser;
use cli::Args;
use handle::handle;
use handle::OnDiskPersister;
use handle::{Handler, HandlerResult};

fn main() -> anyhow::Result<()> {
    let handler = Handler::new(OnDiskPersister::new(), std::io::stdin());
    match handle(handler, &Args::parse())? {
        HandlerResult::Insert(_) => println!("Success"),
    }
    Ok(())
}
