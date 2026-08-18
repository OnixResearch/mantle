# Upstream Snix review

## Review question

Which current Snix changes repair behavior present in Mantle’s adapted vendor tree without replacing Mantle-owned compatibility boundaries?

## Inspected evidence

- Gerrit project query: <https://cl.snix.dev/q/project:snix>
- Open-change inventory observed on 2026-08-01: 48 open changes, including 21 changes that touched vendored crate paths.
- Merged-change review window: 340 changes returned by the inspected paginated query after 2026-04-01.
- Local source inspection under `vendor/nix-compat`, `vendor/snix-build`, `vendor/snix-castore`, `vendor/snix-store`, `vendor/snix-tracing`, and Mantle callers.
- The vendor tree has no upstream revision marker. This review is a semantic gap analysis, not a complete commit-range proof.

## Selected backports

| Gerrit change | Upstream status observed | Local surface | Disposition | Mantle adaptation |
|---|---|---|---|---|
| [31145](https://cl.snix.dev/c/snix/+/31145) | Merged | `vendor/snix-store/src/pathinfoservice/nix_http.rs`, `crates/crunch-store/src/handle.rs` | Adapt | Reject a returned store-path digest mismatch in the HTTP service and again before Mantle persistence or side effects. |
| [31491](https://cl.snix.dev/c/snix/+/31491) | Merged | `vendor/snix-store/src/pathinfoservice/nix_http.rs`, `crates/crunch-store/src/pull.rs` | Adapt | Enable complete multi-frame zstd decoding in both paths and reject malformed later frames without partial admission. |
| [31492](https://cl.snix.dev/c/snix/+/31492) | Merged | `vendor/snix-store/src/pathinfoservice/nix_http.rs` | Adapt | Normalize cache bases as directory URLs while preserving Mantle credential and query handling. |
| [31478](https://cl.snix.dev/c/snix/+/31478) | Open | `vendor/snix-castore/src/nodes/directory.rs` | Adapt | Independently prove the duplicate size contribution and port the bounded pure repair with local tests. Do not claim upstream acceptance. |
| [31496](https://cl.snix.dev/c/snix/+/31496) | Open | `vendor/snix-castore/src/fs/mod.rs` | Adapt | Use FUSE `DT_*` values for directory entries and add local node-kind tests. Do not claim upstream acceptance. |
| [31495](https://cl.snix.dev/c/snix/+/31495) | Merged | `vendor/snix-castore/src/fs/mod.rs` | Adapt | Set the adopted valid nonzero link count and test every supported node kind. |
| [31306](https://cl.snix.dev/c/snix/+/31306) | Merged | `vendor/snix-castore/src/directoryservice/redb.rs` | Adapt | Create and use the write transaction inside `spawn_blocking` with owned database state. |
| [30571](https://cl.snix.dev/c/snix/+/30571) | Open | `vendor/snix-store/src/pathinfoservice/cache.rs` | Adapt | List the writable near service only. This keeps signing and mutation scope local. Do not claim upstream acceptance. |
| [31157](https://cl.snix.dev/c/snix/+/31157) | Merged | `vendor/snix-castore/src/import/fs.rs` | Adapt | Use `copy_buf` with the maintained default bounded reader capacity instead of adding an unevidenced fixed oversized buffer. |
| [31150](https://cl.snix.dev/c/snix/+/31150) | Merged | `vendor/snix-tracing/src/lib.rs` | Adapt | Apply the environment filter to Mantle's combined layers while preserving progress and additional-layer behavior. |

## Deferred or rejected changes

| Gerrit change | Upstream status observed | Disposition | Reason and reopen trigger |
|---|---|---|---|
| [31448](https://cl.snix.dev/c/snix/+/31448) | Merged | Deferred, rechecked 2026-08-08 | No non-vendor manifest enables the affected virtiofs route, and no local used-length call is present. Reopen when that backend becomes a supported runtime route or local inspection finds an active caller. |
| [31272](https://cl.snix.dev/c/snix/+/31272) | Merged | Deferred, rechecked 2026-08-08 | Current read-only redb opens use `redb::Database::builder().open_read_only(path)` without non-default builder configuration. Reopen when Mantle configures that path or tests expose a difference. |
| [30386](https://cl.snix.dev/c/snix/+/30386) | Open | Deferred | The defensive wire-reader assertion is useful but is outside the selected correctness paths. Reopen with a focused daemon or wire-reader maintenance change. |
| [31487](https://cl.snix.dev/c/snix/+/31487) | Open | Deferred | The concurrent blob uploader refactor had unresolved review discussion. Reopen after merge and after a local bounded-concurrency design review. |
| [30598](https://cl.snix.dev/c/snix/+/30598) | Open | Deferred | Mantle-owned code does not use the affected OCI build backend. Reopen when that backend gains a supported Mantle caller. |
| [30982](https://cl.snix.dev/c/snix/+/30982) | Merged | Rejected | The change depends on upstream protobuf directory serialization. Mantle intentionally uses postcard directory identity and incompatible persisted data. |
| 31472, 31512, 31476, and 31477 | Open | Deferred | These derivation-builder, output-model, and store-trait refactors change broad interfaces. Reopen only through a separate architecture change. |

## Decision

Implement the ten selected backports as local behavior adaptations. Keep the other changes deferred or rejected under ADR 0054.

## Owner

Mantle store and vendored-runtime maintainers own implementation, local tests, and final disposition updates.

## Next action

Run the V1 baseline commands before changing vendor or first-party runtime code. Then implement the remote request-identity guard first.

## Claim boundary

This review identifies selected semantic gaps. It does not prove complete upstream parity, upstream acceptance of open changes, whole-store correctness, or release eligibility.
