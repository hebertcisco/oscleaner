use std::path::PathBuf;

use crate::context::ScanContext;
use crate::types::OsKind;

pub fn detect_browser_caches(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    match ctx.os {
        OsKind::Windows => {
            if let Some(local) = &ctx.local_app_data {
                paths.push(local.join("Google/Chrome/User Data/Default/Cache"));
                paths.push(local.join("Google/Chrome/User Data/Profile 1/Cache"));
                paths.push(local.join("Microsoft/Edge/User Data/Default/Cache"));
                paths.push(local.join("BraveSoftware/Brave-Browser/User Data/Default/Cache"));
                paths.push(local.join("Mozilla/Firefox/Profiles"));
            }
            if let Some(roaming) = &ctx.roaming_app_data {
                paths.push(roaming.join("Mozilla/Firefox/Profiles"));
            }
        }
        OsKind::Mac => {
            paths.push(ctx.home.join("Library/Caches/Google/Chrome"));
            paths.push(
                ctx.home
                    .join("Library/Application Support/Google/Chrome/Default/Cache"),
            );
            paths.push(ctx.home.join("Library/Caches/Firefox/Profiles"));
            paths.push(
                ctx.home
                    .join("Library/Application Support/Firefox/Profiles"),
            );
            paths.push(ctx.home.join("Library/Caches/com.apple.Safari"));
            paths.push(
                ctx.home
                    .join("Library/Application Support/Microsoft Edge/Default/Cache"),
            );
            paths.push(
                ctx.home
                    .join("Library/Application Support/BraveSoftware/Brave-Browser/Default/Cache"),
            );
        }
        OsKind::Linux | OsKind::FreeBSD => {
            paths.push(ctx.home.join(".config/google-chrome/Default/Cache"));
            paths.push(ctx.home.join(".config/google-chrome/Profile 1/Cache"));
            paths.push(ctx.home.join(".config/chromium/Default/Cache"));
            paths.push(ctx.home.join(".mozilla/firefox"));
            paths.push(
                ctx.home
                    .join(".config/BraveSoftware/Brave-Browser/Default/Cache"),
            );
            paths.push(ctx.home.join(".config/microsoft-edge/Default/Cache"));
        }
        OsKind::Other => {}
    }
    paths.into_iter().filter(|p| p.exists()).collect()
}
