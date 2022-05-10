use gpgme::{Context, Data, Protocol};

pub struct Gpg {
    protocol: Protocol,
}

impl Gpg {
    pub fn new() -> Self {
        Self {
            protocol: Protocol::OpenPgp,
        }
    }

    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut context = Context::from_protocol(self.protocol)?;
        let key = context.find_keys(Some(key_id))?.next().unwrap().unwrap();
        let mut ciphertext = Vec::new();
        context.encrypt(Some(&key), plaintext, &mut ciphertext)?;
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> anyhow::Result<String> {
        let mut context = Context::from_protocol(self.protocol)?;
        let mut input = Data::from_bytes(ciphertext)?;
        let mut output = Vec::new();
        context.decrypt(&mut input, &mut output)?;
        Ok(std::str::from_utf8(&output)?.to_owned())
    }
}

#[cfg(test)]
mod test {
    use crate::{validate, Args, Gpg};
    use std::{
        env,
        io::Write,
        process::{Command, Stdio},
    };
    use tempfile::NamedTempFile;

    fn import_keys() {
        let public = include_bytes!("../tests/resources/public.key");
        let secret = include_bytes!("../tests/resources/secret.key");
        import_key(public);
        import_key(secret);
    }

    fn import_key(key: &[u8]) {
        let gpg = env::var_os("GPG").unwrap_or_else(|| "gpg".into());
        let mut child = Command::new(&gpg)
            .arg("--no-permission-warning")
            .arg("--batch")
            .arg("--passphrase")
            .arg("abc")
            .arg("--import")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child.stdin.as_mut().unwrap().write_all(key).unwrap();
        assert!(child.wait().unwrap().success());
    }

    #[test]
    fn should_fail_if_file_does_not_exist() {
        let path = "test";
        let args = Args { path: path.into() };
        let result = validate(&args);
        assert!(result.is_err(), "expected error, got {:#?}", result);
        assert_eq!(
            format!("{}", result.err().unwrap()),
            format!(r#"The path "{}" does not exist!"#, path)
        );
    }

    #[test]
    fn should_succeed_if_file_exists() {
        let path = NamedTempFile::new().expect("temp file");
        let args = Args {
            path: path.path().into(),
        };
        let result = validate(&args);
        assert!(result.is_ok(), "expected error, got {:#?}", result);
    }

    #[test]
    fn encrypt_and_decrypt_should_be_isomorphic() {
        import_keys();
        let expected = "test";
        let gpg = Gpg::new();
        let ciphertext = gpg
            .encrypt("passrs-tests@nocht.io", expected.as_bytes())
            .expect("ciphertext encryption error");
        let plaintext = gpg.decrypt(&ciphertext).expect("plaintext");
        assert_eq!(plaintext, expected);
    }
}
