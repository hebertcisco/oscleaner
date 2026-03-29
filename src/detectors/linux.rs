use std::path::PathBuf;

use walkdir::WalkDir;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

fn is_linux(ctx: &ScanContext) -> bool {
    ctx.os == OsKind::Linux
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
