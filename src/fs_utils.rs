use std::io;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

const SKIPPABLE_DIRS: &[&str] = &[
    ".git",
    ".idea",
    ".vscode",
    ".npm",
    ".pyenv",
    ".rbenv",
    ".nvm",
    ".oh-my-zsh",
    "target",
];

pub fn is_skippable(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    SKIPPABLE_DIRS.contains(&name.as_ref())
}

pub fn walk_roots(roots: &[PathBuf], max_depth: usize) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        let walker = WalkDir::new(root)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| !is_skippable(e));

        for entry in walker.filter_map(|e| e.ok()) {
            entries.push(entry);
        }
    }
    entries
}

pub fn search_for_dir(roots: &[PathBuf], name: &str, max_depth: usize) -> Vec<PathBuf> {
    walk_roots(roots, max_depth)
        .into_iter()
        .filter(|e| e.file_type().is_dir() && e.file_name() == name)
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn calc_size(path: &Path) -> io::Result<u64> {
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    let mut size = 0u64;
    for entry in WalkDir::new(path).follow_links(false).into_iter().flatten() {
        if entry.file_type().is_file()
            && let Ok(meta) = entry.metadata()
        {
            size = size.saturating_add(meta.len());
        }
    }
    Ok(size)
}

pub fn list_children(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect(),
        Err(_) => Vec::new(),
    }
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
