use crate::cli::{Action, Args};
use crate::gpg::Gpg;
use crate::secrets::{SecretReader, ZeroizedByteVec, ZeroizedString};
use crate::store::Store;
use anyhow::Context;

pub fn handle<R: SecretReader, S: Store>(
    handler: &mut Handler<R, S>,
    args: &Args,
) -> anyhow::Result<HandlerResult> {
    match &args.action {
        Action::Insert { name, key_id } => handler.insert(name, key_id),
        Action::Retrieve { name } => handler.retrieve(name),
    }
}

pub enum HandlerResult {
    Insert(String),
    Retrieve(ZeroizedString),
}

pub struct Handler<R, S> {
    gpg: Gpg,
    store: S,
    reader: R,
}

impl<R: SecretReader, S: Store> Handler<R, S> {
    pub fn new(store: S, reader: R) -> Self {
        Self {
            gpg: Gpg::default(),
            store,
            reader,
        }
    }

    /// Create a file named with the value of `name` whose contents are taken
    /// from the [`Handler`]'s [`SecretReader`] instance, and encrypted via
    /// the [`Gpg::encrypt`] call.
    pub fn insert(&mut self, name: &str, key_id: &str) -> anyhow::Result<HandlerResult> {
        let buf = &self.read_in_secret_value()?;
        let ciphertext = self.gpg.encrypt(key_id, buf.as_ref())?;
        self.write_out_value(name, &ciphertext)?;
        Ok(HandlerResult::Insert(name.to_owned()))
    }

    fn read_in_secret_value(&mut self) -> anyhow::Result<ZeroizedByteVec> {
        self.reader.read_secret()
    }

    fn write_out_value(&self, name: &str, ciphertext: &[u8]) -> anyhow::Result<()> {
        self.store.insert(name, ciphertext).with_context(|| {
            format!("An error occurred when attempting to insert the entry `{name}`:")
        })
    }

    /// Retrieve a secret from the entry with the value of `name`.
    pub fn retrieve(&self, name: &str) -> anyhow::Result<HandlerResult> {
        let value = self.store.get(name).with_context(|| {
            format!("An error occurred when attempting to retrieve the entry `{name}`:")
        })?;
        let plaintext = self
            .gpg
            .decrypt(&value)
            .with_context(|| format!(r#"The entry "{name}" could not be decrypted!"#))?;
        Ok(HandlerResult::Retrieve(plaintext))
    }
}

#[cfg(test)]
mod test {
    use super::HandlerResult;
    use crate::{
        cli::Action,
        gpg::{
            test::{import_keys, GPG_KEY_ID},
            Gpg,
        },
        input::handle,
        secrets::{SecretReader, ZeroizedByteVec},
        store::{Store, StoreError},
        Args, Handler,
    };
    use std::{cell::RefCell, collections::HashMap, fs::File, rc::Rc};

    #[derive(Clone)]
    struct InMemoryStore {
        store: Rc<RefCell<HashMap<String, Vec<u8>>>>,
    }

    impl InMemoryStore {
        fn new() -> Self {
            Self {
                store: Rc::new(RefCell::new(HashMap::new())),
            }
        }
    }

    impl Store for InMemoryStore {
        fn insert<S: AsRef<str>>(&self, name: S, value: &[u8]) -> Result<(), StoreError> {
            self.store
                .clone()
                .borrow_mut()
                .insert(name.as_ref().to_owned(), value.to_vec());
            Ok(())
        }

        fn get<S: AsRef<str>>(&self, name: S) -> Result<Vec<u8>, StoreError> {
            if let Some(value) = self.store.clone().borrow().get(name.as_ref()) {
                Ok(value.clone())
            } else {
                Err(StoreError::EntryDoesNotExist(name.as_ref().to_owned()))
            }
        }
    }

    struct IoErrorStore;

    impl Store for IoErrorStore {
        fn insert<S: AsRef<str>>(&self, _: S, _: &[u8]) -> Result<(), StoreError> {
            let _ = File::open("b68eea40-38e3-43e8-bb61-60ec38067feb")?;
            Ok(())
        }

        fn get<S: AsRef<str>>(&self, _: S) -> Result<Vec<u8>, StoreError> {
            let _ = File::open("b68eea40-38e3-43e8-bb61-60ec38067feb")?;
            Ok(Vec::new())
        }
    }

    struct FakeSecretReader<'a> {
        secret: RefCell<&'a [u8]>,
    }

    impl<'a> SecretReader for FakeSecretReader<'a> {
        fn read_secret(&self) -> anyhow::Result<ZeroizedByteVec> {
            Ok(ZeroizedByteVec::new(
                rpassword::read_password_from_bufread(&mut self.secret.take())
                    .unwrap()
                    .into_bytes(),
            ))
        }
    }

    #[test]
    fn should_encrypt_and_store_given_value() {
        import_keys();
        let gpg = Gpg::new();
        let name = "name".to_owned();
        let args = Args {
            action: Action::Insert {
                name,
                key_id: GPG_KEY_ID.to_owned(),
            },
        };
        let input = "password\n";
        let secret_reader = FakeSecretReader {
            secret: RefCell::new(input.as_bytes()),
        };
        let store = InMemoryStore::new();
        let mut handler = Handler::new(store.clone(), secret_reader);
        if let HandlerResult::Insert(name) = handle(&mut handler, &args).expect("expected a result")
        {
            let ciphertext = store.get(name).unwrap();
            let plaintext = gpg.decrypt(&ciphertext).unwrap();
            assert_eq!(&*plaintext, input.trim());
        } else {
            panic!("got unexpected handle result")
        }
    }

    #[test]
    fn should_retrieve_entry_decrypted() {
        import_keys();
        let name = "name".to_string();
        let retrieve_args = Args {
            action: Action::Retrieve { name: name.clone() },
        };
        let input = "password\n";
        let secret_reader = FakeSecretReader {
            secret: RefCell::new(input.as_bytes()),
        };
        let store = InMemoryStore::new();
        let mut handler = Handler::new(store, secret_reader);
        handler.insert(&name, GPG_KEY_ID).unwrap();
        if let HandlerResult::Retrieve(value) =
            handle(&mut handler, &retrieve_args).expect("expected a result")
        {
            assert_eq!(&*value, input.trim());
            return;
        }
        panic!("got unexpected handle result");
    }

    #[test]
    fn should_give_meaningful_error_if_entry_could_not_be_decrypted() {
        let name = "name".to_string();
        let retrieve_args = Args {
            action: Action::Retrieve { name: name.clone() },
        };
        let secret_reader = FakeSecretReader {
            secret: RefCell::new("".as_bytes()),
        };
        let store = InMemoryStore::new();
        store.insert(name.clone(), b"").unwrap();
        let mut handler = Handler::new(store, secret_reader);
        let result = handle(&mut handler, &retrieve_args);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(r#"The entry "{name}" could not be decrypted!"#)
        );
    }

    #[test]
    fn insert_should_give_meaningful_error_if_store_has_an_fs_error() {
        let name = "name".to_string();
        let retrieve_args = Args {
            action: Action::Insert {
                name: name.clone(),
                key_id: GPG_KEY_ID.to_string(),
            },
        };
        let secret_reader = FakeSecretReader {
            secret: RefCell::new("secret\n".as_bytes()),
        };
        let store = IoErrorStore;
        let mut handler = Handler::new(store, secret_reader);
        let result = handle(&mut handler, &retrieve_args);
        assert!(result.is_err());
        let partial_error =
            &format!("An error occurred when attempting to insert the entry `{name}`:");
        let err = result.err().unwrap();
        assert!(
            err.to_string().contains(partial_error),
            "error incorrect, got `{err}`",
        );
    }

    #[test]
    fn retrieve_should_give_meaningful_error_if_store_has_an_fs_error() {
        let name = "name".to_string();
        let retrieve_args = Args {
            action: Action::Retrieve { name: name.clone() },
        };
        let secret_reader = FakeSecretReader {
            secret: RefCell::new("".as_bytes()),
        };
        let store = IoErrorStore;
        let mut handler = Handler::new(store, secret_reader);
        let result = handle(&mut handler, &retrieve_args);
        assert!(result.is_err());
        let partial_error =
            &format!("An error occurred when attempting to retrieve the entry `{name}`:");
        let actual_error = result.err().unwrap();
        assert!(
            actual_error.to_string().contains(partial_error),
            "error incorrect, got `{actual_error}`"
        );
    }
}
