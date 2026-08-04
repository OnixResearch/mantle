# Store decision-core baseline and ownership inventory

Date: 2026-08-03
Baseline revision: `cb27a0442047a78c0770acb35cad6b3daf0ec146`

## Baseline validation

The pre-change focused tests passed in the detached cached development shell:

- `cargo test -p crunch-store gc::`: 13 passed.
- `cargo test -p crunch-store repair::`: 11 passed.
- Log: `target/store-cores-baseline.log`.

The baseline output and mutation contracts are:

- GC reports retain the existing root, candidate, byte, sidecar, blob, action-result, and operation fields.
- GC candidates use canonical store-path order.
- Dry-run and execution use the same candidate set.
- GC execution order remains exported outputs, PathInfo, artifact sidecars, closure sidecars, directories, blob indexes, blob chunks, action results, then CA mappings.
- Final-NAR repair reports keep `mantle-pathinfo-final-nar-repair-v1`.
- Final-NAR facts keep Nix-compatible SHA-256 bytes and hexadecimal report text.
- Current repair facts remain a no-op.
- Repairable facts replace stale signatures with one configured local signature.
- An existing artifact sidecar remains current or refreshes with the repaired NAR digest. An absent sidecar remains absent.
- Failed admission, persistence, or verification does not produce a success report.

## GC observation and authority inventory

The std shell owns these observations before it calls the pure GC core:

| Source | Observation | Authority or non-authority |
|---|---|---|
| Durable root registry | Canonical logical paths with `build`, `self-build`, `bootstrap`, or `pin` source | These records are path-retention authority. The core does not create them. |
| `PathInfoService` snapshot | Store path, references, and declared NAR size | These facts define normalized path reachability. Missing roots or references fail closed. |
| Project and closure builds | Successful selected outputs registered as `build` roots | The root record grants retention. Project metadata or a closure sidecar alone does not grant retention. |
| Artifact and closure sidecars | Existing sidecar files related to retained or dead paths | Sidecars follow path retention. Sidecar presence does not make a path live. |
| Local action-result metadata | Records and index markers selected against the live path set | Action-result presence does not make output content live or trusted. |
| Rust unit action results | Signed retention planning supplies live castore nodes separately | The Rust retention plan protects admitted result nodes. Raw CAS presence does not grant retention. |
| Explicit castore roots | Bounded `Node` values supplied by the caller | These protect castore content only. They do not create PathInfo roots. |
| Filesystem and database scans | Existing exports, sidecars, blob indexes, chunks, and directories | These observations support sweep and byte reporting. Presence does not grant retention. |
| Active Cairn changes | No direct GC input | An active change package does not grant store-retention authority. |

`crunch-gc-core` receives only bounded owned path IDs, ordered references,
declared byte counts, roots, and an execution mode. It owns normalization,
reachability, live/dead classification, checked reclaim summaries, ordered path
mutation intents, dry-run disposition, and BLAKE3 plan identity.

`crunch-store` retains locks, service reads, scans, castore traversal, physical
size observation, deletion, database replacement, cleanup, action-result
retention, Rust result retention, CA mapping persistence, and CLI output.

## Repair observation and authority inventory

The std shell owns these observations before it calls the pure repair core:

| Observation | Owner and rule |
|---|---|
| Exact selected store path and current PathInfo | `crunch-store` parses and loads one exact canonical path. |
| Recorded final-NAR size and SHA-256 | Current PathInfo supplies the recorded facts. |
| Observed final-NAR size and SHA-256 | The std NAR renderer calculates these facts from local castore content. |
| Content completeness | `crunch-store` queries castore services and fails closed on missing content. |
| CA path identity | `crunch-store` validates selected path, CA metadata, node, and references. |
| Existing signatures | The shell supplies a bounded count. Signature bytes and key access never enter the core. |
| Artifact sidecar validity | The shell loads and validates the optional sidecar against stale PathInfo facts. |
| Signing key and signer name | Execution-only std authority. The core returns only a signing disposition. |
| Persistence and verification results | Execution-only std observations used for failure and rollback handling. |

`crunch-repair-core` owns current/repair/rejection admission, target-bound BLAKE3
plan identity, signature and sidecar dispositions, ordered mutation and rollback
intents, and fail-closed report classification.

`crunch-store` retains NAR rendering, SHA-256 calculation, key access, signing,
staging, PathInfo persistence, sidecar publication, cleanup, rollback execution,
post-write verification, and report serialization.

## Claim boundary

A core plan records a deterministic decision over supplied facts. It does not
prove that observations are true or that effects started or completed. It does
not prove content correctness, signer authority, provenance, reproducibility,
release eligibility, source trust, or global cache availability.
