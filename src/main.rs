mod gpg;
use anyhow::anyhow;
use clap::Parser;
use gpg::Gpg;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    validate(&args)?;
    let gpg = Gpg::new();
    let mut reader = BufReader::new(File::open(args.path)?);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let plaintext = gpg.decrypt(&buffer)?;
    println!("{}", plaintext);
    Ok(())
}

pub fn validate(args: &Args) -> anyhow::Result<()> {
    if !args.path.exists() {
        // TODO paths are not utf8, we should deal with that here explicitly
        // either fail, or handle them correctly
        return Err(anyhow!(
            r#"The path "{}" does not exist!"#,
            args.path.to_str().unwrap()
        ));
    }
    Ok(())
}
