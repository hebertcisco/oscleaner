use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use console::style;
use indicatif::HumanBytes;

use crate::types::{CleanReport, Finding, OsKind};

/// Categories that are safe to clean automatically.
/// These are all regenerable caches and build artifacts.
const SAFE_CATEGORY_IDS: &[&str] = &[
    "node_modules",    // regenerable: npm install / yarn
    "cargo_targets",   // regenerable: cargo build
    "gradle_cache",    // regenerable: gradle sync
    "maven_cache",     // regenerable: mvn install
    "php_vendor",      // regenerable: composer install
    "ruby_vendor",     // regenerable: bundle install
    "python_cache",    // regenerable: __pycache__ auto-created
    "cocoapods_cache", // regenerable: pod install
    "android_builds",  // regenerable: gradle build
    "react_native_ios", // regenerable: pod install + xcodebuild
    "xcode",           // regenerable: DerivedData rebuilt on next build
    "homebrew_cache",  // regenerable: brew downloads
    "browser_caches",  // regenerable: browsers re-download
    "snap_cache",      // regenerable: snap re-downloads
    "flatpak_cache",   // regenerable: flatpak re-downloads
];

/// Categories explicitly excluded from safe mode and why:
/// - docker:           could remove running containers/images in use
/// - ios_backups:      irreplaceable device backups
/// - mail_downloads:   may contain important attachments
/// - mac_caches:       ~/Library/Caches is too broad, includes app state
/// - mac_logs:         system/user logs needed for debugging
/// - mac_tmp:          /tmp may contain files in use by running processes
/// - windows_temp:     may contain files in use by running processes
/// - windows_update:   system-critical update cache
/// - windows_thumbnail: system UI cache
/// - windows_prefetch: system performance data
/// - windows_wer:      error reports useful for debugging
/// - linux_cache:      ~/.cache is too broad, includes app state
/// - linux_logs:       system logs needed for debugging
/// - linux_tmp:        /tmp may contain files in use by running processes
/// - linux_journal:    systemd logs needed for debugging
/// - linux_coredumps:  useful for post-mortem debugging
/// - linux_trash:      user may want to recover deleted files

const DEFAULT_MAX_SIZE_GB: u64 = 20;
const DEFAULT_MIN_AGE_DAYS: u64 = 2;

pub struct SafeConfig {
    pub max_bytes: u64,
    pub min_age: Duration,
    pub protected_paths: Vec<PathBuf>,
}

impl SafeConfig {
    pub fn new(home: &Path, os: OsKind, max_size_gb: Option<u64>, min_age_days: Option<u64>) -> Self {
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
        // User personal directories (all platforms)
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Music"),
        home.join("Videos"),
        // Security-sensitive
        home.join(".ssh"),
        home.join(".gnupg"),
        home.join(".gpg"),
        // Configuration
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

/// Returns true if the path is inside (or equal to) any protected directory.
pub fn is_path_protected(path: &Path, protected: &[PathBuf]) -> bool {
    for protected_path in protected {
        if path.starts_with(protected_path) {
            return true;
        }
    }
    false
}

/// Returns true if the path was last modified more than `min_age` ago.
/// If metadata cannot be read, returns false (skip the item to be safe).
pub fn is_old_enough(path: &Path, min_age: Duration) -> bool {
    let cutoff = match SystemTime::now().checked_sub(min_age) {
        Some(t) => t,
        None => return false,
    };

    let modified = if path.is_dir() {
        // For directories, check the dir's own mtime
        path.metadata().and_then(|m| m.modified())
    } else {
        path.metadata().and_then(|m| m.modified())
    };

    match modified {
        Ok(mtime) => mtime <= cutoff,
        Err(_) => false,
    }
}

/// Filter findings by safe mode rules. Returns (kept, skipped_reasons).
pub fn filter_safe(
    findings: Vec<Finding>,
    config: &SafeConfig,
) -> (Vec<Finding>, Vec<String>) {
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

/// Check if total size exceeds the max allowed.
pub fn check_size_limit(findings: &[Finding], max_bytes: u64) -> Result<(), u64> {
    let total: u64 = findings.iter().map(|f| f.size).sum();
    if total > max_bytes {
        Err(total)
    } else {
        Ok(())
    }
}

/// Write the safe run log to ~/.oscleaner/safe_run.log
pub fn write_safe_log(
    home: &Path,
    report: &CleanReport,
    items: &[&Finding],
    skipped: &[String],
    config: &SafeConfig,
) {
    let log_dir = home.join(".oscleaner");
    if fs::create_dir_all(&log_dir).is_err() {
        eprintln!(
            "{} Failed to create log directory: {}",
            style("WARNING:").yellow(),
            log_dir.display()
        );
        return;
    }

    let log_path = log_dir.join("safe_run.log");
    let mut file = match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!(
                "{} Failed to open log file: {} ({})",
                style("WARNING:").yellow(),
                log_path.display(),
                err
            );
            return;
        }
    };

    let now = chrono_free_timestamp();
    let _ = writeln!(file, "\n{}", "=".repeat(80));
    let _ = writeln!(file, "[SAFE RUN] {}", now);
    let _ = writeln!(
        file,
        "Config: max_size={}, min_age={}d",
        HumanBytes(config.max_bytes),
        config.min_age.as_secs() / 86400
    );
    let _ = writeln!(file, "Mode: {}", if report.dry_run { "DRY-RUN" } else { "LIVE" });
    let _ = writeln!(file, "{}", "-".repeat(40));

    if !skipped.is_empty() {
        let _ = writeln!(file, "SKIPPED ({} items):", skipped.len());
        for reason in skipped {
            let _ = writeln!(file, "  {}", reason);
        }
        let _ = writeln!(file, "{}", "-".repeat(40));
    }

    let _ = writeln!(file, "PROCESSED ({} items):", items.len());
    for item in items {
        let _ = writeln!(
            file,
            "  [{}] {} ({})",
            item.category_id,
            item.path.display(),
            HumanBytes(item.size)
        );
    }

    let _ = writeln!(file, "{}", "-".repeat(40));
    let _ = writeln!(
        file,
        "Result: attempted={}, succeeded={}, freed={}",
        report.attempted,
        report.succeeded,
        HumanBytes(report.freed_bytes)
    );

    if !report.errors.is_empty() {
        let _ = writeln!(file, "Errors:");
        for err in &report.errors {
            let _ = writeln!(file, "  {}", err);
        }
    }

    let _ = writeln!(file, "{}", "=".repeat(80));

    println!(
        "{} {}",
        style("Safe run log written to:").dim(),
        style(log_path.display()).dim()
    );
}

fn chrono_free_timestamp() -> String {
    // Use UNIX_EPOCH to produce a readable timestamp without chrono dependency
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            // Simple UTC breakdown
            let days = secs / 86400;
            let time_secs = secs % 86400;
            let hours = time_secs / 3600;
            let minutes = (time_secs % 3600) / 60;
            let seconds = time_secs % 60;

            // Days since 1970-01-01
            let (year, month, day) = days_to_date(days);
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                year, month, day, hours, minutes, seconds
            )
        }
        Err(_) => "unknown".to_string(),
    }
}

fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
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
        style(format!("{}", HumanBytes(config.max_bytes))).yellow()
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

        // File was just created, so it should NOT be old enough with 2-day min
        assert!(!is_old_enough(&file, Duration::from_secs(2 * 86400)));

        // But it should be old enough with 0-second min
        assert!(is_old_enough(&file, Duration::from_secs(0)));
    }

    #[test]
    fn check_size_limit_enforces_max() {
        let findings = vec![Finding {
            category_id: "test",
            category_name: "Test",
            category_description: "desc",
            path: PathBuf::from("/tmp/a"),
            size: 30 * 1024 * 1024 * 1024, // 30 GB
            is_dir: true,
        }];

        let max_20gb = 20 * 1024 * 1024 * 1024;
        assert!(check_size_limit(&findings, max_20gb).is_err());

        let max_50gb = 50 * 1024 * 1024 * 1024;
        assert!(check_size_limit(&findings, max_50gb).is_ok());
    }

    #[test]
    fn filter_safe_removes_protected_paths() {
        let tmp = tempdir().unwrap();
        // Create actual dirs so is_old_enough can read metadata
        let protected_dir = tmp.path().join("Documents/node_modules");
        let safe_dir = tmp.path().join("Projects/node_modules");
        fs::create_dir_all(&protected_dir).unwrap();
        fs::create_dir_all(&safe_dir).unwrap();

        let config = SafeConfig {
            max_bytes: u64::MAX,
            min_age: Duration::from_secs(0), // no age filter
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
        // 2024-01-01 = 19723 days since epoch
        let (y, m, d) = days_to_date(19723);
        assert_eq!((y, m, d), (2024, 1, 1));
    }
}
