use crate::report::model::AnalysisInfo;
use crate::wasm::sections::SectionFacts;

pub fn normalize(sections: &mut SectionFacts, analysis: &mut AnalysisInfo) {
    // Apply minimal post-parse normalization and emit analysis warnings.
    //
    // This stage exists to annotate cases where the extracted section data
    // may be incomplete or unconventional, without interpreting risk
    // or influencing rule evaluation.
    if sections.memory_count == 0 {
        analysis
            .warnings
            .push("no memory section or imported memory detected".to_string());
    }

    // Ensure deterministic output ordering.
    analysis.warnings.sort();
}
