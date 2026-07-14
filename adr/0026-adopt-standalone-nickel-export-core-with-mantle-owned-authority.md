# ADR 0026: Adopt the standalone Nickel export core without transferring Mantle authority

## Status

Accepted

## Context

Mantle's `export` command historically combined embedded Nickel evaluation,
filesystem access, destination writes, deterministic identity, admission, and
`mantle-nickel-export-receipt-v1` construction in one module. The independent
`nickel-export-core` now owns the evaluator-neutral subset at immutable Git
revision `257fafc1c746f1faf156207043a4c826bfb16d49`.

Adopting the shared core reduces receipt drift across Onix projects, but using
its external CLI or giving it ambient filesystem authority would replace
Mantle's embedded evaluator and product boundary. A direct cutover would also
make rollback unsafe because unexplained identity or projection drift could
become authoritative before a complete positive and negative validation cycle.

## Decision Drivers

- Use one immutable Cargo, Nix, lock, and release source identity.
- Keep `crunch-eval` behavior and diagnostics in Mantle.
- Keep root, symlink, source-byte, destination, build, and release authority in
  Mantle-owned shells.
- Give the standalone core only explicit in-memory requests, bytes, evaluator
  descriptors, and diagnostics.
- Compare legacy and canonical exact identities and the existing Mantle v1
  projection before authority changes.
- Fail closed on unexplained drift and retain a bounded rollback adapter.
- Preserve narrow non-claims: export evidence is not evaluator, build,
  deployability, or release proof.

## Decision

Mantle pins `nickel-export-core` to the exact Git revision in `Cargo.toml`,
`Cargo.lock`, `.cargo/vendor-config.toml`, `flake.nix`, `flake.lock`, and the typed release source record
`config/nickel-export-core-source.ncl`. The generated JSON record and a
repo-owned pin checker reject branch, tag, version, path, or different-revision
release inputs. The flake input is non-flake source material and independently
binds the same revision.

`src/nickel_export_core_adapter.rs` is a pure adapter. It converts explicit
Mantle observations into the standalone request, evaluator, diagnostic, and
artifact types; delegates normalization, admission, exact-byte BLAKE3 identity,
manifest freshness, and the one-way Mantle projection; and contains no file,
environment, process, evaluator, network, destination, build, or release I/O.

`src/nickel_export.rs` remains the imperative shell. It converts CLI paths,
rejects symlink components, captures bounded no-follow source bytes, invokes
`crunch_eval::evaluate_to_json`, verifies the declared bytes did not change
during evaluation, controls destination writes, renders diagnostics, and owns
exit status. Build and release policy remain outside both adapter and shared
core.

During migration, Mantle constructs the bounded legacy v1 projection and the
standalone canonical receipt from the same captured bytes and evaluator result.
It compares the core-owned one-receipt canonical manifest identity and exact
`mantle-nickel-export-receipt-v1` projection. Drift is classified as request
normalization, dependency closure, evaluator descriptor, serialization,
Mantle policy, or projection drift. The legacy selector remains available as a
rollback adapter; it cannot authorize a path override, mixed evaluator,
stale/tampered evidence, secret-marker admission, evaluator error receipt, or
weakened non-claim after canonical authority is selected.

## Alternatives Considered

### Invoke the standalone `nickel-export` CLI

Rejected because it would replace the embedded evaluator path, move filesystem
and destination authority outside Mantle, and introduce external process
semantics into a product-owned shell.

### Copy the shared core into the Mantle workspace

Rejected because a path copy is not the immutable published source, would drift
from the standalone release, and would preserve the duplicate implementation
this decision removes.

### Switch immediately without dual-run evidence

Rejected because receipt or path-policy drift could become authoritative without
bounded diagnostics or a tested rollback route.

### Treat equal output bytes as evaluator equivalence

Rejected because equal bytes under one observed request do not prove evaluator
semantics, dependency closure, future determinism, build correctness, or release
eligibility.

## Consequences

- Standalone updates require an explicit revision change and replay of the full
  positive/negative compatibility cycle.
- Mantle carries a small legacy projection adapter for rollback, but shared
  normalization, admission, identity, freshness, and projection semantics are
  delegated to the pinned core.
- Declared-only dependency policy remains an explicit non-claim because
  `crunch-eval` does not expose a complete observed import closure here.
- Pre/post source capture detects changed declared bytes but is not a general
  filesystem snapshot or evaluator-equivalence proof.
- Canonical receipt admission still cannot promote an export into build success,
  deployability, or release eligibility; those gates remain Mantle-owned.
