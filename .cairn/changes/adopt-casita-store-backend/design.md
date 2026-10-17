# Design: Adopt Casita as a durable store backend

## Goal and scope

With `--store-backend casita`, every admitted output with its signed PathInfo,
including PathInfo-backed action-result outputs, lives durably in one Casita
repository under the state directory. Mantle keeps build execution, NAR
measurement, signature admission, signer trust, retention decisions, and GC
planning. Casita keeps verified bytes, named roots, conditional root
publication, and reachability-based collection. Signer trust comes from a
policy file that the destination operator owns. Snix stays the default backend
and is unchanged. ADR 0082 records the full upstream findings and the exact
experimental API surface.

## Current behavior

- Snix persistence: `blobs/`, `directories.redb`, and `pathinfo.redb` under
  `--state-dir` (`crates/crunch-store/src/handle.rs`). The physical export
  under `--store` is a projection.
- Admission: the builder ingests outputs into castore, computes NAR facts,
  signs PathInfo with the build's Ed25519 key, and persists it through
  `BuildStore`. `mantle build` uses `--signing-key` or loads or generates
  `signing-key` in the state directory, or in `CRUNCH_CONFIG_DIR` when that is
  set (`load_or_generate_signing_keypair` and `config_dir_or`,
  `src/build_cmd.rs:799-913`). `bootstrap --fetch` signs its fetch build with a
  key generated for each run and trusts only that key
  (`src/bootstrap.rs:873-874`).
- In-place PathInfo updates: `store sign` and `store repair-final-nar` rewrite
  a published PathInfo in the Snix PathInfo service (`src/store_cmd.rs`).
- Castore-only payloads: Rust unit action-result outputs (ADR 0035) live only
  in castore. `store gc` passes the Rust retention policy's live nodes to
  `garbage_collect_with_castore_roots` (`src/store_cmd.rs`), which keeps them
  during GC.
- Batch import: Nario v2 import persists a batch through
  `PathInfoService::put_batch_atomic` and reads up to 100,000 records
  (`MAX_RECORDS`, `crates/crunch-store/src/nario.rs`). Native store archive
  import persists one path at a time (`crates/crunch-store/src/archive.rs`).
- Unsigned admission: several commands take `--trust-unsigned`, and Nario v2
  import takes `--nario-trust-unsigned`.
- GC: `crunch-gc-core` plans over store-path entries (`GcEntry`) and returns
  `RemoveStorePath` intents. `mantle store gc` writes a plan, and
  `mantle store gc --execute --plan-id <id>` re-observes state under
  `StoreMutationGuard`, rewrites both databases, sweeps blobs, and removes dead
  exports (`crates/crunch-store/src/gc.rs`, `tests/store_gc_cli.rs`).
- Transport: `mantle store archive export --to <file> <selector>...` writes a
  `mantle-store-archive-v1` closure, and
  `mantle store archive import --from <file> --trusted-public-keys <key>`
  verifies metadata, NAR facts, node identity, signatures, and trust policy
  before persistence (`tests/store_archive_cli.rs`).
- Signer trust: the Snix PathInfo service stores signatures and verifies
  nothing on read. Store archive import checks each path against the
  `--trusted-public-keys` of that run (`ArchiveImportOptions`,
  `crates/crunch-store/src/archive.rs`), so those keys authorize one import
  only. Builds resolve substitution trust from `--trusted-public-key` or a
  configured `trusted-public-keys` file (`load_configured_trusted_public_keys`,
  `src/build_cmd.rs`).
- Overlay trust file: `overlay-trusted-public-keys` (`load_layer_trust_keys`
  and `parse_public_keys`, `crates/crunch-store/src/overlay.rs`) is UTF-8 text
  with `#` comments and keys separated by whitespace or commas. Each key parses
  as a Nix-style Ed25519 verifying key. The loader enforces 1 to 64 keys
  (`MAX_LAYER_TRUST_KEYS`) and the overlay descriptor size bound, then sorts
  and deduplicates. Without the file, a layer trusts only its `signing-key`.
- `mantle attest key-show` prints the verifying key of an existing signing key
  in `name:base64` form (`cmd_key_show`, `src/attest_cmd.rs`).
- Dependency policy: `deny.toml` denies unknown git sources.
- Vendoring: Nix builds vendor dependencies with Crane `vendorCargoDeps` over
  `Cargo.lock`, with `overrideVendorGitCheckout` for admitted git sources
  (`flake.nix`). The ignored checkout-local `vendor-deps/` closure, selected by
  `.cargo/vendor-config.toml`, serves self-build and offline Cargo paths. No
  repository-owned generator for `vendor-deps/` exists today.

## Upstream facts at the pinned revision

Details are in `evidence/casita-review.md` and ADR 0082. At
`90404fcb1cfb3d83f2233715448dfefe913f5fd1`:

- Crate `casita` 0.1.0, edition 2024, `rust-version` 1.94.1, Apache-2.0.
  `native` pulls a pinned pre-release `turso` git revision, `object_store`
  0.14, `fastcdc` 5, `bao-tree`, `iroh-io`, `cap-std` 4, `nix-archive` 0.6,
  `astral-tokio-tar` `=0.6.4`, and `blake3` `1.8`.
- `Repository::local` creates `blobs/` and `casita.sqlite` under its root.
- Rooted `FilesystemImport` publishes its root in the same mutation and offers
  no publication precondition.
- With `experimental`, a `MutationSession` imports
  `UnrootedFilesystemImport::new(envelope)` to an `ObjectKey` and protects the
  staged graph for the session's lifetime.
  `publish_if_roots_match(staged, expectations, changes)` with
  `RootExpectation { name, target }` and `RootChange::Set { name, target }` or
  `RootChange::Remove { name }` returns a `ConditionalPublishResult`. A root
  mismatch commits nothing and reports the first mismatch in root-name order.
  Unrelated revision races are retried. Several expectations and changes
  publish in one revision.
- One commit accepts at most the repository's `max_root_changes` root changes
  and expectations, 1,024 at the pinned defaults
  (`FormatLimits::max_root_changes`).
- Casita verifies object identity and graph completeness. It does not decide
  which signers a Mantle store trusts.
- Local roots are permanent by default. A disk-pressure pass may release only
  roots marked evictable. Manual collection keeps every named root.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Session staging and conditional publication | Unrooted import in a `MutationSession`, then `publish_if_roots_match` expecting absence | Selected | Root-race, idempotent, interrupted-staging, and batch fixtures |
| Conditional envelope replacement | Stage a new envelope, then `RootChange::Set` expecting the current target | Selected | Replacement fixtures |
| Mantle-owned castore payload roots | Permanent roots under `mantle/castore/` with `content` and `node.postcard` | Selected with declared `rust-unit-cache` | Payload reuse, tamper, and GC fixtures |
| Destination-owned trust policy | `casita-trusted-public-keys` when present, otherwise only the local signing key; the file is written by the operator | Selected | Missing, invalid, unauthorized, removed-key, local-signer listed and unlisted, and reopen fixtures |
| Split an oversized batch across commits | Several conditional publications | Rejected: breaks all-or-none | Batch-limit fixtures |
| Reject `store sign` in Casita mode | Fail the command under `casita` | Rejected: drops a core capability | Replacement fixtures |
| Reject `store repair-final-nar` in Casita mode | Fail the command and the library repair calls under `casita` before any state access or effect | Selected on 2026-09-30 until a durable repair fence exists | Casita repair rejection and unchanged `snix` repair fixtures |
| Make PathInfo-backed `ActionResultPort` outputs optional | Declare the port unsupported | Rejected: drops a core capability | Fresh-process action-result reuse fixture |
| Standalone Rust unit cache in session scratch | Keep cache nodes only in per-session castore | Rejected: fresh-process reuse requires durable verified castore roots | Rust cache fresh-process and `store gc` reachability fixtures |
| Name payload roots by content digest only | Root per castore digest without `node.postcard` | Rejected: a digest cannot recover the executable bit or symlink metadata unambiguously | Payload tamper fixtures |
| Honor `--trust-unsigned` in Casita mode | Admit unsigned PathInfo | Rejected: every later read would reject it | Unsigned rejection fixtures |
| Enroll import keys automatically | Add `--trusted-public-keys` keys to the policy during import | Rejected: a one-run flag would become durable trust without review | None |
| Copy the source trust file | Copy `overlay-trusted-public-keys` or `trusted-public-keys` from the source | Rejected: source trust is not destination trust | None |
| Reuse configured build trust | Read the configured `trusted-public-keys` file | Rejected: it configures substitution for build commands and can live outside the state directory | None |
| Rooted `FilesystemImport` after a lookup | Preflight, then unconditional rooted import | Rejected: replaces a target that another Casita client publishes in between | None |
| Rooted staging root, then supported promotion | Import under a staging root, then `compare_and_set_root` | Deferred: interrupted admissions leave permanent staging roots; first fallback if the session API changes | None |
| `CasitarImport`, IPC daemon, or CLI subprocess | Archive transform, JSON-RPC import, or shelling out | Rejected: no gain, or no root listing, removal, or collection | None |
| Casita as castore only, or a PathInfo mapping beside roots | Keep `pathinfo.redb`, or a mapping file | Rejected: two durable authorities, or a mapping that outlives its content | Envelope-tamper fixtures |

## Contract and component ownership

- **Dependency owner**: pins Casita in `Cargo.toml` and `Cargo.lock` with
  exactly `native` and `experimental`, admits the Casita and `turso` git
  sources in `deny.toml`, keeps the Nix vendor closure and `vendor-deps/`
  reproducible, adds the Nix admission assertion, checks the `rust-version`
  floor for every toolchain, and records the experimental APIs in ADR 0082 and
  the dependency audit.
- **Casita adapter (store shell)**: the only code that names Casita types. It
  implements the backend behind the existing capability views, including
  PathInfo-backed `ActionResultPort` outputs.
- **Repository location**: `<state-dir>/casita`, so Casita's `blobs/` and
  `casita.sqlite` never sit at a Snix marker path.
- **Root namespaces**: `mantle/outputs/<40 lowercase hex of the store-path
  digest>` for outputs, with an envelope of exactly `content` and
  `pathinfo.json`, the deterministic serde JSON encoding of the signed
  PathInfo; `mantle/castore/<64 lowercase hex of the BLAKE3 digest of the
  postcard-encoded castore node>` for castore-only payloads, with an envelope
  of exactly `content` and `node.postcard` while the profile declares
  `rust-unit-cache`. Mantle manages and removes roots only in
  these two namespaces. No other durable PathInfo mapping exists in Casita
  mode.
- **Trust policy**: `casita-trusted-public-keys` in the state directory,
  written by the operator and parsed with the overlay trust-key rules. When the
  file exists, the trust set is exactly its keys. Without it, the trust set is
  only the verifying key of the state directory's `signing-key`; a key that
  `CRUNCH_CONFIG_DIR` places outside the state directory is trusted only if the
  file lists it. Mantle validates it at store open and reads it again, with the
  same parser and bounds, for every PathInfo admission, read, and verification
  and every import preflight. It caches no trust set, never writes the file,
  and never adds the local key to it. Keys match by full key material, never by
  name alone. Under `casita`, `bootstrap --fetch` signs with the durable
  signing key that `mantle build` uses by default, written only after the
  backend identity check; under `snix` it keeps the key it generates for each
  run.
- **Capability profile**: every core capability, including PathInfo-backed
  `ActionResultPort` outputs; `store-repair-final-nar` omitted from the core
  list until a durable repair fence exists; `atomic-batch-import` bounded by
  `max_root_changes` (1,024 at the pinned defaults); `overlay-composition` and
  `unsigned-admission` not declared; `rust-unit-cache` declared with durable,
  verified castore roots.
- **Session scratch**: Snix blob, directory, and PathInfo services in memory
  or under a per-session directory outside `--state-dir`, deleted at session
  end.
- **Mantle-owned files**: `store-identity.json`, `casita-trusted-public-keys`,
  retention and root records, CA mappings, action-result records,
  attestations, source bundles, and the GC fence stay files under the state
  directory. Their store-path and castore references resolve only through
  Casita roots.

## Decisions

### Decision: One root binds content and signed PathInfo

**Choice:** The output envelope holds `content` and `pathinfo.json`, and
one root names it. `pathinfo.json` is the serde JSON encoding of the signed
PathInfo, equal PathInfo values encode to equal bytes, and there is no
postcard fallback.

**Rationale:** One publication makes content and PathInfo visible together,
and one root removal hides them together. Postcard cannot read every PathInfo
back: `PathInfo` carries `ca: Option<CAHash>`, and `CAHash` deserializes
through a `serde_json::Map` (`vendor/nix-compat/src/nixhash/ca_hash.rs`),
which needs self-describing deserialization. Equal bytes let idempotent
admission and replacement compare envelope identities.

### Decision: Publication is a Casita conditional change

**Choice:** Stage the envelope with `UnrootedFilesystemImport` in one
`MutationSession`, then call `publish_if_roots_match` with an expectation that
the name is absent. A mismatch commits nothing. If the existing root targets an
identical envelope, admission succeeds without a write; otherwise it fails with
`casita-root-conflict`.

**Rationale:** The check and the root change commit together inside Casita,
so neither Mantle writers nor other Casita clients can be overwritten. The
store mutation guard still orders Mantle's own writers.

### Decision: Replacement is conditional on the current target

**Choice:** `store sign` reads the current target, stages a new envelope with
the same `content` and the updated PathInfo, verifies it as for admission, and
commits `RootChange::Set` with an expectation of the target it read.

**Rationale:** `store sign` stays core in Casita mode, and a concurrent change
by any writer makes the update commit nothing instead of losing that change.

### Decision: Final-NAR repair is unsupported until a durable repair fence exists

**Choice:** Under `casita`, `store repair-final-nar` fails with
`casita-repair-final-nar-unsupported` at CLI dispatch, before the state
directory is created or opened, for a dry run and for `--execute`.
`inspect_final_nar_repair` and `execute_final_nar_repair` fail with the same
blocker for an open Casita store before any effect, so a library caller cannot
bypass the CLI check. The `casita` profile omits `store-repair-final-nar` from
its core list, and `snix` repair is unchanged.

**Rationale:** An executed repair writes the repaired PathInfo, then renames
the staged artifact-attestation sidecar into place, and restores the original
PathInfo if the rename fails (`persist_repaired_state` in
`crates/crunch-store/src/repair.rs`). Under Casita, the original envelope's
signed facts do not match `content`, so restoring it cannot produce a root that
verifies, and publishing the sidecar before the root has the opposite race.
Until a durable repair fence can recover the root and the sidecar together, the
parent chose on 2026-09-30 to fail closed instead of claiming a repair that a
crash could leave inconsistent.

### Decision: Castore payload roots support the declared Rust unit cache

**Choice:** PathInfo-backed action-result outputs are admitted as output roots
and rehydrated like any output, so `ActionResultPort` stays core. The
standalone Rust unit cache is the optional capability `rust-unit-cache`,
declared for Casita together with durable castore payload roots. Every
retained castore-only payload is published under
`mantle/castore/<64-hex BLAKE3 of the postcard-encoded node>` with `content`
and `node.postcard`. A read re-ingests `content` and requires the reproduced
node and name to match. Guarded GC recovers a pending fence before checking
the Rust retention live nodes; verified retained roots stay published and
unretained roots may be collected. A missing or repointed retained root
rejects planning before a new fence or root removal.

**Rationale:** With Snix reduced to session scratch, cache nodes need a durable
home before they can be reused. `node.postcard` keeps the executable bit and
symlink metadata that a digest alone cannot recover.

### Decision: Batches publish all-or-none within the commit limit

**Choice:** Nario v2 import stages every envelope in one session and publishes
every new root with one `publish_if_roots_match` call. A batch over the
repository's `max_root_changes` bound (1,024 at the pinned defaults) fails
with `casita-batch-limit` before the session opens and is never split. Any
staging, verification, signer-trust, or root mismatch error commits no root.

**Rationale:** This keeps the all-or-none behavior that Snix provides with
`put_batch_atomic`, and the profile states the lower bound honestly.

### Decision: Signer trust is a destination-owned policy

**Choice:** When `casita-trusted-public-keys` exists, the trust set is exactly
its keys, and the local signing key is trusted only if the file lists it.
Without the file, the trust set is only the verifying key of
`<state-dir>/signing-key`, so a new state directory can build and read its own
outputs while its builds sign with that key, which is the default. When
`CRUNCH_CONFIG_DIR` places the signing key outside the state directory, builds
and `bootstrap --fetch` sign with a key outside the trust set and fail closed
with `casita-signer-untrusted` unless the policy file lists that key; Mantle
does not move, copy, or trust it on its own. Mantle publishes only envelopes
signed by a key in the trust set. It validates the file at store open, and a
present but invalid file fails the open. It then reads the file again, with the
same parser and the 65,536-byte bound, for every PathInfo admission, read, and
verification and for every import preflight, and it keeps no trust set between
verifications. A key added or removed takes effect at the next verification, in
an open handle too, and a file that has become invalid fails that verification
with `casita-trust-policy-invalid`. Mantle does not lock the file. An operation
that verifies several outputs while the operator edits the file may observe
more than one version, and no atomicity is claimed; operators replace the file
by writing a new one and renaming it into place. Mantle never creates or edits
the file and never enrolls a key. Under `casita`, `bootstrap --fetch` signs
with the durable signing key that `mantle build` uses by default instead of a
key generated for each run, and it writes that key, when absent, only after the
backend identity check. Store archive import and Nario v2 import check that the
file exists and that every named key is in it before a mutation session opens.

**Rationale:** The Snix PathInfo service verifies nothing on read, and
`--trusted-public-keys` authorizes one import. A backend that reverifies after
reopen needs a durable trust source that the operator reviews and Mantle never
extends on its own. A present file is the whole trust set, so the operator can
revoke any key, including the local one, by editing it. Trusting the local key
beside the file would make that key impossible to revoke through the policy.
The parent chose the per-verification read on 2026-09-30. A trust set captured
at open would keep a revoked key trusted for the life of a long-running handle
and would not make concurrent edits safe either. The cost is one bounded read
of the policy file, or of the signing key when no file exists, per
verification. The parent chose the bootstrap signer rule on 2026-09-30: an
output signed by a key generated for one run cannot pass a later read unless
the operator lists that key, and under `snix`, which verifies nothing on read,
bootstrap keeps its per-run key. The parent also chose on 2026-09-30 to
document the `CRUNCH_CONFIG_DIR` condition instead of changing where the
signing key lives or what the adapter trusts.

### Decision: Unsigned admission is not declared

**Choice:** Casita mode rejects `--trust-unsigned` and `--nario-trust-unsigned`
with `casita-trust-unsigned-unsupported` before any state access.

**Rationale:** An unsigned PathInfo could never pass a later read, so honoring
the flag would admit content that every fresh process rejects.

### Decision: Verify before admission and on every read

**Choice:** Before admission is reported and before any read is served,
Mantle checks the envelope entries and types, reads at most 1 MiB of
`pathinfo.json`, decodes it, requires its bytes to equal the canonical
re-encoding of the decoded PathInfo, matches the root-name digest to the store
path, verifies a signature from the trust set, checks content-address facts,
and matches the measured NAR SHA-256 and size of `content`. `store verify`
uses a full content audit.

**Rationale:** Roots and the policy file can change after publication, and a
path name alone is not trust.

### Decision: Snix is session scratch in Casita mode

**Choice:** Sandbox inputs and payloads are rehydrated from Casita into the
session Snix services. Outputs are ingested, measured, and signed there, then
published to Casita. Nothing in the session services outlives the session or
answers a lookup.

**Rationale:** The builder keeps its Snix `BuildService` contract without a
second durable authority.

### Decision: GC is plan-bound, fenced, then collected

**Choice:** `crunch-gc-core` plans over verified output and payload envelopes
and records the exact root target of each candidate. Execution takes the store
mutation guard, rechecks the plan, writes a durable fence record, removes each
root through a conditional change that expects the planned target, deletes
dead exports and index entries, records every outcome in the fence, and only
then runs collection.

**Rationale:** Each root removal unpublishes one envelope atomically, and the
fence makes interruption recoverable without new deletions.

### Decision: Migration reuses the verified archive transport

**Choice:** The operator writes the reviewed source signer key into the
destination policy, exports a closure under `--store-backend snix`, imports it
under `--store-backend casita` with that key, and declares roots again with
`store pin`.

**Rationale:** The archive transport already verifies closures, NAR facts,
signatures, and per-run trust. The destination policy adds durable trust
without a new command.

## Ordering

Admission (outputs and payloads):

1. Build and admit in session scratch with the existing NAR measurement and
   signing.
2. For outputs, check for a valid signature from the trust set, or fail with
   `casita-signer-untrusted`.
3. Take `StoreMutationGuard` and open one Casita mutation session.
4. Stage the envelope unrooted and verify it: NAR facts against the PathInfo
   for outputs, node and root-name reproduction for payloads.
5. Publish with `publish_if_roots_match` expecting the name to be absent. On a
   mismatch, report idempotent success for an identical target or fail with
   `casita-root-conflict`; the staged envelope stays unrooted.
6. Look up and verify the published envelope, then report admission and
   publish the physical export.

Replacement: take the guard, read the current target, stage the new envelope,
verify it, and commit `RootChange::Set` expecting the target read. A mismatch
commits nothing.

Import: check the batch size, the policy file, and every named key before
step 3, and fail with `casita-batch-limit`, `casita-trust-policy-missing`, or
`casita-import-key-unauthorized`.

Batch: stage and verify every envelope in one session, then publish all new
roots with one call. Any error before or during publication commits no root.

Read: read the root's target from a metadata snapshot, check the envelope out
into per-read scratch, verify it under the trust set, re-ingest `content` and
measure its NAR, confirm the root still has the target that was read, then
serve, export, or rehydrate `content`. Every read is a full content audit.

Casita GC runs only through the guarded entry point
(`garbage_collect_with_castore_roots_under_guard`), which takes
`StoreMutationGuard`. A Casita GC call without the guard, including a dry run,
fails with `casita-gc-guard-required` before any recovery, planning, or
mutation.

GC execution:

1. Take `StoreMutationGuard`; re-observe retained roots, Rust retention live
   nodes, envelopes, references, and each candidate's current root target.
   Any drift fails with `gc-plan-stale` before a mutation.
2. Write the fence record (plan identity, root names, expected targets,
   store paths and payload identities) durably.
3. Remove each root through a conditional change that expects its planned
   target. A mismatch stops removal for that root, records the outcome, and
   blocks a success claim.
4. Delete dead physical exports and index entries naming removed paths.
5. Record every outcome in the fence, then run collection. A busy collector
   leaves reclaim incomplete, and the report says so.

While a fence is incomplete, lookup, PathInfo listing, closure walk, export,
admission, root registration, and rehydration fail with `gc-recovery-required`
unless the command first recovered the fence under the guard. Recovery runs
only under `StoreMutationGuard`, at the start of a guarded command and before
it plans, admits, or registers anything: `store gc`, `store usage`, guarded
store commands such as `store pin`, `store sign`, and `store archive import`,
the bootstrap build, provider adoption, and the remote executor. Unguarded
reads such as `store list` and `store info` do not recover. `store gc` is the
operator's explicit remedy. A fenced root that is absent
completes steps 4 and 5. A fenced root that still has its expected target
stays published and is reported. A root with another target fails with
`casita-root-conflict`. Recovery never removes a root.

## Migration procedure

```bash
# Source: record roots and print the signer's public key (name:base64).
mantle --store-backend snix --state-dir OLD --store OLD_OUT --json store roots
mantle --store-backend snix --state-dir OLD --store OLD_OUT attest key-show

# Destination: write the reviewed key into a new policy file.
# Never copy a trust file or signing key from OLD.
# Once the file exists it is the whole trust set: if NEW will also build,
# add the verifying key of NEW's own signing key as another line.
mkdir -p NEW
printf '%s\n' 'SIGNER:BASE64KEY' > NEW/casita-trusted-public-keys

# Move the closure and verify it in a fresh process.
mantle --store-backend snix --state-dir OLD --store OLD_OUT \
  store archive export --to closure.mnar <selector>...
mantle store archive list --from closure.mnar
mantle --store-backend casita --state-dir NEW --store NEW_OUT \
  store archive import --from closure.mnar --trusted-public-keys 'SIGNER:BASE64KEY'
mantle --store-backend casita --state-dir NEW --store NEW_OUT store pin <path>
mantle --store-backend casita --state-dir NEW --store NEW_OUT store verify
```

The operator checks the key printed by `attest key-show` through a reviewed
channel before writing it. Import fails before any Casita mutation when the
policy file is missing or the import key is not in it. `--trust-unsigned`
fails under `casita`. Import keeps the original signatures and adds none.
`OLD` stays byte-identical. Removing the key from the policy later makes those
outputs fail with `casita-signer-untrusted`. Because a present file is the
whole trust set, local builds in `NEW` fail with `casita-signer-untrusted`
until the operator lists `NEW`'s own verifying key, which `attest key-show`
prints; Mantle never adds it.

## Failure behavior

Stable blockers:

- `casita-nar-mismatch`: measured NAR SHA-256 or size of `content` differs
  from the signed PathInfo.
- `casita-envelope-invalid`: wrong entries or types, an undecodable, oversized,
  or non-canonical `pathinfo.json`, an undecodable `node.postcard`, a root-name
  digest that differs from the store path, inconsistent content-address facts,
  or a payload whose re-ingested `content` does not reproduce its decoded node
  and root name.
- `casita-signer-untrusted`: no valid signature from a key in the trust set,
  at publication or on read, including after a key or the policy file was
  removed.
- `casita-trust-policy-missing`: an import found no `casita-trusted-public-keys`
  file.
- `casita-trust-policy-invalid`: the policy file is malformed, empty,
  oversized, or over the key limit.
- `casita-import-key-unauthorized`: an import key is not in the policy file
  by full key material.
- `casita-trust-unsigned-unsupported`: `--trust-unsigned` or
  `--nario-trust-unsigned` under `casita`.
- `casita-batch-limit`: a batch exceeds the repository's `max_root_changes`
  bound.
- `casita-root-conflict`: a conditional publication, replacement, or recovery
  finds another target.
- `casita-root-missing`: a retained output or payload, or a fenced
  expectation, has no root where one must exist.
- `casita-overlay-unsupported`: `--base-store` with `--store-backend casita`.
- `casita-repair-final-nar-unsupported`: `store repair-final-nar` under
  `casita`, as a dry run or with `--execute`, before the state directory is
  created or opened, or a library repair call on an open Casita store before
  any effect.
- `gc-plan-stale`: Casita GC execution observed drift. The `snix` backend
  keeps its existing `stale-gc-plan` blocker for the same condition
  (`crates/crunch-store/src/gc.rs`), so the conformance rail's stale-plan
  fixture expects each backend's own blocker.
- `casita-envelope-invalid` or `casita-root-missing`: a retained Rust cache
  payload root or envelope changed or vanished; neither reuse nor GC proceeds.
- `casita-gc-guard-required`: Casita GC planning, a dry run, or execution
  without `StoreMutationGuard`.
- `gc-recovery-required`: an operation found an incomplete fence record
  without first recovering it under the guard.

A lookup of a path that was never admitted or was collected is a plain miss.

## Tests

Race, replacement, and batch fixtures use a second Casita client on the same
repository to make real conflicting changes. Trust fixtures write real policy
files and sign with real fixture keys. Payload fixtures run real Rust unit
action results. They are behavior tests, not source-text checks.

- Positive: conformance rail on `casita`; build, delete the export, fresh
  process reuses without a rebuild; a local build without a policy file
  publishes and verifies after reopen, and no policy file appears;
  `bootstrap --fetch` in a new state directory without a policy file signs with
  the durable signing key, a fresh process verifies its fetched output, and a
  second run reuses it; absent root publishes; identical root is an idempotent
  success with no write; `store sign` replaces the envelope and a fresh process
  verifies the new PathInfo; `store repair-final-nar` under `snix` still
  repairs; a batch of exactly 1,024 absent paths publishes in one revision at
  the pinned defaults; a PathInfo-backed action-result output is rehydrated by
  a fresh process; `store gc` without Rust unit cache state removes only
  unretained output roots without creating cache state; with declared
  `rust-unit-cache`, a Rust unit payload is reused without recompiling and GC
  releases unretained payload roots while preserving retained ones; collection
  keeps retained content; migration with a written policy
  verifies in a fresh process; guarded `store gc` recovers an interrupted fence
  before planning and removes no root; an output whose signed PathInfo carries
  a content address round-trips through `pathinfo.json` with equal bytes.
- Negative: a second client publishes between staging and commit; process stops
  after staging; a second client repoints a root during `store sign`;
  `store repair-final-nar` under `casita`, as a dry run and with `--execute`,
  and a library repair call on an open Casita store; a batch of 1,025 paths;
  batch with one conflicting root; batch whose later path fails staging or
  verification; Nario v2 import with a key outside the policy;
  `--trust-unsigned` and `--nario-trust-unsigned` under `casita`; import
  without a policy file; import key missing from the policy; policy key with
  the same name but different bytes; malformed, empty, oversized, and
  over-limit policy files; a local build whose signer an existing policy file
  does not list, and the local signer removed from the file, then fresh-process
  lookup; `bootstrap --fetch` with a policy file that omits the durable signing
  key; build signed by a key outside the trust set; substituted output signed
  only by a cache key outside the policy; key removed from the policy and
  policy file removed, then fresh-process lookup, `store verify`, and GC
  planning; a retained Rust cache payload root repointed to different
  `content` or `node.postcard`, or removed, rejects reuse and GC before any
  new fence;
  NAR mismatch; tampered `pathinfo.json`; swapped `content`; root-name digest
  mismatch; stale plan; interruption before and after root removal; lookup,
  listing, admission, root registration, and rehydration while a fence is
  pending; Casita GC or a dry run without the guard; busy collector; root made
  evictable and released by another tool; floating Casita reference, widened
  feature set, or unadmitted git source; vendor closure drift; `--base-store`
  with Casita. Every import and flag negative asserts that no Casita mutation
  happened and that the policy file and source state stayed byte-identical.
- Retention: Casita's eviction of evictable roots under pressure is
  crate-private at the pinned revision, so the disk-pressure fixture drives
  `DiskPressurePolicy::collect_if_needed` with a supplied `DiskUsage`, which
  runs a real `try_vacuum`, and asserts that every Mantle root stays published
  and `Permanent`. A retained root that another client removes stands in for
  one that Casita evicted after another tool marked it evictable. That the
  local profile's eviction pass never selects a permanent root is Casita's
  documented rule, not a Mantle test.
- Architecture: the store capability boundary checker rejects a Casita type
  outside the store shell. This rail checks topology, not behavior.

## Risks / Trade-offs

- Casita is pre-release, and Mantle depends on its `experimental` API. The pin
  is exact, ADR 0082 lists every experimental item used, and bumps require a
  reviewed change.
- `turso` is a pinned pre-release git dependency with an experimental
  multi-process WAL.
- The dependency graph grows: a second `fastcdc` and `nix-archive` major,
  `bao-tree`, `iroh-io`, and `turso`. `blake3` must resolve to the existing
  `=1.8.2` pin.
- Atomic batches are capped at 1,024 PathInfo records per commit in Casita
  mode at the pinned defaults. A caller that passes a path whose root already
  holds an identical envelope spends one on an expectation; Nario v2 import
  checks and skips paths it finds present, so only absent paths count. Snix
  mode has no backend bound; the Nario v2 reader's 100,000 record limit
  applies to both backends.
- Each retained castore payload adds one root under the declared
  `rust-unit-cache` capability. Casita remains interchangeable with `snix`
  only within their declared capabilities.
- Operators must add cache keys and any `--signing-key` other than the local
  key to the policy before those outputs can be published in Casita mode.
- Removing a key makes retained outputs signed only by that key unreadable and
  blocks GC planning until the key is restored or those roots are released.
- Verification on every read costs time. `store verify` forces a full audit.
- Overlay composition and unsigned admission are not available in Casita
  mode; the Rust unit cache uses verified durable castore payload roots.
- After an interrupted GC, Casita reads and admission fail with
  `gc-recovery-required` until the operator runs `store gc`, which recovers
  the fence before it plans.
- `pathinfo.json` is larger than a binary encoding, and a change to the
  PathInfo serde representation changes envelope bytes and identities, so it
  needs a reviewed format change.

## Claim boundary

Casita backend evidence proves the tested admission, conditional publication,
replacement, payload, batch, trust, verification, reuse, retention, GC,
recovery, migration, and rejection behavior for the pinned revision, feature
set, and declared profile. The trust policy proves only that published and read
outputs carry a valid signature from a key in the policy file as read for that
verification, or, when no file exists, from `<state-dir>/signing-key`. Evidence
does not prove Casita correctness, experimental API stability across revisions,
key ownership, signer honesty, revocation freshness, that a state directory
reads its own outputs without a policy file when `CRUNCH_CONFIG_DIR` places the
signing key elsewhere, durability under power loss, crash safety beyond the
tested interruptions, output correctness, sandboxing, or release eligibility.

## Amendments from the ADR 0082 review

- **Pinned patch.** At the pinned revision `crates/casita/src/nar.rs:88`
  calls `hash.finalize().as_bytes()`, which resolves to
  `sha2::Digest::finalize` in Mantle's graph, so the unmodified source fails
  `cargo check`. One tracked, repository-owned patch changes the call to
  `blake3::Hasher::finalize(&hash).as_bytes()`. The architecture owner
  applies it in `overrideVendorGitCheckout` and makes the default
  `nix develop` shell resolve Casita from that patched closure, because a
  plain `cargo build` there otherwise compiles the unpatched git checkout. The
  transport owner applies the same file in the `vendor-deps/` generator. Every
  path fails on drift. No vendor snapshot is hand-edited, no fork is pinned,
  and no evidence claims an unmodified upstream build.
- **Castore parity gate resolved in the store and cache path, with CLI
  reachability still in progress.** Castore roots are published with declared
  `rust-unit-cache`; public cache reuse and the real wrapper checked fresh
  process restoration, retained GC execution, and FailOpen missing-root
  rejection in Casita Run 82. Run 85 exercised literal selected-Casita CLI
  GC rejection of a missing retained payload and guarded recovery of an
  interrupted output GC using the first coherent binary. Run 86 also
  planned and executed signed output GC without Rust cache state, after
  explicitly releasing the unpinned build's legacy-unmanaged root.
  Run 87 passed the signed no-cache selected-Casita CLI GC fixture under
  the initial frozen root-test snapshot. The existing-cache fixture has
  since been migrated to per-interest retention records; its focused CLI
  run and final merged-source verification remain open before claiming
  a completed backend conformance rail.
- **Dependency advisories.** The vendor rail's `cargo deny check` failed
  advisories and sources. Only `proc-macro-error 0.4.12` (unmaintained) is
  new in the lock, from Casita's graph through `genawaiter 0.99.1`. The
  `h2 0.4.13` and `rustls 0.23.37` were vulnerable in HEAD and reachable
  from Casita through `object_store 0.14.0`; the current lock has compatible
  patched `h2 0.4.16` and `rustls 0.23.45` instead.
  `bitmaps 3.2.1`, `imbl-sized-chunks 0.1.3`, and `rsa 0.9.10` are unchanged
  and off the Casita path. Four missing `allow-git` sources predate this
  change. The gate stays open, with no waivers.
- **Trust set.** When `casita-trusted-public-keys` exists, it is the whole
  trust set, and the local signing key is trusted only if the file lists it.
  Without the file, only `<state-dir>/signing-key` is trusted, so a key that
  `CRUNCH_CONFIG_DIR` places elsewhere needs a policy file that lists it. The
  earlier rule also trusted the local key when a file existed, which made that
  key impossible to revoke through the policy. The adapter fix has landed; the
  Run 24 test fails before it and passes after it (Run 33).
- **Fence recovery.** Any command that holds `StoreMutationGuard` recovers a
  pending fence before it plans, admits, or registers anything. Unguarded
  reads such as `store list` still fail with `gc-recovery-required`.
