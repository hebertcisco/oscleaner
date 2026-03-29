use std::path::PathBuf;

use crate::context::ScanContext;
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
    let mut paths = vec![ctx.home.join("Library/Caches"), ctx.temp.clone()];
    paths.retain(|p| p.exists());
    paths
}

pub fn detect_mac_logs(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let paths = vec![
        ctx.home.join("Library/Logs"),
        PathBuf::from("/Library/Logs"),
    ];
    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn detect_mac_temp(ctx: &ScanContext) -> Vec<PathBuf> {
    if ctx.os != OsKind::Mac {
        return Vec::new();
    }
    let paths = vec![
        PathBuf::from("/tmp"),
        ctx.home.join("Library/Application Support/CrashReporter"),
    ];
    paths.into_iter().filter(|p| p.exists()).collect()
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
