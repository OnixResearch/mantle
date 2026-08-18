## Why

Mantle now rejects signed PathInfo records whose recorded final NAR size or SHA-256 differs from a fresh render, but retained records created before the CA/final-NAR identity repair have no explicit recovery path. Ordinary `store sign` cannot repair them because it signs the stale facts, and archive export correctly fails before emitting bytes. Operators need a narrow, reviewable migration that remeasures already-present immutable castore content without changing store-path, node, references, CA path identity, deriver, or output bytes.

## What Changes

- Add `mantle store repair-final-nar <logical-store-path>` as a dry-run-by-default command with explicit `--execute` and optional `--signing-key`.
- Require one exact full logical store path, complete local castore content, valid CA-derived path identity when CA metadata is present, and an already-signed stale PathInfo before mutation.
- Recompute final NAR size/SHA-256, discard signatures bound to stale facts, create one replacement signature, preserve all other PathInfo fields, and refresh an existing artifact attestation without discarding its claims or graph.
- Emit a structured report that binds old and observed facts, signature counts, sidecar disposition, execution mode, and final status; current records are deterministic no-ops.
- Keep ordinary signing, archive import/export, rebuild behavior, CA identity, content, and closure semantics unchanged.

## Impact

- **Files**: `crates/crunch-store/src/{archive,attestation,lib,path_identity,repair}.rs`, `src/{main,store_cmd}.rs`, focused CLI tests, operator documentation, and `store-transports` lifecycle artifacts.
- **Testing**: pure positive/negative repair plans; store-level dry-run, execute, current, missing/incomplete content, unsigned stale, invalid CA, stale sidecar, and failed persistence paths; CLI dry-run/execute JSON and human reports; retained Bison migration followed by archive export when practical; first-party quality, Cairn, Tracey, and Nix gates.
