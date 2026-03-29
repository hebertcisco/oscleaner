use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::types::OsKind;

pub struct ScanContext {
    pub os: OsKind,
    pub home: PathBuf,
    pub temp: PathBuf,
    pub search_roots: Vec<PathBuf>,
    pub local_app_data: Option<PathBuf>,
    pub roaming_app_data: Option<PathBuf>,
    pub program_data: Option<PathBuf>,
    pub xdg_cache_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
}

impl ScanContext {
    pub fn new() -> Result<Self> {
        let os = if cfg!(target_os = "windows") {
            OsKind::Windows
        } else if cfg!(target_os = "macos") {
            OsKind::Mac
        } else if cfg!(target_os = "linux") {
            OsKind::Linux
        } else if cfg!(target_os = "freebsd") {
            OsKind::FreeBSD
        } else {
            OsKind::Other
        };

        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));

        let cwd = env::current_dir().context("Unable to read current directory")?;
        let temp = env::temp_dir();
        let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let roaming_app_data = env::var_os("APPDATA").map(PathBuf::from);
        let program_data = env::var_os("PROGRAMDATA").map(PathBuf::from);

        let xdg_cache_home = Some(
            env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".cache")),
        );
        let xdg_data_home = Some(
            env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/share")),
        );

        let search_roots = build_search_roots(&home, &cwd);

        Ok(Self {
            os,
            home,
            temp,
            search_roots,
            local_app_data,
            roaming_app_data,
            program_data,
            xdg_cache_home,
            xdg_data_home,
        })
    }
}

fn build_search_roots(home: &Path, cwd: &Path) -> Vec<PathBuf> {
    let mut roots = vec![cwd.to_path_buf()];
    let candidates = ["Projects", "projects", "code", "src", "dev"];
    for dir in candidates {
        let p = home.join(dir);
        if p.exists() {
            roots.push(p);
        }
    }
    roots.push(home.to_path_buf());

    let mut seen = HashSet::new();
    roots
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn build_search_roots_includes_existing_candidates_and_home() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        let cwd = home.join("workspace");
        fs::create_dir_all(&cwd).unwrap();
        fs::create_dir_all(home.join("Projects")).unwrap();
        fs::create_dir_all(home.join("dev")).unwrap();

        let roots = build_search_roots(&home, &cwd);
        assert_eq!(roots[0], cwd, "first root should be cwd");
        // On case-insensitive filesystems (macOS), "projects" also matches
        // the existing "Projects" dir, so we just check membership instead
        // of exact positions for the middle entries.
        assert!(roots.contains(&home.join("Projects")));
        assert!(roots.contains(&home.join("dev")));
        assert_eq!(roots.last().unwrap(), &home, "last root should be home");
    }

    #[test]
    fn build_search_roots_deduplicates_entries() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        let roots = build_search_roots(&home, &home);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], home);
    }
}
