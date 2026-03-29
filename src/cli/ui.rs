use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, MultiSelect, Select};
use indicatif::HumanBytes;

use crate::categories::CleanupCategory;
use crate::context::ScanContext;
use crate::types::{CategorySummary, Finding, OsKind, Platform};

pub fn print_banner(ctx: &ScanContext) {
    let os_label = match ctx.os {
        OsKind::Windows => "Windows",
        OsKind::Mac => "macOS",
        OsKind::Linux => "Linux",
        OsKind::FreeBSD => "FreeBSD",
        OsKind::Other => "Other",
    };
    println!(
        "{} {} {}",
        style("oscleaner").bold().blue(),
        style("->").dim(),
        style(os_label).green()
    );
    println!(
        "{}",
        style("Scan, preview, and clean development/system clutter cross-platform.").dim()
    );
}

pub fn prompt_main_action() -> Result<bool> {
    let options = vec!["Scan & preview cleanup targets", "Exit"];
    let choice = Select::new()
        .with_prompt("Choose what to do")
        .items(&options)
        .default(0)
        .interact()?;
    Ok(choice == 0)
}

pub fn show_summary(summaries: &[CategorySummary], duration: Duration) {
    println!(
        "{}",
        style("Scan complete. Categories detected (sorted by size):").bold()
    );
    for summary in summaries {
        println!(
            "{} {} | {} items",
            style(format!("{:<30}", summary.name)).bold(),
            style(format!("{:>10}", HumanBytes(summary.total_size))).yellow(),
            summary.items
        );
    }
    println!(
        "{} {}",
        style("Scan duration:").dim(),
        style(format!("{:.2?}", duration)).dim()
    );
}

pub fn prompt_category_selection(summaries: &[CategorySummary]) -> Result<HashSet<&'static str>> {
    let items: Vec<String> = summaries
        .iter()
        .map(|s| {
            format!(
                "{} - {} ({}, {} items)",
                s.name,
                s.description,
                HumanBytes(s.total_size),
                s.items
            )
        })
        .collect();
    let defaults: Vec<bool> = summaries.iter().map(|_| true).collect();
    let selections = MultiSelect::new()
        .with_prompt("Select categories to clean")
        .items(&items)
        .defaults(&defaults)
        .interact()?;

    let mut ids = HashSet::new();
    for idx in selections {
        if let Some(summary) = summaries.get(idx) {
            ids.insert(summary.id);
        }
    }
    Ok(ids)
}

pub fn prompt_item_selection<'a>(items: &'a [&'a Finding]) -> Result<Vec<&'a Finding>> {
    let options = vec!["Delete all", "Select one by one"];
    let choice = Select::new()
        .with_prompt("How do you want to proceed?")
        .items(&options)
        .default(0)
        .interact()?;

    if choice == 0 {
        return Ok(items.to_vec());
    }

    let labels: Vec<String> = items
        .iter()
        .map(|f| {
            format!(
                "[{}] {} ({})",
                f.category_name,
                f.path.display(),
                HumanBytes(f.size)
            )
        })
        .collect();
    let defaults: Vec<bool> = items.iter().map(|_| true).collect();
    let selections = MultiSelect::new()
        .with_prompt("Select items to delete")
        .items(&labels)
        .defaults(&defaults)
        .interact()?;

    Ok(selections.into_iter().filter_map(|i| items.get(i).copied()).collect())
}

pub fn confirm_dry_run(force_dry_run: bool) -> Result<bool> {
    if force_dry_run {
        println!(
            "{}",
            style("Dry-run flag detected; no files will be deleted.").yellow()
        );
        return Ok(true);
    }

    Confirm::new()
        .with_prompt("Perform a dry run (preview only)?")
        .default(true)
        .interact()
        .map_err(Into::into)
}

pub fn confirm_cleanup(potential_bytes: u64) -> Result<bool> {
    Confirm::new()
        .with_prompt(format!(
            "Proceed to delete and free approximately {}?",
            HumanBytes(potential_bytes)
        ))
        .default(false)
        .interact()
        .map_err(Into::into)
}

pub fn print_categories_table(categories: &[CleanupCategory]) {
    println!("{}", style("Available cleanup categories:").bold());
    for cat in categories {
        let platform = match cat.platform {
            Platform::All => "all",
            Platform::Windows => "windows",
            Platform::Mac => "mac",
            Platform::Linux => "linux",
            Platform::Unix => "unix",
        };
        println!(
            "--{:<20} {:<8} {}",
            cat.id.replace('_', "-"),
            platform,
            cat.description
        );
    }
}
