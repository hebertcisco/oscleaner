use std::path::PathBuf;

use crate::context::ScanContext;
use crate::fs_utils::list_children;
use crate::types::OsKind;

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
