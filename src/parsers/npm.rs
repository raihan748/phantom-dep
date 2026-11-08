use crate::model::{Dependency, DependencyType, Ecosystem};
use std::path::Path;

/// Parses an npm `package.json` file content into a vector of Dependencies.
pub fn parse_package_json(content: &str, file_path: &str) -> Result<Vec<Dependency>, String> {
    let root: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("Invalid package.json JSON: {}", e))?;

    let mut deps = Vec::new();

    let sections = [
        ("dependencies", DependencyType::Standard),
        ("devDependencies", DependencyType::Dev),
        ("peerDependencies", DependencyType::Peer),
        ("optionalDependencies", DependencyType::Optional),
    ];

    for (section_name, dep_type) in sections {
        if let Some(map) = root.get(section_name).and_then(|v| v.as_object()) {
            for (pkg_name, version_val) in map {
                let version_spec = match version_val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };

                deps.push(Dependency {
                    name: pkg_name.trim().to_string(),
                    version_spec: version_spec.trim().to_string(),
                    ecosystem: Ecosystem::Npm,
                    dep_type,
                    manifest_file: file_path.to_string(),
                    line_number: None,
                });
            }
        }
    }

    Ok(deps)
}

/// Helper to check if a file path is an npm package manifest
pub fn is_npm_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| name.eq_ignore_ascii_case("package.json"))
        .unwrap_or(false)
}
