use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Insert a value at the given key.
    Insert { name: String, key_id: String },
}
