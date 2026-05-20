use std::path::PathBuf;

use anyhow::Result;

use crate::utils::fs;

pub struct MemoryStore {
    base_dir: PathBuf,
}

impl MemoryStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn write_note(&self, category: &str, name: &str, contents: &str) -> Result<()> {
        let path = self.base_dir.join(category).join(format!("{}.md", name));
        fs::safe_write(&path, contents)?;
        Ok(())
    }

    pub fn read_note(&self, category: &str, name: &str) -> Result<Option<String>> {
        let path = self.base_dir.join(category).join(format!("{}.md", name));
        if path.exists() {
            Ok(Some(std::fs::read_to_string(&path)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_notes(&self, category: &str) -> Result<Vec<String>> {
        let dir = self.base_dir.join(category);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut notes = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    notes.push(stem.to_string());
                }
            }
        }
        Ok(notes)
    }

    pub fn append_to_note(&self, category: &str, name: &str, text: &str) -> Result<()> {
        let path = self.base_dir.join(category).join(format!("{}.md", name));
        let existing = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };
        let new_contents = format!("{}{}", existing, text);
        fs::safe_write(&path, &new_contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_note() {
        let tmp = tempfile::tempdir().unwrap();
        let store = MemoryStore::new(tmp.path());
        store.write_note("tasks", "TASK-001", "# Task 1").unwrap();
        let contents = store.read_note("tasks", "TASK-001").unwrap();
        assert_eq!(contents, Some("# Task 1".to_string()));
    }

    #[test]
    fn test_list_notes() {
        let tmp = tempfile::tempdir().unwrap();
        let store = MemoryStore::new(tmp.path());
        store.write_note("tasks", "TASK-001", "a").unwrap();
        store.write_note("tasks", "TASK-002", "b").unwrap();
        let notes = store.list_notes("tasks").unwrap();
        assert!(notes.contains(&"TASK-001".to_string()));
        assert!(notes.contains(&"TASK-002".to_string()));
    }

    #[test]
    fn test_append_to_note() {
        let tmp = tempfile::tempdir().unwrap();
        let store = MemoryStore::new(tmp.path());
        store
            .write_note("decisions", "DEC-001", "# Decision\n")
            .unwrap();
        store
            .append_to_note("decisions", "DEC-001", "Updated.\n")
            .unwrap();
        let contents = store.read_note("decisions", "DEC-001").unwrap().unwrap();
        assert!(contents.contains("Updated."));
    }
}
