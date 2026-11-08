use crate::model::{Dependency, DependencyType, Ecosystem};
use std::path::Path;

/// Parses a Python `requirements.txt` file content into a vector of Dependencies.
pub fn parse_requirements_txt(content: &str, file_path: &str) -> Vec<Dependency> {
    let mut deps = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let mut line = raw_line.trim();

        // Strip inline comments starting with '#'
        if let Some(pos) = line.find('#') {
            line = line[..pos].trim();
        }

        if line.is_empty() {
            continue;
        }

        // Ignore pip directives / flags like -r, -f, -i, --extra-index-url
        if line.starts_with('-') {
            continue;
        }

        // Strip environment markers starting with ';'
        if let Some(pos) = line.find(';') {
            line = line[..pos].trim();
        }

        if line.is_empty() {
            continue;
        }

        // Parse package name and version specifier
        // Comparison operators in PEP 508: ==, ===, ~=, !=, <=, >=, <, >
        let op_prefixes = ["===", "==", "~=", "!=", "<=", ">=", "<", ">", "@"];
        let mut split_pos = None;
        let mut matched_op = "";

        for op in op_prefixes {
            if let Some(pos) = line.find(op) {
                if split_pos.map_or(true, |p| pos < p) {
                    split_pos = Some(pos);
                    matched_op = op;
                }
            }
        }

        let (mut pkg_name, version_spec) = if let Some(pos) = split_pos {
            let name_part = line[..pos].trim();
            let ver_part = line[pos..].trim();
            (name_part, ver_part)
        } else {
            (line.trim(), "*")
        };

        // If package name has extras like `pkg[security,tests]`, extract base name
        if let Some(extra_pos) = pkg_name.find('[') {
            pkg_name = pkg_name[..extra_pos].trim();
        }

        if !pkg_name.is_empty() {
            deps.push(Dependency {
                name: pkg_name.to_string(),
                version_spec: if version_spec == "*" && !matched_op.is_empty() {
                    matched_op.to_string()
                } else {
                    version_spec.to_string()
                },
                ecosystem: Ecosystem::PyPi,
                dep_type: DependencyType::Standard,
                manifest_file: file_path.to_string(),
                line_number: Some(line_num),
            });
        }
    }

    deps
}

/// Parses a Python `pyproject.toml` file content into a vector of Dependencies.
pub fn parse_pyproject_toml(content: &str, file_path: &str) -> Result<Vec<Dependency>, String> {
    let toml_val: toml::Value =
        toml::from_str(content).map_err(|e| format!("Invalid pyproject.toml: {}", e))?;

    let mut deps = Vec::new();

    // 1. Standard PEP 621: [project.dependencies] -> array of strings
    if let Some(project) = toml_val.get("project").and_then(|v| v.as_table()) {
        if let Some(arr) = project.get("dependencies").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(dep_str) = item.as_str() {
                    let parsed = parse_requirements_txt(dep_str, file_path);
                    for mut d in parsed {
                        d.dep_type = DependencyType::Standard;
                        deps.push(d);
                    }
                }
            }
        }

        // [project.optional-dependencies] -> table of arrays
        if let Some(opt) = project.get("optional-dependencies").and_then(|v| v.as_table()) {
            for (_group, arr_val) in opt {
                if let Some(arr) = arr_val.as_array() {
                    for item in arr {
                        if let Some(dep_str) = item.as_str() {
                            let parsed = parse_requirements_txt(dep_str, file_path);
                            for mut d in parsed {
                                d.dep_type = DependencyType::Optional;
                                deps.push(d);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Poetry: [tool.poetry.dependencies]
    if let Some(tool) = toml_val.get("tool").and_then(|v| v.as_table()) {
        if let Some(poetry) = tool.get("poetry").and_then(|v| v.as_table()) {
            if let Some(poetry_deps) = poetry.get("dependencies").and_then(|v| v.as_table()) {
                for (pkg_name, val) in poetry_deps {
                    // Poetry specifies python version here as well, skip "python"
                    if pkg_name.eq_ignore_ascii_case("python") {
                        continue;
                    }
                    let ver = match val {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Table(tbl) => tbl
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string(),
                        other => other.to_string(),
                    };
                    deps.push(Dependency {
                        name: pkg_name.clone(),
                        version_spec: ver,
                        ecosystem: Ecosystem::PyPi,
                        dep_type: DependencyType::Standard,
                        manifest_file: file_path.to_string(),
                        line_number: None,
                    });
                }
            }

            // Poetry dev-dependencies or groups
            if let Some(dev_deps) = poetry.get("dev-dependencies").and_then(|v| v.as_table()) {
                for (pkg_name, val) in dev_deps {
                    let ver = match val {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Table(tbl) => tbl
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string(),
                        other => other.to_string(),
                    };
                    deps.push(Dependency {
                        name: pkg_name.clone(),
                        version_spec: ver,
                        ecosystem: Ecosystem::PyPi,
                        dep_type: DependencyType::Dev,
                        manifest_file: file_path.to_string(),
                        line_number: None,
                    });
                }
            }
        }
    }

    Ok(deps)
}

/// Helper to check if a file path is a Python manifest
pub fn is_python_manifest(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    file_name == "requirements.txt"
        || file_name.ends_with(".requirements.txt")
        || file_name.starts_with("requirements-")
        || file_name == "pyproject.toml"
}
