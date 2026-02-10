use crate::rules::eval::TriggeredRule;
use crate::wasm::sections::{ExportFact, ImportFact};

pub fn sort_imports(imports: &mut [ImportFact]) {
    imports.sort_by(|a, b| {
        (a.module.as_str(), a.name.as_str(), a.kind.as_str()).cmp(&(
            b.module.as_str(),
            b.name.as_str(),
            b.kind.as_str(),
        ))
    });
}

pub fn sort_exports(exports: &mut [ExportFact]) {
    exports.sort_by(|a, b| {
        (a.name.as_str(), a.kind.as_str()).cmp(&(b.name.as_str(), b.kind.as_str()))
    });
}

pub fn sort_triggered_rules(rules: &mut [TriggeredRule]) {
    rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
}
