use std::collections::HashSet;

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};

use crate::categories::CleanupCategory;
use crate::safe;

macro_rules! category_flags {
    ($(($field:ident, $id:literal, $help:literal)),+ $(,)?) => {
        #[derive(Args, Debug, Default, Clone, PartialEq, Eq)]
        pub struct CategoryFlags {
            $(
                #[arg(long, help = $help)]
                pub $field: bool,
            )+
        }

        impl CategoryFlags {
            pub fn has_any(&self) -> bool {
                $(self.$field)||+
            }

            pub fn to_ids(&self) -> HashSet<&'static str> {
                let mut ids = HashSet::new();
                $(
                    if self.$field {
                        ids.insert($id);
                    }
                )+
                ids
            }
        }
    };
}

category_flags!(
    (node_modules,      "node_modules",      "Node.js node_modules folders"),
    (docker,            "docker",            "Docker caches, images, containers, and build data"),
    (xcode,             "xcode",             "Xcode DerivedData and archives (macOS)"),
    (android_builds,    "android_builds",    "Android build folders"),
    (react_native_ios,  "react_native_ios",  "React Native iOS Pods/builds"),
    (gradle_cache,      "gradle_cache",      "Gradle cache"),
    (maven_cache,       "maven_cache",       "Maven repository cache"),
    (cargo_targets,     "cargo_targets",     "Cargo target directories"),
    (php_vendor,        "php_vendor",        "PHP vendor directories (Composer)"),
    (ruby_vendor,       "ruby_vendor",       "Ruby vendor directories (Bundler)"),
    (python_cache,      "python_cache",      "Python __pycache__, pyc files, and virtualenvs"),
    (cocoapods_cache,   "cocoapods_cache",   "CocoaPods cache (macOS)"),
    (mac_caches,        "mac_caches",        "macOS user caches"),
    (mac_logs,          "mac_logs",          "macOS system and user logs"),
    (mac_tmp,           "mac_tmp",           "macOS temporary files"),
    (ios_backups,       "ios_backups",       "iOS device backups (macOS)"),
    (homebrew_cache,    "homebrew_cache",    "Homebrew cache (macOS)"),
    (mail_downloads,    "mail_downloads",    "Mail downloads cache (macOS)"),
    (windows_temp,      "windows_temp",      "Windows temp folder"),
    (windows_update,    "windows_update",    "Windows Update cache"),
    (windows_thumbnail, "windows_thumbnail", "Windows thumbnail cache"),
    (windows_prefetch,  "windows_prefetch",  "Windows prefetch files"),
    (windows_wer,       "windows_wer",       "Windows error reporting data"),
    (browser_caches,    "browser_caches",    "Browser caches (Chrome, Firefox, Edge, Brave, Safari)"),
    (linux_cache,       "linux_cache",       "Linux user cache (~/.cache)"),
    (linux_logs,        "linux_logs",        "Linux system and user logs"),
    (linux_tmp,         "linux_tmp",         "Linux temporary files (/tmp, /var/tmp)"),
    (linux_journal,     "linux_journal",     "Systemd journal logs"),
    (linux_coredumps,   "linux_coredumps",   "Linux core dumps"),
    (linux_trash,       "linux_trash",       "Linux XDG Trash"),
    (snap_cache,        "snap_cache",        "Snap package caches"),
    (flatpak_cache,     "flatpak_cache",     "Flatpak app caches"),
);

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
            Some(Command::Scan(targets) | Command::Clean(targets)) => targets,
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
            ids.extend(available_ids);
        }

        if self.safe {
            let safe_ids: HashSet<&str> = safe::safe_category_ids().iter().copied().collect();
            ids.retain(|id| safe_ids.contains(id));
        }

        Ok(ids)
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
