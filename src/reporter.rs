use crate::model::{RiskLevel, ScanResult};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub scanned_path: String,
    pub total_dependencies: usize,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
    pub strict_triggered: bool,
    pub findings: Vec<ScanResult>,
}

impl ScanReport {
    pub fn new(path: &Path, results: Vec<ScanResult>, strict: bool) -> Self {
        let total = results.len();
        let high = results.iter().filter(|r| r.risk_level == RiskLevel::High).count();
        let medium = results.iter().filter(|r| r.risk_level == RiskLevel::Medium).count();
        let low = results.iter().filter(|r| r.risk_level == RiskLevel::Low).count();

        Self {
            scanned_path: path.to_string_lossy().to_string(),
            total_dependencies: total,
            high_risk_count: high,
            medium_risk_count: medium,
            low_risk_count: low,
            strict_triggered: strict && high > 0,
            findings: results,
        }
    }
}

/// Formats the scan results as a colorized terminal table
pub fn render_table(report: &ScanReport) -> String {
    let mut out = String::new();

    out.push_str("\n");
    out.push_str(&format!(
        "{}\n",
        "=== phantom-dep: Hallucinated Dependency Scanner ===".bold().cyan()
    ));
    out.push_str(&format!("Target: {}\n", report.scanned_path.yellow()));
    out.push_str(&format!(
        "Scanned: {} dependencies | High Risk: {} | Medium Risk: {} | Low Risk: {}\n\n",
        report.total_dependencies.to_string().bold(),
        report.high_risk_count.to_string().red().bold(),
        report.medium_risk_count.to_string().yellow().bold(),
        report.low_risk_count.to_string().green().bold()
    ));

    if report.findings.is_empty() {
        out.push_str(&format!("{}\n", "No dependencies found matching filters.".dimmed()));
        return out;
    }

    // Column widths
    let w_eco = 8;
    let w_dep = 32;
    let w_ver = 14;
    let w_risk = 8;
    let w_conf = 8;

    out.push_str(&format!(
        "{:<w_eco$} {:<w_dep$} {:<w_ver$} {:<w_risk$} {:<w_conf$} {}\n",
        "ECOSYSTEM".bold(),
        "DEPENDENCY".bold(),
        "VERSION".bold(),
        "RISK".bold(),
        "CONF".bold(),
        "DIAGNOSIS & SUGGESTION".bold(),
    ));

    out.push_str(&format!(
        "{:-<w_eco$} {:-<w_dep$} {:-<w_ver$} {:-<w_risk$} {:-<w_conf$} {:-<40}\n",
        "", "", "", "", "", ""
    ));

    for res in &report.findings {
        let risk_badge = match res.risk_level {
            RiskLevel::High => "HIGH".red().bold(),
            RiskLevel::Medium => "MEDIUM".yellow(),
            RiskLevel::Low => "LOW".green(),
        };

        let conf_str = format!("{:.0}%", res.confidence * 100.0);

        let dep_name = if res.dependency.name.len() > w_dep - 2 {
            format!("{}...", &res.dependency.name[..w_dep - 5])
        } else {
            res.dependency.name.clone()
        };

        let ver_str = if res.dependency.version_spec.len() > w_ver - 2 {
            format!("{}...", &res.dependency.version_spec[..w_ver - 5])
        } else {
            res.dependency.version_spec.clone()
        };

        let mut diag = res.details.clone();
        if let Some(alt) = &res.suggested_alternative {
            diag.push_str(&format!(" -> Suggestion: '{}'", alt.green().bold()));
        }

        out.push_str(&format!(
            "{:<w_eco$} {:<w_dep$} {:<w_ver$} {:<w_risk$} {:<w_conf$} {}\n",
            res.dependency.ecosystem.as_str(),
            dep_name,
            ver_str,
            risk_badge,
            conf_str,
            diag
        ));
    }

    out.push_str("\n");
    if report.strict_triggered {
        out.push_str(&format!(
            "{}\n",
            "[FAILURE] Strict mode active: High risk / hallucinated dependencies detected!"
                .red()
                .bold()
        ));
    } else {
        out.push_str(&format!(
            "{}\n",
            "[SUCCESS] Scan completed successfully.".green().bold()
        ));
    }

    out
}

/// Formats the scan results as pretty-printed JSON
pub fn render_json(report: &ScanReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}
