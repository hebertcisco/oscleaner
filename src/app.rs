use std::collections::HashSet;
use std::time::Instant;

use anyhow::Result;
use console::style;
use indicatif::HumanBytes;
use serde::Serialize;

use crate::categories::{build_categories, CleanupCategory};
use crate::cleanup::{perform_cleanup, print_report};
use crate::cli::{
    confirm_cleanup, confirm_dry_run, print_banner, print_categories_table,
    prompt_category_selection, prompt_item_selection, prompt_main_action, show_summary, CliOptions,
    RunMode,
};
use crate::context::ScanContext;
use crate::safe::{self, SafeConfig};
use crate::scanner::{filter_findings, scan_categories, summarize_findings};
use crate::types::{CategorySummary, CleanReport, Finding};

#[derive(Serialize)]
struct JsonScanOutput {
    findings: Vec<JsonFinding>,
    summaries: Vec<CategorySummary>,
    scan_duration_ms: u64,
}

#[derive(Serialize)]
struct JsonFinding {
    category_id: &'static str,
    category_name: &'static str,
    path: String,
    size: u64,
    is_dir: bool,
}

impl From<&Finding> for JsonFinding {
    fn from(f: &Finding) -> Self {
        Self {
            category_id: f.category_id,
            category_name: f.category_name,
            path: f.path.display().to_string(),
            size: f.size,
            is_dir: f.is_dir,
        }
    }
}

#[derive(Serialize)]
struct JsonCleanOutput {
    report: CleanReport,
}

pub fn run() -> Result<()> {
    let opts = CliOptions::from_env();
    let ctx = ScanContext::new()?;

    if !opts.json {
        print_banner(&ctx);
    }

    let categories = build_categories();
    let mode = opts.mode();
    let requested_ids = opts.resolve_category_ids(&categories)?;

    match mode {
        RunMode::List => {
            if opts.json {
                let cats: Vec<_> = categories
                    .iter()
                    .filter(|c| c.platform.matches(ctx.os))
                    .map(|c| serde_json::json!({
                        "id": c.id,
                        "name": c.name,
                        "description": c.description,
                    }))
                    .collect();
                println!("{}", serde_json::to_string(&cats)?);
            } else {
                print_categories_table(&categories);
            }
        }
        RunMode::Scan => {
            run_scan_command(&categories, &ctx, &requested_ids, opts.targets().all, opts.json)?;
        }
        RunMode::Clean => {
            run_clean_command(&categories, &ctx, &requested_ids, &opts)?;
        }
        RunMode::Interactive => {
            if opts.effective_yes() {
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
    json: bool,
) -> Result<()> {
    let categories_to_scan = pick_categories(categories, requested_ids, select_all);
    let (findings, scan_duration) = run_scan(&categories_to_scan, ctx, json)?;

    if json {
        let summaries = summarize_findings(&findings);
        let output = JsonScanOutput {
            findings: findings.iter().map(JsonFinding::from).collect(),
            summaries,
            scan_duration_ms: scan_duration.as_millis() as u64,
        };
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }

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
    let targets = opts.targets();
    let yes = opts.effective_yes();
    let json = opts.json;
    let safe_config = if opts.safe {
        let cfg = SafeConfig::new(&ctx.home, ctx.os, opts.max_size_gb, opts.min_age_days);
        if !json {
            safe::print_safe_banner(&cfg);
        }
        Some(cfg)
    } else {
        None
    };

    let categories_to_scan = pick_categories(categories, requested_ids, targets.all || opts.safe);
    let (findings, scan_duration) = run_scan(&categories_to_scan, ctx, json)?;

    if findings.is_empty() {
        if json {
            println!("{}", serde_json::to_string(&JsonCleanOutput {
                report: CleanReport {
                    dry_run: opts.dry_run,
                    attempted: 0,
                    succeeded: 0,
                    freed_bytes: 0,
                    errors: vec![],
                },
            })?);
        } else {
            println!(
                "{}",
                style("No cleanup targets detected. You're already tidy!").green()
            );
        }
        return Ok(());
    }

    let (findings, skipped_reasons) = if let Some(ref cfg) = safe_config {
        let (kept, skipped) = safe::filter_safe(findings, cfg);
        if !json && !skipped.is_empty() {
            println!(
                "{} {} items skipped (protected or too recent)",
                style("Safe filter:").bold(),
                skipped.len()
            );
        }
        (kept, skipped)
    } else {
        (findings, Vec::new())
    };

    if findings.is_empty() {
        if json {
            println!("{}", serde_json::to_string(&JsonCleanOutput {
                report: CleanReport {
                    dry_run: opts.dry_run,
                    attempted: 0,
                    succeeded: 0,
                    freed_bytes: 0,
                    errors: vec![],
                },
            })?);
        } else {
            println!(
                "{}",
                style("No cleanup targets remain after safe filtering.").green()
            );
        }
        return Ok(());
    }

    if let Some(ref cfg) = safe_config
        && let Some(total) = safe::exceeds_size_limit(&findings, cfg.max_bytes)
    {
        if !json {
            println!(
                "{} Total size {} exceeds safe limit of {}. Aborting.",
                style("SAFE ABORT:").red().bold(),
                style(HumanBytes(total)).yellow(),
                style(HumanBytes(cfg.max_bytes)).yellow()
            );
        }
        return Ok(());
    }

    let summaries = summarize_findings(&findings);
    if !json {
        show_summary(&summaries, scan_duration);
    }

    let mut selected_ids = requested_ids.clone();
    if selected_ids.is_empty() && (targets.all || yes || opts.safe) {
        selected_ids.extend(summaries.iter().map(|s| s.id));
    } else if selected_ids.is_empty() && !json {
        selected_ids = prompt_category_selection(&summaries)?;
    }

    if selected_ids.is_empty() {
        if !json {
            println!("{}", style("No categories selected. Exiting.").yellow());
        }
        return Ok(());
    }

    let (selected_items, potential_bytes) = filter_findings(&findings, &selected_ids);
    if selected_items.is_empty() {
        if !json {
            println!("{}", style("No matching targets to clean.").yellow());
        }
        return Ok(());
    }

    let final_items: Vec<&Finding>;
    let final_bytes: u64;

    if json || yes {
        final_bytes = potential_bytes;
        final_items = selected_items;
    } else {
        let chosen = prompt_item_selection(&selected_items)?;
        if chosen.is_empty() {
            println!("{}", style("No items selected. Exiting.").yellow());
            return Ok(());
        }
        final_bytes = chosen.iter().map(|f| f.size).sum();
        final_items = chosen;
    }

    if !json {
        println!(
            "{} {} across {} items will be removed.",
            style("Potential reclaim:").bold(),
            style(HumanBytes(final_bytes)).yellow().bold(),
            final_items.len()
        );
    }

    let mut dry_run = opts.dry_run;
    if !json && !yes && !opts.dry_run {
        dry_run = confirm_dry_run(false)?;
    }

    if !dry_run && !json && !yes {
        let proceed = confirm_cleanup(final_bytes)?;
        if !proceed {
            println!("{}", style("Cancelled by user.").yellow());
            return Ok(());
        }
    }

    let report = perform_cleanup(&final_items, dry_run);

    if json {
        println!("{}", serde_json::to_string(&JsonCleanOutput { report })?);
    } else {
        print_report(&report);

        if let Some(ref cfg) = safe_config
            && let Err(err) = safe::write_safe_log(&ctx.home, &report, &final_items, &skipped_reasons, cfg)
        {
            eprintln!(
                "{} Failed to write safe run log: {}",
                style("WARNING:").yellow(),
                err
            );
        }
    }

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

    let (findings, scan_duration) = run_scan(categories, ctx, false)?;
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

    let (selected_items, _potential_bytes) = filter_findings(&findings, &selected_ids);
    if selected_items.is_empty() {
        println!("{}", style("No matching targets to clean.").yellow());
        return Ok(());
    }

    let chosen = prompt_item_selection(&selected_items)?;
    if chosen.is_empty() {
        println!("{}", style("No items selected. Exiting.").yellow());
        return Ok(());
    }
    let final_bytes: u64 = chosen.iter().map(|f| f.size).sum();

    println!(
        "{} {} across {} items will be removed.",
        style("Potential reclaim:").bold(),
        style(HumanBytes(final_bytes)).yellow().bold(),
        chosen.len()
    );

    let mut dry_run = opts.dry_run;
    if !opts.dry_run {
        dry_run = confirm_dry_run(false)?;
    }
    if !dry_run {
        let proceed = confirm_cleanup(final_bytes)?;
        if !proceed {
            println!("{}", style("Cancelled by user.").yellow());
            return Ok(());
        }
    }

    let report = perform_cleanup(&chosen, dry_run);
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
    json: bool,
) -> Result<(Vec<Finding>, std::time::Duration)> {
    let start = Instant::now();
    let findings = if json {
        scan_categories_quiet(categories, ctx)?
    } else {
        scan_categories(categories, ctx)?
    };
    let duration = start.elapsed();
    Ok((findings, duration))
}

fn scan_categories_quiet(
    categories: &[CleanupCategory],
    ctx: &ScanContext,
) -> Result<Vec<Finding>> {
    use crate::fs_utils::calc_size;
    use std::collections::HashSet;
    use std::path::PathBuf;

    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut findings = Vec::new();

    for cat in categories {
        if !cat.platform.matches(ctx.os) {
            continue;
        }

        let paths = (cat.detector)(ctx);
        for path in paths {
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }

            let is_dir = path.is_dir();
            if let Ok(size) = calc_size(&path) {
                if size > 0 {
                    findings.push(Finding {
                        category_id: cat.id,
                        category_name: cat.name,
                        category_description: cat.description,
                        path,
                        size,
                        is_dir,
                    });
                }
            }
        }
    }

    Ok(findings)
}
