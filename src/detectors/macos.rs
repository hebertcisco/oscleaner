use std::collections::HashSet;
use std::path::PathBuf;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

#[derive(Clone, Copy)]
enum MacRoot {
    Home,
    Applications,
    UserApplications,
    LibraryApplicationSupport,
    LibraryCaches,
    LibraryPreferences,
    LibrarySavedState,
}

#[derive(Clone, Copy)]
struct MacPathSpec {
    root: MacRoot,
    relative: &'static str,
}

struct MacLeftoverApp {
    install_markers: &'static [MacPathSpec],
    leftovers: &'static [MacPathSpec],
}

const fn mac_path(root: MacRoot, relative: &'static str) -> MacPathSpec {
    MacPathSpec { root, relative }
}

const CURSOR_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Cursor.app"),
    mac_path(MacRoot::UserApplications, "Cursor.app"),
];
const CURSOR_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".cursor"),
    mac_path(MacRoot::LibraryApplicationSupport, "Cursor"),
    mac_path(MacRoot::LibraryCaches, "Cursor"),
    mac_path(MacRoot::LibraryPreferences, "com.todesktop.230313mzl4w4u92.plist"),
    mac_path(
        MacRoot::LibrarySavedState,
        "com.todesktop.230313mzl4w4u92.savedState",
    ),
];

const WINDSURF_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Windsurf.app"),
    mac_path(MacRoot::UserApplications, "Windsurf.app"),
];
const WINDSURF_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".codeium/windsurf"),
    mac_path(MacRoot::LibraryApplicationSupport, "Windsurf"),
    mac_path(MacRoot::LibraryCaches, "Windsurf"),
    mac_path(MacRoot::LibraryPreferences, "com.exafunction.windsurf.plist"),
];

const ZED_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Zed.app"),
    mac_path(MacRoot::UserApplications, "Zed.app"),
];
const ZED_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::LibraryApplicationSupport, "Zed"),
    mac_path(MacRoot::LibraryCaches, "Zed"),
    mac_path(MacRoot::LibraryPreferences, "dev.zed.Zed.plist"),
];

const WARP_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Warp.app"),
    mac_path(MacRoot::UserApplications, "Warp.app"),
];
const WARP_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".warp"),
    mac_path(MacRoot::LibraryApplicationSupport, "dev.warp.Warp-Stable"),
    mac_path(MacRoot::LibraryCaches, "dev.warp.Warp-Stable"),
    mac_path(
        MacRoot::LibraryPreferences,
        "dev.warp.Warp-Stable.plist",
    ),
];

const POSTMAN_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Postman.app"),
    mac_path(MacRoot::UserApplications, "Postman.app"),
];
const POSTMAN_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::LibraryApplicationSupport, "Postman"),
    mac_path(MacRoot::LibraryCaches, "Postman"),
    mac_path(MacRoot::LibraryPreferences, "com.postmanlabs.mac.plist"),
];

const INSOMNIA_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Insomnia.app"),
    mac_path(MacRoot::UserApplications, "Insomnia.app"),
];
const INSOMNIA_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::LibraryApplicationSupport, "Insomnia"),
    mac_path(MacRoot::LibraryCaches, "Insomnia"),
    mac_path(MacRoot::LibraryPreferences, "rest.insomnia.app.plist"),
];

const CLAUDE_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "Claude.app"),
    mac_path(MacRoot::UserApplications, "Claude.app"),
];
const CLAUDE_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::LibraryApplicationSupport, "Claude"),
    mac_path(MacRoot::LibraryCaches, "Claude"),
    mac_path(MacRoot::LibraryPreferences, "com.anthropic.claudefordesktop.plist"),
];

const GITHUB_DESKTOP_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Applications, "GitHub Desktop.app"),
    mac_path(MacRoot::UserApplications, "GitHub Desktop.app"),
];
const GITHUB_DESKTOP_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::LibraryApplicationSupport, "GitHub Desktop"),
    mac_path(MacRoot::LibraryCaches, "GitHub Desktop"),
    mac_path(MacRoot::LibraryPreferences, "com.github.GitHubClient.plist"),
];

const GEMINI_MAC_INSTALL_MARKERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".npm-global/bin/gemini"),
    mac_path(MacRoot::Home, ".bun/bin/gemini"),
    mac_path(MacRoot::Home, ".local/bin/gemini"),
];
const GEMINI_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".gemini"),
    mac_path(MacRoot::LibraryApplicationSupport, "gemini"),
    mac_path(MacRoot::LibraryCaches, "gemini"),
];

const JUNIE_MAC_LEFTOVERS: &[MacPathSpec] = &[
    mac_path(MacRoot::Home, ".junie"),
    mac_path(MacRoot::LibraryApplicationSupport, "Junie"),
    mac_path(MacRoot::LibraryCaches, "Junie"),
];

const MAC_DEV_LEFTOVER_APPS: &[MacLeftoverApp] = &[
    MacLeftoverApp {
        install_markers: CURSOR_MAC_INSTALL_MARKERS,
        leftovers: CURSOR_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: WINDSURF_MAC_INSTALL_MARKERS,
        leftovers: WINDSURF_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: ZED_MAC_INSTALL_MARKERS,
        leftovers: ZED_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: WARP_MAC_INSTALL_MARKERS,
        leftovers: WARP_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: POSTMAN_MAC_INSTALL_MARKERS,
        leftovers: POSTMAN_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: INSOMNIA_MAC_INSTALL_MARKERS,
        leftovers: INSOMNIA_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: CLAUDE_MAC_INSTALL_MARKERS,
        leftovers: CLAUDE_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: GITHUB_DESKTOP_MAC_INSTALL_MARKERS,
        leftovers: GITHUB_DESKTOP_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: GEMINI_MAC_INSTALL_MARKERS,
        leftovers: GEMINI_MAC_LEFTOVERS,
    },
    MacLeftoverApp {
        install_markers: &[],
        leftovers: JUNIE_MAC_LEFTOVERS,
    },
];

fn resolve_mac_path(ctx: &ScanContext, spec: MacPathSpec) -> PathBuf {
    match spec.root {
        MacRoot::Home => ctx.home.join(spec.relative),
        MacRoot::Applications => PathBuf::from("/Applications").join(spec.relative),
        MacRoot::UserApplications => ctx.home.join("Applications").join(spec.relative),
        MacRoot::LibraryApplicationSupport => ctx
            .home
            .join("Library/Application Support")
            .join(spec.relative),
        MacRoot::LibraryCaches => ctx.home.join("Library/Caches").join(spec.relative),
        MacRoot::LibraryPreferences => ctx.home.join("Library/Preferences").join(spec.relative),
        MacRoot::LibrarySavedState => ctx
            .home
            .join("Library/Saved Application State")
            .join(spec.relative),
    }
}

fn mac_install_markers_exist(ctx: &ScanContext, markers: &[MacPathSpec]) -> bool {
    markers
        .iter()
        .map(|spec| resolve_mac_path(ctx, *spec))
        .any(|path| path.exists())
}

fn existing_mac_paths(ctx: &ScanContext, specs: &[MacPathSpec]) -> Vec<PathBuf> {
    specs
        .iter()
        .map(|spec| resolve_mac_path(ctx, *spec))
        .filter(|path| path.exists())
        .collect()
}

pub fn detect_xcode_data(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    vec![
        ctx.home.join("Library/Developer/Xcode/DerivedData"),
        ctx.home.join("Library/Developer/Xcode/Archives"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

pub fn detect_cocoapods_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let path = ctx.home.join("Library/Caches/CocoaPods");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_mac_user_caches(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let mut paths = Vec::new();
    let caches = ctx.home.join("Library/Caches");
    if caches.exists() {
        paths.extend(list_children(&caches));
    }
    if ctx.temp.exists() {
        paths.extend(list_children(&ctx.temp));
    }
    paths
}

pub fn detect_mac_logs(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let mut paths = Vec::new();
    for dir in &[ctx.home.join("Library/Logs"), PathBuf::from("/Library/Logs")] {
        if dir.exists() {
            paths.extend(list_children(dir));
        }
    }
    paths
}

pub fn detect_mac_temp(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let mut paths = Vec::new();
    let tmp = PathBuf::from("/tmp");
    if tmp.exists() {
        paths.extend(list_children(&tmp));
    }
    let crash = ctx.home.join("Library/Application Support/CrashReporter");
    if crash.exists() {
        paths.push(crash);
    }
    paths
}

pub fn detect_ios_backups(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let path = ctx
        .home
        .join("Library/Application Support/MobileSync/Backup");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_homebrew_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let path = ctx.home.join("Library/Caches/Homebrew");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_mail_downloads(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let path = ctx
        .home
        .join("Library/Containers/com.apple.mail/Data/Library/Mail Downloads");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_mac_dev_tool_leftovers(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut leftovers = Vec::new();

    for app in MAC_DEV_LEFTOVER_APPS {
        if mac_install_markers_exist(ctx, app.install_markers) {
            continue;
        }

        for path in existing_mac_paths(ctx, app.leftovers) {
            if seen.insert(path.clone()) {
                leftovers.push(path);
            }
        }
    }

    leftovers
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn test_ctx(root: &std::path::Path) -> ScanContext {
        let home = root.join("home");
        fs::create_dir_all(home.join("Library/Application Support")).unwrap();
        fs::create_dir_all(home.join("Library/Caches")).unwrap();
        fs::create_dir_all(home.join("Library/Preferences")).unwrap();
        fs::create_dir_all(home.join("Library/Saved Application State")).unwrap();
        fs::create_dir_all(home.join("Applications")).unwrap();

        ScanContext {
            os: OsKind::Mac,
            home: home.clone(),
            temp: root.join("tmp"),
            search_roots: vec![home.clone()],
            local_app_data: None,
            roaming_app_data: None,
            program_data: None,
            program_files: None,
            program_files_x86: None,
            xdg_cache_home: Some(home.join(".cache")),
            xdg_config_home: Some(home.join(".config")),
            xdg_data_home: Some(home.join(".local/share")),
        }
    }

    #[test]
    fn detects_cursor_leftovers_on_mac_when_app_bundle_is_missing() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("state.json"), b"{}").unwrap();

        let found = detect_mac_dev_tool_leftovers(&ctx);
        assert!(found.contains(&leftover));
    }

    #[test]
    fn skips_cursor_leftovers_on_mac_when_user_app_exists() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        let app = ctx.home.join("Applications/Cursor.app");

        fs::create_dir_all(&leftover).unwrap();
        fs::create_dir_all(&app).unwrap();

        let found = detect_mac_dev_tool_leftovers(&ctx);
        assert!(!found.contains(&leftover));
    }

    #[test]
    fn detects_junie_leftovers_on_mac_without_install_markers() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".junie");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("config.json"), b"{}").unwrap();

        let found = detect_mac_dev_tool_leftovers(&ctx);
        assert!(found.contains(&leftover));
    }
}
