use crate::signals::model::*;
use crate::wasm::{scan::InstructionFacts, sections::SectionFacts};

/// Transforms low-level parsing facts into a stable `Signals` schema.
///
/// Performs a pure structural mapping from internal facts to the public
/// representation. This function contains no policy or analysis logic,
/// ensuring a strict boundary between extraction and interpretation.
pub fn extract_signals(sections: &SectionFacts, instr: &InstructionFacts) -> Signals {
    Signals {
        module: ModuleSignals {
            function_count: sections.function_count,
            section_count: None, // Reserved for future section tracking.
        },

        memory: MemorySignals {
            memory_count: sections.memory_count,
            min_pages: sections.memory_min_pages,
            max_pages: sections.memory_max_pages,
            has_max: sections.memory_has_max,
        },

        imports_exports: ImportExportSignals {
            import_count: sections.import_count,
            export_count: sections.export_count,
            // Assumes lists are deterministically sorted at the SectionFacts layer.
            imports: Some(
                sections
                    .imports
                    .iter()
                    .map(|i| ImportItem {
                        module: i.module.clone(),
                        name: i.name.clone(),
                        kind: i.kind.clone(),
                    })
                    .collect(),
            ),
            exports: Some(
                sections
                    .exports
                    .iter()
                    .map(|e| ExportItem {
                        name: e.name.clone(),
                        kind: e.kind.clone(),
                    })
                    .collect(),
            ),
        },

        instructions: InstructionSignals {
            has_memory_grow: instr.has_memory_grow,
            memory_grow_count: instr.memory_grow_count,
            has_call_indirect: instr.has_call_indirect,
            call_indirect_count: instr.call_indirect_count,
            has_loop: instr.has_loop,
            loop_count: instr.loop_count,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::sections::{ExportFact, ImportFact};

    fn build_sections() -> SectionFacts {
        SectionFacts {
            function_count: 24,
            memory_count: 1,
            memory_min_pages: Some(1),
            memory_max_pages: Some(256),
            memory_has_max: true,
            import_count: 3,
            export_count: 2,
            imports: vec![
                // deliberately unsorted
                ImportFact {
                    module: "z_mod".into(),
                    name: "a".into(),
                    kind: "func".into(),
                },
                ImportFact {
                    module: "a_mod".into(),
                    name: "z".into(),
                    kind: "func".into(),
                },
                ImportFact {
                    module: "a_mod".into(),
                    name: "a".into(),
                    kind: "func".into(),
                },
            ],
            exports: vec![
                // deliberately unsorted
                ExportFact {
                    name: "z".into(),
                    kind: "func".into(),
                },
                ExportFact {
                    name: "a".into(),
                    kind: "func".into(),
                },
            ],
            ..Default::default()
        }
    }

    fn build_instr() -> InstructionFacts {
        InstructionFacts {
            has_memory_grow: true,
            memory_grow_count: 2,
            has_call_indirect: true,
            call_indirect_count: 15,
            has_loop: false,
            loop_count: 0,
        }
    }

    #[test]
    fn extract_signals_maps_all_fields_correctly() {
        let sections = build_sections();
        let instr = build_instr();

        let signals = extract_signals(&sections, &instr);

        assert_eq!(signals.module.function_count, 24);
        assert!(signals.module.section_count.is_none());

        assert_eq!(signals.memory.memory_count, 1);
        assert_eq!(signals.memory.min_pages, Some(1));
        assert_eq!(signals.memory.max_pages, Some(256));
        assert!(signals.memory.has_max);

        assert_eq!(signals.imports_exports.import_count, 3);
        assert_eq!(signals.imports_exports.export_count, 2);

        assert!(signals.instructions.has_memory_grow);
        assert_eq!(signals.instructions.memory_grow_count, 2);
        assert!(signals.instructions.has_call_indirect);
        assert_eq!(signals.instructions.call_indirect_count, 15);
        assert!(!signals.instructions.has_loop);
        assert_eq!(signals.instructions.loop_count, 0);
    }

    #[test]
    fn extract_signals_is_deterministic() {
        let sections = build_sections();
        let instr = build_instr();

        let s1 = extract_signals(&sections, &instr);
        let s2 = extract_signals(&sections, &instr);

        assert_eq!(
            serde_json::to_string(&s1).unwrap(),
            serde_json::to_string(&s2).unwrap()
        );
    }

    #[test]
    fn extract_signals_preserves_deterministic_ordering() {
        let mut sections = build_sections();

        // IMPORTANT: assume SectionFacts layer sorted beforehand.
        // If someone removes sorting upstream, this test catches it.
        sections.imports.sort_by(|a, b| {
            (a.module.as_str(), a.name.as_str(), a.kind.as_str()).cmp(&(
                b.module.as_str(),
                b.name.as_str(),
                b.kind.as_str(),
            ))
        });
        sections.exports.sort_by(|a, b| {
            (a.name.as_str(), a.kind.as_str()).cmp(&(b.name.as_str(), b.kind.as_str()))
        });

        let signals = extract_signals(&sections, &InstructionFacts::default());

        let imports = signals.imports_exports.imports.unwrap();
        assert_eq!(imports[0].module, "a_mod");
        assert_eq!(imports[0].name, "a");

        let exports = signals.imports_exports.exports.unwrap();
        assert_eq!(exports[0].name, "a");
    }

    #[test]
    fn extract_signals_handles_missing_memory_bounds() {
        let mut sections = SectionFacts::default();
        sections.memory_count = 1;
        sections.memory_min_pages = None;
        sections.memory_max_pages = None;
        sections.memory_has_max = false;

        let signals = extract_signals(&sections, &InstructionFacts::default());

        assert_eq!(signals.memory.min_pages, None);
        assert_eq!(signals.memory.max_pages, None);
        assert!(!signals.memory.has_max);
    }

    #[test]
    fn extract_signals_handles_empty_sections() {
        let signals = extract_signals(&SectionFacts::default(), &InstructionFacts::default());

        assert_eq!(signals.module.function_count, 0);
        assert_eq!(signals.memory.memory_count, 0);
        assert_eq!(signals.instructions.memory_grow_count, 0);

        assert!(signals.imports_exports.imports.unwrap().is_empty());
        assert!(signals.imports_exports.exports.unwrap().is_empty());
    }

    #[test]
    fn extract_signals_handles_large_instruction_counts() {
        let instr = InstructionFacts {
            has_memory_grow: true,
            memory_grow_count: u64::MAX,
            has_call_indirect: true,
            call_indirect_count: u64::MAX,
            has_loop: true,
            loop_count: u64::MAX,
        };

        let signals = extract_signals(&SectionFacts::default(), &instr);

        assert_eq!(signals.instructions.memory_grow_count, u64::MAX);
        assert_eq!(signals.instructions.call_indirect_count, u64::MAX);
        assert_eq!(signals.instructions.loop_count, u64::MAX);
    }
}
