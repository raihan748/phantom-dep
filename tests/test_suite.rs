use phantom_dep::engine::catalog::{get_catalog, get_standard_libraries, is_standard_library};
use phantom_dep::engine::heuristics::{
    check_ai_hallucination_pattern, check_impossible_version,
};
use phantom_dep::engine::scanner::{scan_dependency, scan_path};
use phantom_dep::engine::typo::{find_typosquatting, levenshtein_distance, similarity_ratio};
use phantom_dep::model::{
    Dependency, DependencyType, DetectionReason, Ecosystem, RiskLevel,
};
use phantom_dep::parsers::cargo::parse_cargo_toml;
use phantom_dep::parsers::go::parse_go_mod;
use phantom_dep::parsers::npm::parse_package_json;
use phantom_dep::parsers::python::{parse_pyproject_toml, parse_requirements_txt};
use phantom_dep::reporter::{render_json, render_table, ScanReport};
use std::fs;

#[test]
fn test_npm_package_json_parsing() {
    let json_content = r#"{
        "name": "sample-web",
        "version": "1.0.0",
        "dependencies": {
            "express": "^4.18.2",
            "lodash": "4.17.21"
        },
        "devDependencies": {
            "typescript": "^5.0.0",
            "jest": "^29.5.0"
        },
        "peerDependencies": {
            "react": ">=18.0.0"
        },
        "optionalDependencies": {
            "chokidar": "^3.5.3"
        }
    }"#;

    let deps = parse_package_json(json_content, "package.json").expect("Failed to parse package.json");
    assert_eq!(deps.len(), 6);

    let express_dep = deps.iter().find(|d| d.name == "express").unwrap();
    assert_eq!(express_dep.version_spec, "^4.18.2");
    assert_eq!(express_dep.dep_type, DependencyType::Standard);
    assert_eq!(express_dep.ecosystem, Ecosystem::Npm);

    let ts_dep = deps.iter().find(|d| d.name == "typescript").unwrap();
    assert_eq!(ts_dep.dep_type, DependencyType::Dev);

    let react_dep = deps.iter().find(|d| d.name == "react").unwrap();
    assert_eq!(react_dep.dep_type, DependencyType::Peer);

    let chokidar_dep = deps.iter().find(|d| d.name == "chokidar").unwrap();
    assert_eq!(chokidar_dep.dep_type, DependencyType::Optional);
}

#[test]
fn test_python_requirements_txt_parsing() {
    let req_content = r#"
        # Core requirements
        requests >= 2.28.0, < 3.0.0
        numpy == 1.24.3
        pandas ~= 2.0.0
        scipy[extra] >= 1.9.0; python_version >= '3.8'
        -r requirements-base.txt
        --extra-index-url https://download.pytorch.org/whl/cu118
        torch
        sys
        langchain-plugin-openai-gpt4-turbo-v2
    "#;

    let deps = parse_requirements_txt(req_content, "requirements.txt");
    let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();

    assert!(names.contains(&"requests"));
    assert!(names.contains(&"numpy"));
    assert!(names.contains(&"pandas"));
    assert!(names.contains(&"scipy"));
    assert!(names.contains(&"torch"));
    assert!(names.contains(&"sys"));
    assert!(names.contains(&"langchain-plugin-openai-gpt4-turbo-v2"));

    let requests_dep = deps.iter().find(|d| d.name == "requests").unwrap();
    assert_eq!(requests_dep.version_spec, ">= 2.28.0, < 3.0.0");
    assert_eq!(requests_dep.ecosystem, Ecosystem::PyPi);

    let scipy_dep = deps.iter().find(|d| d.name == "scipy").unwrap();
    assert_eq!(scipy_dep.version_spec, ">= 1.9.0");
}

#[test]
fn test_python_pyproject_toml_parsing() {
    let toml_content = r#"
        [project]
        name = "data-pipeline"
        version = "0.1.0"
        dependencies = [
            "requests >= 2.28.0",
            "pandas",
            "pydantic >= 2.0.0"
        ]

        [project.optional-dependencies]
        test = [
            "pytest >= 7.0.0"
        ]

        [tool.poetry.dependencies]
        python = "^3.10"
        fastapi = "^0.100.0"
        uvicorn = { version = "^0.23.0", extras = ["standard"] }
    "#;

    let deps = parse_pyproject_toml(toml_content, "pyproject.toml").expect("Failed to parse pyproject.toml");
    let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();

    assert!(names.contains(&"requests"));
    assert!(names.contains(&"pandas"));
    assert!(names.contains(&"pydantic"));
    assert!(names.contains(&"pytest"));
    assert!(names.contains(&"fastapi"));
    assert!(names.contains(&"uvicorn"));
    // 'python' key should be excluded as a package dependency
    assert!(!names.contains(&"python"));
}

#[test]
fn test_cargo_toml_parsing() {
    let cargo_content = r#"
        [package]
        name = "test-app"
        version = "0.1.0"

        [dependencies]
        serde = "1.0"
        tokio = { version = "1.35", features = ["full"] }
        my_json = { package = "serde_json", version = "1.0" }

        [dev-dependencies]
        tempfile = "3.8"

        [build-dependencies]
        syn = "2.0"

        [target.'cfg(windows)'.dependencies]
        colored = "2.1"
    "#;

    let deps = parse_cargo_toml(cargo_content, "Cargo.toml").expect("Failed to parse Cargo.toml");

    let serde_dep = deps.iter().find(|d| d.name == "serde").unwrap();
    assert_eq!(serde_dep.version_spec, "1.0");
    assert_eq!(serde_dep.dep_type, DependencyType::Standard);

    let tokio_dep = deps.iter().find(|d| d.name == "tokio").unwrap();
    assert_eq!(tokio_dep.version_spec, "1.35");

    let json_dep = deps.iter().find(|d| d.name == "serde_json").unwrap();
    assert_eq!(json_dep.version_spec, "1.0");

    let temp_dep = deps.iter().find(|d| d.name == "tempfile").unwrap();
    assert_eq!(temp_dep.dep_type, DependencyType::Dev);

    let syn_dep = deps.iter().find(|d| d.name == "syn").unwrap();
    assert_eq!(syn_dep.dep_type, DependencyType::Build);

    let colored_dep = deps.iter().find(|d| d.name == "colored").unwrap();
    assert_eq!(colored_dep.version_spec, "2.1");
}

#[test]
fn test_go_mod_parsing() {
    let go_mod_content = r#"
        module github.com/example/demo

        go 1.21

        require (
            github.com/gin-gonic/gin v1.9.1
            github.com/google/uuid v1.6.0 // indirect
            github.com/stretchr/testify v1.8.4
        )

        require github.com/spf13/cobra v1.8.0
    "#;

    let deps = parse_go_mod(go_mod_content, "go.mod").expect("Failed to parse go.mod");
    assert_eq!(deps.len(), 4);

    let gin = deps.iter().find(|d| d.name == "github.com/gin-gonic/gin").unwrap();
    assert_eq!(gin.version_spec, "v1.9.1");
    assert_eq!(gin.dep_type, DependencyType::Standard);

    let uuid = deps.iter().find(|d| d.name == "github.com/google/uuid").unwrap();
    assert_eq!(uuid.dep_type, DependencyType::Optional); // indirect marked as optional

    let cobra = deps.iter().find(|d| d.name == "github.com/spf13/cobra").unwrap();
    assert_eq!(cobra.version_spec, "v1.8.0");
}

#[test]
fn test_levenshtein_and_similarity() {
    assert_eq!(levenshtein_distance("requests", "requests"), 0);
    assert_eq!(levenshtein_distance("reqeusts", "requests"), 1);
    assert_eq!(levenshtein_distance("lodahs", "lodash"), 1);
    assert_eq!(levenshtein_distance("panads", "pandas"), 1);
    assert_eq!(levenshtein_distance("toki0", "tokio"), 1);
    assert_eq!(levenshtein_distance("sred", "serde"), 2);

    let sim = similarity_ratio("reqeusts", "requests");
    assert!(sim >= 0.85);
}

#[test]
fn test_typosquatting_detection() {
    let typo_pypi = find_typosquatting(Ecosystem::PyPi, "reqeusts").unwrap();
    assert_eq!(typo_pypi.suggested_name, "requests");
    assert_eq!(typo_pypi.distance, 1);

    let typo_npm = find_typosquatting(Ecosystem::Npm, "expres").unwrap();
    assert_eq!(typo_npm.suggested_name, "express");

    let typo_cargo = find_typosquatting(Ecosystem::Cargo, "toki0").unwrap();
    assert_eq!(typo_cargo.suggested_name, "tokio");

    // Exact catalog match returns None for typosquatting
    assert!(find_typosquatting(Ecosystem::PyPi, "requests").is_none());
}

#[test]
fn test_standard_library_collision() {
    // Python
    assert!(is_standard_library(Ecosystem::PyPi, "sys"));
    assert!(is_standard_library(Ecosystem::PyPi, "os"));
    assert!(is_standard_library(Ecosystem::PyPi, "json"));
    assert!(is_standard_library(Ecosystem::PyPi, "subprocess"));

    // Node.js
    assert!(is_standard_library(Ecosystem::Npm, "fs"));
    assert!(is_standard_library(Ecosystem::Npm, "path"));
    assert!(is_standard_library(Ecosystem::Npm, "http"));

    // Go
    assert!(is_standard_library(Ecosystem::Go, "fmt"));
    assert!(is_standard_library(Ecosystem::Go, "net/http"));

    // Rust
    assert!(is_standard_library(Ecosystem::Cargo, "std"));
    assert!(is_standard_library(Ecosystem::Cargo, "core"));

    // Check scanning result
    let dep = Dependency {
        name: "sys".to_string(),
        version_spec: "*".to_string(),
        ecosystem: Ecosystem::PyPi,
        dep_type: DependencyType::Standard,
        manifest_file: "requirements.txt".to_string(),
        line_number: Some(1),
    };
    let scan = scan_dependency(&dep);
    assert_eq!(scan.risk_level, RiskLevel::High);
    assert_eq!(scan.reason, DetectionReason::StandardLibraryCollision);
    assert!(scan.confidence >= 0.90);
    assert!(scan.suggested_alternative.is_some());
}

#[test]
fn test_ai_hallucination_pattern_detection() {
    let hallucinated_dep = Dependency {
        name: "langchain-plugin-openai-gpt4-turbo-v2".to_string(),
        version_spec: "^1.0.0".to_string(),
        ecosystem: Ecosystem::PyPi,
        dep_type: DependencyType::Standard,
        manifest_file: "requirements.txt".to_string(),
        line_number: Some(1),
    };

    let scan = scan_dependency(&hallucinated_dep);
    assert_eq!(scan.risk_level, RiskLevel::High);
    assert_eq!(scan.reason, DetectionReason::HallucinatedPattern);
    assert!(scan.confidence >= 0.85);

    let pattern = check_ai_hallucination_pattern("fastapi-ai-agent-framework");
    assert!(pattern.is_some());
}

#[test]
fn test_impossible_version_detection() {
    let (conf, details) = check_impossible_version("^99.0.0").unwrap();
    assert!(conf >= 0.80);
    assert!(details.contains("99"));

    let neg = check_impossible_version("-1.2.3");
    assert!(neg.is_some());

    let normal = check_impossible_version("^1.28.3");
    assert!(normal.is_none());
}

#[test]
fn test_verified_catalog_package() {
    let dep = Dependency {
        name: "requests".to_string(),
        version_spec: "^2.31.0".to_string(),
        ecosystem: Ecosystem::PyPi,
        dep_type: DependencyType::Standard,
        manifest_file: "requirements.txt".to_string(),
        line_number: Some(1),
    };

    let scan = scan_dependency(&dep);
    assert_eq!(scan.risk_level, RiskLevel::Low);
    assert_eq!(scan.reason, DetectionReason::VerifiedInCatalog);
    assert!(scan.confidence <= 0.10);
}

#[test]
fn test_unverified_package_fallback() {
    let dep = Dependency {
        name: "my-internal-telecom-lib".to_string(),
        version_spec: "1.0.0".to_string(),
        ecosystem: Ecosystem::Cargo,
        dep_type: DependencyType::Standard,
        manifest_file: "Cargo.toml".to_string(),
        line_number: None,
    };

    let scan = scan_dependency(&dep);
    assert_eq!(scan.risk_level, RiskLevel::Medium);
    assert_eq!(scan.reason, DetectionReason::UnverifiedPackage);
}

#[test]
fn test_scan_path_and_filtering() {
    let temp_dir = std::env::temp_dir().join("phantom_dep_test_env");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let req_path = temp_dir.join("requirements.txt");
    fs::write(
        &req_path,
        "requests>=2.28\nreqeusts>=2.28\nsys\nlangchain-plugin-openai-gpt4-turbo-v2\n",
    )
    .unwrap();

    let all_findings = scan_path(&temp_dir, Some(Ecosystem::PyPi), 0.0).unwrap();
    assert_eq!(all_findings.len(), 4);

    // Only high confidence >= 0.70
    let high_conf = scan_path(&temp_dir, Some(Ecosystem::PyPi), 0.70).unwrap();
    assert_eq!(high_conf.len(), 3); // reqeusts, sys, langchain-plugin-...

    // Test ScanReport
    let report = ScanReport::new(&temp_dir, all_findings, true);
    assert_eq!(report.total_dependencies, 4);
    assert_eq!(report.high_risk_count, 3);
    assert_eq!(report.low_risk_count, 1);
    assert!(report.strict_triggered);

    // Test reporting formats
    let table = render_table(&report);
    assert!(table.contains("phantom-dep"));
    assert!(table.contains("requests"));
    assert!(table.contains("reqeusts"));

    let json_output = render_json(&report).unwrap();
    assert!(json_output.contains("\"high_risk_count\": 3"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_catalog_inventory_completeness() {
    assert!(get_catalog(Ecosystem::Npm).len() >= 50);
    assert!(get_catalog(Ecosystem::PyPi).len() >= 50);
    assert!(get_catalog(Ecosystem::Cargo).len() >= 50);
    assert!(get_catalog(Ecosystem::Go).len() >= 30);

    assert!(get_standard_libraries(Ecosystem::Npm).len() >= 20);
    assert!(get_standard_libraries(Ecosystem::PyPi).len() >= 30);
    assert!(get_standard_libraries(Ecosystem::Cargo).len() >= 5);
    assert!(get_standard_libraries(Ecosystem::Go).len() >= 20);
}

#[test]
fn test_cli_argument_parsing() {
    use clap::Parser;
    use phantom_dep::cli::{CliArgs, EcosystemArg, OutputFormat};

    let args = CliArgs::try_parse_from([
        "phantom-dep",
        "--path",
        "./test-project",
        "--ecosystem",
        "npm",
        "--format",
        "json",
        "--strict",
        "--min-confidence",
        "0.75",
    ])
    .expect("CLI args parse failed");

    assert_eq!(args.path.to_str().unwrap(), "./test-project");
    assert_eq!(args.ecosystem, EcosystemArg::Npm);
    assert_eq!(args.format, OutputFormat::Json);
    assert!(args.strict);
    assert_eq!(args.min_confidence, 0.75);
}

#[test]
fn test_ecosystem_parsing_strings() {
    assert_eq!(Ecosystem::parse("npm"), Some(Ecosystem::Npm));
    assert_eq!(Ecosystem::parse("node"), Some(Ecosystem::Npm));
    assert_eq!(Ecosystem::parse("pypi"), Some(Ecosystem::PyPi));
    assert_eq!(Ecosystem::parse("python"), Some(Ecosystem::PyPi));
    assert_eq!(Ecosystem::parse("cargo"), Some(Ecosystem::Cargo));
    assert_eq!(Ecosystem::parse("rust"), Some(Ecosystem::Cargo));
    assert_eq!(Ecosystem::parse("go"), Some(Ecosystem::Go));
    assert_eq!(Ecosystem::parse("golang"), Some(Ecosystem::Go));
    assert_eq!(Ecosystem::parse("unknown_lang"), None);
}

#[test]
fn test_invalid_manifest_syntax_handling() {
    let bad_json = "{ invalid json: 123 ";
    let res = parse_package_json(bad_json, "package.json");
    assert!(res.is_err());

    let bad_toml = "[project\nversion = 1";
    let res2 = parse_cargo_toml(bad_toml, "Cargo.toml");
    assert!(res2.is_err());
}

#[test]
fn test_mixed_workspace_multi_manifest_scanning() {
    let temp_dir = std::env::temp_dir().join("phantom_dep_multi_manifest_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let npm_file = temp_dir.join("package.json");
    fs::write(&npm_file, r#"{"dependencies": {"react": "^18.0.0", "fs": "*" }}"#).unwrap();

    let py_file = temp_dir.join("requirements.txt");
    fs::write(&py_file, "requests>=2.28\nsys\n").unwrap();

    let cargo_file = temp_dir.join("Cargo.toml");
    fs::write(&cargo_file, "[package]\nname=\"sample\"\nversion=\"0.1.0\"\n[dependencies]\nserde=\"1.0\"\nstd=\"*\"\n").unwrap();

    let go_file = temp_dir.join("go.mod");
    fs::write(&go_file, "module example\ngo 1.20\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n\tfmt v1.0\n)\n").unwrap();

    let all_results = scan_path(&temp_dir, None, 0.0).expect("Scan multi-manifest failed");
    assert_eq!(all_results.len(), 8);

    let high_risks: Vec<_> = all_results.iter().filter(|r| r.risk_level == RiskLevel::High).collect();
    // fs, sys, std, fmt are all standard library collisions!
    assert_eq!(high_risks.len(), 4);

    let low_risks: Vec<_> = all_results.iter().filter(|r| r.risk_level == RiskLevel::Low).collect();
    // react, requests, serde, gin are all genuine catalog packages!
    assert_eq!(low_risks.len(), 4);

    let _ = fs::remove_dir_all(&temp_dir);
}
