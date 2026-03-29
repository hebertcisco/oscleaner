use std::path::PathBuf;

use walkdir::WalkDir;

use crate::context::ScanContext;
use crate::fs_utils::search_for_dir;
use crate::types::OsKind;

pub fn detect_node_modules(ctx: &ScanContext) -> Vec<PathBuf> {
    search_for_dir(&ctx.search_roots, "node_modules", 5)
}

pub fn detect_docker_data(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(ctx.home.join(".docker"));

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
        OsKind::Other => {}
    }

    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn detect_android_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if entry.file_name() == "build" {
                if let Some(parent) = entry.path().parent() {
                    let name = parent
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if name.contains("android") || name == "app" {
                        hits.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    hits
}

pub fn detect_react_native_ios(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let name = entry.file_name();
            if name == "Pods" {
                if let Some(parent) = entry.path().parent() {
                    if parent.file_name().map(|f| f == "ios").unwrap_or(false) {
                        hits.push(entry.path().to_path_buf());
                    }
                }
            }
            if name == "build" {
                if let Some(parent) = entry.path().parent() {
                    if parent.file_name().map(|f| f == "ios").unwrap_or(false) {
                        hits.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    hits
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
    let mut hits = Vec::new();
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            if entry.file_name() == "target" {
                let p = entry.path();
                let has_rustc_info = p.join(".rustc_info.json").exists();
                let has_cargo_lock = p.join("debug/.cargo-lock").exists();
                if has_rustc_info || has_cargo_lock {
                    hits.push(p.to_path_buf());
                }
            }
        }
    }
    hits
}

pub fn detect_php_vendor(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let name = entry.file_name();
            if name == "vendor" {
                let p = entry.path();
                if p.join("autoload.php").exists() {
                    hits.push(p.to_path_buf());
                }
            }
        }
    }
    hits
}

pub fn detect_ruby_vendor(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let name = entry.file_name();
            if name == "vendor" {
                let p = entry.path();
                // Ruby vendor: has bundle/ subdir or parent has Gemfile
                let has_bundle = p.join("bundle").is_dir();
                let parent_has_gemfile = p
                    .parent()
                    .map(|parent| parent.join("Gemfile").exists())
                    .unwrap_or(false);
                if has_bundle || parent_has_gemfile {
                    hits.push(p.to_path_buf());
                }
            }
        }
    }
    hits
}

pub fn detect_python_artifacts(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    let venv_names = [".venv", "venv", "env", "envs", "virtualenv", "virtualenvs"];
    for root in &ctx.search_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let name = entry.file_name();
            if entry.file_type().is_dir() {
                if name == "__pycache__" || venv_names.iter().any(|v| name == *v) {
                    hits.push(entry.path().to_path_buf());
                }
            } else if entry.path().extension().and_then(|ext| ext.to_str()) == Some("pyc") {
                hits.push(entry.path().to_path_buf());
            }
        }
    }
    hits
}
