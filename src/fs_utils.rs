use std::io;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

pub fn is_skippable(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(name.as_ref(), ".git" | ".idea" | ".vscode" | "target")
}

pub fn search_for_dir(roots: &[PathBuf], name: &str, max_depth: usize) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if is_skippable(&entry) {
                continue;
            }
            if entry.file_name() == name {
                hits.push(entry.path().to_path_buf());
            }
        }
    }
    hits
}

pub fn calc_size(path: &Path) -> io::Result<u64> {
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    let mut size = 0u64;
    for entry in WalkDir::new(path).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                size = size.saturating_add(meta.len());
            }
        }
    }
    Ok(size)
}

pub fn shorten_path(path: &Path) -> String {
    let display = path.display().to_string();
    if display.len() > 60 {
        let start = &display[..30];
        let end = &display[display.len().saturating_sub(20)..];
        format!("{start}...{end}")
    } else {
        display
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn calc_size_counts_files_and_nested_directories() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("root");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("a.txt"), b"hello").unwrap();
        fs::write(nested.join("b.bin"), vec![0u8; 3]).unwrap();

        assert_eq!(calc_size(&root).unwrap(), 8);
        assert_eq!(calc_size(&root.join("a.txt")).unwrap(), 5);
    }

    #[test]
    fn shorten_path_returns_original_for_short_paths() {
        let path = PathBuf::from("C:\\temp\\short");
        assert_eq!(shorten_path(&path), path.display().to_string());
    }

    #[test]
    fn shorten_path_truncates_long_paths() {
        let long_path = format!("C:\\{}", "a".repeat(70));
        let path = PathBuf::from(&long_path);

        let shortened = shorten_path(&path);
        let expected_start = &long_path[..30];
        let expected_end = &long_path[long_path.len().saturating_sub(20)..];

        assert!(shortened.contains("..."));
        assert!(shortened.starts_with(expected_start));
        assert!(shortened.ends_with(expected_end));
    }
}
