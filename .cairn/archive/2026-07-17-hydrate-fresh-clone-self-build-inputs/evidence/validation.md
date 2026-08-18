# Validation evidence: fresh-clone self-build source hydration

Date: 2026-07-17

## Bounded claim

This evidence proves that an independently identity-bound `mantle-source-bundle-v1` handoff can hydrate the ignored Cargo directory source and pinned legacy-provider source state into a fresh clone, pass locked Cargo metadata with an initially empty `CARGO_HOME` and Cargo networking disabled, and satisfy `bootstrap --fetch --offline-source-preflight` without network fallback.

It does not prove fixed-point self-build success, complete capture of future undeclared bootstrap sources, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment readiness, or full Cargo compatibility.

## Real fresh-clone handoff

The producer exported the actual locked `vendor-deps/`, unpacked pinned musl.cc provider, and runtime-generated `provider.json` with:

```text
format: mantle-bootstrap-source-profile-v1
mode: fresh-clone-inputs
manifest_blake3: 0f09dcc3de85a3def04592efe55d3e8813cf6238b5f4340d2c8d139d38517351
required_record_count: 3
provider_kind: musl.cc-native-reduced-v1
serialized_bytes: 2216806169
```

A local Git clone was asserted not to contain `vendor-deps/` before hydration. The contracted result was:

```text
format: mantle-self-build-source-hydration-v1
manifest_blake3: 0f09dcc3de85a3def04592efe55d3e8813cf6238b5f4340d2c8d139d38517351
vendor_content_blake3: e66ff44bfb95a05ea91aece6965e230c7b356ee8c2ab08f4f195614d7fafefdc
provider_archive_content_blake3: 567774c93019f99b9f3e8988b476fb43235e843b71f3660d3588fdaa67500a9c
imported_record_count: 3
existing_record_count: 0
pinned: true
```

The structured transcript is `evidence/fresh-clone-summary.json`.

## Empty-cache locked Cargo validation

Command shape:

```text
CARGO_HOME=<new-empty-dir> CARGO_NET_OFFLINE=true \
  cargo metadata --offline --locked --format-version 1 \
  --config .cargo/vendor-config.toml
```

Result: exit zero, 3,449,220-byte metadata document. The new Cargo home contained only `.global-cache`, `.package-cache`, and `registry/CACHEDIR.TAG`; it contained no downloaded source payload.

## Offline legacy-provider preflight

Final command shape:

```text
CRUNCH_NO_FUSE=1 mantle \
  --store <fresh-store> \
  --state-dir <hydrated-state> \
  bootstrap --fetch --offline-source-preflight \
  --output <seed.ncl>
```

Pueue task `307` completed successfully with:

```text
offline bootstrap source profile ready: records=1 source_state_blake3=b76723aaf06e15776cc0713145f58aea98764034de62ffd458e2d8cd63e98ad0
raw musl-gcc-raw -> .../m7vilqp92wlp139x4xa2krz562h5a3gb-musl-gcc-raw
.../m7vilqp92wlp139x4xa2krz562h5a3gb-musl-gcc-raw (fetched)
.../nzb1x8p85ph7jvlqy0kmkfsw7783rbjh-musl-seed-toolchain (reduced provider built)
Wrote .../offline-provider-seed-final.ncl
```

`--offline-source-preflight` rejects missing/unpinned provider state before the builder and does not permit a live network fallback. The generated provider metadata records `provider_id = musl.cc-native-reduced-v1`, raw SHA-256 SRI `sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY=`, raw size 282,505,293 bytes, and reduced size 197,603,646 bytes.

## Focused positive and negative tests

Pueue task `265` ran the complete focused surfaces:

```text
source_bundle::tests: 55 passed; 0 failed
bootstrap::tests: 18 passed; 0 failed
source_bundle_hydration_cli: 2 passed; 0 failed
machine_schema_contracts: 4 passed; 0 failed
```

Coverage includes valid hydration, wrong external manifest identity, missing vendor record, an existing destination, Cargo checksum drift, state-persistence rollback, unpinned provider state, source traversal, over-limit files, safe Cargo names containing `..`, case-distinct provider headers, read-only provider directories, and source-fetch tempdir lifetime.

## Repository gates

- Rustfmt: `cargo fmt --check -p mantle -v` passed.
- Strict first-party Clippy: pueue task `287` passed after the focused read-only-directory test.
- Tiger Style: pueue task `283` passed.
- Dependency policy: pueue task `291` reported `advisories ok, bans ok, licenses ok, sources ok`.
- Machine contracts: pueue task `300` reported `machine schema contract generation: PASS (19 contracted, 48 classified)` and `machine schema contract check: PASS (19 contracted, 48 classified)`.
- First-party quality: pueue task `292` passed Rustfmt, strict Clippy, and serialized first-party tests. Both root binary suites reported `1570 passed; 0 failed`; focused integration suites, including machine schemas, runbook docs, source-bundle CLI, and hydration CLI, also passed.
- Nix evaluation: pueue task `300` ended with `all checks passed!`; incompatible non-host systems were explicitly omitted by Nix.
- Cairn pre-sync validation and proposal/design/tasks gates passed in task `322`.
- Executed sync receipt: `1d4e3bebe778445f2c73a180d488c48499b32c3df26894fc406a8a25b236da7a`; the accepted requirement was inspected in `cairn/specs/bootstrap-inventory/spec.md`.
- Pre-archive validation remained valid and Tracey reported `145/145` referenced.
- Implementation commit: `86e9d3a7`.

## Corrections discovered by real payload validation

The real provider exposed constraints that synthetic fixtures had not exercised: compiler files above 16 MiB, legitimate Cargo names containing `..` within a component, case-distinct Linux kernel headers, runtime metadata using `provider_id`, read-only source directories, fresh-profile compatibility with legacy provider lookup, and tempdir ownership across asynchronous fetch execution. Each correction is bounded by a positive and negative/unit assertion, and the final real-provider preflight was rerun after the last implementation change.

## Archive and exact post-archive state

Archive dry-run/execution used `CAIRN_ARCHIVE_DATE=2026-07-17`; execute receipt:
`4ffac6b1f63f7da771758b22d8533bc32310a746d414a3247ecbc7f88b6c0bda`.

Pueue task `329` produced exact command outputs beside this transcript:

- `post-archive-validation.json` reports `"changes": 0`, empty issue/finding lists, and `"valid": true`.
- `post-archive-change-list.json` reports an empty `changes` array.
- `post-archive-tracey.txt` reports `traceability coverage ok: 145/145 referenced (profile mantle-default)`.

The implementation commit is `86e9d3a7`; the archive/evidence commit and push are the remaining mechanical operations.
