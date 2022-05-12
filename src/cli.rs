use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Insert a value of the given name.
    Insert { name: String, key_id: String },

    /// Retrieve the value of the given name.
    Retrieve { name: String },
}
