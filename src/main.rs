mod categories;
mod cleanup;
mod cli;
mod context;
mod detectors;
mod fs_utils;
mod scanner;
mod types;

use std::collections::HashSet;
use std::time::Instant;

use anyhow::Result;
use console::style;
use indicatif::HumanBytes;

use crate::categories::{build_categories, CleanupCategory};
use crate::cleanup::{perform_cleanup, print_report};
use crate::cli::{
    confirm_cleanup, confirm_dry_run, print_banner, print_categories_table,
    prompt_category_selection, prompt_main_action, show_summary, CliOptions, RunMode,
};
use crate::context::ScanContext;
use crate::scanner::{filter_findings, scan_categories, summarize_findings};
use crate::types::Finding;

fn main() -> Result<()> {
    let opts = CliOptions::from_env();
    let ctx = ScanContext::new()?;
    print_banner(&ctx);

    let categories = build_categories();
    let mode = opts.mode();
    let requested_ids = opts.resolve_category_ids(&categories)?;

    match mode {
        RunMode::List => {
            print_categories_table(&categories);
        }
        RunMode::Scan => {
            run_scan_command(&categories, &ctx, &requested_ids, opts.all)?;
        }
        RunMode::Clean => {
            run_clean_command(&categories, &ctx, &requested_ids, &opts)?;
        }
        RunMode::Interactive => {
            if opts.yes {
                run_clean_command(&categories, &ctx, &requested_ids, &opts)?;
            } else {
                run_interactive_flow(&categories, &ctx, &opts)?;
            }
        }
    }

    Ok(())
}

fn run_scan_command(
    categories: &[CleanupCategory],
    ctx: &ScanContext,
    requested_ids: &HashSet<&'static str>,
    select_all: bool,
) -> Result<()> {
    let categories_to_scan = pick_categories(categories, requested_ids, select_all);
    let (findings, scan_duration) = run_scan(&categories_to_scan, ctx)?;

    if findings.is_empty() {
        println!(
            "{}",
            style("No cleanup targets detected. You're already tidy!").green()
        );
        return Ok(());
    }

    let summaries = summarize_findings(&findings);
    show_summary(&summaries, scan_duration);
    Ok(())
}

fn run_clean_command(
    categories: &[CleanupCategory],
    ctx: &ScanContext,
    requested_ids: &HashSet<&'static str>,
    opts: &CliOptions,
) -> Result<()> {
    let categories_to_scan = pick_categories(categories, requested_ids, opts.all);
    let (findings, scan_duration) = run_scan(&categories_to_scan, ctx)?;

    if findings.is_empty() {
        println!(
            "{}",
            style("No cleanup targets detected. You're already tidy!").green()
        );
        return Ok(());
    }

    let summaries = summarize_findings(&findings);
    show_summary(&summaries, scan_duration);

    let mut selected_ids = requested_ids.clone();
    if selected_ids.is_empty() && (opts.all || opts.yes) {
        selected_ids.extend(summaries.iter().map(|s| s.id));
    } else if selected_ids.is_empty() {
        selected_ids = prompt_category_selection(&summaries)?;
    }

    if selected_ids.is_empty() {
        println!("{}", style("No categories selected. Exiting.").yellow());
        return Ok(());
    }

    let (selected_items, potential_bytes) = filter_findings(&findings, &selected_ids);
    if selected_items.is_empty() {
        println!("{}", style("No matching targets to clean.").yellow());
        return Ok(());
    }

    println!(
        "{} {} across {} items will be removed.",
        style("Potential reclaim:").bold(),
        style(HumanBytes(potential_bytes)).yellow().bold(),
        selected_items.len()
    );

    let mut dry_run = opts.dry_run;
    if !opts.yes && !opts.dry_run {
        dry_run = confirm_dry_run(false)?;
    }

    if !dry_run && !opts.yes {
        let proceed = confirm_cleanup(potential_bytes)?;
        if !proceed {
            println!("{}", style("Cancelled by user.").yellow());
            return Ok(());
        }
    }

    let report = perform_cleanup(&selected_items, dry_run);
    print_report(&report);
    Ok(())
}

fn run_interactive_flow(
    categories: &[CleanupCategory],
    ctx: &ScanContext,
    opts: &CliOptions,
) -> Result<()> {
    if !prompt_main_action()? {
        println!("{}", style("Goodbye!").cyan());
        return Ok(());
    }

    let (findings, scan_duration) = run_scan(categories, ctx)?;
    if findings.is_empty() {
        println!(
            "{}",
            style("No cleanup targets detected. You're already tidy!").green()
        );
        return Ok(());
    }

    let summaries = summarize_findings(&findings);
    show_summary(&summaries, scan_duration);

    let selected_ids = prompt_category_selection(&summaries)?;
    if selected_ids.is_empty() {
        println!("{}", style("No categories selected. Exiting.").yellow());
        return Ok(());
    }

    let (selected_items, potential_bytes) = filter_findings(&findings, &selected_ids);
    println!(
        "{} {} across {} items will be removed.",
        style("Potential reclaim:").bold(),
        style(HumanBytes(potential_bytes)).yellow().bold(),
        selected_items.len()
    );

    let mut dry_run = opts.dry_run;
    if !opts.dry_run {
        dry_run = confirm_dry_run(false)?;
    }
    if !dry_run {
        let proceed = confirm_cleanup(potential_bytes)?;
        if !proceed {
            println!("{}", style("Cancelled by user.").yellow());
            return Ok(());
        }
    }

    let report = perform_cleanup(&selected_items, dry_run);
    print_report(&report);
    Ok(())
}

fn pick_categories(
    all: &[CleanupCategory],
    requested_ids: &HashSet<&'static str>,
    select_all: bool,
) -> Vec<CleanupCategory> {
    if select_all || requested_ids.is_empty() {
        all.to_vec()
    } else {
        all.iter()
            .filter(|cat| requested_ids.contains(&cat.id))
            .cloned()
            .collect()
    }
}

fn run_scan(
    categories: &[CleanupCategory],
    ctx: &ScanContext,
) -> Result<(Vec<Finding>, std::time::Duration)> {
    let start = Instant::now();
    let findings = scan_categories(categories, ctx)?;
    let duration = start.elapsed();
    Ok((findings, duration))
}
