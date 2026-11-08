use crate::engine::catalog::{is_in_popular_catalog, is_standard_library};
use crate::engine::heuristics::{
    check_ai_hallucination_pattern, check_impossible_version, check_standard_lib,
};
use crate::engine::typo::find_typosquatting;
use crate::model::{
    Dependency, DetectionReason, Ecosystem, RiskLevel, ScanResult,
};
use crate::parsers::discover_and_parse;
use std::path::Path;

/// Evaluates a single dependency entry and computes hallucination risk
pub fn scan_dependency(dep: &Dependency) -> ScanResult {
    // 1. Standard library collision check
    if is_standard_library(dep.ecosystem, &dep.name) {
        let details = check_standard_lib(dep.ecosystem, &dep.name)
            .unwrap_or_else(|| format!("'{}' is a standard library module.", dep.name));

        return ScanResult {
            dependency: dep.clone(),
            risk_level: RiskLevel::High,
            confidence: 0.95,
            reason: DetectionReason::StandardLibraryCollision,
            details,
            suggested_alternative: Some("Use built-in standard library module (remove from manifest)".to_string()),
        };
    }

    // 2. Verified popular catalog check
    if is_in_popular_catalog(dep.ecosystem, &dep.name) {
        if let Some((ver_conf, ver_details)) = check_impossible_version(&dep.version_spec) {
            return ScanResult {
                dependency: dep.clone(),
                risk_level: RiskLevel::Medium,
                confidence: ver_conf,
                reason: DetectionReason::ImpossibleVersion,
                details: format!("Verified package, but suspicious version: {}", ver_details),
                suggested_alternative: None,
            };
        }

        return ScanResult {
            dependency: dep.clone(),
            risk_level: RiskLevel::Low,
            confidence: 0.05,
            reason: DetectionReason::VerifiedInCatalog,
            details: "Package verified in popular offline catalog".to_string(),
            suggested_alternative: None,
        };
    }

    // 3. Typosquatting / typo similarity check against catalog
    if let Some(typo) = find_typosquatting(dep.ecosystem, &dep.name) {
        let confidence = (0.75 + (typo.similarity * 0.20)).min(0.95);
        return ScanResult {
            dependency: dep.clone(),
            risk_level: RiskLevel::High,
            confidence,
            reason: DetectionReason::TyposquattingSuspected,
            details: format!(
                "Probable typo of popular package '{}' (Levenshtein distance {}, similarity {:.0}%)",
                typo.suggested_name,
                typo.distance,
                typo.similarity * 100.0
            ),
            suggested_alternative: Some(typo.suggested_name),
        };
    }

    // 4. AI hallucination buzzword chaining
    if let Some((conf, details)) = check_ai_hallucination_pattern(&dep.name) {
        return ScanResult {
            dependency: dep.clone(),
            risk_level: RiskLevel::High,
            confidence: conf,
            reason: DetectionReason::HallucinatedPattern,
            details,
            suggested_alternative: None,
        };
    }

    // 5. Impossible version format check
    if let Some((conf, details)) = check_impossible_version(&dep.version_spec) {
        return ScanResult {
            dependency: dep.clone(),
            risk_level: RiskLevel::High,
            confidence: conf,
            reason: DetectionReason::ImpossibleVersion,
            details,
            suggested_alternative: None,
        };
    }

    // 6. Unverified package fallback
    ScanResult {
        dependency: dep.clone(),
        risk_level: RiskLevel::Medium,
        confidence: 0.45,
        reason: DetectionReason::UnverifiedPackage,
        details: "Package not found in offline verified catalog; verify registry existence".to_string(),
        suggested_alternative: None,
    }
}

/// Scans all manifests found in path and filters by confidence
pub fn scan_path(
    path: &Path,
    ecosystem: Option<Ecosystem>,
    min_confidence: f64,
) -> Result<Vec<ScanResult>, String> {
    let deps = discover_and_parse(path, ecosystem)?;
    let mut results = Vec::new();

    for dep in deps {
        let result = scan_dependency(&dep);
        if result.confidence >= min_confidence {
            results.push(result);
        }
    }

    Ok(results)
}
