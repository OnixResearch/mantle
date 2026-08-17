# Audit: External Pin Import Surface

Audited against `r[project_workflows.project_lock_importers]` and
`r[project_workflows.nixtamal_importer]` requirement clauses, and the change's
own `r[project_workflows.lock_importer_proof_rail]` requirement.

## Source Files Audited

- `crates/crunch-project-core/src/importer.rs` (400 lines)
- `crates/crunch-project-core/src/lib.rs` (importer exports)
- `crates/crunch-project/src/lib.rs` (importer re-exports)
- `src/pin_import.rs` (358 lines)
- `src/cargo_import.rs` (300 lines)
- `src/main.rs` (import CLI dispatch)
- `tests/pin_import_cli.rs` (94 lines)
- `tests/cargo_import_cli.rs` (300 lines)

## Core Import Implementation

### Types
- ✅ `ExternalPin` with `name`, `kind` (`ExternalPinKind`), `url`, `repository`,
  `reference`, `rev`, `hash_algo`, `hash`, `frozen`, `mirrors`, `patches`,
  `freshness`, `fetch_policy`, `trust_policy`.
- ✅ `ExternalPinKind`: `File`, `Tarball`, `Git`, `Darcs`, `Pijul`, `Fossil`.
- ✅ `ExternalPatch`, `ExternalPatchSource`.
- ✅ `ExternalExistingFile`, `ExternalHash` for plan drift detection.
- ✅ `PinImportPlan`, `PinImportFileOperation`, `PinImportMappedInput`,
  `PinImportMappedPatch`, `PinImportBlocker`.
- ✅ `PinImportSemantic` for composition semantics.
- ✅ `PIN_IMPORT_PLAN_SCHEMA` constant.
- ✅ `build_pin_import_plan()` — pure core planning function.

### Supported Importer: Nixtamal
- ✅ `PIN_IMPORT_SUPPORTED_IMPORTER = "nixtamal"`.
- ✅ `PIN_IMPORT_DEFAULT_PROJECT_FILE`, `PIN_IMPORT_DEFAULT_LOCK_FILE`,
  `PIN_IMPORT_DEFAULT_INPUTS_FILE`.
- ✅ JSON deserialization in `src/pin_import.rs` for Nixtamal fixture format.

### Plan/Apply Boundary
- ✅ `PinImportOptions` with `plan`/`apply` mode.
- ✅ `build_pin_import_plan()` produces deterministic plan without I/O.
- ✅ Plan reports: file operations, mapped inputs, mapped patches, blockers.
- ✅ Plan is reviewable and side-effect free (pure core).

## Shell Implementation

### `pin_import.rs` CLI
- ✅ Reads Nixtamal fixture file, deserializes, calls core `build_pin_import_plan()`.
- ✅ Renders plan to stdout (JSON or text).
- ✅ Apply writes: `mantle-project.ncl`, `mantle.lock`, `.mantle/inputs.ncl`.
- ✅ Handles unsupported semantics as blockers.
- ✅ `PIN_IMPORTER_NIXTAMAL`, `PIN_IMPORTER_FLAKE`, `PIN_IMPORTER_NPINS`, `PIN_IMPORTER_NIV`.

### `cargo_import.rs`
- ✅ Cargo workspace import scaffold (separate from pin import).
- ✅ Offline vendor detection.
- ✅ Tests in `tests/cargo_import_cli.rs`.

### CLI Dispatch
- ✅ `mantle import pins` command tree in `src/main.rs`.
- ✅ Plan → Apply workflow with `--apply` flag.

## Coverage Against Requirement

### Plan Is Side-Effect Free
- ✅ Core `build_pin_import_plan()` is pure: no I/O, no mutation.
- ✅ Shell `pin_import.rs` calls plan first, renders, exits before apply.
- ✅ Test: `pin_import_cli_plan_no_mutate` asserts no files written.

### Apply Writes Only Planned Files
- ✅ `PinImportFileOperation::Create` / `Update` / `Unchanged` / `Stale` / `Conflict`.
- ✅ Apply logic in `pin_import.rs` limits writes to `mantle-project.ncl`,
  `mantle.lock`, `.mantle/inputs.ncl`.
- ✅ Test: `pin_import_cli_apply_creates_project_files` verifies written files.
- ✅ Test: `pin_import_cli_apply_fails_on_missing_fixture` checks error handling.

### Composition Semantics Are Blockers
- ✅ `PinImportSemantic` enum with `RecursiveGraph`, `FlakeOutputComposition`,
  `ModuleLayer`, `FollowsRewriting`, `Overlay`.
- ✅ `build_pin_import_plan()` converts detected composition semantics to blockers.
- ✅ `composition_semantics` field on `NixtamalInput` in test fixture.
- ⚠️ Test: `pin_import_cli_unsupported_input_reports_blocker_and_apply_fails`
  covers unsupported `darcs` kind + `composition_semantics`.

### Unsupported Nixtamal Semantics Block Apply
- ✅ `NixtamalFixture` JSON fixture has `unsupported_semantics` field.
- ✅ Blocker output produced.
- ⚠️ No test for each blocker type individually.

### Evidence Record
- ⚠️ No versioned evidence record for import plan/apply.
- ✅ Plan schema constant `PIN_IMPORT_PLAN_SCHEMA` for stable schema reference.
- ✅ Non-claim: import is not build success, deployability, or frontend module
  correctness — stated in docs but not rendered as evidence.

## Gaps for Proof Rail

| Feature | Code | Test |
|---|---|---|
| Plan side-effect free | ✅ | ✅ |
| Apply writes planned files | ✅ | ✅ |
| Unsupported semantics blocker | ✅ | ✅ |
| Composition semantics blocker | ✅ | ⚠️ partial |
| Plan drift rejection | ✅ | ❌ |
| Apply conflict rejection | ✅ | ❌ |
| Versioned evidence record | ❌ | ❌ |
| Non-claim assertion in evidence | ❌ | ❌ |
| Build-tool handoff assertion | ❌ | ❌ |
| Composed plan→apply proof rail | ✅ partial | ❌ |

**Gaps to address:**
1. Composed offline proof rail that exercises full plan→apply flow with evidence
2. Versioned, redacted evidence record with non-claims
3. Negative cases: plan drift, apply conflict, individual composition semantic
   blockers, unsupported Nixtamal semantics
4. Build-tool handoff non-claim assertion in evidence