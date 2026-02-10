use anyhow::Result;
use wasmparser::{FunctionBody, Operator};

/// Aggregated facts about WASM instructions that affect execution boundaries.
///
/// This struct records **capability presence** and **occurrence counts**
/// for instructions that complicate static reasoning:
///
/// - `memory.grow`   → dynamic memory expansion
/// - `call_indirect` → dynamic control flow
/// - `loop`          → potentially unbounded execution
///
/// These facts are **pure observations**:
/// - no interpretation
/// - no policy
/// - no control-flow analysis
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstructionFacts {
    pub has_memory_grow: bool,
    pub memory_grow_count: u64,

    pub has_call_indirect: bool,
    pub call_indirect_count: u64,

    pub has_loop: bool,
    pub loop_count: u64,
}

/// Scans a single WASM function body and updates instruction facts.
///
/// The scan:
/// - performs a single linear pass over operators
/// - does not build a control-flow graph
/// - does not attempt to reason about termination or semantics
///
/// This function is designed to be called once per `CodeSectionEntry`
/// and accumulates results into the provided `InstructionFacts`.
pub fn on_code_entry(facts: &mut InstructionFacts, body: FunctionBody) -> Result<()> {
    let mut reader = body.get_operators_reader()?;

    while !reader.eof() {
        match reader.read()? {
            Operator::MemoryGrow { .. } => {
                facts.has_memory_grow = true;
                facts.memory_grow_count += 1;
            }
            Operator::CallIndirect { .. } => {
                facts.has_call_indirect = true;
                facts.call_indirect_count += 1;
            }
            Operator::Loop { .. } => {
                facts.has_loop = true;
                facts.loop_count += 1;
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::{Parser, Payload};

    /// Extracts all function bodies from a WASM module.
    ///
    /// Mirrors parsing behavior while allowing
    /// tests to operate on small, readable WAT fixtures.
    fn extract_bodies(wasm: &[u8]) -> Vec<FunctionBody<'_>> {
        Parser::new(0)
            .parse_all(wasm)
            .filter_map(|payload| match payload.unwrap() {
                Payload::CodeSectionEntry(body) => Some(body),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_cumulative_detection_multiple_functions() {
        let wasm = wat::parse_str(
            r#"
            (module
              (type (func))
              (table 1 funcref)
              (memory 1)
              (func $f1 (loop (br 0)) (loop (nop)))
              (func $f2 (drop (memory.grow (i32.const 1))))
              (func $f3 (call_indirect (type 0) (i32.const 0)))
            )
            "#,
        )
        .unwrap();

        let mut facts = InstructionFacts::default();
        for body in extract_bodies(&wasm) {
            on_code_entry(&mut facts, body).expect("scan failed");
        }

        assert_eq!(facts.loop_count, 2);
        assert_eq!(facts.memory_grow_count, 1);
        assert_eq!(facts.call_indirect_count, 1);

        assert!(facts.has_loop);
        assert!(facts.has_memory_grow);
        assert!(facts.has_call_indirect);
    }

    #[test]
    fn test_deeply_nested_loops() {
        let wasm = wat::parse_str(
            r#"
            (module
              (func (loop (loop (loop (nop)))))
            )
            "#,
        )
        .unwrap();

        let mut facts = InstructionFacts::default();
        let body = extract_bodies(&wasm).pop().unwrap();
        on_code_entry(&mut facts, body).unwrap();

        assert_eq!(facts.loop_count, 3);
    }

    #[test]
    fn test_empty_function_is_noop() {
        let wasm = wat::parse_str("(module (func))").unwrap();

        let mut facts = InstructionFacts::default();
        let body = extract_bodies(&wasm).pop().unwrap();
        on_code_entry(&mut facts, body).unwrap();

        assert_eq!(facts, InstructionFacts::default());
    }
}
