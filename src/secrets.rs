use anyhow::Context;
use zeroize::ZeroizeOnDrop;

/// [`String`] whose memory is zeroed out when dropped.
#[derive(Clone, ZeroizeOnDrop)]
pub struct ZeroizedString(String);

impl ZeroizedString {
    pub fn new(inner: String) -> Self {
        Self(inner)
    }

    pub fn into_bytes(self) -> ZeroizedByteVec {
        ZeroizedByteVec::new(self.0.clone().into_bytes())
    }
}

impl AsRef<str> for ZeroizedString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// [`Vec<u8>`] whose memory is zeroed out when dropped.
#[derive(ZeroizeOnDrop)]
pub struct ZeroizedByteVec {
    inner: Vec<u8>,
}

impl ZeroizedByteVec {
    pub fn new(inner: Vec<u8>) -> Self {
        Self { inner }
    }
}

impl AsRef<[u8]> for ZeroizedByteVec {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
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
        let secret = ZeroizedString::new(
            rpassword::prompt_password("Enter your secret: ")
                .with_context(|| "failed to read from input source")?,
        );
        Ok(secret.into_bytes())
    }
}
