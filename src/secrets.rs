use std::ops::{Deref, DerefMut};

use anyhow::Context;
use zeroize::ZeroizeOnDrop;

/// [`String`] whose memory is zeroed out when dropped.
#[derive(Clone, ZeroizeOnDrop)]
pub struct ZeroizedString(String);

impl ZeroizedString {
    pub fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl AsRef<str> for ZeroizedString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// [`Vec<u8>`] whose memory is zeroed out when dropped.
#[derive(ZeroizeOnDrop)]
pub struct ZeroizedByteVec(Vec<u8>);

impl ZeroizedByteVec {
    pub fn new(inner: Vec<u8>) -> Self {
        Self(inner)
    }

    pub fn into_zeroized_string(self) -> ZeroizedString {
        ZeroizedString::new(
            std::str::from_utf8(self.0.clone().as_ref())
                .expect("only utf-8 secrets are supported")
                .to_string(),
        )
    }
}

impl AsRef<[u8]> for ZeroizedByteVec {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for ZeroizedByteVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZeroizedByteVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Read a secret into a [`ZeroizedByteVec`].
pub trait SecretReader {
    fn read_secret(&self) -> anyhow::Result<ZeroizedByteVec>;
}

pub struct StdinSecretReader;

impl SecretReader for StdinSecretReader {
    /// Read from stdin without echoing back the characeters.
    fn read_secret(&self) -> anyhow::Result<ZeroizedByteVec> {
        Ok(ZeroizedByteVec::new(
            rpassword::prompt_password("Enter your secret: ")
                .with_context(|| "failed to read from input source")?
                .into_bytes(),
        ))
    }
}
