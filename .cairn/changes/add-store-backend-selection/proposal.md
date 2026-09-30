# Proposal: Select the store backend explicitly

## Why

Mantle has one persistent store engine, and the choice is implicit.
`StoreHandle::open` always opens Snix services under `--state-dir`: `blobs/`,
`directories.redb`, and `pathinfo.redb`
(`crates/crunch-store/src/handle.rs`, `open_blob_service`,
`open_directory_service`, `open_pathinfo_service`). GC execution rewrites those
two databases and sweeps `blobs/` (`crates/crunch-store/src/gc.rs`).

The state identity record `store-identity.json` (`mantle-store-state-v1`,
`crates/crunch-store/src/overlay.rs`) binds the logical prefix and trust policy,
but not the storage engine. A second engine could open the same directory
without detecting the mismatch.

`adopt-casita-store-backend` adds Casita as a durable backend. Without a
selection seam and a recorded backend, that change would either replace Snix
silently or let two engines share one state directory. A new backend may also
lack optional behavior that Snix has today, such as overlay composition,
atomic batch import without a backend limit, unsigned admission, or the
standalone Rust unit cache, and that gap must be explicit.

Store construction is spread across the CLI and libraries. `StoreConfig` and
`StoreHandle::open` are built in about 30 files under `src/`, `crates/`, and
`tests/`. Five launchers forward `--state-dir` or `--store-prefix` to child
processes (`src/main.rs` local remote worker, `src/bootstrap_validate.rs`,
`src/source_built_fixed_point_shell.rs`, `src/transcript_cmd.rs`,
`crates/crunch-rustc-wrapper/src/bin/mantle-rust-cache-daemon.rs`). A child
that does not receive the backend could reopen the state under another engine.

## What Changes

- Add a Mantle-owned backend identifier and one global `--store-backend <id>`
  option. The CLI composition root defaults to `snix`. `StoreConfig` requires
  the identifier, and no library constructor supplies a default.
  r[mantle.store_backends.explicit_selection]
- Forward the selected identifier to every child process that opens the same
  state. Reject an unknown identifier before any state access.
  r[mantle.store_backends.explicit_selection]
- Record the backend in a versioned state identity record for new state
  directories. Read a legacy `mantle-store-state-v1` record as `snix` without
  rewriting it. r[mantle.store_backends.state_identity]
- Reject a state directory whose recorded backend differs from the selection,
  or one without an identity record that holds a `casita` repository marker.
  Under `snix`, other state without an identity record is legacy Snix state:
  Mantle adds a `snix` identity and changes no existing file before the Snix
  services open. Under `casita`, such a directory is rejected when it holds
  anything besides the lock, the signing key, and trust-policy files. The check
  runs before any storage service opens or any file changes. Every overlay
  layer must record the same backend.
  r[mantle.store_backends.mixed_open_rejection]
- Never read another local backend's state after a miss or failure. Configured
  remote substitution keeps its existing policy.
  r[mantle.store_backends.no_silent_fallback]
- State the backend-neutral admission invariants: store paths, NAR facts,
  trusted-key admission, Mantle identities, and store capability views. For
  the same signing key and deterministic fixtures, signatures are equal across
  backends; with different keys, only unsigned fields are compared.
  r[mantle.store_backends.admission_invariants]
- Add a Mantle-owned capability profile per backend. Every backend implements
  the core capabilities, including in-place PathInfo update and PathInfo-backed
  `ActionResultPort` outputs. Optional capabilities (overlay composition, atomic
  batch import with any bound the backend imposes, unsigned admission, and the
  standalone Rust unit cache `rust-unit-cache`) are declared per backend. An
  undeclared one fails closed before any state access, and a request beyond a
  declared bound fails before any state mutation. `snix` declares all four and
  imposes no batch bound.
  r[mantle.store_backends.capability_profile]
- Add a backend-parameterized conformance rail with positive and negative
  fixtures, driven by each backend's profile and run with an explicitly
  provisioned fixture signing key. Run it on `snix` against pre-change goldens.
  r[mantle.store_backends.conformance_rail]
- Regenerate the operator command contract for the new global option and
  document selection, identity, profiles, mismatch remediation, and
  non-claims. r[mantle.store_backends.claim_boundary]

## Impact

- **Immediate consumer**: every store-opening Mantle command (`build`,
  `store *`, `attest`, `foreign-import`, remote build and worker, Rust cache
  daemon) gains an explicit, recorded backend. `adopt-casita-store-backend`
  admits `casita` through this seam, declares its profile, and runs the same
  conformance rail.
- **Immediate outcome**: a state directory cannot be opened by the wrong
  engine. Existing Snix state keeps working without migration. Operators can
  see which optional capabilities the selected backend supports.
- **Durable capability**: one selection seam, one identity check, one
  capability profile, and one conformance rail for every future backend.
- **Maintenance owner**: Mantle store lifecycle owner (`crunch-store`), with the
  CLI composition-root owner for option plumbing.
- **Repeatability evidence**: unknown-identifier, recorded-mismatch,
  foreign-state, mixed-overlay, undeclared-capability, child-forwarding, and
  legacy-identity fixtures; pre- and post-change goldens for store paths, NAR
  SHA-256, signed PathInfo, and GC plan identities on `snix`, recorded with an
  explicitly provisioned fixture signing key and environment.
- **Compatibility**: the `snix` default keeps on-disk formats, derivation
  hashes, output paths, signatures, action refs, GC plan identities, report
  schemas, overlay composition, atomic batch import, unsigned admission, and
  the Rust unit cache. The operator command
  contract artifacts regenerate for the new global option.

## Scope

The change covers the identifier type, the global option, the `StoreConfig`
cutover at every construction site, child-process forwarding, the identity
record, mismatch and foreign-state rejection, the overlay layer check, the
no-fallback rule, the capability profile, the conformance rail, operator
surfaces, documentation, and an ADR.

## Non-Goals

- Implementing Casita or any non-Snix backend. `adopt-casita-store-backend`
  owns that work.
- Moving content between backends.
- Changing Snix on-disk formats, GC decision rules, or retention records.
  `add-retention-interest-records` owns retention records.
- Changing remote cache, substituter, action-result discovery, or remote
  builder protocols.
- Inferring a backend from directory contents, environment variables, or build
  features.
- Describing backends as interchangeable beyond their declared capability
  profiles.

## Success Criteria

- With the fixture signing key and environment recorded with the pre-change
  goldens, commands without `--store-backend` and with `--store-backend snix`
  reproduce the recorded store paths, NAR SHA-256 values, signed PathInfo, and
  GC plan identities.
- State directories with different signing keys agree on every unsigned field,
  and each signature verifies under its own key.
- A state directory recorded for another backend fails with
  `store-backend-mismatch` and stays byte-identical.
- An unknown identifier fails with `store-backend-unknown` before any state
  access.
- Every child process launched against the same state receives the selected
  identifier.
- `snix` declares overlay composition, atomic batch import with no backend
  bound, unsigned admission, and the Rust unit cache, and its profile fixtures
  pass. An undeclared optional capability or a request beyond a declared bound
  fails closed in the negative fixtures.
