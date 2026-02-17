# SEBI Report Schema

**Version:** `0.1.0`

This document defines the **official report schema** produced by **SEBI (Stylus Execution Boundary Inspector)**.

The schema is a **stable contract** between SEBI and its users (CI systems, auditors, developers).
All SEBI outputs **must conform** to this specification.

---

## 1. Design Principles

### 1.1 Separation of concerns

SEBI strictly separates:

* **Signals** → raw, objective observations derived from a WASM artifact
* **Rules** → policy interpretations applied to signals
* **Classification** → final verdict derived from triggered rules

Parsing, interpretation, and judgment must never be mixed.

---

### 1.2 Determinism

Given the same input artifact, SEBI **must produce identical JSON output**.

To guarantee this:

* All arrays are **sorted deterministically**
* No timestamps or nondeterministic values are included
* Field meanings never change without a schema version bump

---

### 1.3 Versioning policy

* `schema_version` follows **Semantic Versioning**
* **MAJOR**: breaking changes (field removal, renaming, or semantic change)
* **MINOR**: additive, backward-compatible changes
* **PATCH**: non-semantic or documentation-only changes

Once published, a schema version must **never silently change meaning**.

---

## 2. Relationship to Rule Catalog

This document defines the **report structure and signal vocabulary** used by SEBI.

Interpretation of signals into execution-boundary risk assessments is defined separately in
[`RULES.md`](./RULES.md).

### Separation of responsibilities

* `SCHEMA.md` defines:

  * the shape of SEBI reports
  * the meaning and types of signals
  * determinism and versioning guarantees

* `RULES.md` defines:

  * how signals are interpreted
  * which conditions trigger rules
  * how rules combine into a final classification

Signals defined in this schema **must not encode policy or judgment**.
All interpretation logic belongs exclusively to the rule catalog.

---

## 3. Top-Level Structure

A SEBI report contains exactly the following top-level fields:

```text
schema_version
tool
artifact
signals
analysis
rules
classification
```

Each field is mandatory unless explicitly marked optional.

---

## 4. Field Specifications

---

### 4.1 `schema_version` (string)

The version of this report schema.

Example:

```json
"schema_version": "0.1.0"
```

---

### 4.2 `tool` (object)

Identifies the producer of the report.

| Field     | Type   | Required | Description                  |
| --------- | ------ | -------- | ---------------------------- |
| `name`    | string | yes      | Tool name (e.g. `"sebi-cli"`) |
| `version` | string | yes      | SEBI tool version            |
| `commit`  | string | no       | Git commit hash of the build |

**Note:** No timestamps are included to preserve determinism.

---

### 4.3 `artifact` (object)

Identifies the analyzed WASM artifact.

| Field        | Type    | Required | Description                          |
| ------------ | ------- | -------- | ------------------------------------ |
| `path`       | string  | no       | Path to the artifact (informational) |
| `size_bytes` | integer | yes      | File size in bytes                   |
| `hash`       | object  | yes      | Cryptographic file hash              |

#### `artifact.hash`

| Field       | Type   | Required | Description                      |
| ----------- | ------ | -------- | -------------------------------- |
| `algorithm` | string | yes      | Hash algorithm (e.g. `"sha256"`) |
| `value`     | string | yes      | Hex-encoded hash value           |

The hash uniquely binds the report to the **exact artifact analyzed**.

---

## 5. Signals

Signals are **raw factual observations** derived directly from the WASM binary.

They:

* contain **no interpretation**
* encode **no severity**
* do **not** imply risk on their own

Signals are consumed by rules defined in `RULES.md`.

---

### 5.1 `signals.module`

| Field            | Type    | Description                         |
| ---------------- | ------- | ----------------------------------- |
| `function_count` | integer | Number of defined functions         |
| `section_count`  | integer | Total number of sections (optional) |

---

### 5.2 `signals.memory`

| Field          | Type           | Description                             |
| -------------- | -------------- | --------------------------------------- |
| `memory_count` | integer        | Number of memories declared or imported |
| `min_pages`    | integer | null | Minimum memory pages                    |
| `max_pages`    | integer | null | Maximum memory pages                    |
| `has_max`      | boolean        | Whether a maximum is declared           |

---

### 5.3 `signals.imports_exports`

| Field          | Type    | Description                   |
| -------------- | ------- | ----------------------------- |
| `import_count` | integer | Total number of imports       |
| `export_count` | integer | Total number of exports       |
| `imports`      | array   | Optional detailed import list |
| `exports`      | array   | Optional detailed export list |

#### Import item

| Field    | Type                                                 |
| -------- | ---------------------------------------------------- |
| `module` | string                                               |
| `name`   | string                                               |
| `kind`   | `"func" \| "memory" \| "table" \| "global" \| "tag"` |

#### Export item

| Field  | Type                 |
| ------ | -------------------- |
| `name` | string               |
| `kind` | same enum as imports |

**Ordering rule:**

* imports sorted by `(module, name, kind)`
* exports sorted by `(name, kind)`

---

### 5.4 `signals.instructions`

| Field                 | Type    | Description                 |
| --------------------- | ------- | --------------------------- |
| `has_memory_grow`     | boolean | Presence of `memory.grow`   |
| `memory_grow_count`   | integer | Number of occurrences       |
| `has_call_indirect`   | boolean | Presence of `call_indirect` |
| `call_indirect_count` | integer | Number of occurrences       |
| `has_loop`            | boolean | Presence of `loop`          |
| `loop_count`          | integer | Number of loop instructions |

---

## 6. Analysis

Runtime and parsing status information.

| Field      | Type   | Description                                           |
| ---------- | ------ | ----------------------------------------------------- |
| `status`   | string | `"ok" \| "parse_error" \| "unsupported"` |
| `warnings` | array  | Sorted list of warning messages                       |

This section provides **diagnostic context only** and must not affect rule evaluation.

---

## 7. Rules

---

### 7.1 `rules.catalog`

Identifies the rule catalog used.

| Field             | Type   | Description          |
| ----------------- | ------ | -------------------- |
| `catalog_version` | string | Rule catalog version |
| `ruleset`         | string | Rule set identifier  |

---

### 7.2 `rules.triggered`

A list of triggered rules.

Each item contains:

| Field      | Type   | Description                |
| ---------- | ------ | -------------------------- |
| `rule_id`  | string | Unique rule identifier     |
| `severity` | string | `"Low" \| "Med" \| "High"` |
| `title`    | string | Short rule name            |
| `message`  | string | Human-readable explanation |
| `evidence` | object | Key-value evidence         |

**Ordering rule:** sorted by `rule_id`.

---

## 8. Classification

Final verdict derived from triggered rules.

| Field                | Type    | Description                          |
| -------------------- | ------- | ------------------------------------ |
| `level`              | string  | `"SAFE" \| "RISK" \| "HIGH_RISK"`    |
| `policy`             | string  | Classification policy identifier     |
| `reason`             | string  | Summary explanation                  |
| `highest_severity`   | string  | `"NONE" \| "Low" \| "Med" \| "High"` |
| `triggered_rule_ids` | array   | Sorted list of rule IDs              |
| `exit_code`          | integer | CI exit code (`0`, `1`, `2`)         |

The logic used to populate this object is defined in `RULES.md`.

---

## 9. Determinism Guarantees

SEBI guarantees that:

* Identical artifacts produce identical reports
* No timestamps are included in JSON output
* All arrays are sorted deterministically
* Rule evaluation does not depend on signal discovery order
* Field semantics do not change within a schema version

---

## 10. Non-Goals

The SEBI report schema does **not** attempt to:

* execute or simulate WASM
* estimate gas or runtime cost
* prove correctness or safety
* infer developer intent

SEBI reports **structural execution-boundary signals only**.

---

## 11. Schema Evolution

Future schema versions may:

* add new optional signals
* extend existing signal groups
* add new rule metadata fields

Breaking changes require a **major version bump**.
