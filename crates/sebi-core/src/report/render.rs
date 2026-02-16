use crate::TOOL_NAME;
use crate::report::model::Report;

pub fn render_text(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("{} {}\n", TOOL_NAME, report.tool.version));
    out.push_str(&format!(
        "Artifact size: {} bytes\n",
        report.artifact.size_bytes
    ));
    out.push_str(&format!(
        "Classification: {:?}\n",
        report.classification.level
    ));
    out.push_str("Triggered rules:\n");
    for r in &report.rules.triggered {
        out.push_str(&format!("  - {} [{}] {}\n", r.rule_id, r.severity, r.title));
    }
    out
}
