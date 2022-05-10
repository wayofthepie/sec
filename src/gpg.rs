use gpgme::{Context, Data, Protocol};

/// Wrapper for GPG functionality.
pub struct Gpg {
    protocol: Protocol,
}

/// High level implementation of GPG functionality.
impl Gpg {
    pub fn new() -> Self {
        Self {
            protocol: Protocol::OpenPgp,
        }
    }

    /// Encrypt the given plaintext bytes with the key indentified by the key ID.
    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut context = Context::from_protocol(self.protocol)?;
        let key = context.find_keys(Some(key_id))?.next().unwrap().unwrap();
        let mut ciphertext = Vec::new();
        context.encrypt(Some(&key), plaintext, &mut ciphertext)?;
        Ok(ciphertext)
    }

    /// Decrypt the given ciphertext.
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
    use crate::Gpg;
    use std::{
        env,
        io::Write,
        process::{Command, Stdio},
    };

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
