use std::fs;

use console::style;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};

use crate::fs_utils::shorten_path;
use crate::types::{CleanReport, Finding};

pub fn perform_cleanup(items: &[&Finding], dry_run: bool) -> CleanReport {
    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} {pos}/{len}")
            .expect("hardcoded progress template is valid")
            .tick_chars("|/-\\"),
    );

    let mut freed = 0u64;
    let mut errors = Vec::new();
    let mut succeeded = 0usize;

    for item in items {
        let label = shorten_path(&item.path);
        pb.set_message(format!(
            "{} {}",
            if dry_run { "Preview" } else { "Removing" },
            label
        ));

        if dry_run {
            pb.inc(1);
            continue;
        }

        let result = if item.is_dir {
            fs::remove_dir_all(&item.path)
        } else {
            fs::remove_file(&item.path)
        };

        match result {
            Ok(_) => {
                freed += item.size;
                succeeded += 1;
            }
            Err(err) => errors.push(format!("{}: {}", item.path.display(), err)),
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    CleanReport {
        dry_run,
        attempted: items.len(),
        succeeded,
        freed_bytes: freed,
        errors,
    }
}

pub fn print_report(report: &CleanReport) {
    if report.dry_run {
        println!(
            "{}",
            style("Dry-run complete. No files were deleted.")
                .yellow()
                .bold()
        );
    } else {
        println!("{}", style("Cleanup complete.").green().bold());
    }

    println!(
        "{} {}, {} succeeded, {} failed",
        style("Items processed:").bold(),
        report.attempted,
        report.succeeded,
        report.attempted.saturating_sub(report.succeeded)
    );

    if !report.dry_run {
        println!(
            "{} {}",
            style("Disk space reclaimed:").bold(),
            style(HumanBytes(report.freed_bytes)).yellow()
        );
    }

    if !report.errors.is_empty() {
        println!("{}", style("Errors:").red().bold());
        for err in &report.errors {
            println!("  - {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_finding(path: &std::path::Path, size: u64, is_dir: bool) -> Finding {
        Finding {
            category_id: "test",
            category_name: "Test",
            category_description: "desc",
            path: path.to_path_buf(),
            size,
            is_dir,
        }
    }

    #[test]
    fn perform_cleanup_respects_dry_run() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("sample.txt");
        fs::write(&file_path, b"12345").unwrap();

        let finding = make_finding(&file_path, 5, false);
        let report = perform_cleanup(&[&finding], true);

        assert!(file_path.exists());
        assert!(report.dry_run);
        assert_eq!(report.attempted, 1);
        assert_eq!(report.succeeded, 0);
        assert_eq!(report.freed_bytes, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn perform_cleanup_removes_files_and_directories() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file.txt");
        let dir_path = tmp.path().join("dir");
        let nested_file = dir_path.join("nested.bin");

        fs::write(&file_path, b"hello").unwrap();
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&nested_file, vec![1u8; 2]).unwrap();

        let file_finding = make_finding(&file_path, 5, false);
        let dir_finding = make_finding(&dir_path, 2, true);
        let items = vec![&file_finding, &dir_finding];

        let report = perform_cleanup(&items, false);

        assert!(!file_path.exists());
        assert!(!dir_path.exists());
        assert!(report.errors.is_empty());
        assert!(!report.dry_run);
        assert_eq!(report.attempted, 2);
        assert_eq!(report.succeeded, 2);
        assert_eq!(report.freed_bytes, 7);
    }
}
