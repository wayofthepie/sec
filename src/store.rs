use anyhow::anyhow;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::Path,
};

trait Store {
    fn insert<S: AsRef<str>>(&self, name: S, value: &[u8]) -> anyhow::Result<()>;
    fn get<S: AsRef<str>>(&self, name: S) -> anyhow::Result<Vec<u8>>;
}

pub struct OnDiskStore {
    base_dir: String,
}

impl OnDiskStore {
    pub fn new<S: Into<String>>(base_dir: S) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn build_entry_path<S: AsRef<str>>(&self, name: S) -> String {
        format!("{}/{}", self.base_dir, name.as_ref())
    }
}

impl Store for OnDiskStore {
    fn insert<S: AsRef<str>>(&self, name: S, value: &[u8]) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.build_entry_path(name))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(value).map_err(|e| e.into())
    }

    fn get<S: AsRef<str>>(&self, name: S) -> anyhow::Result<Vec<u8>> {
        let name = name.as_ref();
        let path = self.build_entry_path(name);
        if !Path::new(&path).exists() {
            return Err(anyhow!(r#"The entry "{name}" does not exist!"#));
        }
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod test {
    use crate::store::{OnDiskStore, Store};
    use std::{fs::File, io::Read, path::PathBuf, str::FromStr};
    use tempfile::tempdir;

    #[test]
    fn should_create_file_with_name_of_entry() {
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let name = "key";
        let value = b"value";
        let store = OnDiskStore::new(base_dir);
        store.insert(name, value).unwrap();
        assert!(PathBuf::from_str(&format!("{base_dir}/{name}"))
            .unwrap()
            .exists());
    }

    #[test]
    fn create_files_contents_should_be_the_value_passed() {
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let name = "key";
        let value = b"value";
        let store = OnDiskStore::new(base_dir);
        store.insert(name, value).unwrap();
        let path = &format!("{base_dir}/{name}");
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, value);
    }

    #[test]
    fn should_retrieve_value_of_the_given_name() {
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let name = "key";
        let value = b"value";
        let store = OnDiskStore::new(base_dir);
        store.insert(name, value).unwrap();
        let retrieved = store.get(name).unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn should_give_meaningful_error_if_entry_with_given_name_does_not_exist() {
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let name = "key";
        let store = OnDiskStore::new(base_dir);
        let result = store.get(name);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(r#"The entry "{name}" does not exist!"#)
        );
    }
}
