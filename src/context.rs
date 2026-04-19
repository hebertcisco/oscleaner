use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::types::OsKind;

#[derive(Clone)]
pub struct ScanContext {
    pub os: OsKind,
    pub home: PathBuf,
    pub temp: PathBuf,
    pub search_roots: Vec<PathBuf>,
    pub local_app_data: Option<PathBuf>,
    pub roaming_app_data: Option<PathBuf>,
    pub program_data: Option<PathBuf>,
    pub program_files: Option<PathBuf>,
    pub program_files_x86: Option<PathBuf>,
    pub xdg_cache_home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub system_drive: Option<PathBuf>,
    pub selected_drive: Option<PathBuf>,
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
        let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
        let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);

        let xdg_cache_home = Some(
            env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".cache")),
        );
        let xdg_config_home = Some(
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config")),
        );
        let xdg_data_home = Some(
            env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/share")),
        );
        let system_drive = env::var_os("SystemDrive")
            .map(PathBuf::from)
            .map(|path| normalize_windows_drive_root(&path));

        let search_roots = build_search_roots(&home, &cwd);

        Ok(Self {
            os,
            home,
            temp,
            search_roots,
            local_app_data,
            roaming_app_data,
            program_data,
            program_files,
            program_files_x86,
            xdg_cache_home,
            xdg_config_home,
            xdg_data_home,
            system_drive,
            selected_drive: None,
        })
    }

    pub fn with_selected_drive(&self, drive_root: &Path) -> Self {
        let mut ctx = self.clone();
        if self.os == OsKind::Windows {
            let drive_root = normalize_windows_drive_root(drive_root);
            ctx.search_roots = vec![drive_root.clone()];
            ctx.selected_drive = Some(drive_root);
        }
        ctx
    }

    pub fn is_path_in_scope(&self, path: &Path) -> bool {
        if self.os != OsKind::Windows {
            return true;
        }

        match self.selected_drive.as_deref() {
            Some(selected_drive) => same_windows_drive(path, selected_drive),
            None => true,
        }
    }

    pub fn system_drive(&self) -> Option<&Path> {
        self.system_drive.as_deref()
    }

    pub fn selected_drive(&self) -> Option<&Path> {
        self.selected_drive.as_deref()
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

fn normalize_windows_drive_root(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy().replace('/', "\\");
    let trimmed = raw.trim_end_matches('\\');

    match trimmed.as_bytes() {
        [drive, b':', ..] => PathBuf::from(format!("{}:\\", (*drive as char).to_ascii_uppercase())),
        _ => path.to_path_buf(),
    }
}

fn same_windows_drive(path: &Path, root: &Path) -> bool {
    match (windows_drive_letter(path), windows_drive_letter(root)) {
        (Some(path_drive), Some(root_drive)) => path_drive == root_drive,
        _ => path.starts_with(root),
    }
}

fn windows_drive_letter(path: &Path) -> Option<char> {
    let raw = path.to_string_lossy().replace('/', "\\");
    match raw.as_bytes() {
        [drive, b':', ..] => Some((*drive as char).to_ascii_uppercase()),
        _ => None,
    }
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

    #[test]
    fn with_selected_drive_replaces_search_roots_and_scopes_paths() {
        let ctx = ScanContext {
            os: OsKind::Windows,
            home: PathBuf::from("C:\\Users\\test"),
            temp: PathBuf::from("C:\\Temp"),
            search_roots: vec![PathBuf::from("C:\\Users\\test")],
            local_app_data: None,
            roaming_app_data: None,
            program_data: None,
            program_files: None,
            program_files_x86: None,
            xdg_cache_home: None,
            xdg_config_home: None,
            xdg_data_home: None,
            system_drive: Some(PathBuf::from("C:\\")),
            selected_drive: None,
        };

        let scoped = ctx.with_selected_drive(Path::new("d:"));
        assert_eq!(scoped.search_roots, vec![PathBuf::from("D:\\")]);
        assert_eq!(scoped.selected_drive(), Some(Path::new("D:\\")));
        assert!(scoped.is_path_in_scope(Path::new("D:\\Projects\\app")));
        assert!(!scoped.is_path_in_scope(Path::new("C:\\Users\\test\\.cargo")));
    }
}
