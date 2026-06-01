# Tracey baseline and first backfill batch

Task-ID: baseline-first-batch
Covers: verification_evidence.tracey_coverage_readiness

## Baseline command

```sh
/nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn tracey coverage --root . --json > cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-baseline-2026-06-01.json
```

The command exited with status `1` because coverage was red. This is expected for the baseline.

Baseline artifacts:

- Full JSON: `tracey-coverage-baseline-2026-06-01.json`
- Grouped missing IDs: `tracey-coverage-baseline-grouped-2026-06-01.json`

Baseline summary:

```json
{
  "valid": false,
  "total_requirements": 202,
  "referenced_count": 1,
  "missing_count": 201,
  "dangling_count": 0,
  "missing_by_group": [
    { "group": "build_tool_boundary", "count": 3 },
    { "group": "compiled-eval", "count": 21 },
    { "group": "rust_package_planning", "count": 176 },
    { "group": "verification_evidence", "count": 1 }
  ]
}
```

## Classification

| Requirement group | Count | Classification | Owner | Next action |
| --- | ---: | --- | --- | --- |
| `build_tool_boundary` | 3 | evidence-backed bridge-marker | Mantle | Completed first batch by adding bounded bridge refs to `tools/tracey_refs.rs`; root implementation/tests live under `src/`, `tests/`, ADR `adr/0010-keep-mantle-build-tool-boundary.md`, and archived Cairn evidence. |
| `compiled-eval` | 21 | legacy scanner-gap plus evidence-led bridge-marker candidate | Mantle | Inspect `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/` and either migrate/bridge only the archived prototype claims, or leave explicit debt for unsupported future semantics. |
| `rust_package_planning` | 176 | mixed direct-marker, scanner-gap, and bridge-marker debt | Mantle | Backfill in small archived-change batches; prefer direct refs where scanned files exist and bridge refs only when root `src/`/`tests/` evidence is durable. |
| `verification_evidence.tracey_coverage_readiness` | 1 | active-change self-coverage debt | Mantle | Keep missing until this change has durable validation evidence; do not self-mark coverage readiness as implemented before the rail is actually maintained. |

## First batch change

Added bounded bridge refs in `tools/tracey_refs.rs` for:

- `build_tool_boundary.mantle_not_module_layer`
- `build_tool_boundary.onix_owns_module_lowering`
- `build_tool_boundary.synthetic_system_eval_not_integration`

The bridge text explicitly limits the claim to Mantle-side boundary traceability. It does **not** claim Onix module lowering is implemented inside Mantle and does **not** claim global Tracey coverage is green.

Existing source-built toolchain closure refs were preserved in `tools/tracey_refs.rs` for `rust_package_planning.source_built_toolchain_closure`; that requirement was already outside the baseline missing list.

## After first batch coverage

```sh
/nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn tracey coverage --root . --json > cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-2026-06-01.json
```

The command exited with status `1` because global coverage remains red. This is expected after a bounded first batch.

After-batch artifacts:

- Full JSON: `tracey-coverage-after-first-batch-2026-06-01.json`
- Grouped remaining missing IDs: `tracey-coverage-after-first-batch-grouped-2026-06-01.json`

After-batch summary:

```json
{
  "valid": false,
  "total_requirements": 202,
  "referenced_count": 4,
  "missing_count": 198,
  "dangling_count": 0,
  "missing_by_group": [
    { "group": "compiled-eval", "count": 21 },
    { "group": "rust_package_planning", "count": 176 },
    { "group": "verification_evidence", "count": 1 }
  ]
}
```

Decision: first batch reduced missing requirements from `201` to `198`; remaining missing IDs are explicitly recorded in grouped JSON and are not claimed complete.
