use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// A Transaction tracks changes to the file system, allowing them to
/// be rolled back in case of an error.
pub(crate) struct Transaction {
    operations: Vec<Operation>,
    version: i32,
    committed: bool,

    temp_dir: tempfile::TempDir,
}

impl Transaction {
    pub(crate) fn new(temp_dir: tempfile::TempDir) -> Self {
        Self {
            operations: Vec::new(),
            version: 0,
            committed: false,
            temp_dir,
        }
    }

    pub(crate) fn commit(&mut self) {
        self.committed = true;
    }

    pub(crate) fn rollback_to(&mut self, version: i32) -> io::Result<()> {
        if self.committed {
            return Ok(());
        }
        while let Some(op) = self.operations.pop() {
            op.rollback()?;
            self.version -= 1;
            if self.version == version {
                break;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn version(&self) -> i32 {
        self.version
    }

    pub(crate) fn create_dir(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        fs::create_dir(&path)?;
        self.change(Operation::CreateDir(path));
        Ok(())
    }

    pub(crate) fn create_dir_all(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut stack = Vec::new();
        let mut path = path.as_ref();
        stack.push(path);

        while let Some(parent) = path.parent() {
            if parent.exists() {
                break;
            }
            stack.push(parent);
            path = parent;
        }

        while let Some(p) = stack.pop() {
            if p.is_dir() {
                continue;
            }
            self.create_dir(p)?;
        }
        Ok(())
    }

    pub(crate) fn write_file(
        &mut self,
        path: impl Into<PathBuf>,
        contents: impl AsRef<[u8]>,
    ) -> io::Result<()> {
        let path = path.into();
        if let Some(dirname) = path.parent() {
            self.create_dir_all(dirname)?;
        }

        let mut file = loop {
            match fs::File::create_new(&path) {
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    self.remove_file(&path)?;
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(file) => {
                    break file;
                }
            }
        };

        file.write_all(contents.as_ref())?;
        file.flush()?;
        self.change(Operation::WriteFile(path));
        Ok(())
    }

    pub(crate) fn remove_file(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        if !path.is_file() && !path.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{} not a file or symlink", path.display()),
            ));
        }
        let backup_path = self.backup_path(&path);

        fs::rename(&path, &backup_path)?;
        self.change(Operation::RemoveFile {
            removed: path,
            backup: backup_path,
        });
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn remove_dir(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{} not a directory", path.display()),
            ));
        }

        let backup_path = self.backup_path(&path);

        fs::rename(&path, &backup_path)?;
        self.change(Operation::RemoveDir {
            removed: path,
            backup: backup_path,
        });
        Ok(())
    }

    fn backup_path(&self, path: &Path) -> PathBuf {
        let mut filename = path
            .file_name()
            .expect("path should have a file name")
            .to_owned();
        filename.push(format!(".backup.{}", self.version));
        self.temp_dir.path().join(filename)
    }

    fn change(&mut self, op: Operation) {
        self.operations.push(op);
        self.version += 1;
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Err(e) = self.rollback_to(0) {
            panic!("failed to rollback: {e}");
        }
    }
}

#[derive(Debug)]
enum Operation {
    CreateDir(PathBuf),
    WriteFile(PathBuf),
    RemoveFile {
        removed: PathBuf,
        backup: PathBuf,
    },
    #[allow(dead_code)]
    RemoveDir {
        removed: PathBuf,
        backup: PathBuf,
    },
}

impl Operation {
    fn rollback(&self) -> io::Result<()> {
        match self {
            Operation::CreateDir(p) => fs::remove_dir(p),
            Operation::WriteFile(p) => fs::remove_file(p),
            Operation::RemoveFile { removed, backup }
            | Operation::RemoveDir { removed, backup } => fs::rename(backup, removed),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn remove_file() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let filepath = testdir.path().join("foo");
        fs::write(&filepath, "").unwrap();

        tx.remove_file(&filepath).unwrap();
        tx.commit();

        assert!(!filepath.is_file());
    }

    #[test]
    fn remove_file_then_rollback() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let filepath = testdir.path().join("foo");
        fs::write(&filepath, "").unwrap();

        tx.remove_file(&filepath).unwrap();
        drop(tx);

        assert!(filepath.is_file());
    }

    #[test]
    fn remove_file_that_not_exists() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let res = tx.remove_file(testdir.path().join("foo"));
        assert!(res.is_err());
    }

    #[test]
    fn remove_dir() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        fs::create_dir_all(testdir.path().join("foo")).unwrap();
        fs::write(testdir.path().join("foo/bar"), "").unwrap();

        tx.remove_dir(testdir.path().join("foo")).unwrap();
        tx.commit();

        assert!(!testdir.path().join("foo").exists());
    }

    #[test]
    fn remove_dir_then_rollback() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        fs::create_dir_all(testdir.path().join("foo")).unwrap();
        fs::write(testdir.path().join("foo/bar"), "").unwrap();

        tx.remove_dir(testdir.path().join("foo")).unwrap();
        drop(tx);

        assert!(testdir.path().join("foo").exists());
    }

    #[test]
    fn remove_dir_that_not_exists() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let res = tx.remove_dir(testdir.path().join("foo"));
        assert!(res.is_err());
    }

    #[test]
    fn write_file() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let contents = "hi".to_string();
        let filepath = testdir.path().join("foo/bar");
        tx.write_file(&filepath, contents.clone()).unwrap();
        tx.commit();

        assert!(filepath.is_file());
        let file_content = fs::read_to_string(&filepath).unwrap();
        assert_eq!(contents, file_content);
    }

    #[test]
    fn write_file_then_rollback() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let contents = "hi".to_string();
        let filepath = testdir.path().join("a/b/c/d/e/f");
        tx.write_file(&filepath, contents.clone()).unwrap();
        drop(tx);

        assert!(!filepath.is_file());
    }

    #[test]
    fn write_file_that_exists() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let contents1 = "hi".to_string();
        let contents2 = "ih".to_string();
        let filepath = &testdir.path().join("a");
        fs::write(&filepath, &contents1).unwrap();
        tx.write_file(&filepath, &contents2).unwrap();
        tx.commit();

        assert_eq!(fs::read_to_string(&filepath).unwrap(), contents2);
    }

    #[test]
    fn write_file_that_exists_then_rollback() {
        let testdir = tempfile::tempdir().unwrap();
        let txdir = tempfile::Builder::new()
            .prefix("bip39-keygen")
            .tempdir()
            .unwrap();

        let mut tx = Transaction::new(txdir);

        let contents1 = "hi".to_string();
        let contents2 = "ih".to_string();
        let filepath = &testdir.path().join("a");
        fs::write(&filepath, &contents1).unwrap();
        tx.write_file(&filepath, &contents2).unwrap();
        drop(tx);

        assert_eq!(fs::read_to_string(&filepath).unwrap(), contents1);
    }
}
