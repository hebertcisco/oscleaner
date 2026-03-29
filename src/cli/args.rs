use std::collections::HashSet;

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};

use crate::categories::CleanupCategory;
use crate::safe;

#[derive(Parser, Debug, Default)]
#[command(name = "oscleaner", version, about = "Scan, preview, and clean development/system clutter")]
pub struct CliOptions {
    #[arg(
        short = 'n',
        long = "dry-run",
        global = true,
        help = "Preview deletions without removing files"
    )]
    pub dry_run: bool,
    #[arg(
        short = 'Y',
        long = "yes",
        global = true,
        help = "Skip interactive confirmations and proceed directly"
    )]
    pub yes: bool,
    #[arg(
        long = "safe",
        global = true,
        help = "Safe mode: only regenerable caches, age/size limits, auto-confirm (ideal for cron)"
    )]
    pub safe: bool,
    #[arg(
        long = "max-size",
        value_name = "GB",
        global = true,
        help = "Maximum total GB allowed to delete (default: 20 in safe mode)"
    )]
    pub max_size_gb: Option<u64>,
    #[arg(
        long = "min-age",
        value_name = "DAYS",
        global = true,
        help = "Only delete items older than N days (default: 2 in safe mode)"
    )]
    pub min_age_days: Option<u64>,
    #[command(flatten)]
    pub targets: TargetArgs,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Scan(TargetArgs),
    Clean(TargetArgs),
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Interactive,
    Scan,
    Clean,
    List,
}

#[derive(Args, Debug, Default, Clone, PartialEq, Eq)]
pub struct TargetArgs {
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
}

#[derive(Args, Debug, Default, Clone, PartialEq, Eq)]
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
    #[arg(long, help = "PHP vendor directories (Composer)")]
    pub php_vendor: bool,
    #[arg(long, help = "Ruby vendor directories (Bundler)")]
    pub ruby_vendor: bool,
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
    #[arg(long, help = "Linux user cache (~/.cache)")]
    pub linux_cache: bool,
    #[arg(long, help = "Linux system and user logs")]
    pub linux_logs: bool,
    #[arg(long, help = "Linux temporary files (/tmp, /var/tmp)")]
    pub linux_tmp: bool,
    #[arg(long, help = "Systemd journal logs")]
    pub linux_journal: bool,
    #[arg(long, help = "Linux core dumps")]
    pub linux_coredumps: bool,
    #[arg(long, help = "Linux XDG Trash")]
    pub linux_trash: bool,
    #[arg(long, help = "Snap package caches")]
    pub snap_cache: bool,
    #[arg(long, help = "Flatpak app caches")]
    pub flatpak_cache: bool,
}

impl CliOptions {
    pub fn from_env() -> Self {
        CliOptions::parse()
    }

    pub fn mode(&self) -> RunMode {
        match &self.command {
            Some(Command::Scan(_)) => RunMode::Scan,
            Some(Command::Clean(_)) => RunMode::Clean,
            Some(Command::List) => RunMode::List,
            None => {
                if self.safe
                    || self.targets.all
                    || self.targets.category_flags.has_any()
                    || !self.targets.categories.is_empty()
                {
                    RunMode::Clean
                } else {
                    RunMode::Interactive
                }
            }
        }
    }

    /// In safe mode, --yes is implicit.
    pub fn effective_yes(&self) -> bool {
        self.yes || self.safe
    }

    pub fn targets(&self) -> &TargetArgs {
        match &self.command {
            Some(Command::Scan(targets)) | Some(Command::Clean(targets)) => targets,
            Some(Command::List) | None => &self.targets,
        }
    }

    pub fn resolve_category_ids(
        &self,
        available: &[CleanupCategory],
    ) -> Result<HashSet<&'static str>> {
        let targets = self.targets();
        let mut ids = targets.category_flags.to_ids();
        let available_ids: HashSet<&'static str> = available.iter().map(|c| c.id).collect();

        for name in &targets.categories {
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

        if targets.all || self.safe {
            ids.extend(available_ids.into_iter());
        }

        // In safe mode, restrict to only safe categories
        if self.safe {
            let safe_ids: HashSet<&str> = safe::safe_category_ids().iter().copied().collect();
            ids.retain(|id| safe_ids.contains(id));
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
            || self.php_vendor
            || self.ruby_vendor
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
            || self.linux_cache
            || self.linux_logs
            || self.linux_tmp
            || self.linux_journal
            || self.linux_coredumps
            || self.linux_trash
            || self.snap_cache
            || self.flatpak_cache
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
        if self.php_vendor {
            ids.insert("php_vendor");
        }
        if self.ruby_vendor {
            ids.insert("ruby_vendor");
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
        if self.linux_cache {
            ids.insert("linux_cache");
        }
        if self.linux_logs {
            ids.insert("linux_logs");
        }
        if self.linux_tmp {
            ids.insert("linux_tmp");
        }
        if self.linux_journal {
            ids.insert("linux_journal");
        }
        if self.linux_coredumps {
            ids.insert("linux_coredumps");
        }
        if self.linux_trash {
            ids.insert("linux_trash");
        }
        if self.snap_cache {
            ids.insert("snap_cache");
        }
        if self.flatpak_cache {
            ids.insert("flatpak_cache");
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::{CliOptions, Command, RunMode};
    use clap::Parser;

    #[test]
    fn clean_subcommand_accepts_category_flags_after_subcommand() {
        let opts = CliOptions::parse_from([
            "oscleaner",
            "clean",
            "--xcode",
            "--android-builds",
            "--dry-run",
        ]);

        assert_eq!(opts.mode(), RunMode::Clean);
        assert!(opts.dry_run);
        assert!(matches!(opts.command, Some(Command::Clean(_))));
        assert!(opts.targets().category_flags.xcode);
        assert!(opts.targets().category_flags.android_builds);
    }

    #[test]
    fn top_level_category_flags_still_default_to_clean_mode() {
        let opts = CliOptions::parse_from(["oscleaner", "--node-modules", "--cargo-targets"]);

        assert_eq!(opts.mode(), RunMode::Clean);
        assert!(opts.command.is_none());
        assert!(opts.targets().category_flags.node_modules);
        assert!(opts.targets().category_flags.cargo_targets);
    }
}
