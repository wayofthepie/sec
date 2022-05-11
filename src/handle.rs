use crate::cli::{Action, Args};
use crate::gpg::Gpg;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

pub fn handle<R: Read, P: Persister>(
    mut handler: Handler<R, P>,
    args: &Args,
) -> anyhow::Result<HandlerResult> {
    match &args.action {
        Action::Insert { key } => handler.insert(key),
    }
}

pub enum HandlerResult {
    Insert(File),
}

pub struct Handler<R, P> {
    key_id: String,
    gpg: Gpg,
    persister: P,
    reader: R,
}

impl<R: Read, P: Persister> Handler<R, P> {
    pub fn new(key_id: String, persister: P, reader: R) -> Self {
        Self {
            key_id,
            gpg: Gpg::default(),
            persister,
            reader,
        }
    }

    pub fn insert(&mut self, key: &str) -> anyhow::Result<HandlerResult> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        let ciphertext = self.gpg.encrypt(&self.key_id, &buf)?;
        let mut file = self.persister.create(key)?;
        file.write_all(&ciphertext)?;
        Ok(HandlerResult::Insert(file))
    }
}

pub trait Persister {
    fn create(&self, name: &str) -> anyhow::Result<File>;
}

pub struct OnDiskPersister;

impl OnDiskPersister {
    pub fn new() -> Self {
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
        cli::Action,
        gpg::{
            test::{import_keys, GPG_KEY_ID},
            Gpg,
        },
        handle::{handle, OnDiskPersister},
        Args, Handler, HandlerResult,
    };
    use memfile::CreateOptions;
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::Path,
    };
    use tempfile::tempdir;

    use super::Persister;

    struct InMemoryPersister;

    impl Persister for InMemoryPersister {
        fn create(&self, name: &str) -> anyhow::Result<File> {
            Ok(CreateOptions::new().create(name)?.into_file())
        }
    }

    #[test]
    fn osdiskpersister_should_create_file() {
        let tmpdir = tempdir().unwrap();
        let dir = tmpdir.path().to_str().unwrap();
        let path = format!("{dir}/test.file");
        let persister = Box::new(OnDiskPersister::new());
        persister.create(&path).unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn should_encrypt_and_write_the_given_value() {
        import_keys();
        let gpg = Gpg::new();
        let key = "key".to_owned();
        let args = Args {
            action: Action::Insert { key },
        };
        let mut buf = Vec::new();
        let input = "value";
        let handler = Handler::new(GPG_KEY_ID.to_owned(), InMemoryPersister, input.as_bytes());
        let HandlerResult::Insert(mut file) = handle(handler, &args).expect("expected a result");
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut buf).unwrap();
        let plaintext = gpg.decrypt(&buf).unwrap();
        assert_eq!(plaintext, input);
    }
}
