use anyhow::Context;
use std::path::{Path, PathBuf};

pub trait FileSystemOperator {
    fn home_dir(&self) -> Option<PathBuf>;
    fn create_dir<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()>;
}

pub struct FileSystemOperations;

impl FileSystemOperator for FileSystemOperations {
    fn home_dir(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        std::fs::create_dir(path).with_context(|| "failed to create directory {path}")
    }
}
