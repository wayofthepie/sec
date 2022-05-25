use anyhow::Context;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub trait FileSystemOperator {
    fn home_dir(&self) -> Option<PathBuf>;
    fn mkdir<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()>;
    fn touch<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<File>;
}

pub struct FileSystemOperations;

impl FileSystemOperator for FileSystemOperations {
    fn home_dir(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn mkdir<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        std::fs::create_dir(path).with_context(|| "failed to create directory {path}")
    }

    fn touch<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<File> {
        Ok(std::fs::File::create(path)?)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::{FileSystemOperations, FileSystemOperator};
    use tempfile::tempdir;

    #[test]
    fn should_create_dir() {
        let leaf = "leaf";
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let fs_ops = FileSystemOperations;
        fs_ops.mkdir(format!("{}/{}", base_dir, leaf)).unwrap();
        assert!(Path::new(&format!("{}/{}", base_dir, leaf)).exists());
    }

    #[test]
    fn should_create_file() {
        let leaf = "leaf";
        let tmpdir = tempdir().unwrap();
        let base_dir = tmpdir.path().to_str().unwrap();
        let fs_ops = FileSystemOperations;
        fs_ops.touch(format!("{}/{}", base_dir, leaf)).unwrap();
        assert!(Path::new(&format!("{}/{}", base_dir, leaf)).exists());
    }
}
