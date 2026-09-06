# Checked legacy archive migration

The migration preserves package bytes and signed facts. It changes only a proven legacy castore node representation. See [ADR 0119](../adr/0119-migrate-legacy-archive-representations-without-resigning.md).

## Interface

The standalone helper is `scripts/migrate-legacy-archive.rs`. It compiles only the helper and its library dependencies, not the Mantle executable. The pure owner is `crates/crunch-repair-core/src/legacy_archive.rs`.

The request has this closed schema:

```json
{
  "schema": "mantle-legacy-archive-migration-request-v1",
  "expected_input_blake3": "<64 lowercase hexadecimal characters>",
  "store_prefix": "/mantle/store",
  "roots": ["<32-character-store-digest>-<name>"],
  "trusted_public_keys": ["<name>:<base64-public-key>"]
}
```

The roots are store basenames, not physical paths. The expected archive identity must come from an independently retained producer observation. Public keys are explicit input. The helper never reads private keys or ambient trust configuration.

With a Cargo-script-capable toolchain, run:

```sh
CARGO_TARGET_DIR=target/archive-compat \
  cargo -Zscript test --offline --manifest-path scripts/migrate-legacy-archive.rs
CARGO_TARGET_DIR=target/archive-compat \
  cargo -Zscript run --offline --manifest-path scripts/migrate-legacy-archive.rs -- \
  request.json input.archive target/new-migration
```

The parent output directory must already exist. The destination must be absent. The helper publishes `archive` and `receipt.json` atomically on Linux. Missing cached development dependencies cause `--offline` to fail. They never trigger a package or toolchain bootstrap.

After comparing the receipt and output archive BLAKE3, use the existing Mantle binary:

```sh
mantle --state-dir fresh-state --store fresh-store --store-prefix /mantle/store \
  store archive import --from target/new-migration/archive \
  --trusted-public-keys '<name>:<base64-public-key>'
mantle --state-dir fresh-state --store fresh-store --store-prefix /mantle/store \
  store verify --trusted-public-keys '<name>:<base64-public-key>'
```

Import success requires the full expected path set, not only exit zero. Verification requires every expected object and trusted signature. A newly initialized Mantle state can generate its own private key. That key is not part of this migration and must not be shared.

## Admitted subset

- Native archive v1 with exact requested roots and logical prefix.
- Current postcard directory nodes or the exact historical doubled-count nodes.
- Absent CA, final-NAR recursive SHA-256 CA, or Mantle's `crunch-ca-marker:out` recursive SHA-256 CA.
- Explicit trusted signatures and complete reachable references.
- Linux atomic no-replace publication into an operator-owned parent.

Other hash modes, marker output names, and metadata representations fail closed. The helper does not erase CA or replace expected hashes with observations. It rejects fields that deserialize into a different non-node JSON value.

## Resource and authority bounds

The input archive is at most 512 MiB. A request is at most 64 KiB. Each metadata frame is at most 1 MiB. There are at most 64 records, 64 references per record, 64 signatures per record, and 16 trusted keys.

A NAR is at most 256 MiB. Its tree contains at most 131072 nodes with a maximum recursion depth of 64. Marker occurrences have the same 131072 limit. Payloads remain in memory as one pinned archive observation. The helper does not restore file modes, symlinks, or executables.

The core owns migration and closure decisions. The protocol adapter supplies cryptographic and NAR observations. The shell owns explicit file reads, temporary files, synchronization, and publication. No store service or database capability enters the helper.

The output directory is a same-user capability. The helper makes no hostile-parent or power-loss durability claim. Abrupt process death can leave an unpublished temporary directory. It cannot publish a partial archive/receipt pair through the no-replace commit.

## Evidence scope

`mantle-legacy-archive-migration-v1` records original and current node identities, CA evidence, and input/output archive BLAKE3 values. Payload hashes and signed fields stay unchanged. The receipt explicitly sets `package_realization = false`.

The desktop fixture imported eleven objects through an unchanged Mantle binary and passed all eleven hash/signature checks. Evidence is under `verification/legacy-archive-2026-09-06/`.

This does not prove compiler correctness, ABI compatibility, a new Darkhttpd output, reproducibility, release eligibility, or physical readiness. It does not repair ordinary NAR-cache import. Full workspace checks and a reproducible helper packaging contract remain outside this focused operator migration.
