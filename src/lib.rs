//! phantom-dep: Multi-ecosystem hallucinated dependency & typosquatting scanner.

pub mod cli;
pub mod engine;
pub mod model;
pub mod parsers;
pub mod reporter;

pub use cli::{CliArgs, EcosystemArg, OutputFormat};
pub use engine::scanner::{scan_dependency, scan_path};
pub use model::{Dependency, DependencyType, DetectionReason, Ecosystem, RiskLevel, ScanResult};
pub use reporter::{render_json, render_table, ScanReport};

/// Core version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
