pub mod cargo;
pub mod go;
pub mod npm;
pub mod python;

use crate::model::{Dependency, Ecosystem};
use std::fs;
use std::path::{Path, PathBuf};

/// Scans a file or directory for package manifests and extracts dependencies
pub fn discover_and_parse(
    target_path: &Path,
    ecosystem_filter: Option<Ecosystem>,
) -> Result<Vec<Dependency>, String> {
    if !target_path.exists() {
        return Err(format!("Target path does not exist: {}", target_path.display()));
    }

    let mut dependencies = Vec::new();

    if target_path.is_file() {
        let deps = parse_file(target_path, ecosystem_filter)?;
        dependencies.extend(deps);
    } else if target_path.is_dir() {
        let manifest_files = find_manifests(target_path, ecosystem_filter);
        for file in manifest_files {
            if let Ok(deps) = parse_file(&file, ecosystem_filter) {
                dependencies.extend(deps);
            }
        }
    }

    Ok(dependencies)
}

/// Parses a single manifest file
pub fn parse_file(
    file_path: &Path,
    ecosystem_filter: Option<Ecosystem>,
) -> Result<Vec<Dependency>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let path_str = file_path.to_string_lossy();

    if npm::is_npm_manifest(file_path) {
        if ecosystem_filter.map_or(true, |e| e == Ecosystem::Npm) {
            return npm::parse_package_json(&content, &path_str);
        }
    } else if cargo::is_cargo_manifest(file_path) {
        if ecosystem_filter.map_or(true, |e| e == Ecosystem::Cargo) {
            return cargo::parse_cargo_toml(&content, &path_str);
        }
    } else if go::is_go_manifest(file_path) {
        if ecosystem_filter.map_or(true, |e| e == Ecosystem::Go) {
            return go::parse_go_mod(&content, &path_str);
        }
    } else if python::is_python_manifest(file_path) {
        if ecosystem_filter.map_or(true, |e| e == Ecosystem::PyPi) {
            let file_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            if file_name == "pyproject.toml" {
                return python::parse_pyproject_toml(&content, &path_str);
            } else {
                return Ok(python::parse_requirements_txt(&content, &path_str));
            }
        }
    }

    Ok(Vec::new())
}

/// Recursively discovers all supported manifests in a directory
pub fn find_manifests(dir: &Path, ecosystem_filter: Option<Ecosystem>) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    walk_directory(dir, &mut manifests, ecosystem_filter, 0);
    manifests
}

fn walk_directory(
    dir: &Path,
    results: &mut Vec<PathBuf>,
    ecosystem_filter: Option<Ecosystem>,
    depth: usize,
) {
    if depth > 10 {
        return; // Prevent excessive recursion
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();

            // Skip VCS and build artifacts
            if dir_name.starts_with('.')
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "vendor"
                || dir_name == "__pycache__"
                || dir_name == ".venv"
                || dir_name == "venv"
            {
                continue;
            }

            walk_directory(&path, results, ecosystem_filter, depth + 1);
        } else if path.is_file() {
            let is_match = match ecosystem_filter {
                Some(Ecosystem::Npm) => npm::is_npm_manifest(&path),
                Some(Ecosystem::Cargo) => cargo::is_cargo_manifest(&path),
                Some(Ecosystem::Go) => go::is_go_manifest(&path),
                Some(Ecosystem::PyPi) => python::is_python_manifest(&path),
                None => {
                    npm::is_npm_manifest(&path)
                        || cargo::is_cargo_manifest(&path)
                        || go::is_go_manifest(&path)
                        || python::is_python_manifest(&path)
                }
            };

            if is_match {
                results.push(path);
            }
        }
    }
}
