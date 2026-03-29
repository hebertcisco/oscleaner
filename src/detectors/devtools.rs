use std::path::PathBuf;

use crate::context::ScanContext;
use crate::fs_utils::{search_for_dir, walk_roots};
use crate::types::OsKind;

pub fn detect_node_modules(ctx: &ScanContext) -> Vec<PathBuf> {
    search_for_dir(&ctx.search_roots, "node_modules", 5)
}

pub fn detect_docker_data(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut paths = vec![ctx.home.join(".docker")];

    match ctx.os {
        OsKind::Mac => {
            paths.push(ctx.home.join("Library/Containers/com.docker.docker/Data"));
            paths.push(PathBuf::from("/var/lib/docker"));
        }
        OsKind::Windows => {
            if let Some(program_data) = &ctx.program_data {
                paths.push(program_data.join("Docker"));
                paths.push(program_data.join("DockerDesktop"));
            }
            if let Some(local) = &ctx.local_app_data {
                paths.push(local.join("Docker"));
            }
        }
        OsKind::Linux | OsKind::FreeBSD => {
            paths.push(PathBuf::from("/var/lib/docker"));
        }
        OsKind::Other => {}
    }

    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn detect_android_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 6)
        .into_iter()
        .filter(|e| e.file_type().is_dir() && e.file_name() == "build")
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|name| {
                    let lower = name.to_lowercase();
                    lower.contains("android") || lower == "app"
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_react_native_ios(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 6)
        .into_iter()
        .filter(|e| {
            e.file_type().is_dir() && (e.file_name() == "Pods" || e.file_name() == "build")
        })
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.file_name())
                .map(|f| f == "ios")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_gradle_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    let path = ctx.home.join(".gradle/caches");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_maven_cache(ctx: &ScanContext) -> Vec<PathBuf> {
    let path = ctx.home.join(".m2/repository");
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

pub fn detect_cargo_targets(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 4)
        .into_iter()
        .filter(|e| e.file_type().is_dir() && e.file_name() == "target")
        .filter(|e| {
            let p = e.path();
            p.join(".rustc_info.json").exists() || p.join("debug/.cargo-lock").exists()
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_php_vendor(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 5)
        .into_iter()
        .filter(|e| e.file_type().is_dir() && e.file_name() == "vendor")
        .filter(|e| e.path().join("autoload.php").exists())
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_ruby_vendor(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 5)
        .into_iter()
        .filter(|e| e.file_type().is_dir() && e.file_name() == "vendor")
        .filter(|e| {
            let p = e.path();
            p.join("bundle").is_dir()
                || p.parent()
                    .map(|parent| parent.join("Gemfile").exists())
                    .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_python_artifacts(ctx: &ScanContext) -> Vec<PathBuf> {
    let venv_names: &[&str] = &[".venv", "venv", "env", "envs", "virtualenv", "virtualenvs"];

    walk_roots(&ctx.search_roots, 5)
        .into_iter()
        .filter(|e| {
            let name = e.file_name();
            if e.file_type().is_dir() {
                name == "__pycache__" || venv_names.iter().any(|v| name == *v)
            } else {
                e.path().extension().and_then(|ext| ext.to_str()) == Some("pyc")
            }
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}
