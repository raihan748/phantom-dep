use crate::model::{Dependency, DependencyType, Ecosystem};
use std::path::Path;

/// Parses a Rust `Cargo.toml` file content into a vector of Dependencies.
pub fn parse_cargo_toml(content: &str, file_path: &str) -> Result<Vec<Dependency>, String> {
    let toml_val: toml::Value =
        toml::from_str(content).map_err(|e| format!("Invalid Cargo.toml: {}", e))?;

    let root_table = match toml_val.as_table() {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut deps = Vec::new();

    let standard_sections = [
        ("dependencies", DependencyType::Standard),
        ("dev-dependencies", DependencyType::Dev),
        ("build-dependencies", DependencyType::Build),
    ];

    for (section, dep_type) in standard_sections {
        if let Some(table) = root_table.get(section).and_then(|v| v.as_table()) {
            extract_table_dependencies(table, dep_type, file_path, &mut deps);
        }
    }

    // Target specific sections: [target.'cfg(...)'.dependencies]
    if let Some(target_table) = root_table.get("target").and_then(|v| v.as_table()) {
        for (_target_key, target_val) in target_table {
            if let Some(target_cfg) = target_val.as_table() {
                for (section, dep_type) in standard_sections {
                    if let Some(table) = target_cfg.get(section).and_then(|v| v.as_table()) {
                        extract_table_dependencies(table, dep_type, file_path, &mut deps);
                    }
                }
            }
        }
    }

    // Workspace dependencies: [workspace.dependencies]
    if let Some(workspace_table) = root_table.get("workspace").and_then(|v| v.as_table()) {
        if let Some(table) = workspace_table.get("dependencies").and_then(|v| v.as_table()) {
            extract_table_dependencies(table, DependencyType::Standard, file_path, &mut deps);
        }
    }

    Ok(deps)
}

fn extract_table_dependencies(
    table: &toml::map::Map<String, toml::Value>,
    dep_type: DependencyType,
    file_path: &str,
    out: &mut Vec<Dependency>,
) {
    for (alias_name, val) in table {
        let (actual_name, version_spec) = match val {
            toml::Value::String(s) => (alias_name.clone(), s.clone()),
            toml::Value::Table(t) => {
                let pkg_name = t
                    .get("package")
                    .and_then(|p| p.as_str())
                    .unwrap_or(alias_name.as_str())
                    .to_string();

                let ver = if let Some(v) = t.get("version").and_then(|v| v.as_str()) {
                    v.to_string()
                } else if let Some(p) = t.get("path").and_then(|p| p.as_str()) {
                    format!("path:{}", p)
                } else if let Some(g) = t.get("git").and_then(|g| g.as_str()) {
                    format!("git:{}", g)
                } else {
                    "*".to_string()
                };
                (pkg_name, ver)
            }
            other => (alias_name.clone(), other.to_string()),
        };

        out.push(Dependency {
            name: actual_name,
            version_spec,
            ecosystem: Ecosystem::Cargo,
            dep_type,
            manifest_file: file_path.to_string(),
            line_number: None,
        });
    }
}

/// Helper to check if a file path is a Cargo manifest
pub fn is_cargo_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| name.eq_ignore_ascii_case("Cargo.toml"))
        .unwrap_or(false)
}
