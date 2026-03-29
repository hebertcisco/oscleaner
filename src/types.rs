use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OsKind {
    Windows,
    Mac,
    Linux,
    FreeBSD,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Platform {
    All,
    Windows,
    Mac,
    Linux,
    Unix,
}

impl Platform {
    pub fn matches(self, os: OsKind) -> bool {
        match (self, os) {
            (Platform::All, _) => true,
            (Platform::Windows, OsKind::Windows) => true,
            (Platform::Mac, OsKind::Mac) => true,
            (Platform::Linux, OsKind::Linux) => true,
            (Platform::Unix, OsKind::Mac | OsKind::Linux | OsKind::FreeBSD) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Finding {
    pub category_id: &'static str,
    pub category_name: &'static str,
    pub category_description: &'static str,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug)]
pub struct CategorySummary {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub total_size: u64,
    pub items: usize,
}

#[derive(Debug)]
pub struct CleanReport {
    pub dry_run: bool,
    pub attempted: usize,
    pub succeeded: usize,
    pub freed_bytes: u64,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_matching_respects_os() {
        assert!(Platform::All.matches(OsKind::Windows));
        assert!(Platform::All.matches(OsKind::Mac));
        assert!(Platform::All.matches(OsKind::Linux));
        assert!(Platform::All.matches(OsKind::FreeBSD));
        assert!(Platform::All.matches(OsKind::Other));

        assert!(Platform::Windows.matches(OsKind::Windows));
        assert!(!Platform::Windows.matches(OsKind::Mac));
        assert!(!Platform::Windows.matches(OsKind::Linux));

        assert!(Platform::Mac.matches(OsKind::Mac));
        assert!(!Platform::Mac.matches(OsKind::Windows));
        assert!(!Platform::Mac.matches(OsKind::Linux));

        assert!(Platform::Linux.matches(OsKind::Linux));
        assert!(!Platform::Linux.matches(OsKind::Windows));
        assert!(!Platform::Linux.matches(OsKind::Mac));

        assert!(Platform::Unix.matches(OsKind::Mac));
        assert!(Platform::Unix.matches(OsKind::Linux));
        assert!(Platform::Unix.matches(OsKind::FreeBSD));
        assert!(!Platform::Unix.matches(OsKind::Windows));
        assert!(!Platform::Unix.matches(OsKind::Other));
    }
}
