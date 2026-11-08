//! phantom-dep: Multi-ecosystem hallucinated dependency & typosquatting scanner.

pub mod engine;
pub mod model;
pub mod parsers;

pub use engine::scanner::{scan_dependency, scan_path};
pub use model::{Dependency, DependencyType, DetectionReason, Ecosystem, RiskLevel, ScanResult};

/// Core version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
