# Tasks

## Contract

- [x] [serial] Define the versioned source-bundle manifest, source record kinds, BLAKE3 record/payload refs, deterministic ordering, store-prefix binding for store-path records, and named limits. r[source_transports.offline_source_bundle_format]
- [x] [serial] Define the language-neutral source adapter contract for package managers and build ecosystems, including lock identity, offline knobs, allowed source roots, cache isolation, generated-source boundaries, and unsupported behavior. r[source_transports.source_adapter_contract]
- [x] [serial] Define source-bundle plan/export semantics for fixed fetcher inputs, local path sources, VCS snapshots, language package-manager mirrors with generic plus adapter-specific metadata, bootstrap archives, provider manifests, toolchain/source-root inputs, and proof inputs. r[source_transports.source_bundle_export_plan]
- [x] [serial] Define filesystem source canonicalization and rejection rules for paths, symlinks, modes, file kinds, hardlinks, devices, timestamps, Unicode, and case collisions. r[source_transports.source_filesystem_canonicalization]
- [x] [serial] Define source-bundle import, list, and verify semantics with idempotent skip, digest verification, unsupported-feature rejection, no network behavior, atomic commit, crash safety, and pin/root policy. r[source_transports.source_bundle_import_verify] r[source_transports.source_bundle_atomic_pinning]
- [x] [serial] Define offline build preflight over planned roots and imported source state, including missing, stale, unsupported, untrusted, and network-required diagnostics. r[source_transports.offline_build_preflight]
- [x] [serial] Define proof-before-claim wording that limits source-bundle evidence to input/source availability and identity. r[source_transports.source_bundle_non_claims]

## Implementation

- [x] [serial] Implement pure source-bundle core types and validators for manifests, source records, adapter metadata, filesystem payload summaries, limits, BLAKE3 refs, deterministic ordering, and compatibility/non-claim classification. r[source_transports.offline_source_bundle_format] r[source_transports.source_adapter_contract] r[source_transports.source_filesystem_canonicalization]
- [x] [serial] Implement the plan/export shell that resolves supported source/input closures and writes deterministic source-bundle streams without executing builds. r[source_transports.source_bundle_export_plan]
- [x] [serial] Implement import/list/verify shells that stream bundles, persist source state atomically, skip already-present records, pin/root imported records when requested, and never fetch network in import/list/verify modes. r[source_transports.source_bundle_import_verify] r[source_transports.source_bundle_atomic_pinning]
- [x] [serial] Implement offline preflight integration for local build planning and remote-builder input preparation. r[source_transports.offline_build_preflight]

## Verification

- [x] [serial] Add pure positive tests for deterministic manifest digesting, source record normalization, adapter contract acceptance, safe filesystem canonicalization, import skip/import planning, atomic pin/root planning, and ready preflight classification. r[source_transports.offline_source_bundle_format] r[source_transports.source_adapter_contract] r[source_transports.source_filesystem_canonicalization] r[source_transports.source_bundle_import_verify] r[source_transports.source_bundle_atomic_pinning] r[source_transports.offline_build_preflight]
- [x] [serial] Add pure negative tests for unsupported versions, duplicate records, oversized lists/chunks, digest mismatch, source-kind mismatch, stale identity, missing source records, adapter contract violations, unsafe filesystem payloads, interrupted import state, unpinned readiness, and overbroad claims. r[source_transports.offline_source_bundle_format] r[source_transports.source_adapter_contract] r[source_transports.source_filesystem_canonicalization] r[source_transports.source_bundle_atomic_pinning] r[source_transports.source_bundle_non_claims]
- [x] [serial] Add fixture tests covering generated fetcher inputs, local path sources, VCS checkout snapshots, at least one non-Cargo package-manager mirror fixture, Cargo as one adapter fixture, bootstrap source archives, and toolchain/source-root inputs. r[source_transports.source_bundle_export_plan] r[source_transports.source_bundle_import_verify]
- [x] [serial] Add CLI tests proving plan is no-mutate, import/list/verify do not use network, and offline preflight fails before build execution when source material is missing or stale. r[source_transports.offline_build_preflight]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks complete. r[source_transports.offline_source_bundle_format]

## Evidence summary

- Core/source-state and initial implementation: `evidence/implementation-slice-2026-06-30.md`, `evidence/build-root-plan-slice-2026-06-30.md`, `evidence/export-materialization-slice-2026-06-30.md`, `evidence/offline-preflight-slice-2026-06-30.md`.
- Hardening, fixtures, imported handoff, VCS, and remote input preparation: `evidence/hardening-cli-slice-2026-07-01.md`, `evidence/fixture-slice-2026-07-01.md`, `evidence/imported-source-handoff-2026-07-01.md`, `evidence/vcs-revision-guard-2026-07-01.md`, `evidence/remote-source-state-handoff-2026-07-01.md`, `evidence/vcs-local-revision-check-2026-07-01.md`.
- Final readiness coverage: `evidence/cli-preflight-failures-2026-07-01.md`, `evidence/adapter-readiness-classes-2026-07-01.md`, `evidence/current-blocker.md`.
