use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::categories::CleanupCategory;
use crate::context::ScanContext;
use crate::fs_utils::calc_size;
use crate::types::{CategorySummary, Finding};

pub fn scan_categories(categories: &[CleanupCategory], ctx: &ScanContext) -> Result<Vec<Finding>> {
    let pb = ProgressBar::new(categories.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg} ({pos}/{len})")
            .expect("hardcoded progress template is valid")
            .tick_chars("|/-\\"),
    );

    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut findings = Vec::new();

    for cat in categories {
        if !cat.platform.matches(ctx.os) {
            pb.inc(1);
            continue;
        }

        pb.set_message(format!("Scanning {}", cat.name));
        let paths = (cat.detector)(ctx);
        for path in paths {
            if !path.exists() {
                continue;
            }
            if !ctx.is_path_in_scope(&path) {
                continue;
            }
            if !seen.insert(path.clone()) {
                continue;
            }

            let is_dir = path.is_dir();
            match calc_size(&path) {
                Ok(size) if size > 0 => findings.push(Finding {
                    category_id: cat.id,
                    category_name: cat.name,
                    category_description: cat.description,
                    path,
                    size,
                    is_dir,
                }),
                Ok(_) => {}
                Err(err) => eprintln!(
                    "{} {} ({})",
                    style("Failed to inspect").red(),
                    path.display(),
                    err
                ),
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    Ok(findings)
}

pub fn summarize_findings(findings: &[Finding]) -> Vec<CategorySummary> {
    let mut map: HashMap<&'static str, CategorySummary> = HashMap::new();
    for finding in findings {
        let entry = map.entry(finding.category_id).or_insert(CategorySummary {
            id: finding.category_id,
            name: finding.category_name,
            description: finding.category_description,
            total_size: 0,
            items: 0,
        });
        entry.total_size += finding.size;
        entry.items += 1;
    }

    let mut summaries: Vec<_> = map.into_values().collect();
    summaries.sort_by_key(|s| std::cmp::Reverse(s.total_size));
    summaries
}

pub fn filter_findings<'a>(
    findings: &'a [Finding],
    category_ids: &HashSet<&'static str>,
) -> (Vec<&'a Finding>, u64) {
    let mut items = Vec::new();
    let mut total = 0;
    for finding in findings {
        if category_ids.contains(&finding.category_id) {
            total += finding.size;
            items.push(finding);
        }
    }
    items.sort_by_key(|f| std::cmp::Reverse(f.size));
    (items, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_findings_groups_and_sorts_by_size() {
        let findings = vec![
            Finding {
                category_id: "a",
                category_name: "A",
                category_description: "desc",
                path: PathBuf::from("a1"),
                size: 100,
                is_dir: true,
            },
            Finding {
                category_id: "a",
                category_name: "A",
                category_description: "desc",
                path: PathBuf::from("a2"),
                size: 50,
                is_dir: false,
            },
            Finding {
                category_id: "b",
                category_name: "B",
                category_description: "desc",
                path: PathBuf::from("b1"),
                size: 200,
                is_dir: true,
            },
        ];

        let summaries = summarize_findings(&findings);
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].id, "b");
        assert_eq!(summaries[0].total_size, 200);
        assert_eq!(summaries[1].id, "a");
        assert_eq!(summaries[1].total_size, 150);
        assert_eq!(summaries[1].items, 2);
    }

    #[test]
    fn filter_findings_selects_and_sorts_subset() {
        let findings = vec![
            Finding {
                category_id: "keep",
                category_name: "Keep",
                category_description: "desc",
                path: PathBuf::from("first"),
                size: 10,
                is_dir: false,
            },
            Finding {
                category_id: "other",
                category_name: "Other",
                category_description: "desc",
                path: PathBuf::from("skip"),
                size: 999,
                is_dir: false,
            },
            Finding {
                category_id: "keep",
                category_name: "Keep",
                category_description: "desc",
                path: PathBuf::from("second"),
                size: 30,
                is_dir: true,
            },
        ];
        let mut selected = HashSet::new();
        selected.insert("keep");

        let (filtered, bytes) = filter_findings(&findings, &selected);
        assert_eq!(bytes, 40);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].path, PathBuf::from("second"));
        assert_eq!(filtered[1].path, PathBuf::from("first"));
    }
}
