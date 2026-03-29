use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use console::style;
use indicatif::HumanBytes;

use crate::types::{CleanReport, Finding, OsKind};

const SAFE_CATEGORY_IDS: &[&str] = &[
    "node_modules",
    "cargo_targets",
    "gradle_cache",
    "maven_cache",
    "php_vendor",
    "ruby_vendor",
    "python_cache",
    "cocoapods_cache",
    "android_builds",
    "react_native_ios",
    "xcode",
    "homebrew_cache",
    "browser_caches",
    "snap_cache",
    "flatpak_cache",
];

const DEFAULT_MAX_SIZE_GB: u64 = 20;
const DEFAULT_MIN_AGE_DAYS: u64 = 2;

pub struct SafeConfig {
    pub max_bytes: u64,
    pub min_age: Duration,
    pub protected_paths: Vec<PathBuf>,
}

impl SafeConfig {
    pub fn new(
        home: &Path,
        os: OsKind,
        max_size_gb: Option<u64>,
        min_age_days: Option<u64>,
    ) -> Self {
        let gb = max_size_gb.unwrap_or(DEFAULT_MAX_SIZE_GB);
        let days = min_age_days.unwrap_or(DEFAULT_MIN_AGE_DAYS);

        Self {
            max_bytes: gb * 1024 * 1024 * 1024,
            min_age: Duration::from_secs(days * 86400),
            protected_paths: build_protected_paths(home, os),
        }
    }
}

pub fn safe_category_ids() -> &'static [&'static str] {
    SAFE_CATEGORY_IDS
}

fn build_protected_paths(home: &Path, os: OsKind) -> Vec<PathBuf> {
    let mut paths = vec![
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Music"),
        home.join("Videos"),
        home.join(".ssh"),
        home.join(".gnupg"),
        home.join(".gpg"),
        home.join(".config"),
        home.join(".local/share"),
    ];

    match os {
        OsKind::Mac => {
            paths.extend([
                PathBuf::from("/System"),
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                home.join("Library/Keychains"),
                home.join("Library/Application Support"),
                home.join("Library/Preferences"),
                home.join("Library/Mail"),
            ]);
        }
        OsKind::Linux | OsKind::FreeBSD => {
            paths.extend([
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/boot"),
                PathBuf::from("/lib"),
                PathBuf::from("/lib64"),
            ]);
        }
        OsKind::Windows => {
            paths.extend([
                PathBuf::from("C:\\Windows"),
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
            ]);
        }
        OsKind::Other => {}
    }

    paths
}

fn is_path_protected(path: &Path, protected: &[PathBuf]) -> bool {
    protected.iter().any(|p| path.starts_with(p))
}

fn is_old_enough(path: &Path, min_age: Duration) -> bool {
    let cutoff = match SystemTime::now().checked_sub(min_age) {
        Some(t) => t,
        None => return false,
    };

    path.metadata()
        .and_then(|m| m.modified())
        .map(|mtime| mtime <= cutoff)
        .unwrap_or(false)
}

pub fn filter_safe(findings: Vec<Finding>, config: &SafeConfig) -> (Vec<Finding>, Vec<String>) {
    let mut kept = Vec::new();
    let mut skipped = Vec::new();

    for finding in findings {
        if is_path_protected(&finding.path, &config.protected_paths) {
            skipped.push(format!(
                "PROTECTED: {} (inside protected directory)",
                finding.path.display()
            ));
            continue;
        }

        if !is_old_enough(&finding.path, config.min_age) {
            skipped.push(format!(
                "TOO_RECENT: {} (modified within minimum age window)",
                finding.path.display()
            ));
            continue;
        }

        kept.push(finding);
    }

    (kept, skipped)
}

pub fn exceeds_size_limit(findings: &[Finding], max_bytes: u64) -> Option<u64> {
    let total: u64 = findings.iter().map(|f| f.size).sum();
    if total > max_bytes {
        Some(total)
    } else {
        None
    }
}

pub fn write_safe_log(
    home: &Path,
    report: &CleanReport,
    items: &[&Finding],
    skipped: &[String],
    config: &SafeConfig,
) -> Result<()> {
    let log_dir = home.join(".oscleaner");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

    let log_path = log_dir.join("safe_run.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

    let now = utc_timestamp();
    writeln!(file, "\n{}", "=".repeat(80))?;
    writeln!(file, "[SAFE RUN] {now}")?;
    writeln!(
        file,
        "Config: max_size={}, min_age={}d",
        HumanBytes(config.max_bytes),
        config.min_age.as_secs() / 86400
    )?;
    writeln!(
        file,
        "Mode: {}",
        if report.dry_run { "DRY-RUN" } else { "LIVE" }
    )?;
    writeln!(file, "{}", "-".repeat(40))?;

    if !skipped.is_empty() {
        writeln!(file, "SKIPPED ({} items):", skipped.len())?;
        for reason in skipped {
            writeln!(file, "  {reason}")?;
        }
        writeln!(file, "{}", "-".repeat(40))?;
    }

    writeln!(file, "PROCESSED ({} items):", items.len())?;
    for item in items {
        writeln!(
            file,
            "  [{}] {} ({})",
            item.category_id,
            item.path.display(),
            HumanBytes(item.size)
        )?;
    }

    writeln!(file, "{}", "-".repeat(40))?;
    writeln!(
        file,
        "Result: attempted={}, succeeded={}, freed={}",
        report.attempted, report.succeeded, HumanBytes(report.freed_bytes)
    )?;

    if !report.errors.is_empty() {
        writeln!(file, "Errors:")?;
        for err in &report.errors {
            writeln!(file, "  {err}")?;
        }
    }

    writeln!(file, "{}", "=".repeat(80))?;

    println!(
        "{} {}",
        style("Safe run log written to:").dim(),
        style(log_path.display()).dim()
    );

    Ok(())
}

fn utc_timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let days = secs / 86400;
            let time_secs = secs % 86400;
            let hours = time_secs / 3600;
            let minutes = (time_secs % 3600) / 60;
            let seconds = time_secs % 60;
            let (year, month, day) = days_to_date(days);
            format!("{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02} UTC")
        }
        Err(_) => "unknown".to_string(),
    }
}

fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn print_safe_banner(config: &SafeConfig) {
    println!(
        "{} {} {}",
        style("SAFE MODE").bold().green(),
        style("->").dim(),
        style("cron-friendly automatic cleanup").dim()
    );
    println!(
        "  {} {}",
        style("Max total size:").dim(),
        style(HumanBytes(config.max_bytes)).yellow()
    );
    println!(
        "  {} {} days",
        style("Min file age:").dim(),
        style(config.min_age.as_secs() / 86400).yellow()
    );
    println!(
        "  {} {}",
        style("Protected paths:").dim(),
        style(format!("{} directories", config.protected_paths.len())).yellow()
    );
    println!(
        "  {} only regenerable caches/build artifacts",
        style("Categories:").dim(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn safe_category_ids_are_not_empty() {
        assert!(!SAFE_CATEGORY_IDS.is_empty());
    }

    #[test]
    fn is_path_protected_detects_nested_paths() {
        let protected = vec![PathBuf::from("/home/user/Documents")];
        assert!(is_path_protected(
            Path::new("/home/user/Documents/important.txt"),
            &protected
        ));
        assert!(!is_path_protected(
            Path::new("/home/user/Projects/node_modules"),
            &protected
        ));
    }

    #[test]
    fn is_old_enough_returns_true_for_old_files() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("old.txt");
        fs::write(&file, b"data").unwrap();

        assert!(!is_old_enough(&file, Duration::from_secs(2 * 86400)));
        assert!(is_old_enough(&file, Duration::from_secs(0)));
    }

    #[test]
    fn exceeds_size_limit_enforces_max() {
        let findings = vec![Finding {
            category_id: "test",
            category_name: "Test",
            category_description: "desc",
            path: PathBuf::from("/tmp/a"),
            size: 30 * 1024 * 1024 * 1024,
            is_dir: true,
        }];

        let max_20gb = 20 * 1024 * 1024 * 1024;
        assert!(exceeds_size_limit(&findings, max_20gb).is_some());

        let max_50gb = 50 * 1024 * 1024 * 1024;
        assert!(exceeds_size_limit(&findings, max_50gb).is_none());
    }

    #[test]
    fn filter_safe_removes_protected_paths() {
        let tmp = tempdir().unwrap();
        let protected_dir = tmp.path().join("Documents/node_modules");
        let safe_dir = tmp.path().join("Projects/node_modules");
        fs::create_dir_all(&protected_dir).unwrap();
        fs::create_dir_all(&safe_dir).unwrap();

        let config = SafeConfig {
            max_bytes: u64::MAX,
            min_age: Duration::from_secs(0),
            protected_paths: vec![tmp.path().join("Documents")],
        };

        let findings = vec![
            Finding {
                category_id: "test",
                category_name: "Test",
                category_description: "desc",
                path: protected_dir,
                size: 100,
                is_dir: true,
            },
            Finding {
                category_id: "test",
                category_name: "Test",
                category_description: "desc",
                path: safe_dir,
                size: 200,
                is_dir: true,
            },
        ];

        let (kept, skipped) = filter_safe(findings, &config);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].size, 200);
        assert_eq!(skipped.len(), 1);
        assert!(skipped[0].contains("PROTECTED"));
    }

    #[test]
    fn days_to_date_known_value() {
        let (y, m, d) = days_to_date(19723);
        assert_eq!((y, m, d), (2024, 1, 1));
    }
}
