use std::collections::HashSet;
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

fn is_linux(ctx: &ScanContext) -> bool {
    ctx.os == OsKind::Linux
}

#[derive(Clone, Copy)]
enum LinuxRoot {
    Home,
    XdgConfig,
    XdgCache,
    XdgData,
    LocalBin,
    UserShareApplications,
    Opt,
    UsrBin,
    UsrLocalBin,
    UsrShareApplications,
}

#[derive(Clone, Copy)]
struct LinuxPathSpec {
    root: LinuxRoot,
    relative: &'static str,
}

struct LinuxLeftoverApp {
    install_markers: &'static [LinuxPathSpec],
    leftovers: &'static [LinuxPathSpec],
}

const fn linux_path(root: LinuxRoot, relative: &'static str) -> LinuxPathSpec {
    LinuxPathSpec { root, relative }
}

const CURSOR_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Cursor/cursor"),
    linux_path(LinuxRoot::UsrBin, "cursor"),
    linux_path(LinuxRoot::UsrLocalBin, "cursor"),
    linux_path(LinuxRoot::UserShareApplications, "cursor.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "cursor.desktop"),
];
const CURSOR_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Home, ".cursor"),
    linux_path(LinuxRoot::XdgConfig, "Cursor"),
    linux_path(LinuxRoot::XdgCache, "Cursor"),
    linux_path(LinuxRoot::XdgData, "Cursor"),
];

const WINDSURF_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Windsurf/windsurf"),
    linux_path(LinuxRoot::UsrBin, "windsurf"),
    linux_path(LinuxRoot::UsrLocalBin, "windsurf"),
    linux_path(LinuxRoot::UserShareApplications, "windsurf.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "windsurf.desktop"),
];
const WINDSURF_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Home, ".codeium/windsurf"),
    linux_path(LinuxRoot::XdgConfig, "Windsurf"),
    linux_path(LinuxRoot::XdgCache, "Windsurf"),
    linux_path(LinuxRoot::XdgData, "Windsurf"),
];

const ZED_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::UsrBin, "zed"),
    linux_path(LinuxRoot::UsrLocalBin, "zed"),
    linux_path(LinuxRoot::LocalBin, "zed"),
    linux_path(LinuxRoot::UserShareApplications, "dev.zed.Zed.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "dev.zed.Zed.desktop"),
];
const ZED_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::XdgConfig, "zed"),
    linux_path(LinuxRoot::XdgCache, "zed"),
    linux_path(LinuxRoot::XdgData, "zed"),
];

const WARP_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Warp/warp"),
    linux_path(LinuxRoot::UsrBin, "warp-terminal"),
    linux_path(LinuxRoot::UsrLocalBin, "warp-terminal"),
    linux_path(LinuxRoot::UserShareApplications, "warp-terminal.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "warp-terminal.desktop"),
];
const WARP_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Home, ".warp"),
    linux_path(LinuxRoot::XdgConfig, "Warp"),
    linux_path(LinuxRoot::XdgCache, "Warp"),
    linux_path(LinuxRoot::XdgData, "warp-terminal"),
];

const POSTMAN_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Postman/Postman"),
    linux_path(LinuxRoot::UsrBin, "postman"),
    linux_path(LinuxRoot::UsrLocalBin, "postman"),
    linux_path(LinuxRoot::UserShareApplications, "postman.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "postman.desktop"),
];
const POSTMAN_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::XdgConfig, "Postman"),
    linux_path(LinuxRoot::XdgCache, "Postman"),
    linux_path(LinuxRoot::XdgData, "Postman"),
];

const INSOMNIA_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Insomnia/insomnia"),
    linux_path(LinuxRoot::UsrBin, "insomnia"),
    linux_path(LinuxRoot::UsrLocalBin, "insomnia"),
    linux_path(LinuxRoot::UserShareApplications, "insomnia.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "insomnia.desktop"),
];
const INSOMNIA_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::XdgConfig, "Insomnia"),
    linux_path(LinuxRoot::XdgCache, "Insomnia"),
    linux_path(LinuxRoot::XdgData, "Insomnia"),
];

const CLAUDE_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Opt, "Claude/claude"),
    linux_path(LinuxRoot::UsrBin, "claude"),
    linux_path(LinuxRoot::UsrLocalBin, "claude"),
    linux_path(LinuxRoot::UserShareApplications, "claude.desktop"),
    linux_path(LinuxRoot::UsrShareApplications, "claude.desktop"),
];
const CLAUDE_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::XdgConfig, "Claude"),
    linux_path(LinuxRoot::XdgCache, "Claude"),
    linux_path(LinuxRoot::XdgData, "Claude"),
];

const GEMINI_LINUX_INSTALL_MARKERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::LocalBin, "gemini"),
    linux_path(LinuxRoot::UsrLocalBin, "gemini"),
    linux_path(LinuxRoot::UsrBin, "gemini"),
    linux_path(LinuxRoot::Home, ".bun/bin/gemini"),
    linux_path(LinuxRoot::Home, ".npm-global/bin/gemini"),
];
const GEMINI_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Home, ".gemini"),
    linux_path(LinuxRoot::XdgConfig, "gemini"),
    linux_path(LinuxRoot::XdgCache, "gemini"),
    linux_path(LinuxRoot::XdgData, "gemini"),
];

const JUNIE_LINUX_LEFTOVERS: &[LinuxPathSpec] = &[
    linux_path(LinuxRoot::Home, ".junie"),
    linux_path(LinuxRoot::XdgConfig, "Junie"),
    linux_path(LinuxRoot::XdgCache, "Junie"),
    linux_path(LinuxRoot::XdgData, "Junie"),
];

const LINUX_DEV_LEFTOVER_APPS: &[LinuxLeftoverApp] = &[
    LinuxLeftoverApp {
        install_markers: CURSOR_LINUX_INSTALL_MARKERS,
        leftovers: CURSOR_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: WINDSURF_LINUX_INSTALL_MARKERS,
        leftovers: WINDSURF_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: ZED_LINUX_INSTALL_MARKERS,
        leftovers: ZED_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: WARP_LINUX_INSTALL_MARKERS,
        leftovers: WARP_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: POSTMAN_LINUX_INSTALL_MARKERS,
        leftovers: POSTMAN_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: INSOMNIA_LINUX_INSTALL_MARKERS,
        leftovers: INSOMNIA_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: CLAUDE_LINUX_INSTALL_MARKERS,
        leftovers: CLAUDE_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: GEMINI_LINUX_INSTALL_MARKERS,
        leftovers: GEMINI_LINUX_LEFTOVERS,
    },
    LinuxLeftoverApp {
        install_markers: &[],
        leftovers: JUNIE_LINUX_LEFTOVERS,
    },
];

fn resolve_linux_path(ctx: &ScanContext, spec: LinuxPathSpec) -> Option<PathBuf> {
    let path = match spec.root {
        LinuxRoot::Home => ctx.home.join(spec.relative),
        LinuxRoot::XdgConfig => ctx.xdg_config_home.as_ref()?.join(spec.relative),
        LinuxRoot::XdgCache => ctx.xdg_cache_home.as_ref()?.join(spec.relative),
        LinuxRoot::XdgData => ctx.xdg_data_home.as_ref()?.join(spec.relative),
        LinuxRoot::LocalBin => ctx.home.join(".local/bin").join(spec.relative),
        LinuxRoot::UserShareApplications => ctx
            .home
            .join(".local/share/applications")
            .join(spec.relative),
        LinuxRoot::Opt => PathBuf::from("/opt").join(spec.relative),
        LinuxRoot::UsrBin => PathBuf::from("/usr/bin").join(spec.relative),
        LinuxRoot::UsrLocalBin => PathBuf::from("/usr/local/bin").join(spec.relative),
        LinuxRoot::UsrShareApplications => {
            PathBuf::from("/usr/share/applications").join(spec.relative)
        }
    };

    Some(path)
}

fn linux_install_markers_exist(ctx: &ScanContext, markers: &[LinuxPathSpec]) -> bool {
    markers
        .iter()
        .filter_map(|spec| resolve_linux_path(ctx, *spec))
        .any(|path| path.exists())
}

fn existing_linux_paths(ctx: &ScanContext, specs: &[LinuxPathSpec]) -> Vec<PathBuf> {
    specs
        .iter()
        .filter_map(|spec| resolve_linux_path(ctx, *spec))
        .filter(|path| path.exists())
        .collect()
}

pub fn detect_linux_user_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    ctx.xdg_cache_home
        .as_ref()
        .filter(|p| p.exists())
        .map(|p| list_children(p))
        .unwrap_or_default()
}

pub fn detect_linux_logs(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let mut paths = Vec::new();
    let var_log = PathBuf::from("/var/log");
    if var_log.exists() {
        paths.extend(list_children(&var_log));
    }
    let user_syslog = ctx.home.join(".local/share/syslog");
    if user_syslog.exists() {
        paths.push(user_syslog);
    }
    paths
}

pub fn detect_linux_tmp(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let mut paths = Vec::new();
    for dir in &[PathBuf::from("/tmp"), PathBuf::from("/var/tmp")] {
        if dir.exists() {
            paths.extend(list_children(dir));
        }
    }
    paths
}

pub fn detect_linux_journal(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let paths = vec![
        PathBuf::from("/var/log/journal"),
        ctx.home.join(".local/share/systemd/journal"),
    ];
    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn detect_linux_coredumps(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let paths = vec![
        PathBuf::from("/var/lib/systemd/coredump"),
        ctx.home.join("coredumps"),
    ];
    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn detect_linux_trash(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    ctx.xdg_data_home
        .as_ref()
        .map(|p| p.join("Trash"))
        .filter(|p| p.exists())
        .into_iter()
        .collect()
}

pub fn detect_snap_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let mut paths = Vec::new();

    let snap_dir = ctx.home.join("snap");
    if snap_dir.exists() {
        for entry in WalkDir::new(&snap_dir)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if entry.file_name() == ".cache" {
                paths.push(entry.path().to_path_buf());
            }
        }
    }

    let system_snap = PathBuf::from("/var/lib/snapd/cache");
    if system_snap.exists() {
        paths.push(system_snap);
    }

    paths
}

pub fn detect_flatpak_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }
    let mut paths = Vec::new();

    let flatpak_apps = ctx.home.join(".var/app");
    if flatpak_apps.exists() {
        for entry in WalkDir::new(&flatpak_apps)
            .min_depth(1)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if entry.file_name() == "cache" {
                paths.push(entry.path().to_path_buf());
            }
        }
    }

    let var_tmp = PathBuf::from("/var/tmp");
    if var_tmp.exists() {
        for entry in WalkDir::new(&var_tmp)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("flatpak-cache-")
            {
                paths.push(entry.path().to_path_buf());
            }
        }
    }

    paths
}

pub fn detect_linux_dev_tool_leftovers(ctx: &ScanContext) -> Vec<PathBuf> {
    if !is_linux(ctx) {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut leftovers = Vec::new();

    for app in LINUX_DEV_LEFTOVER_APPS {
        if linux_install_markers_exist(ctx, app.install_markers) {
            continue;
        }

        for path in existing_linux_paths(ctx, app.leftovers) {
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
        let cache = home.join(".cache");
        let config = home.join(".config");
        let data = home.join(".local/share");

        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(data.join("applications")).unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();

        ScanContext {
            os: OsKind::Linux,
            home: home.clone(),
            temp: root.join("tmp"),
            search_roots: vec![home.clone()],
            local_app_data: None,
            roaming_app_data: None,
            program_data: None,
            program_files: None,
            program_files_x86: None,
            xdg_cache_home: Some(cache),
            xdg_config_home: Some(config),
            xdg_data_home: Some(data),
            system_drive: None,
            selected_drive: None,
        }
    }

    #[test]
    fn detects_cursor_leftovers_on_linux_when_binary_is_missing() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("state.json"), b"{}").unwrap();

        let found = detect_linux_dev_tool_leftovers(&ctx);
        assert!(found.contains(&leftover));
    }

    #[test]
    fn skips_cursor_leftovers_on_linux_when_desktop_entry_exists() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        let marker = ctx.home.join(".local/share/applications/cursor.desktop");

        fs::create_dir_all(&leftover).unwrap();
        fs::write(&marker, "[Desktop Entry]").unwrap();

        let found = detect_linux_dev_tool_leftovers(&ctx);
        assert!(!found.contains(&leftover));
    }

    #[test]
    fn detects_junie_leftovers_on_linux_without_install_markers() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".junie");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("config.json"), b"{}").unwrap();

        let found = detect_linux_dev_tool_leftovers(&ctx);
        assert!(found.contains(&leftover));
    }
}
