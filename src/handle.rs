use crate::cli::{Action, Args};
use crate::gpg::Gpg;
use anyhow::Context;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub fn handle<R: SecretReader, P: Persister>(
    mut handler: Handler<R, P>,
    args: &Args,
) -> anyhow::Result<HandlerResult> {
    match &args.action {
        Action::Insert { name: key, key_id } => handler.insert(key, key_id),
    }
}

pub enum HandlerResult {
    Insert(File),
}

pub struct Handler<R, P> {
    gpg: Gpg,
    persister: P,
    reader: R,
}

impl<R: SecretReader, P: Persister> Handler<R, P> {
    pub fn new(persister: P, reader: R) -> Self {
        Self {
            gpg: Gpg::default(),
            persister,
            reader,
        }
    }

    pub fn insert(&mut self, name: &str, key_id: &str) -> anyhow::Result<HandlerResult> {
        let buf = &self.read_in_secret_value()?;
        let ciphertext = self.gpg.encrypt(key_id, buf)?;
        let file = self.write_out_value(name, &ciphertext)?;
        Ok(HandlerResult::Insert(file))
    }

    fn read_in_secret_value(&mut self) -> anyhow::Result<Vec<u8>> {
        self.reader.read_secret()
    }

    fn write_out_value(&self, name: &str, ciphertext: &[u8]) -> anyhow::Result<File> {
        let mut file = self.persister.create(name)?;
        file.write_all(ciphertext)?;
        Ok(file)
    }
}

pub trait SecretReader {
    fn read_secret(&self) -> anyhow::Result<Vec<u8>>;
}

pub struct StdinSecretReader;

impl SecretReader for StdinSecretReader {
    fn read_secret(&self) -> anyhow::Result<Vec<u8>> {
        let secret = rpassword::prompt_password("Enter your secret: ")
            .with_context(|| "failed to read from input source")?;
        Ok(secret.into_bytes())
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
            .create(true)
            .write(true)
            .open(name)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {
    use super::{Persister, SecretReader};
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
        cell::RefCell,
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::Path,
    };
    use tempfile::tempdir;

    struct FakeSecretReader<'a> {
        secret: RefCell<&'a [u8]>,
    }

    impl<'a> SecretReader for FakeSecretReader<'a> {
        fn read_secret(&self) -> anyhow::Result<Vec<u8>> {
            Ok(rpassword::read_password_from_bufread(&mut self.secret.take())?.into_bytes())
        }
    }

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
        let name = "name".to_owned();
        let args = Args {
            action: Action::Insert {
                name,
                key_id: GPG_KEY_ID.to_owned(),
            },
        };
        let mut buf = Vec::new();
        let input = "password\n";
        let secret_reader = FakeSecretReader {
            secret: RefCell::new(input.as_bytes()),
        };
        let handler = Handler::new(InMemoryPersister, secret_reader);
        let HandlerResult::Insert(mut file) = handle(handler, &args).expect("expected a result");
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut buf).unwrap();
        let plaintext = gpg.decrypt(&buf).unwrap();
        assert_eq!(plaintext, input.trim());
    }
}
