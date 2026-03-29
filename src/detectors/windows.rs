use std::path::PathBuf;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

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
