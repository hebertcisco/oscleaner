use std::collections::HashSet;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use console::style;
use dialoguer::{Confirm, MultiSelect, Select};
use indicatif::HumanBytes;

use crate::categories::CleanupCategory;
use crate::context::ScanContext;
use crate::types::{CategorySummary, OsKind, Platform};

#[derive(Parser, Debug, Default)]
#[command(name = "oscleaner", version, about = "Scan, preview, and clean development/system clutter")]
pub struct CliOptions {
    #[arg(short = 'n', long = "dry-run", help = "Preview deletions without removing files")]
    pub dry_run: bool,
    #[arg(short = 'Y', long = "yes", help = "Skip interactive confirmations and proceed directly")]
    pub yes: bool,
    #[arg(long = "all", help = "Target every available category")]
    pub all: bool,
    #[arg(
        short = 'c',
        long = "category",
        value_name = "ID",
        help = "Cleanup category by id (repeatable)",
        action = clap::ArgAction::Append
    )]
    pub categories: Vec<String>,
    #[command(flatten)]
    pub category_flags: CategoryFlags,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Scan,
    Clean,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Interactive,
    Scan,
    Clean,
    List,
}

#[derive(Args, Debug, Default, Clone)]
pub struct CategoryFlags {
    #[arg(long, help = "Node.js node_modules folders")]
    pub node_modules: bool,
    #[arg(long, help = "Docker caches, images, containers, and build data")]
    pub docker: bool,
    #[arg(long, help = "Xcode DerivedData and archives (macOS)")]
    pub xcode: bool,
    #[arg(long, help = "Android build folders")]
    pub android_builds: bool,
    #[arg(long, help = "React Native iOS Pods/builds")]
    pub react_native_ios: bool,
    #[arg(long, help = "Gradle cache")]
    pub gradle_cache: bool,
    #[arg(long, help = "Maven repository cache")]
    pub maven_cache: bool,
    #[arg(long, help = "Cargo target directories")]
    pub cargo_targets: bool,
    #[arg(long, help = "Python __pycache__, pyc files, and virtualenvs")]
    pub python_cache: bool,
    #[arg(long, help = "CocoaPods cache (macOS)")]
    pub cocoapods_cache: bool,
    #[arg(long, help = "macOS user caches")]
    pub mac_caches: bool,
    #[arg(long, help = "macOS system and user logs")]
    pub mac_logs: bool,
    #[arg(long, help = "macOS temporary files")]
    pub mac_tmp: bool,
    #[arg(long, help = "iOS device backups (macOS)")]
    pub ios_backups: bool,
    #[arg(long, help = "Homebrew cache (macOS)")]
    pub homebrew_cache: bool,
    #[arg(long, help = "Mail downloads cache (macOS)")]
    pub mail_downloads: bool,
    #[arg(long, help = "Windows temp folder")]
    pub windows_temp: bool,
    #[arg(long, help = "Windows Update cache")]
    pub windows_update: bool,
    #[arg(long, help = "Windows thumbnail cache")]
    pub windows_thumbnail: bool,
    #[arg(long, help = "Windows prefetch files")]
    pub windows_prefetch: bool,
    #[arg(long, help = "Windows error reporting data")]
    pub windows_wer: bool,
    #[arg(long, help = "Browser caches (Chrome, Firefox, Edge, Brave, Safari)")]
    pub browser_caches: bool,
}

impl CliOptions {
    pub fn from_env() -> Self {
        CliOptions::parse()
    }

    pub fn mode(&self) -> RunMode {
        match self.command {
            Some(Command::Scan) => RunMode::Scan,
            Some(Command::Clean) => RunMode::Clean,
            Some(Command::List) => RunMode::List,
            None => {
                if self.all || self.category_flags.has_any() || !self.categories.is_empty() {
                    RunMode::Clean
                } else {
                    RunMode::Interactive
                }
            }
        }
    }

    pub fn resolve_category_ids(
        &self,
        available: &[CleanupCategory],
    ) -> Result<HashSet<&'static str>> {
        let mut ids = self.category_flags.to_ids();
        let available_ids: HashSet<&'static str> = available.iter().map(|c| c.id).collect();

        for name in &self.categories {
            let normalized = name.replace('-', "_").to_lowercase();
            if let Some(found) = available_ids
                .iter()
                .find(|id| id.to_lowercase() == normalized)
            {
                ids.insert(*found);
            } else {
                bail!(
                    "Unknown category '{}'. Use `oscleaner list` to see available categories.",
                    name
                );
            }
        }

        if self.all {
            ids.extend(available_ids.into_iter());
        }

        Ok(ids)
    }
}

impl CategoryFlags {
    pub fn has_any(&self) -> bool {
        self.node_modules
            || self.docker
            || self.xcode
            || self.android_builds
            || self.react_native_ios
            || self.gradle_cache
            || self.maven_cache
            || self.cargo_targets
            || self.python_cache
            || self.cocoapods_cache
            || self.mac_caches
            || self.mac_logs
            || self.mac_tmp
            || self.ios_backups
            || self.homebrew_cache
            || self.mail_downloads
            || self.windows_temp
            || self.windows_update
            || self.windows_thumbnail
            || self.windows_prefetch
            || self.windows_wer
            || self.browser_caches
    }

    pub fn to_ids(&self) -> HashSet<&'static str> {
        let mut ids = HashSet::new();
        if self.node_modules {
            ids.insert("node_modules");
        }
        if self.docker {
            ids.insert("docker");
        }
        if self.xcode {
            ids.insert("xcode");
        }
        if self.android_builds {
            ids.insert("android_builds");
        }
        if self.react_native_ios {
            ids.insert("react_native_ios");
        }
        if self.gradle_cache {
            ids.insert("gradle_cache");
        }
        if self.maven_cache {
            ids.insert("maven_cache");
        }
        if self.cargo_targets {
            ids.insert("cargo_targets");
        }
        if self.python_cache {
            ids.insert("python_cache");
        }
        if self.cocoapods_cache {
            ids.insert("cocoapods_cache");
        }
        if self.mac_caches {
            ids.insert("mac_caches");
        }
        if self.mac_logs {
            ids.insert("mac_logs");
        }
        if self.mac_tmp {
            ids.insert("mac_tmp");
        }
        if self.ios_backups {
            ids.insert("ios_backups");
        }
        if self.homebrew_cache {
            ids.insert("homebrew_cache");
        }
        if self.mail_downloads {
            ids.insert("mail_downloads");
        }
        if self.windows_temp {
            ids.insert("windows_temp");
        }
        if self.windows_update {
            ids.insert("windows_update");
        }
        if self.windows_thumbnail {
            ids.insert("windows_thumbnail");
        }
        if self.windows_prefetch {
            ids.insert("windows_prefetch");
        }
        if self.windows_wer {
            ids.insert("windows_wer");
        }
        if self.browser_caches {
            ids.insert("browser_caches");
        }
        ids
    }
}

pub fn print_banner(ctx: &ScanContext) {
    let os_label = match ctx.os {
        OsKind::Windows => "Windows",
        OsKind::Mac => "macOS",
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
        };
        println!(
            "--{:<20} {:<8} {}",
            cat.id.replace('_', "-"),
            platform,
            cat.description
        );
    }
}
