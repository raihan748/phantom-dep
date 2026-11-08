use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported software package ecosystems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    Npm,
    PyPi,
    Cargo,
    Go,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::Npm => "npm",
            Ecosystem::PyPi => "pypi",
            Ecosystem::Cargo => "cargo",
            Ecosystem::Go => "go",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "npm" | "node" | "javascript" | "js" => Some(Ecosystem::Npm),
            "pypi" | "python" | "py" => Some(Ecosystem::PyPi),
            "cargo" | "rust" | "rs" => Some(Ecosystem::Cargo),
            "go" | "golang" => Some(Ecosystem::Go),
            _ => None,
        }
    }
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Dependency category in manifest
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Standard,
    Dev,
    Peer,
    Optional,
    Build,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::Standard => write!(f, "dependencies"),
            DependencyType::Dev => write!(f, "devDependencies"),
            DependencyType::Peer => write!(f, "peerDependencies"),
            DependencyType::Optional => write!(f, "optionalDependencies"),
            DependencyType::Build => write!(f, "buildDependencies"),
        }
    }
}

/// Parsed dependency entry from a manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_spec: String,
    pub ecosystem: Ecosystem,
    pub dep_type: DependencyType,
    pub manifest_file: String,
    pub line_number: Option<usize>,
}

/// Risk level for dependency hallucination or squatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
        }
    }
}

/// Primary category of detection reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionReason {
    VerifiedInCatalog,
    StandardLibraryCollision,
    TyposquattingSuspected,
    HallucinatedPattern,
    ImpossibleVersion,
    UnverifiedPackage,
}

impl fmt::Display for DetectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionReason::VerifiedInCatalog => write!(f, "Verified Catalog Package"),
            DetectionReason::StandardLibraryCollision => write!(f, "Standard Library Collision"),
            DetectionReason::TyposquattingSuspected => write!(f, "Typosquatting Suspected"),
            DetectionReason::HallucinatedPattern => write!(f, "Hallucinated Pattern"),
            DetectionReason::ImpossibleVersion => write!(f, "Impossible Version Specifier"),
            DetectionReason::UnverifiedPackage => write!(f, "Unverified Package"),
        }
    }
}

/// Scan result for a single dependency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanResult {
    pub dependency: Dependency,
    pub risk_level: RiskLevel,
    pub confidence: f64,
    pub reason: DetectionReason,
    pub details: String,
    pub suggested_alternative: Option<String>,
}
