# SEBI Rule Catalog

**Catalog Version:** `0.1.0`

This document defines the **official rule catalog** used by **SEBI (Stylus Execution Boundary Inspector)**.

Rules interpret **raw execution-boundary signals** extracted from Stylus-compiled WASM artifacts and classify them into **risk categories**.

Rules are **deterministic, explainable, and policy-driven**.

---

## Relationship to Report Schema

This rule catalog operates **exclusively on signals defined in the SEBI report schema**, as specified in
[`SCHEMA.md`](./SCHEMA.md).

Rules:

* **do not parse WASM directly**
* **do not inspect instructions directly**
* **do not derive signals**
* **do not depend on runtime execution**

All rule trigger conditions reference **explicit schema paths**.

---

## 1. Rule Design Principles

### 1.1 Signals first, rules second

Rules operate **only on signals** defined in the SEBI report schema.

Rules **must never**:

* parse WASM directly
* inspect instructions directly
* depend on runtime execution
* rely on probabilistic heuristics

This separation guarantees:

* explainability
* testability
* deterministic behavior

---

### 1.2 Explainability by construction

Each rule:

* has a stable identifier
* documents *why* it exists
* emits structured evidence when triggered

Every triggered rule must answer:

> *“Why did this rule fire?”*

---

### 1.3 Conservative by design

Rules detect **potential execution-boundary risks**, not confirmed failures.

SEBI favors:

* early, conservative warnings
* false positives over silent misses

---

## 2. Severity Levels

Each rule has a fixed severity.

| Severity | Meaning                                                   |
| -------- | --------------------------------------------------------- |
| LOW      | Informational signal                                      |
| MED      | Potential execution-boundary concern                      |
| HIGH     | Strong indicator of unbounded or hard-to-reason execution |

Severity reflects **structural execution risk**, not exploitability.

---

## 3. Rule Definitions

---

### R-MEM-01 — Missing Declared Memory Maximum

| Field          | Value        |
| -------------- | ------------ |
| **Rule ID**    | `R-MEM-01`   |
| **Severity**   | MED          |
| **Category**   | Memory       |
| **Applies to** | Module-level |

#### Trigger condition

```
signals.memory.has_max == false
```

#### Schema dependencies

* `signals.memory.has_max`
* `signals.memory.min_pages`

#### Rationale

WASM memory without a declared maximum may grow until constrained by the host environment.
This reduces predictability and complicates static resource bounding.

#### Evidence emitted

* `signals.memory.has_max`
* `signals.memory.min_pages`

#### Notes

This rule does **not** imply incorrect or malicious behavior.
It highlights reduced *static enforceability* of memory limits.

---

### R-MEM-02 — Runtime Memory Growth Detected

| Field          | Value             |
| -------------- | ----------------- |
| **Rule ID**    | `R-MEM-02`        |
| **Severity**   | HIGH              |
| **Category**   | Memory            |
| **Applies to** | Instruction-level |

#### Trigger condition

```
signals.instructions.has_memory_grow == true
```

#### Schema dependencies

* `signals.instructions.has_memory_grow`
* `signals.instructions.memory_grow_count`

#### Rationale

The presence of `memory.grow` indicates that the contract may expand memory dynamically at runtime.
This complicates static reasoning about execution boundaries.

#### Evidence emitted

* `signals.instructions.has_memory_grow`
* `signals.instructions.memory_grow_count`

#### Notes

SEBI does not infer *why* memory growth occurs (e.g., allocator behavior vs user logic).
The rule flags **capability**, not intent.

---

### R-CALL-01 — Dynamic Dispatch via Function Tables

| Field          | Value             |
| -------------- | ----------------- |
| **Rule ID**    | `R-CALL-01`       |
| **Severity**   | HIGH              |
| **Category**   | Control Flow      |
| **Applies to** | Instruction-level |

#### Trigger condition

```
signals.instructions.has_call_indirect == true
```

#### Schema dependencies

* `signals.instructions.has_call_indirect`
* `signals.instructions.call_indirect_count`

#### Rationale

`call_indirect` enables dynamic function dispatch via tables, reducing static call-graph predictability and complicating execution analysis.

#### Evidence emitted

* `signals.instructions.has_call_indirect`
* `signals.instructions.call_indirect_count`

#### Notes

Dynamic dispatch is not inherently unsafe.
This rule highlights **analysis complexity**, not a guaranteed failure.

---

### R-LOOP-01 — Loop Constructs Detected

| Field          | Value             |
| -------------- | ----------------- |
| **Rule ID**    | `R-LOOP-01`       |
| **Severity**   | MED               |
| **Category**   | Control Flow      |
| **Applies to** | Instruction-level |

#### Trigger condition

```
signals.instructions.has_loop == true
```

#### Schema dependencies

* `signals.instructions.has_loop`
* `signals.instructions.loop_count`

#### Rationale

Loop constructs may result in unbounded control flow depending on runtime conditions.
Termination cannot always be proven statically.

#### Evidence emitted

* `signals.instructions.has_loop`
* `signals.instructions.loop_count`

#### Notes

Loops are common and often safe.
This rule flags **potential analysis uncertainty**, not infinite execution.

---

### R-SIZE-01 — Large WASM Artifact

| Field          | Value                        |
| -------------- | ---------------------------- |
| **Rule ID**    | `R-SIZE-01`                  |
| **Severity**   | MED or HIGH (policy-defined) |
| **Category**   | Complexity                   |
| **Applies to** | Artifact-level               |

#### Trigger condition

```
artifact.size_bytes > SIZE_THRESHOLD
```

#### Schema dependencies

* `artifact.size_bytes`

#### Rationale

Larger binaries tend to correlate with increased complexity and reduced static analyzability.
Size is used as a **proxy signal**, not a direct risk indicator.

#### Evidence emitted

* `artifact.size_bytes`
* configured `SIZE_THRESHOLD`

#### Notes

The size threshold is **policy-configurable** and not hard-coded into the rule definition.

---

## 4. Classification Policy

Rules are combined using a **transparent, deterministic policy**.

### Default policy

* If **any HIGH** severity rule is triggered → `HIGH_RISK`
* Else if **two or more MED** severity rules are triggered → `RISK`
* Else if **one MED** severity rule is triggered → `RISK`
* Else → `SAFE`

This policy is the authoritative source for populating the `classification` object defined in `SCHEMA.md`.

---

## 5. Rule Stability and Ordering

* Triggered rules are always sorted by `rule_id`
* Rule identifiers are stable and must never be reused
* Changing a rule’s meaning or severity requires a **catalog version bump**

---

## 6. Extending the Rule Catalog

New rules may be introduced if they:

* rely only on existing or newly defined signals
* include explicit schema dependencies
* include clear rationale and evidence mapping
* preserve determinism and explainability

Breaking changes require:

* new rule identifiers
* explicit documentation
* catalog version updates

---

## 7. Non-Goals of the Rule System

The SEBI rule system does **not** attempt to:

* estimate gas or execution cost
* detect exploits
* infer developer intent
* replace audits or runtime enforcement

Rules highlight **structural execution-boundary risk only**.

---

## 8. Summary

SEBI rules focus on detecting structural patterns that:

* reduce static predictability
* complicate resource bounding
* increase execution-boundary uncertainty

They provide **early, explainable signals** to support informed deployment decisions.