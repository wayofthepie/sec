pub mod cli;
pub mod gpg;
mod handle;

use clap::Parser;
use cli::Args;
use handle::handle;
use handle::OnDiskPersister;
use handle::{Handler, HandlerResult};

fn main() -> anyhow::Result<()> {
    let input = b"test";
    let handler = Handler::new("".to_owned(), OnDiskPersister::new(), &input[..]);
    match handle(handler, &Args::parse())? {
        HandlerResult::Insert(_) => println!("Success"),
    }
    Ok(())
}
