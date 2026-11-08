# phantom-dep 👻

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Ecosystems](https://img.shields.io/badge/ecosystems-npm%20%7C%20pypi%20%7C%20cargo%20%7C%20go-purple.svg)]()
[![Offline First](https://img.shields.io/badge/network-offline--first-success.svg)]()

> **Multi-ecosystem hallucinated dependency & typosquatting scanner for AI-assisted vibecoding.**

---

## 🎯 The Problem: AI Package Hallucination

When modern developers ("vibecoders") generate applications using LLMs (Claude, GPT-4, Cursor, GitHub Copilot), models frequently:
1. **Hallucinate non-existent package names** (e.g., `langchain-plugin-openai-gpt4-turbo-v2`, `fastapi-ai-agent-framework`).
2. **Mistake standard libraries for external packages** (declaring `sys`, `fs`, `os`, or `fmt` inside `requirements.txt`, `package.json`, or `go.mod`).
3. **Produce subtle typosquatting vectors** (e.g. `reqeusts`, `lodahs`, `panads`, `toki0`), exposing developers to dependency confusion and malicious supply chain takeovers.

Attackers actively scan public prompts and forums to register these hallucinated package names on npm and PyPI, resulting in automated Remote Code Execution upon `npm install` or `pip install`.

**`phantom-dep`** runs **completely offline** before you install dependencies, scanning manifests to flag hallucinated packages, typosquats, and runtime standard library collisions before a single socket connection is opened.

---

## 🏗️ Architecture

```
                      +-----------------------------+
                      |   Target Directory / File   |
                      +--------------+--------------+
                                     |
           +-------------------------+-------------------------+
           |                         |                         |
    [package.json]          [requirements.txt /       [Cargo.toml] / [go.mod]
    (npm AST Parser)         pyproject.toml]          (Cargo / Go Parsers)
           |                (PyPI AST Parser)                  |
           +-------------------------+-------------------------+
                                     |
                                     v
                 +---------------------------------------+
                 |       Normalized Dependency AST       |
                 +-------------------+-------------------+
                                     |
                +--------------------+--------------------+
                |                                         |
                v                                         v
   +--------------------------+              +--------------------------+
   |  Offline Verified Top    |              |  Standard Library        |
   |  Ecosystem Catalogs      |              |  Collision Detector      |
   +------------+-------------+              +------------+-------------+
                |                                         |
                +--------------------+--------------------+
                                     |
                                     v
                 +---------------------------------------+
                 |    Damerau-Levenshtein Typo Matcher   |
                 |     & AI Buzzword Chaining Scorer     |
                 +-------------------+-------------------+
                                     |
                                     v
                 +---------------------------------------+
                 |   Confidence Score & Risk Evaluation  |
                 |       (LOW / MEDIUM / HIGH RISK)      |
                 +-------------------+-------------------+
                                     |
                     +---------------+---------------+
                     |                               |
                     v                               v
          +---------------------+         +---------------------+
          | Colorized Terminal  |         | Machine-Parsable    |
          | Table Report        |         | CI/CD JSON Output   |
          +---------------------+         +---------------------+
```

---

## 🚀 Key Features

- **Multi-Ecosystem Coverage**:
  - **npm / Node.js**: `package.json` (`dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`).
  - **Python**: `requirements.txt` (handles PEP 508 operators, extras `[security]`, environment markers, pip flags) and `pyproject.toml` (PEP 621 & Poetry formats).
  - **Rust / Cargo**: `Cargo.toml` (`dependencies`, `dev-dependencies`, `build-dependencies`, target-specific configs).
  - **Go**: `go.mod` (single-line & multi-line `require (...)` blocks, indirect tracking).
- **Standard Library Collision Guard**: Detects when standard built-in modules (`fs`, `path`, `sys`, `os`, `fmt`, `std`) are mistakenly declared in manifests.
- **Damerau-Levenshtein Keystroke Engine**: Flags dangerous typosquats (`reqeusts` -> `requests`, `lodahs` -> `lodash`) with automatic authentic recommendations.
- **AI Buzzword Chaining Heuristics**: Detects hallucinated multi-keyword package hallucinations like `chatgpt-api-wrapper-v3` or `langchain-plugin-openai-...`.
- **Impossible Semver Validation**: Flags hallucinated release tags (e.g. `^99.0.0`, `-1.0.0`).
- **Zero-Network Offline First**: Runs 100% offline with zero external network calls, ideal for air-gapped security pipelines and pre-commit hooks.
- **CI/CD Strict Mode**: Returns exit code `1` when `--strict` is enabled and any High Risk dependency is identified.

---

## 📦 Installation & Build

Requires Rust 1.75+ (Edition 2021):

```bash
# Clone the repository
git clone https://github.com/raihan748/phantom-dep.git
cd phantom-dep

# Build release binary
cargo build --release

# The compiled binary is at target/release/phantom-dep
```

---

## 💻 CLI Usage

```text
Offline hallucinated dependency and typosquatting scanner for AI-assisted codebases

Usage: phantom-dep.exe [OPTIONS]

Options:
  -p, --path <PATH>
          Path to file or directory to scan [default: .]
  -e, --ecosystem <ECOSYSTEM>
          Ecosystem filter (npm, pypi, cargo, go, all) [default: all] [possible values: npm, pypi, cargo, go, all]
  -f, --format <FORMAT>
          Output report format (table, json) [default: table] [possible values: table, json]
  -s, --strict
          Fail with exit code 1 if any high-risk / hallucinated dependencies are detected
  -m, --min-confidence <MIN_CONFIDENCE>
          Minimum confidence threshold to report (0.0 to 1.0) [default: 0]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Examples

#### 1. Scan current directory with terminal table output
```bash
phantom-dep --path .
```

#### 2. Enforce strict exit code in CI/CD pipeline
```bash
phantom-dep --path ./src --strict
```

#### 3. Scan only Python manifests with JSON output
```bash
phantom-dep --path ./backend --ecosystem pypi --format json
```

#### 4. Filter only high-confidence hallucination risks (>= 75%)
```bash
phantom-dep --path . --min-confidence 0.75
```

---

## 📊 Sample Output

### Terminal Table Output

```text
=== phantom-dep: Hallucinated Dependency Scanner ===
Target: ./sample-project
Scanned: 5 dependencies | High Risk: 3 | Medium Risk: 1 | Low Risk: 1

ECOSYSTEM DEPENDENCY                       VERSION        RISK     CONF     DIAGNOSIS & SUGGESTION
-------------------------------------------------------------------------------------------------------------
pypi      requests                         >= 2.28.0      LOW      5%       Package verified in popular offline catalog
pypi      reqeusts                         >= 2.28.0      HIGH     95%      Probable typo of popular package 'requests' (Levenshtein distance 1, similarity 88%) -> Suggestion: 'requests'
pypi      sys                              *              HIGH     95%      Package 'sys' is a built-in Python standard library module; does not need to be installed as an external dependency. -> Suggestion: 'Use built-in standard library module (remove from manifest)'
pypi      langchain-plugin-openai-gpt4-... ^1.0.0         HIGH     90%      Suspicious AI hallucination pattern detected: multiple buzzwords chained together (langchain, plugin, openai, gpt, turbo)
cargo     internal-company-shared-utils    0.1.0          MEDIUM   45%      Package not found in offline verified catalog; verify registry existence

[FAILURE] Strict mode active: High risk / hallucinated dependencies detected!
```

### JSON Output (`--format json`)

```json
{
  "scanned_path": "./sample-project",
  "total_dependencies": 2,
  "high_risk_count": 1,
  "medium_risk_count": 0,
  "low_risk_count": 1,
  "strict_triggered": true,
  "findings": [
    {
      "dependency": {
        "name": "requests",
        "version_spec": ">= 2.28.0",
        "ecosystem": "pypi",
        "dep_type": "dependencies",
        "manifest_file": "requirements.txt",
        "line_number": 1
      },
      "risk_level": "LOW",
      "confidence": 0.05,
      "reason": "verified_in_catalog",
      "details": "Package verified in popular offline catalog",
      "suggested_alternative": null
    },
    {
      "dependency": {
        "name": "reqeusts",
        "version_spec": ">= 2.28.0",
        "ecosystem": "pypi",
        "dep_type": "dependencies",
        "manifest_file": "requirements.txt",
        "line_number": 2
      },
      "risk_level": "HIGH",
      "confidence": 0.95,
      "reason": "typosquatting_suspected",
      "details": "Probable typo of popular package 'requests' (Levenshtein distance 1, similarity 88%)",
      "suggested_alternative": "requests"
    }
  ]
}
```

---

## 🧪 Testing

Run the automated test suite:

```bash
cargo test
```

All 18 unit and integration tests execute with zero warnings:

```text
running 18 tests
test test_ecosystem_parsing_strings ... ok
test test_go_mod_parsing ... ok
test test_impossible_version_detection ... ok
test test_catalog_inventory_completeness ... ok
test test_invalid_manifest_syntax_handling ... ok
test test_levenshtein_and_similarity ... ok
test test_cli_argument_parsing ... ok
test test_npm_package_json_parsing ... ok
test test_python_requirements_txt_parsing ... ok
test test_standard_library_collision ... ok
test test_cargo_toml_parsing ... ok
test test_python_pyproject_toml_parsing ... ok
test test_verified_catalog_package ... ok
test test_typosquatting_detection ... ok
test test_ai_hallucination_pattern_detection ... ok
test test_unverified_package_fallback ... ok
test test_scan_path_and_filtering ... ok
test test_mixed_workspace_multi_manifest_scanning ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
Created with high-impact software engineering by **SMK Telkom Developer**.
