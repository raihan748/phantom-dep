use crate::engine::catalog::is_standard_library;
use crate::model::Ecosystem;

/// Evaluates if a package name collides with a built-in standard library module
pub fn check_standard_lib(ecosystem: Ecosystem, name: &str) -> Option<String> {
    if is_standard_library(ecosystem, name) {
        let eco_name = match ecosystem {
            Ecosystem::Npm => "Node.js runtime",
            Ecosystem::PyPi => "Python standard library",
            Ecosystem::Cargo => "Rust core/std library",
            Ecosystem::Go => "Go standard library",
        };
        Some(format!(
            "Package '{}' is a built-in {} module; does not need to be installed as an external dependency.",
            name, eco_name
        ))
    } else {
        None
    }
}

/// Evaluates if a package name exhibits common AI-hallucinated naming patterns
pub fn check_ai_hallucination_pattern(name: &str) -> Option<(f64, String)> {
    let lower = name.to_ascii_lowercase();

    // Split on '-' or '_'
    let tokens: Vec<&str> = lower
        .split(|c| c == '-' || c == '_')
        .filter(|t| !t.is_empty())
        .collect();

    let ai_buzzwords = [
        "langchain",
        "plugin",
        "openai",
        "gpt",
        "gpt3",
        "gpt4",
        "turbo",
        "copilot",
        "agent",
        "autonomous",
        "chatgpt",
        "wrapper",
        "llm",
        "claude",
        "gemini",
        "deepseek",
        "connector",
        "super",
        "ultra",
        "framework",
        "assist",
    ];

    let mut buzzword_count = 0;
    let mut matched_words = Vec::new();

    for token in &tokens {
        for &bw in &ai_buzzwords {
            if *token == bw || token.starts_with(bw) || token.ends_with(bw) {
                buzzword_count += 1;
                if !matched_words.contains(&bw) {
                    matched_words.push(bw);
                }
                break;
            }
        }
    }

    // Heuristic 1: 3 or more tokens with 2 or more distinct AI buzzwords or high buzzword density
    if (tokens.len() >= 3 && matched_words.len() >= 2) || buzzword_count >= 3 {
        let confidence = (0.75 + (matched_words.len() as f64 * 0.05)).min(0.95);
        return Some((
            confidence,
            format!(
                "Suspicious AI hallucination pattern detected: multiple buzzwords chained together ({})",
                matched_words.join(", ")
            ),
        ));
    }

    // Heuristic 2: Known AI invention templates like `langchain-plugin-...`, `fastapi-ai-agent-...`
    if (lower.starts_with("langchain-plugin-") && lower.len() > 17)
        || (lower.starts_with("chatgpt-") && lower.contains("plugin"))
        || (lower.starts_with("auto-gpt-") && lower.contains("plugin"))
    {
        return Some((
            0.92,
            "Matches classic LLM hallucination naming pattern for non-existent third-party AI plugins".to_string(),
        ));
    }

    None
}

/// Evaluates if a version specifier is impossible or suspicious
pub fn check_impossible_version(version: &str) -> Option<(f64, String)> {
    let trimmed = version.trim();
    if trimmed.is_empty() || trimmed == "*" {
        return None;
    }

    // Check negative numbers
    if trimmed.starts_with('-') || trimmed.contains("=-") {
        return Some((
            0.90,
            format!("Invalid negative version specification: '{}'", trimmed),
        ));
    }

    // Extract digits for major version
    // e.g. "^99.0.0", ">=100.0", "v999.1"
    let clean_ver = trimmed
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .trim();

    let parts: Vec<&str> = clean_ver.split('.').collect();
    if let Some(first) = parts.first() {
        if let Ok(major) = first.parse::<u64>() {
            if major >= 90 {
                return Some((
                    0.85,
                    format!(
                        "Suspiciously high major version '{}' ({}) - likely an AI hallucinated release",
                        major, trimmed
                    ),
                ));
            }
        }
    }

    // Check for weird non-semver tokens in versions
    if trimmed.contains("gpt") || trimmed.contains("turbo") || trimmed.contains("latest-") {
        return Some((
            0.88,
            format!("Version string contains non-standard text token: '{}'", trimmed),
        ));
    }

    None
}
