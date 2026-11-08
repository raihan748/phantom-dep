use crate::model::{Dependency, DependencyType, Ecosystem};
use std::path::Path;

/// Parses a Go `go.mod` file content into a vector of Dependencies.
pub fn parse_go_mod(content: &str, file_path: &str) -> Result<Vec<Dependency>, String> {
    let mut deps = Vec::new();
    let mut in_require_block = false;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let mut line = raw_line.trim();

        // Strip inline comments, keeping note if it's indirect
        let is_indirect = line.contains("// indirect");
        if let Some(pos) = line.find("//") {
            line = line[..pos].trim();
        }

        if line.is_empty() {
            continue;
        }

        if line.starts_with("require (") || line == "require (" {
            in_require_block = true;
            continue;
        }

        if in_require_block {
            if line == ")" || line.starts_with(')') {
                in_require_block = false;
                continue;
            }

            // Inside require block: <module_path> <version>
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if !tokens.is_empty() {
                let mod_name = tokens[0].trim();
                let ver = if tokens.len() > 1 {
                    tokens[1].trim()
                } else {
                    "*"
                };

                deps.push(Dependency {
                    name: mod_name.to_string(),
                    version_spec: ver.to_string(),
                    ecosystem: Ecosystem::Go,
                    dep_type: if is_indirect {
                        DependencyType::Optional
                    } else {
                        DependencyType::Standard
                    },
                    manifest_file: file_path.to_string(),
                    line_number: Some(line_num),
                });
            }
        } else if line.starts_with("require ") {
            // Single line require: require <module_path> <version>
            let remainder = line.strip_prefix("require ").unwrap().trim();
            let tokens: Vec<&str> = remainder.split_whitespace().collect();
            if !tokens.is_empty() {
                let mod_name = tokens[0].trim();
                let ver = if tokens.len() > 1 {
                    tokens[1].trim()
                } else {
                    "*"
                };

                deps.push(Dependency {
                    name: mod_name.to_string(),
                    version_spec: ver.to_string(),
                    ecosystem: Ecosystem::Go,
                    dep_type: if is_indirect {
                        DependencyType::Optional
                    } else {
                        DependencyType::Standard
                    },
                    manifest_file: file_path.to_string(),
                    line_number: Some(line_num),
                });
            }
        }
    }

    Ok(deps)
}

/// Helper to check if a file path is a Go manifest
pub fn is_go_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| name.eq_ignore_ascii_case("go.mod"))
        .unwrap_or(false)
}
