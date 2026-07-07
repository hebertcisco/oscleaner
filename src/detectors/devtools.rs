use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::context::ScanContext;
use crate::fs_utils::{search_for_dir, walk_roots};
use crate::types::OsKind;

pub fn detect_node_modules(ctx: &ScanContext) -> Vec<PathBuf> {
    let caches_dir = ctx.home.join("Library/Caches");
    search_for_dir(&ctx.search_roots, "node_modules", 5)
        .into_iter()
        .filter(|p| !p.starts_with(&caches_dir))
        .collect()
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
    unique_existing_paths(
        walk_roots(&ctx.search_roots, 6)
            .into_iter()
            .filter(|e| {
                e.file_type().is_dir()
                    && matches!(
                        e.file_name().to_str(),
                        Some("build" | ".cxx" | ".externalNativeBuild")
                    )
            })
            .filter(|e| {
                let path = e.path();
                let parent = match path.parent() {
                    Some(parent) => parent,
                    None => return false,
                };

                is_android_module_dir(parent)
                    || ancestor_matches(parent, 3, is_android_project_dir)
                    || parent
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| {
                            let lower = name.to_lowercase();
                            lower.contains("android") || lower == "app"
                        })
                        .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf()),
    )
}

pub fn detect_react_native_ios(ctx: &ScanContext) -> Vec<PathBuf> {
    unique_existing_paths(
        walk_roots(&ctx.search_roots, 6)
            .into_iter()
            .filter(|e| {
                e.file_type().is_dir() && (e.file_name() == "Pods" || e.file_name() == "build")
            })
            .filter(|e| {
                let Some(ios_dir) = e.path().parent() else {
                    return false;
                };
                if ios_dir.file_name().map(|f| f == "ios").unwrap_or(false) {
                    return ios_dir
                        .parent()
                        .map(is_react_native_project_root)
                        .unwrap_or(false);
                }
                false
            })
            .map(|e| e.path().to_path_buf()),
    )
}

pub fn detect_ios_project_artifacts(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, is_ios_or_swift_project_root);
    unique_existing_paths(
        project_roots
            .into_iter()
            .flat_map(|root| existing_children(&root, &["Pods", "build", ".build"])),
    )
}

pub fn detect_web_project_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, |dir| {
        package_json(dir).is_some()
            && (is_expo_project_root(dir)
                || is_next_project_root(dir)
                || is_angular_project_root(dir)
                || is_nest_project_root(dir))
    });

    unique_existing_paths(project_roots.into_iter().flat_map(|root| {
        let mut paths = Vec::new();

        if is_expo_project_root(&root) {
            paths.extend(existing_children(
                &root,
                &[".expo", "web-build", "dist", "build"],
            ));
        }
        if is_next_project_root(&root) {
            paths.extend(existing_children(&root, &[".next", "out", "dist"]));
        }
        if is_angular_project_root(&root) {
            paths.extend(existing_children(&root, &["dist", ".angular/cache"]));
        }
        if is_nest_project_root(&root) {
            paths.extend(existing_children(&root, &["dist", "build"]));
        }

        paths
    }))
}

pub fn detect_jvm_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, |dir| {
        is_jvm_project_root(dir) && !is_android_project_dir(dir)
    });

    unique_existing_paths(
        project_roots
            .into_iter()
            .flat_map(|root| existing_children(&root, &["target", "build", "out"])),
    )
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
    let project_roots = project_dirs(ctx, 4, |dir| dir.join("Cargo.toml").is_file());
    unique_existing_paths(
        project_roots
            .into_iter()
            .map(|root| root.join("target"))
            .filter(|target| target.exists()),
    )
}

pub fn detect_rails_artifacts(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, is_rails_project_root);
    unique_existing_paths(project_roots.into_iter().flat_map(|root| {
        existing_children(
            &root,
            &[
                "tmp/cache",
                "public/assets",
                "public/packs",
                "public/packs-test",
            ],
        )
    }))
}

pub fn detect_go_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, |dir| dir.join("go.mod").is_file());
    unique_existing_paths(
        project_roots
            .into_iter()
            .flat_map(|root| existing_children(&root, &["build", "dist"])),
    )
}

pub fn detect_cpp_builds(ctx: &ScanContext) -> Vec<PathBuf> {
    let project_roots = project_dirs(ctx, 5, is_cpp_project_root);
    unique_existing_paths(project_roots.into_iter().flat_map(|root| {
        let mut paths = existing_children(
            &root,
            &[
                "build",
                "out",
                "cmake-build-debug",
                "cmake-build-release",
                "cmake-build-relwithdebinfo",
                "cmake-build-minsizerel",
            ],
        );

        if let Ok(entries) = fs::read_dir(&root) {
            paths.extend(entries.filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                (path.is_dir() && name.starts_with("cmake-build-")).then_some(path)
            }));
        }

        paths
    }))
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

pub fn detect_java_heap_dumps(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 6)
        .into_iter()
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("hprof"))
                    .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn detect_apk_artifacts(ctx: &ScanContext) -> Vec<PathBuf> {
    walk_roots(&ctx.search_roots, 6)
        .into_iter()
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("apk"))
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

fn project_dirs(
    ctx: &ScanContext,
    max_depth: usize,
    predicate: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    unique_existing_paths(
        walk_roots(&ctx.search_roots, max_depth)
            .into_iter()
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.path().to_path_buf())
            .filter(|path| predicate(path)),
    )
}

fn existing_children(root: &Path, relative_paths: &[&str]) -> Vec<PathBuf> {
    relative_paths
        .iter()
        .map(|relative| root.join(relative))
        .filter(|path| path.exists())
        .collect()
}

fn unique_existing_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| path.exists())
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

fn ancestor_matches(path: &Path, max_levels: usize, predicate: fn(&Path) -> bool) -> bool {
    path.ancestors().take(max_levels + 1).any(predicate)
}

fn has_file(dir: &Path, names: &[&str]) -> bool {
    names.iter().any(|name| dir.join(name).is_file())
}

fn has_child_extension(dir: &Path, extension: &str) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.path().is_dir()
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == extension)
                    .unwrap_or(false)
        })
}

fn package_json(dir: &Path) -> Option<Value> {
    let contents = fs::read_to_string(dir.join("package.json")).ok()?;
    serde_json::from_str(&contents).ok()
}

fn package_json_has_dep(dir: &Path, package_names: &[&str]) -> bool {
    let Some(json) = package_json(dir) else {
        return false;
    };

    [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .iter()
    .filter_map(|section| json.get(section).and_then(Value::as_object))
    .any(|deps| package_names.iter().any(|name| deps.contains_key(*name)))
}

fn package_json_script_contains(dir: &Path, needles: &[&str]) -> bool {
    let Some(json) = package_json(dir) else {
        return false;
    };

    json.get("scripts")
        .and_then(Value::as_object)
        .map(|scripts| {
            scripts.values().filter_map(Value::as_str).any(|script| {
                let script = script.to_ascii_lowercase();
                needles.iter().any(|needle| script.contains(needle))
            })
        })
        .unwrap_or(false)
}

fn file_contains_any(path: &Path, needles: &[&str]) -> bool {
    fs::read_to_string(path)
        .map(|contents| {
            let contents = contents.to_ascii_lowercase();
            needles.iter().any(|needle| contents.contains(needle))
        })
        .unwrap_or(false)
}

fn is_react_native_project_root(dir: &Path) -> bool {
    package_json_has_dep(dir, &["react-native", "expo"])
        || package_json_script_contains(dir, &["react-native", "expo"])
}

fn is_expo_project_root(dir: &Path) -> bool {
    package_json_has_dep(dir, &["expo"]) || package_json_script_contains(dir, &["expo"])
}

fn is_next_project_root(dir: &Path) -> bool {
    package_json_has_dep(dir, &["next"]) || package_json_script_contains(dir, &["next"])
}

fn is_angular_project_root(dir: &Path) -> bool {
    dir.join("angular.json").is_file()
        || package_json_has_dep(dir, &["@angular/core", "@angular/cli"])
        || package_json_script_contains(dir, &["ng build"])
}

fn is_nest_project_root(dir: &Path) -> bool {
    dir.join("nest-cli.json").is_file()
        || package_json_has_dep(dir, &["@nestjs/core", "@nestjs/cli"])
        || package_json_script_contains(dir, &["nest build"])
}

fn is_ios_or_swift_project_root(dir: &Path) -> bool {
    has_child_extension(dir, "xcodeproj")
        || has_child_extension(dir, "xcworkspace")
        || dir.join("Podfile").is_file()
        || dir.join("Package.swift").is_file()
}

fn is_android_project_dir(dir: &Path) -> bool {
    has_file(
        dir,
        &[
            "settings.gradle",
            "settings.gradle.kts",
            "gradlew",
            "gradlew.bat",
        ],
    ) && (dir.join("local.properties").is_file()
        || dir.join("gradle.properties").is_file()
        || dir.join("app/src/main/AndroidManifest.xml").is_file()
        || gradle_files_contain(dir, &["com.android."]))
}

fn is_android_module_dir(dir: &Path) -> bool {
    dir.join("src/main/AndroidManifest.xml").is_file()
        || gradle_files_contain(dir, &["com.android."])
        || (dir.join("CMakeLists.txt").is_file()
            && ancestor_matches(dir, 3, is_android_project_dir))
}

fn gradle_files_contain(dir: &Path, needles: &[&str]) -> bool {
    [
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
    ]
    .iter()
    .any(|name| file_contains_any(&dir.join(name), needles))
}

fn is_jvm_project_root(dir: &Path) -> bool {
    dir.join("pom.xml").is_file()
        || dir.join("build.gradle").is_file()
        || dir.join("build.gradle.kts").is_file()
        || dir.join("settings.gradle").is_file()
        || dir.join("settings.gradle.kts").is_file()
}

fn is_rails_project_root(dir: &Path) -> bool {
    dir.join("Gemfile").is_file()
        && (dir.join("config/application.rb").is_file() || dir.join("config.ru").is_file())
}

fn is_cpp_project_root(dir: &Path) -> bool {
    has_file(
        dir,
        &[
            "CMakeLists.txt",
            "Makefile",
            "meson.build",
            "conanfile.txt",
            "conanfile.py",
            "vcpkg.json",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn test_context(root: PathBuf) -> ScanContext {
        ScanContext {
            os: OsKind::Mac,
            home: root.clone(),
            temp: root.join("tmp"),
            search_roots: vec![root],
            local_app_data: None,
            roaming_app_data: None,
            program_data: None,
            program_files: None,
            program_files_x86: None,
            xdg_cache_home: None,
            xdg_config_home: None,
            xdg_data_home: None,
            system_drive: None,
            selected_drive: None,
        }
    }

    #[test]
    fn detects_react_native_ios_pods_and_builds_only_inside_rn_projects() {
        let tmp = tempdir().unwrap();
        let rn = tmp.path().join("rn-app");
        fs::create_dir_all(rn.join("ios/Pods")).unwrap();
        fs::create_dir_all(rn.join("ios/build")).unwrap();
        fs::write(
            rn.join("package.json"),
            r#"{"dependencies":{"react-native":"0.80.0"}}"#,
        )
        .unwrap();

        let native = tmp.path().join("native-app");
        fs::create_dir_all(native.join("ios/Pods")).unwrap();

        let paths = detect_react_native_ios(&test_context(tmp.path().to_path_buf()));
        assert!(paths.contains(&rn.join("ios/Pods")));
        assert!(paths.contains(&rn.join("ios/build")));
        assert!(!paths.contains(&native.join("ios/Pods")));
    }

    #[test]
    fn detects_native_ios_and_swift_project_artifacts() {
        let tmp = tempdir().unwrap();
        let app = tmp.path().join("ios-native");
        fs::create_dir_all(app.join("App.xcodeproj")).unwrap();
        fs::create_dir_all(app.join("Pods")).unwrap();
        fs::create_dir_all(app.join("build")).unwrap();

        let swift = tmp.path().join("swift-package");
        fs::create_dir_all(swift.join(".build")).unwrap();
        fs::write(swift.join("Package.swift"), "// swift-tools-version: 6.0").unwrap();

        let paths = detect_ios_project_artifacts(&test_context(tmp.path().to_path_buf()));
        assert!(paths.contains(&app.join("Pods")));
        assert!(paths.contains(&app.join("build")));
        assert!(paths.contains(&swift.join(".build")));
    }

    #[test]
    fn detects_web_framework_build_outputs() {
        let tmp = tempdir().unwrap();
        let app = tmp.path().join("web-app");
        fs::create_dir_all(app.join(".next")).unwrap();
        fs::create_dir_all(app.join("dist")).unwrap();
        fs::write(
            app.join("package.json"),
            r#"{"dependencies":{"next":"latest","@nestjs/core":"latest"}}"#,
        )
        .unwrap();

        let paths = detect_web_project_builds(&test_context(tmp.path().to_path_buf()));
        assert!(paths.contains(&app.join(".next")));
        assert!(paths.contains(&app.join("dist")));
    }

    #[test]
    fn detects_android_native_and_ndk_outputs() {
        let tmp = tempdir().unwrap();
        let app = tmp.path().join("android-app");
        fs::create_dir_all(app.join("app/build")).unwrap();
        fs::create_dir_all(app.join("app/.cxx")).unwrap();
        fs::write(app.join("settings.gradle"), "pluginManagement {}").unwrap();
        fs::write(app.join("local.properties"), "sdk.dir=/tmp/android-sdk").unwrap();
        fs::write(
            app.join("app/build.gradle"),
            "plugins { id 'com.android.application' }",
        )
        .unwrap();

        let paths = detect_android_builds(&test_context(tmp.path().to_path_buf()));
        assert!(paths.contains(&app.join("app/build")));
        assert!(paths.contains(&app.join("app/.cxx")));
    }

    #[test]
    fn detects_jvm_go_rails_and_cpp_build_outputs() {
        let tmp = tempdir().unwrap();

        let java = tmp.path().join("spring-app");
        fs::create_dir_all(java.join("target")).unwrap();
        fs::write(java.join("pom.xml"), "<project />").unwrap();

        let go = tmp.path().join("go-app");
        fs::create_dir_all(go.join("dist")).unwrap();
        fs::write(go.join("go.mod"), "module example.com/app").unwrap();

        let rails = tmp.path().join("rails-app");
        fs::create_dir_all(rails.join("config")).unwrap();
        fs::create_dir_all(rails.join("tmp/cache")).unwrap();
        fs::write(rails.join("Gemfile"), "gem 'rails'").unwrap();
        fs::write(rails.join("config/application.rb"), "module App; end").unwrap();

        let cpp = tmp.path().join("cpp-app");
        fs::create_dir_all(cpp.join("cmake-build-debug")).unwrap();
        fs::write(
            cpp.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.22)",
        )
        .unwrap();

        let rust = tmp.path().join("rust-app");
        fs::create_dir_all(rust.join("target")).unwrap();
        fs::write(rust.join("Cargo.toml"), "[package]\nname = \"rust-app\"").unwrap();

        let ctx = test_context(tmp.path().to_path_buf());
        assert!(detect_jvm_builds(&ctx).contains(&java.join("target")));
        assert!(detect_go_builds(&ctx).contains(&go.join("dist")));
        assert!(detect_rails_artifacts(&ctx).contains(&rails.join("tmp/cache")));
        assert!(detect_cpp_builds(&ctx).contains(&cpp.join("cmake-build-debug")));
        assert!(detect_cargo_targets(&ctx).contains(&rust.join("target")));
    }
}
