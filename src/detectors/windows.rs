use std::collections::HashSet;
use std::path::PathBuf;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

#[derive(Clone, Copy)]
enum WindowsRoot {
    Home,
    LocalAppData,
    RoamingAppData,
    ProgramFiles,
    ProgramFilesX86,
}

#[derive(Clone, Copy)]
struct WindowsPathSpec {
    root: WindowsRoot,
    relative: &'static str,
}

struct WindowsLeftoverApp {
    install_markers: &'static [WindowsPathSpec],
    leftovers: &'static [WindowsPathSpec],
}

const fn win_path(root: WindowsRoot, relative: &'static str) -> WindowsPathSpec {
    WindowsPathSpec { root, relative }
}

const CURSOR_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Cursor/Cursor.exe"),
    win_path(WindowsRoot::ProgramFiles, "Cursor/Cursor.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Cursor/Cursor.exe"),
];
const CURSOR_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::Home, ".cursor"),
    win_path(WindowsRoot::LocalAppData, "Cursor"),
    win_path(WindowsRoot::RoamingAppData, "Cursor"),
    win_path(WindowsRoot::LocalAppData, "Programs/Cursor"),
];

const WINDSURF_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Windsurf/Windsurf.exe"),
    win_path(WindowsRoot::ProgramFiles, "Windsurf/Windsurf.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Windsurf/Windsurf.exe"),
];
const WINDSURF_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::Home, ".codeium/windsurf"),
    win_path(WindowsRoot::LocalAppData, "Windsurf"),
    win_path(WindowsRoot::RoamingAppData, "Windsurf"),
    win_path(WindowsRoot::LocalAppData, "Programs/Windsurf"),
];

const ZED_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Zed/Zed.exe"),
    win_path(WindowsRoot::ProgramFiles, "Zed/Zed.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Zed/Zed.exe"),
];
const ZED_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "Zed"),
    win_path(WindowsRoot::LocalAppData, "Zed"),
    win_path(WindowsRoot::LocalAppData, "Programs/Zed"),
];

const WARP_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Warp/Warp.exe"),
    win_path(WindowsRoot::ProgramFiles, "Warp/Warp.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Warp/Warp.exe"),
];
const WARP_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::Home, ".warp"),
    win_path(WindowsRoot::RoamingAppData, "Warp"),
    win_path(WindowsRoot::LocalAppData, "Warp"),
    win_path(WindowsRoot::LocalAppData, "Programs/Warp"),
];

const POSTMAN_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Postman/Postman.exe"),
    win_path(WindowsRoot::LocalAppData, "Programs/Postman/Postman.exe"),
    win_path(WindowsRoot::ProgramFiles, "Postman/Postman.exe"),
];
const POSTMAN_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "Postman"),
    win_path(WindowsRoot::LocalAppData, "Postman"),
    win_path(WindowsRoot::LocalAppData, "Programs/Postman"),
];

const INSOMNIA_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Insomnia/Insomnia.exe"),
    win_path(WindowsRoot::ProgramFiles, "Insomnia/Insomnia.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Insomnia/Insomnia.exe"),
];
const INSOMNIA_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "Insomnia"),
    win_path(WindowsRoot::LocalAppData, "Insomnia"),
    win_path(WindowsRoot::LocalAppData, "Programs/Insomnia"),
];

const GITHUB_DESKTOP_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "GitHubDesktop/GitHubDesktop.exe"),
    win_path(
        WindowsRoot::LocalAppData,
        "Programs/GitHub Desktop/GitHubDesktop.exe",
    ),
];
const GITHUB_DESKTOP_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "GitHub Desktop"),
    win_path(WindowsRoot::LocalAppData, "GitHubDesktop"),
    win_path(WindowsRoot::LocalAppData, "Programs/GitHub Desktop"),
];

const CLAUDE_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::LocalAppData, "Programs/Claude/Claude.exe"),
    win_path(WindowsRoot::ProgramFiles, "Claude/Claude.exe"),
    win_path(WindowsRoot::ProgramFilesX86, "Claude/Claude.exe"),
];
const CLAUDE_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "Claude"),
    win_path(WindowsRoot::LocalAppData, "Claude"),
    win_path(WindowsRoot::LocalAppData, "Programs/Claude"),
];

const GEMINI_INSTALL_MARKERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::RoamingAppData, "npm/gemini.cmd"),
    win_path(WindowsRoot::RoamingAppData, "npm/gemini.ps1"),
    win_path(WindowsRoot::RoamingAppData, "npm/gemini"),
];
const GEMINI_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::Home, ".gemini"),
    win_path(WindowsRoot::RoamingAppData, "gemini"),
    win_path(WindowsRoot::LocalAppData, "gemini"),
];

const JUNIE_LEFTOVERS: &[WindowsPathSpec] = &[
    win_path(WindowsRoot::Home, ".junie"),
    win_path(WindowsRoot::RoamingAppData, "Junie"),
    win_path(WindowsRoot::LocalAppData, "Junie"),
];

const WINDOWS_DEV_LEFTOVER_APPS: &[WindowsLeftoverApp] = &[
    WindowsLeftoverApp {
        install_markers: CURSOR_INSTALL_MARKERS,
        leftovers: CURSOR_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: WINDSURF_INSTALL_MARKERS,
        leftovers: WINDSURF_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: ZED_INSTALL_MARKERS,
        leftovers: ZED_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: WARP_INSTALL_MARKERS,
        leftovers: WARP_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: POSTMAN_INSTALL_MARKERS,
        leftovers: POSTMAN_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: INSOMNIA_INSTALL_MARKERS,
        leftovers: INSOMNIA_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: GITHUB_DESKTOP_INSTALL_MARKERS,
        leftovers: GITHUB_DESKTOP_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: CLAUDE_INSTALL_MARKERS,
        leftovers: CLAUDE_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: GEMINI_INSTALL_MARKERS,
        leftovers: GEMINI_LEFTOVERS,
    },
    WindowsLeftoverApp {
        install_markers: &[],
        leftovers: JUNIE_LEFTOVERS,
    },
];

fn resolve_windows_path(ctx: &ScanContext, spec: WindowsPathSpec) -> Option<PathBuf> {
    let root = match spec.root {
        WindowsRoot::Home => Some(ctx.home.clone()),
        WindowsRoot::LocalAppData => ctx.local_app_data.clone(),
        WindowsRoot::RoamingAppData => ctx.roaming_app_data.clone(),
        WindowsRoot::ProgramFiles => ctx.program_files.clone(),
        WindowsRoot::ProgramFilesX86 => ctx.program_files_x86.clone(),
    }?;

    Some(root.join(spec.relative))
}

fn install_markers_exist(ctx: &ScanContext, markers: &[WindowsPathSpec]) -> bool {
    markers
        .iter()
        .filter_map(|spec| resolve_windows_path(ctx, *spec))
        .any(|path| path.exists())
}

fn existing_paths(ctx: &ScanContext, specs: &[WindowsPathSpec]) -> Vec<PathBuf> {
    specs
        .iter()
        .filter_map(|spec| resolve_windows_path(ctx, *spec))
        .filter(|path| path.exists())
        .collect()
}

pub fn detect_windows_temp(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    if ctx.temp.exists() {
        list_children(&ctx.temp)
    } else {
        Vec::new()
    }
}

pub fn detect_windows_update_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    let dir = PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download");
    if dir.exists() {
        list_children(&dir)
    } else {
        Vec::new()
    }
}

pub fn detect_windows_thumbnail_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    ctx.local_app_data
        .as_ref()
        .map(|p| p.join("Microsoft/Windows/Explorer"))
        .filter(|p| p.exists())
        .into_iter()
        .collect()
}

pub fn detect_windows_prefetch(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    let path = PathBuf::from("C:\\Windows\\Prefetch");
    if path.exists() {
        list_children(&path)
    } else {
        Vec::new()
    }
}

pub fn detect_windows_wer(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    ctx.program_data
        .as_ref()
        .map(|p| p.join("Microsoft/Windows/WER"))
        .filter(|p| p.exists())
        .into_iter()
        .collect()
}

pub fn detect_windows_dev_tool_leftovers(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut leftovers = Vec::new();

    for app in WINDOWS_DEV_LEFTOVER_APPS {
        if install_markers_exist(ctx, app.install_markers) {
            continue;
        }

        for path in existing_paths(ctx, app.leftovers) {
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
        let local = root.join("AppData/Local");
        let roaming = root.join("AppData/Roaming");
        let program_data = root.join("ProgramData");
        let program_files = root.join("Program Files");
        let program_files_x86 = root.join("Program Files (x86)");

        for dir in [
            &home,
            &local,
            &roaming,
            &program_data,
            &program_files,
            &program_files_x86,
        ] {
            fs::create_dir_all(dir).unwrap();
        }

        ScanContext {
            os: OsKind::Windows,
            home: home.clone(),
            temp: root.join("Temp"),
            search_roots: vec![home.clone()],
            local_app_data: Some(local),
            roaming_app_data: Some(roaming),
            program_data: Some(program_data),
            program_files: Some(program_files),
            program_files_x86: Some(program_files_x86),
            xdg_cache_home: None,
            xdg_config_home: None,
            xdg_data_home: None,
            system_drive: Some(PathBuf::from("C:\\")),
            selected_drive: None,
        }
    }

    #[test]
    fn detects_cursor_leftovers_when_cursor_binary_is_missing() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("state.json"), b"{}").unwrap();

        let found = detect_windows_dev_tool_leftovers(&ctx);
        assert_eq!(found, vec![leftover]);
    }

    #[test]
    fn skips_cursor_leftovers_when_cursor_is_still_installed() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".cursor");
        let install_marker = ctx
            .local_app_data
            .as_ref()
            .unwrap()
            .join("Programs/Cursor/Cursor.exe");

        fs::create_dir_all(&leftover).unwrap();
        fs::create_dir_all(install_marker.parent().unwrap()).unwrap();
        fs::write(&install_marker, b"exe").unwrap();

        let found = detect_windows_dev_tool_leftovers(&ctx);
        assert!(!found.contains(&leftover));
    }

    #[test]
    fn detects_gemini_dotdir_when_no_global_shim_exists() {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        let leftover = ctx.home.join(".gemini");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("config.json"), b"{}").unwrap();

        let found = detect_windows_dev_tool_leftovers(&ctx);
        assert!(found.contains(&leftover));
    }
}
