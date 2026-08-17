## Why

Mantle examples should teach the product in a progressive path: smallest derivation, local build, package composition, project outputs, fetchers, trust/provenance, and advanced bootstrap. The current gallery has many useful pieces, but it does not yet present a complete learning path or enough provenance/trust examples for operators who need proof-before-claim workflows.

## What Changes

- Expand the examples gallery with a beginner-to-advanced path and clear capability labels.
- Add examples that demonstrate consuming multi-output packages, project checks, and built-output execution.
- Add lightweight trust/provenance examples for artifact attestations or store-side evidence where the commands can stay deterministic and local.
- Keep advanced/bootstrap examples explicit about what they prove and what they do not prove.

## Impact

- **Files**: `examples/`, `examples/project/`, README sections, tests for example evaluation/build/output, and optional docs under `docs/`.
- **Testing**: examples inventory/eval/build rails, provenance command smoke tests if added, `cairn validate --root .`, and task-gate evidence.
