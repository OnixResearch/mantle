## Why

Mantle now has several offline-build pieces: ordinary derivations deny network by default, `--no-substitute` disables cache substitution, `--offline-source-preflight` checks imported source state, `mantle source bundle` can export/import/pin sources, and `mantle.offlineCargoPackage` runs Cargo with isolated offline inputs. These pieces are not presented as one clear operator workflow, and key commands are missing from the quick reference.

Operators need a concise, proof-before-claim runbook that says exactly how to prepare offline source material, import and pin it, preflight a selected root, run the build without substitutes, and read the resulting evidence. Diagnostics should also point from common blockers to the next command without implying that source readiness is build success.

## What Changes

- Add a documented offline-build runbook covering source-bundle export, transfer/copy, import with pinning, offline source preflight, `--no-substitute`, build/run, and evidence inspection.
- Surface `mantle source bundle` and `--offline-source-preflight` in the root README command reference and operator workflow docs.
- Add stable human/JSON diagnostics or hints for missing source state, unpinned source records, network-required inputs, unsupported adapters, and malformed offline Cargo evidence.
- Keep every runbook claim bounded to the command evidence produced.
- Add tests that reject stale docs, missing commands, and overclaiming language.

## Impact

- **Files**: `README.md`, `docs/operator-workflows.md`, `docs/operator-proof-guide.md`, `examples/README.md`, `src/source_bundle.rs`, `src/main.rs` help text, docs drift tests, and this Cairn spec delta.
- **Testing**: docs/runbook drift checks, CLI help assertions, source-preflight diagnostic tests, JSON output shape tests, and claim-boundary text checks.

## Out of Scope

- Implementing new source-bundle execution semantics; that is covered by `source-bundle-realization-route`.
- Proving any specific build succeeds merely from running the runbook.
- Adding a hosted cache, remote build farm, or package-manager-specific vendoring command.
