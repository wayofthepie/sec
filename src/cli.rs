use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    Initialize,
    /// Insert a value of the given name.
    Insert {
        /// name of the entry
        name: String,
        /// key id for the key used to encrypt this entry
        key_id: String,
    },

    /// Retrieve the value of the given name.
    Retrieve {
        /// name of the secret to retrieve
        name: String,
    },
}
