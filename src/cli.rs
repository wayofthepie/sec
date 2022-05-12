use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Insert a value of the given name.
    Insert {
        /// The name of the entry - this is a file path, a file will be created at the given
        /// path
        name: String,
        /// The key id for the key used to encrypt this entry
        key_id: String,
    },

    /// Retrieve the value of the given name.
    Retrieve {
        /// The name of the secret to retrieve. Like insert, this should be a file path to an
        /// existing entry.
        name: String,
    },
}
