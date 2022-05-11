mod gpg;
use clap::Parser;
use gpg::Gpg;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Insert a value at the given key.
    Insert { key: String },
}

fn main() -> anyhow::Result<()> {
    let handler = Handler::new("".to_owned(), Box::new(OnDiskPersister));
    match handle(handler, &Args::parse())? {
        HandlerResult::Insert(_) => println!("Success"),
    }
    Ok(())
}

fn handle(handler: Handler, args: &Args) -> anyhow::Result<HandlerResult> {
    match &args.action {
        // TODO get value from stdin
        Action::Insert { key } => handler.insert(key, "test"),
    }
}

enum HandlerResult {
    Insert(File),
}

struct Handler {
    key_id: String,
    gpg: Gpg,
    persister: Box<dyn Persister>,
}

impl Handler {
    pub fn new(key_id: String, persister: Box<dyn Persister>) -> Self {
        Self {
            key_id,
            gpg: Gpg::new(),
            persister,
        }
    }

    pub fn insert(&self, key: &str, value: &str) -> anyhow::Result<HandlerResult> {
        let ciphertext = self.gpg.encrypt(&self.key_id, value.as_bytes())?;
        let mut file = self.persister.create(key)?;
        file.write_all(&ciphertext)?;
        Ok(HandlerResult::Insert(file))
    }
}

trait Persister {
    fn create(&self, name: &str) -> anyhow::Result<File>;
}

struct OnDiskPersister;

impl OnDiskPersister {
    fn new() -> Self {
        Self
    }
}

impl Persister for OnDiskPersister {
    fn create(&self, name: &str) -> anyhow::Result<File> {
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(name)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        gpg::{
            test::{import_keys, GPG_KEY_ID},
            Gpg,
        },
        Action, Args, Handler, HandlerResult, OnDiskPersister, Persister,
    };
    use memfile::CreateOptions;
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::Path,
    };
    use tempfile::tempdir;

    struct InMemoryPersister;

    impl Persister for InMemoryPersister {
        fn create(&self, name: &str) -> anyhow::Result<File> {
            Ok(CreateOptions::new().create(name)?.into_file())
        }
    }

    #[test]
    fn should_create_file() {
        let tmpdir = tempdir().unwrap();
        let dir = tmpdir.path().to_str().unwrap();
        let path = format!("{dir}/test.file");
        println!("{:#?}", path);
        let persister = Box::new(OnDiskPersister::new());
        persister.create(&path).unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn should_encrypt_and_write_the_given_value() {
        import_keys();
        let gpg = Gpg::new();
        let value = "test";
        let handler = Handler::new(GPG_KEY_ID.to_owned(), Box::new(InMemoryPersister));
        let mut buf = Vec::new();
        let HandlerResult::Insert(mut file) = handler.insert("test", value).expect("a result");
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut buf).unwrap();
        let plaintext = gpg.decrypt(&buf).unwrap();
        assert_eq!(plaintext, value);
    }
}
