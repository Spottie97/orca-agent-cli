use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub project_type: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub file_summary: Vec<FileSummary>,
    pub detected_frameworks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileSummary {
    pub path: String,
    pub size: u64,
    pub language: Option<String>,
}

pub fn scan_repo(path: &Path) -> Result<ScanResult> {
    let mut result = ScanResult::default();
    let mut frameworks = HashSet::new();

    for entry in ignore::Walk::new(path) {
        let entry = entry?;
        let metadata = entry.metadata();
        let path_ref = entry.path();

        if path_ref.is_dir() {
            result.total_dirs += 1;
            continue;
        }

        if let Ok(meta) = metadata {
            result.total_files += 1;
            let rel_path = path_ref.strip_prefix(path).unwrap_or(path_ref);
            let rel_str = rel_path.to_string_lossy().to_string();

            let lang = detect_language(rel_path);
            result.file_summary.push(FileSummary {
                path: rel_str.clone(),
                size: meta.len(),
                language: lang.clone(),
            });

            // Detect project type from key files
            if rel_path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
                frameworks.insert("rust".to_string());
            } else if rel_path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                frameworks.insert("node".to_string());
            } else if rel_path.file_name() == Some(std::ffi::OsStr::new("go.mod")) {
                frameworks.insert("go".to_string());
            } else if rel_path.file_name() == Some(std::ffi::OsStr::new("pyproject.toml"))
                || rel_path.file_name() == Some(std::ffi::OsStr::new("requirements.txt"))
            {
                frameworks.insert("python".to_string());
            }
        }
    }

    result.detected_frameworks = frameworks.into_iter().collect();
    result.project_type = if result.detected_frameworks.is_empty() {
        "unknown".to_string()
    } else {
        result.detected_frameworks.join(", ")
    };

    Ok(result)
}

fn detect_language(path: &Path) -> Option<String> {
    path.extension().and_then(|e| e.to_str()).map(|ext| {
        match ext {
            "rs" => "rust",
            "py" => "python",
            "js" | "ts" | "jsx" | "tsx" => "javascript/typescript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" | "cc" => "cpp",
            "yaml" | "yml" => "yaml",
            "json" => "json",
            "toml" => "toml",
            "md" => "markdown",
            "sh" => "shell",
            _ => "other",
        }
        .to_string()
    })
}

pub fn render_scan_summary(result: &ScanResult) -> String {
    let mut md = String::new();
    md.push_str("# Scan Summary\n\n");
    md.push_str(&format!("- **Project type**: {}\n", result.project_type));
    md.push_str(&format!("- **Total files**: {}\n", result.total_files));
    md.push_str(&format!("- **Total directories**: {}\n", result.total_dirs));

    if !result.detected_frameworks.is_empty() {
        md.push_str("\n## Detected Frameworks\n");
        for fw in &result.detected_frameworks {
            md.push_str(&format!("- {}\n", fw));
        }
    }

    md.push_str("\n## File Summary\n");
    for f in result.file_summary.iter().take(50) {
        if let Some(ref lang) = f.language {
            md.push_str(&format!("- `{}` ({} — {} bytes)\n", f.path, lang, f.size));
        } else {
            md.push_str(&format!("- `{}` ({} bytes)\n", f.path, f.size));
        }
    }

    if result.file_summary.len() > 50 {
        md.push_str(&format!(
            "\n*... and {} more files*\n",
            result.file_summary.len() - 50
        ));
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_current_repo() {
        let result = scan_repo(Path::new(".")).unwrap();
        assert!(result.total_files > 0);
        assert!(result.total_dirs > 0);
    }

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(
            detect_language(Path::new("src/main.rs")),
            Some("rust".to_string())
        );
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(
            detect_language(Path::new("app.py")),
            Some("python".to_string())
        );
    }

    #[test]
    fn test_render_scan_summary_includes_project_type() {
        let result = ScanResult {
            project_type: "rust".to_string(),
            total_files: 10,
            total_dirs: 3,
            ..Default::default()
        };
        let md = render_scan_summary(&result);
        assert!(md.contains("rust"));
        assert!(md.contains("10"));
        assert!(md.contains("3"));
    }
}
