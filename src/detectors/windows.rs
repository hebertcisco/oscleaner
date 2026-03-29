use std::path::PathBuf;

use crate::context::ScanContext;
use crate::types::OsKind;

pub fn detect_windows_temp(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    vec![ctx.temp.clone()]
}

pub fn detect_windows_update_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    vec![PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download")]
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}

pub fn detect_windows_thumbnail_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    ctx.local_app_data
        .as_ref()
        .map(|p| p.join("Microsoft/Windows/Explorer"))
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}

pub fn detect_windows_prefetch(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Windows {
        return Vec::new();
    }
    let path = PathBuf::from("C:\\Windows\\Prefetch");
    if path.exists() {
        vec![path]
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
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}
