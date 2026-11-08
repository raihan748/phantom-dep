use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EcosystemArg {
    Npm,
    Pypi,
    Cargo,
    Go,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "phantom-dep",
    author = "SMK Telkom Developer <dev@smktelkom-mks.sch.id>",
    version,
    about = "Offline hallucinated dependency and typosquatting scanner for AI-assisted codebases"
)]
pub struct CliArgs {
    /// Path to file or directory to scan
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Ecosystem filter (npm, pypi, cargo, go, all)
    #[arg(short, long, value_enum, default_value_t = EcosystemArg::All)]
    pub ecosystem: EcosystemArg,

    /// Output report format (table, json)
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Fail with exit code 1 if any high-risk / hallucinated dependencies are detected
    #[arg(short, long, default_value_t = false)]
    pub strict: bool,

    /// Minimum confidence threshold to report (0.0 to 1.0)
    #[arg(short, long, default_value_t = 0.0)]
    pub min_confidence: f64,
}
