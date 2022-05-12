use crate::cli::{Action, Args};
use crate::gpg::Gpg;
use crate::secrets::{SecretReader, ZeroizedByteVec};
use anyhow::Context;
use std::io::{BufReader, Read};
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub fn handle<R: SecretReader, P: Persister>(
    handler: &mut Handler<R, P>,
    args: &Args,
) -> anyhow::Result<HandlerResult> {
    match &args.action {
        Action::Insert { name, key_id } => handler.insert(name, key_id),
        Action::Retrieve { name } => handler.retrieve(name),
    }
}

pub enum HandlerResult {
    Insert(File),
    Retrieve(String),
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

    /// Create a file named with the value of `name` whose contents are taken
    /// from the [`Handler`]'s [`SecretReader`] instance, and encrypted via
    /// the [`Gpg::encrypt`] call.
    pub fn insert(&mut self, name: &str, key_id: &str) -> anyhow::Result<HandlerResult> {
        let buf = &self.read_in_secret_value()?;
        let ciphertext = self.gpg.encrypt(key_id, buf.as_ref())?;
        let file = self.write_out_value(name, &ciphertext)?;
        Ok(HandlerResult::Insert(file))
    }

    fn read_in_secret_value(&mut self) -> anyhow::Result<ZeroizedByteVec> {
        self.reader.read_secret()
    }

    fn write_out_value(&self, name: &str, ciphertext: &[u8]) -> anyhow::Result<File> {
        let mut file = self.persister.create(name)?;
        file.write_all(ciphertext)?;
        Ok(file)
    }

    /// Retrieve a secret from the entry with the value of `name`.
    pub fn retrieve(&self, name: &str) -> anyhow::Result<HandlerResult> {
        let file =
            File::open(name).with_context(|| format!(r#"The entry "{name}" does not exist!"#))?;
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let plaintext = self
            .gpg
            .decrypt(&buf)
            .with_context(|| format!(r#"The entry "{name}" could not be decrypted!"#))?;
        Ok(HandlerResult::Retrieve(plaintext))
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

impl Default for OnDiskPersister {
    fn default() -> Self {
        Self::new()
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
    use super::{HandlerResult, Persister};
    use crate::{
        cli::Action,
        gpg::{
            test::{import_keys, GPG_KEY_ID},
            Gpg,
        },
        input::{handle, OnDiskPersister},
        secrets::{SecretReader, ZeroizedByteVec},
        Args, Handler,
    };
    use memfile::CreateOptions;
    use std::{
        cell::RefCell,
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::Path,
    };
    use tempfile::{tempdir, NamedTempFile};

    struct FakeSecretReader<'a> {
        secret: RefCell<&'a [u8]>,
    }

    impl<'a> SecretReader for FakeSecretReader<'a> {
        fn read_secret(&self) -> anyhow::Result<ZeroizedByteVec> {
            Ok(ZeroizedByteVec::new(
                rpassword::read_password_from_bufread(&mut self.secret.take())?.into_bytes(),
            ))
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
        let mut handler = Handler::new(InMemoryPersister, secret_reader);
        if let HandlerResult::Insert(mut file) =
            handle(&mut handler, &args).expect("expected a result")
        {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.read_to_end(&mut buf).unwrap();
            let plaintext = gpg.decrypt(&buf).unwrap();
            assert_eq!(plaintext, input.trim());
        } else {
            panic!("got unexpected handle result")
        }
    }

    #[test]
    fn should_decrypt_already_encrypted_file() {
        import_keys();
        let tempdir = tempdir().unwrap();
        let tempdir = tempdir.path().to_str().unwrap();
        let name = format!("{tempdir}/name");
        let retrieve_args = Args {
            action: Action::Retrieve { name: name.clone() },
        };
        let input = "password\n";
        let secret_reader = FakeSecretReader {
            secret: RefCell::new(input.as_bytes()),
        };
        let mut handler = Handler::new(OnDiskPersister, secret_reader);
        handler.insert(&name, GPG_KEY_ID).unwrap();
        if let HandlerResult::Retrieve(value) =
            handle(&mut handler, &retrieve_args).expect("expected a result")
        {
            assert_eq!(value, input.trim());
            return;
        }
        panic!("got unexpected handle result");
    }

    #[test]
    fn should_give_meaningful_error_if_entry_with_name_does_not_exist() {
        let missing_entry = "missing_entry".to_string();
        let value = "".to_owned();
        let retrieve_args = Args {
            action: Action::Retrieve {
                name: missing_entry.clone(),
            },
        };
        let secret_reader = FakeSecretReader {
            secret: RefCell::new(value.as_bytes()),
        };
        let mut handler = Handler::new(OnDiskPersister, secret_reader);
        let result = handle(&mut handler, &retrieve_args);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(r#"The entry "{missing_entry}" does not exist!"#)
        );
    }

    #[test]
    fn should_give_meaningful_error_if_entry_could_not_be_decrypted() {
        let file = NamedTempFile::new().unwrap();
        let file_path = file.path().to_str().unwrap().to_string();
        let retrieve_args = Args {
            action: Action::Retrieve {
                name: file_path.clone(),
            },
        };
        let secret_reader = FakeSecretReader {
            secret: RefCell::new("".as_bytes()),
        };
        let mut handler = Handler::new(OnDiskPersister, secret_reader);
        let result = handle(&mut handler, &retrieve_args);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(r#"The entry "{file_path}" could not be decrypted!"#)
        );
    }
}
