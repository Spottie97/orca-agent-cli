use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::Result;

pub fn safe_write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    }

    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.txt");
        safe_write(&path, "hello world").unwrap();
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello world");
    }

    #[test]
    fn test_ensure_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a/b/c");
        ensure_dir(&path).unwrap();
        assert!(path.exists());
    }
}
