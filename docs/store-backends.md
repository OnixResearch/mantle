# Store backends

Mantle keeps admitted outputs in one durable store backend per state directory.
`snix` is the default backend and keeps its existing formats.
`casita` keeps outputs in a pinned, pre-release [Casita](https://github.com/cachix/casita) repository and declares a smaller capability profile.
[ADR 0082](../adr/0082-select-store-backends-explicitly-and-admit-casita.md) records the decision and is still Proposed.

Casita support covers only the paths listed under [Validation](#validation).
Fresh `mantle build` processes have reused a fixed-output `file://` fetch and the sandboxed `examples/hello.ncl` derivation, in content-addressed and input-addressed form, from Casita after their physical exports were deleted.
A bounded offline fixture also built a signed two-path source closure under each backend from an imported pinned bundle after its local HTTP endpoint closed, with equal source-preflight reports and byte-identical archive exports.
Other `mantle build` derivation shapes have no passing validation under `casita` yet.
The backends are interchangeable only within their declared profiles.

## Select a backend

Pass the global option `--store-backend snix` or `--store-backend casita`; the default is `snix`.
Environment variables, directory contents, and build features never select a backend.
An unknown value fails with `store-backend-unknown` before Mantle reads or creates state.

Mantle passes the selected backend to the child processes that it starts for the same state directory: the local remote worker, bootstrap validation, the source-built fixed-point shell, and transcript runs.
`mantle-rust-cache-daemon` requires an explicit `--store-backend`.

## State identity

Each state directory records its backend in `store-identity.json`.
A new directory gets schema `mantle-store-state-v2` with a `backend` field.
Mantle reads a legacy `mantle-store-state-v1` record as `snix` and never rewrites it.

Mantle checks the record before it creates directories, takes the store lock, or opens services:

- A record for another backend fails with `store-backend-mismatch: requested <selected>, state declares <recorded>`.
- A different logical prefix or trust policy fails with `store-identity-mismatch`.
- A directory without a record may hold `signing-key`, `casita-trusted-public-keys`, `overlay-trusted-public-keys`, and `store-mutation.lock`; other content is handled by these rules:
  - An entry named `casita` fails with `store-backend-mismatch` under either backend, also next to Snix files.
  - Under `--store-backend casita`, any other content fails with `store-backend-mismatch`.
  - Under `--store-backend snix`, Mantle treats the content as Snix state from an older Mantle build and writes a `mantle-store-state-v2` record with `backend: snix`.
    It deletes and rewrites none of the existing files, but opening the Snix databases can update their internal bookkeeping.

The failures above change no file.
Rerun with the recorded backend, or [move the content](#migrate-from-snix-to-casita) into a new state directory.
Never edit or delete `store-identity.json`.
Mantle reads only the selected backend and never falls back to another backend's files.

Mantle does not check where unrecorded content came from.
Any command that selects `snix`, including one that omits `--store-backend`, claims such a directory for `snix`, so run Mantle against an unrecorded directory only when you know that Mantle's Snix backend wrote it.

When comparing signed results across backend selections or state directories, provision
the same signing key explicitly in every run and compare the NAR and PathInfo
with the same logical store prefix. Store paths and unsigned PathInfo fields may
match with different keys, but signatures cannot: verify each signature under
the corresponding public key instead of treating a different signature as a
backend regression. A generated per-state key is not a reproducible signing
fixture.

## Capability profiles

`mantle store info <path>` prints the selected backend and its profile; with `--json`, see `backend` and `backend_capabilities`.
Both profiles declare the core capabilities `output-admission`, `lookup`, `fresh-process-reopen`, `closure-resolution`, `export`, `store-archive-export`, `store-archive-import`, `gc-plan`, `plan-bound-gc`, `action-result-pathinfo-output-reuse`, and `store-sign`.
Only `snix` also declares `store-repair-final-nar`; under `casita`, `store repair-final-nar` fails with `casita-repair-final-nar-unsupported` before Mantle reads or creates state, both as a dry run and with `--execute`.
A declared capability is not validated behavior.

| Optional capability | `snix` | `casita` | Blocker under `casita` |
|---|---|---|---|
| `overlay-composition` | `true` | `false` | `casita-overlay-unsupported` |
| `atomic-batch-import` | `true` | `true` | `casita-batch-limit` above the bound |
| `rust-unit-cache` | `true` | `true` | — |
| `unsigned-admission` | `true` | `false` | `casita-trust-unsigned-unsupported` |
| `max-root-changes` | `unbounded-by-backend` | `1024` | `casita-batch-limit` |

JSON field names use underscores, and `max_root_changes` is `null` for `snix`.
The 1,024 bound is the number of root changes that one Casita commit accepts at the pinned defaults; it does not cap how many outputs a store holds.
It is separate from the 100,000-record limit of the Nario v2 reader.

Under `casita`, `--base-store`, every `--trust-unsigned` option, and `--nario-trust-unsigned` still fail before Mantle reads or creates state. Local `rust-plan` and shared-origin Rust cache reuse use durable Casita payload roots. The `rust-cache serve` and wrapper daemon preflight rejection is removed; the focused fixtures do not establish end-to-end daemon traffic.
`store usage` and `store gc` recover a pending fence under the store mutation guard before loading Rust cache retention; an invalid or missing retained payload root fails closed before a GC fence or root removal. A store without Rust cache state plans output GC without opening the cache.

The store capability boundary keeps Casita repository types and mutation
sessions inside the `crunch-store` store shell. The declared
`crunch-rust-cache` adapter may own private Snix services, but that exception
does **not** authorize Casita types or sessions in the adapter or any other
first-party shell. `tools/check_store_capability_boundary.rs` scans production
Rust sources for known escape patterns and exercises negative fixtures; zero
reported escapes prove only this bounded source check, not runtime isolation,
absence of other escape forms, Casita correctness, or release safety.

## Casita layout and trust

The Casita repository lives in `<state-dir>/casita`, next to Mantle-owned files such as `store-identity.json`, `casita-trusted-public-keys`, and `gc-roots.json`.
Mantle keeps session Snix blob and directory services in memory as scratch space.
Each admitted output has one root, `mantle/outputs/<40 lowercase hex of the store-path digest>`.
The output root targets an envelope directory with exactly two entries: `content`, the output file, directory, or symlink; and `pathinfo.json`, the signed PathInfo in canonical serde JSON.
Retained castore-only Rust cache payloads instead use permanent `mantle/castore/<64 lowercase hex of the BLAKE3 digest of the postcard-encoded node>` roots. Their envelope has exactly `content` and `node.postcard`; each read re-ingests `content` and requires its reproduced node and root name to match `node.postcard`. A repointed or malformed payload fails with `casita-envelope-invalid`; a missing retained root fails with `casita-root-missing`, rather than silently rebuilding the Rust unit.
Mantle releases output and payload roots only through GC and never marks a root evictable.
Commands that open the repository can change Casita's own files under `<state-dir>/casita`, even when they fail.
To check a failed command, compare roots, fence files, and Mantle-owned files, not the whole repository directory.

When `<state-dir>/casita-trusted-public-keys` exists, Mantle trusts exactly the keys in it; the local signing key is trusted only if the file lists it.
Without the file, Mantle trusts only the public key of `<state-dir>/signing-key`, so a build signed with another key, for example through `--signing-key` or `$CRUNCH_CONFIG_DIR/signing-key`, needs a policy file that lists that key.
The operator writes the policy file; Mantle never creates, edits, copies, or extends it.
It uses the `overlay-trusted-public-keys` syntax: UTF-8 text, `#` comments, and `<name>:<base64-ed25519-public-key>` entries separated by whitespace or commas.
It holds 1 to 64 keys in at most 65,536 bytes.
The policy path must be a regular file, not a symlink.
An empty, malformed, oversized, or over-limit file, or any symlink or other non-regular file at that path, fails the store open with `casita-trust-policy-invalid`.

Mantle validates the policy file when it opens the store.
It then reads the file again, up to 65,536 bytes, for every admission, every read, and every import preflight, and it keeps no cached trust set, so a key added or removed while a process runs takes effect at that process's next check.
Repeated lookup or reuse within one process also rechecks the Casita root, signed envelope, content, and current policy; the in-memory Snix session-node cache is not an alternate Casita read authority.
Mantle does not lock the file, so a check that overlaps an edit can see a partial file; write the new file elsewhere and rename it into place.
Mantle publishes an output only when its PathInfo carries a valid signature from the trust set; otherwise the build fails with `casita-signer-untrusted`.
After a key leaves the file, reads of outputs signed only by that key fail with `casita-signer-untrusted`, including outputs of the local signer and reads during GC planning.
Under `casita`, `mantle bootstrap --fetch` signs the raw seed that it fetches with the local signing key that `mantle build` uses without `--signing-key`, so these rules apply to it; with `CRUNCH_CONFIG_DIR` unset, it creates `<state-dir>/signing-key` with mode `0600` when that file is missing.
It opens the state directory with the `/crunch/store` logical prefix, so give it a new state directory or one that already records that prefix, and pass `--store-prefix /crunch/store` to later commands on that directory, such as `store verify`.

`store archive import` and Nario v2 import take their keys from `--trusted-public-keys`, or from the configured `trusted-public-keys` file when that option is absent.
Before any Casita mutation, an import fails with `casita-trust-policy-missing` when the policy file is absent or the import names no key.
It fails with `casita-import-key-unauthorized` when an import key is not in the policy file with the same name and key bytes.
An archive path that no import key signed fails with `untrusted-signature`, and an archive path without any signature fails with `casita-signer-untrusted`; Mantle publishes neither.
`--trusted-public-keys` authorizes one import and never adds durable trust.

## Admission and reads

Casita publication starts only after Mantle's existing admission checks pass.
Mantle checks the envelope's NAR size and SHA-256 against the signed PathInfo, stages the envelope without a root, and publishes the root in one conditional Casita commit that expects the root to be absent.
If the root already targets an identical envelope, the admission succeeds without a write.
If the root targets anything else, the admission fails with `casita-root-conflict` and leaves that root unchanged.

`store sign` reads the current output, stages a replacement envelope with the
same `content` and updated signed `pathinfo.json`, and conditionally replaces
the root only if it still targets the envelope that Mantle read. A second
client repointing that root makes signing fail with `casita-root-conflict`;
Mantle leaves the second client's target unchanged. A successful replacement
leaves the old envelope unrooted for later collection, not immediately deleted.

Native store archive import publishes one path per commit in archive order, so the 1,024 bound does not limit the archive size.
If a later path fails, the paths before it stay published.
Nario v2 import publishes all new paths in one commit or none of them; a conflicting root or a later path that fails verification publishes nothing.
A Nario v2 import with more than 1,024 paths that are not already in the store fails with `casita-batch-limit` before Mantle stages anything in Casita, and Mantle never splits it.
Paths that the import finds already in the store are checked, skipped, and not counted.

Every read checks out the envelope, re-ingests `content`, and re-measures its NAR; Mantle does not reuse NAR facts that Casita caches.
The read checks the root target before and after checkout, the two envelope entries, canonical `pathinfo.json`, the root name against the store path, the content-address identity, a trusted signature, the content node, and the NAR size and SHA-256.
Failures report `casita-envelope-invalid`, `casita-signer-untrusted`, `casita-nar-mismatch`, or `casita-root-conflict`.
A retained path without its root reports `casita-root-missing`.
Before `store archive export` writes any bytes, Mantle verifies the selected
Casita root and every referenced closure member under the current policy.
Revoking a reference's sole signer rejects the entire export with
`casita-signer-untrusted`, without changing either root or writing a partial
archive. Restoring the policy allows the same signed closure to export again.

Under `casita`, `store verify` also checks signatures against the keys named with `--trusted-public-keys`, in the configured `trusted-public-keys` file, or given with `--signing-key`.
It never adds or creates the local signing key on its own, so name the signer's public key when you run it.

## Garbage collection

Under `snix`, the execution plan ID binds the ordered reclaim observations,
including physical paths. An older plan issued before dead-blob observation
ordering is made deterministic may be rejected as stale even when its
candidates are unchanged. This rejection is safe: inspect a new
`store gc --dry-run` report and execute its newly issued plan ID rather than
retrying or overriding the old one.

Casita GC keeps the two-step flow: `store gc` writes a plan, and `store gc --execute --plan-id <blake3-plan-id>` runs it.
Under `casita`, `store usage` and `store gc` take the store mutation lock even when they only plan.
Execution replans under the lock, and any drift fails with `gc-plan-stale` before Mantle writes a fence or changes a root.
Casita output and castore payload root targets are bound into the plan ID. GC verifies every payload root before planning and takes the Rust cache retention live nodes into account; an unretained payload can be collected while a retained one remains rooted.

Execution then writes `casita-gc-fence.json`.
It removes each candidate root only while the root still has its planned target, records each outcome in `casita-gc-fence.progress`, deletes dead exports and index entries, and runs Casita collection.
A root with another target stops the execution and reports `casita-root-conflict`.
If the collector is busy, the report has `execution_complete: false` and the fence stays.
A Casita GC plan holds at most 65,536 candidates.
Casita candidates report no reclaimable byte count; their reclaim observations carry `casita-physical-size-not-observed`.

While a fence is pending, reads such as `store list` and `store info`, admissions, and root registry changes fail with `gc-recovery-required`.
The next guarded store command, such as `store usage`, `store gc`, `store pin`, or `store unpin`, finishes the pending fence first.
Recovery never removes a root.
It cleans up only fenced roots that are already absent, fails with `casita-root-conflict` when a fenced root changed, and then runs collection.
Recovery checks every fenced root against its expected target before cleaning any removed root; a later repointed root leaves earlier exports and indexes untouched and keeps the fence pending for operator review.
The retained set comes from the same root records and pins as under `snix`.

## Migrate from Snix to Casita

Move content with the verified store archive; Mantle has no in-place conversion.

```bash
# 1. Export the closure from the Snix state directory.
mantle --store-backend snix --state-dir OLD --store OLD_OUT \
  store archive export --to closure.mnar <selector>...

# 2. Provision destination trust before the first Casita open.
mkdir -p NEW
printf '%s\n' '<name>:<base64-ed25519-public-key>' > NEW/casita-trusted-public-keys

# 3. Import with the same key, then pin the paths to retain.
mantle --store-backend casita --state-dir NEW --store NEW_OUT \
  store archive import --from closure.mnar \
  --trusted-public-keys '<name>:<base64-ed25519-public-key>'
mantle --store-backend casita --state-dir NEW --store NEW_OUT \
  store pin <logical-store-path>

# 4. Check signatures, then re-export and compare.
mantle --store-backend casita --state-dir NEW --store NEW_OUT \
  store verify --trusted-public-keys '<name>:<base64-ed25519-public-key>' <selector>
mantle --store-backend casita --state-dir NEW --store NEW_OUT \
  store archive export --to check.mnar <selector>...
cmp closure.mnar check.mnar
```

Write the public key of every signer on the exported paths.
`store info` under `OLD` lists each path's signatures, and `mantle attest key-show --signing-key <key-file>` prints a Mantle key in `name:base64` form.
Confirm each key through an independent reviewed channel, and do not copy `signing-key` or any trust file from `OLD`.
Because `NEW` now has a policy file, local builds there publish only after you add the public key of the key that signs them.

The import and the later Casita commands do not change `OLD`.
The import keeps the original signatures, adds none, and registers no root, so pin every path that must stay retained.

## Pinned Casita dependency

Mantle depends on `casita` 0.1.0 from `https://github.com/cachix/casita` at revision `90404fcb1cfb3d83f2233715448dfefe913f5fd1`, with default features off and only `native` and `experimental` enabled.
Casita has no release, and upstream says that its `experimental` API may change between revisions.
The pinned revision does not compile unmodified in Mantle's dependency graph.
The tracked one-line `patches/casita-blake3-finalize.patch` is applied to the
Nix Crane `overrideVendorGitCheckout` source and to the ignored checkout-local
`vendor-deps/` closure. Nix admission checks the upstream revision, feature
set, `rust-version = 1.94.1`, original `nar.rs` digest, and patch digest before
vendoring. The generator checks these inputs and the patched file digest, then
updates Cargo's per-file vendor checksum. Do not edit a vendor snapshot or use
a fork.

Casita's `native` graph selects `bao-tree` 0.16.1. Mantle carries the exact
Bao candidate Git tree in `third_party/bao-tree/` and selects it with the root
`[patch.crates-io]`, rather than requiring an unpublished fork or a machine-local
path. The original upstream base is
`2be9abd144783455606424424c29bd3a57f926f8`; the local validation-preserving
candidate is `eecfbbb458cc684fd85e056881580d307a1d1868`. The tracked
`third_party/bao-tree-source.json` records BLAKE3 identity for every one of its
35 Git-tracked files, including both licenses, the unchanged Bao package and
workspace manifests, source, tests, examples, and regression fixtures.
`python3 scripts/import-bao-tree.py check` verifies the committed snapshot
offline without the candidate checkout; pass a checkout path only to compare
its clean, exact candidate revision and parent. An import refuses to replace
an existing snapshot. Neither a matching local receipt nor Mantle's review
confers upstream approval, publication, or release eligibility.

The Bao `validate` feature remains enabled by Bao's default feature set and
uses `futures-lite`, not `genawaiter`. `genawaiter` remains elsewhere in the
locked Turso graph; do not infer its global absence from the Bao path. The
checkout-local vendor generator verifies the local Bao snapshot before
regenerating or checking external dependencies. After a lock change,
`python3 scripts/vendor-deps.py refresh` preserves the former ignored closure
in a named sibling backup before installing a newly generated closure;
`check` then compares every generated entry. The Nix
`bao-source-integrity` check validates the tracked snapshot in clean source,
and the Casita Nix vendor/check rails still verify the separately pinned
Casita source and its exact `nar.rs` patch.

From `nix develop`, run `python3 scripts/vendor-deps.py generate` to create
the ignored checkout-local closure, or `python3 scripts/vendor-deps.py check`
to compare a fresh generation file by file and resolve locked offline Cargo
metadata. `generate` refuses to replace an existing directory. Run
`python3 scripts/vendor-deps.py dev-shell-check` to verify that plain Cargo
metadata (without `--config`) resolves the pinned, patched Casita from Crane's
immutable default dev-shell source replacement rather than an unpatched git
checkout. Compile the immutable default shell source with
`cargo check --locked -p crunch-store` without `--config`. For an explicit
checkout-local compile, use a separate empty Cargo home and a fresh target
directory. Combining both source maps defines the same git sources twice;
reusing build-script output can also try to overwrite read-only copied vendor
headers.

```bash
mkdir -p target
CARGO_HOME="$(mktemp -d)" \
CARGO_TARGET_DIR="$(mktemp -d "$PWD/target/checkout-vendor.XXXXXXXX")" \
  cargo check --locked -p crunch-store --config .cargo/vendor-config.toml
```

`nix build .#checks.x86_64-linux.casita-vendor-closure`
builds the clean-source Crane vendor closure, while
`nix build .#checks.x86_64-linux.casita-crunch-store-check` also runs
`cargo check --locked -p crunch-store` using that closure. These are separate
compile and source-admission checks, not substitutes for one another.

The Nix Tigerstyle consumer passes that same Crane vendor config directly to
its Cargo check; it does not resolve a second unpatched Casita checkout. Its
source map uses revision `90404fcb1cfb3d83f2233715448dfefe913f5fd1` and
the original tracked `patches/casita-blake3-finalize.patch` (SHA-256
`c0def0527dcc56beafa8d3f89c418a071b1d93df418245ebcd6a69aeb580840a`).
An evaluated source map is not a passing Tigerstyle or full-flake gate.

Crane's `buildDepsOnly` normally replaces path sources with stub Rust files.
`flake.nix` restores just the complete tracked Bao snapshot into that dummy
dependency source before compiling Casita; otherwise the cached Casita
compiles against an empty Bao lib and fails its public API imports. The
final package and check still consume the normal filtered real source tree.

Both self-build Rust source-bundle plans select a final 1.94.1 compiler.
Their plan values alone do not establish that the source-built compiler exists
or that any of these three Cargo checks passed.

The [dependency audit](dependency-audit.md#live-combined-workspace-admission-2026-10-06) records the configured locked offline `cargo-deny` exit 0 without new waivers on 2026-10-06, alongside historical failures; it does not qualify source-built Rust, hosted CI, or release.

## Validation

Run these rails from `nix develop`:

```bash
cargo test -p mantle --test store_archive_cli casita_
cargo test -p mantle --test store_gc_cli
cargo test -p mantle --test integration_build signed_fixed_output_cli_cache_restores_missing_export_in_fresh_processes
cargo test -p mantle --test integration_build casita_explicit_trust_policy_revokes_local_signer_across_fresh_processes
cargo test -p mantle --test integration_build casita_legacy_bootstrap_fetch_uses_durable_local_signer_and_exclusive_policy
cargo test -p mantle --test integration_build casita_external_config_signer_requires_destination_policy_for_build_and_read
cargo test -p mantle --test integration_build independently_built_backend_closures_have_identical_archives_and_offline_source_reports
cargo test -p mantle --test store_archive_cli admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail
cargo test -p mantle --test attest_cli casita_rejects_substituted_output_signed_only_by_cache_key_outside_policy
cargo test -p mantle --test integration casita_repair_final_nar_rejects_before_creating_or_mutating_state
cargo test -p mantle --test integration store_repair_final_nar_dry_run_then_execute_is_explicit_and_idempotent
cargo test -p crunch-store --lib repair::tests::casita_repair_library_entrypoints_reject_without_publishing
cargo test -p crunch-store --lib casita::tests::
cargo test -p crunch-store --lib overlay::tests::
cargo test -p crunch-store --lib archive::tests:: -- --test-threads 1
cargo test -p crunch-store --lib nario::tests:: -- --test-threads 1
cargo test -p crunch-store --lib gc::tests::casita_ -- --test-threads 1
cargo test -p mantle --test rust_plan_cli casita_rust_plan_reuses_compiled_unit_in_fresh_process
cargo test -p mantle --test example_projects casita_local_action_result_reuses_verified_output_after_export_removal
cargo test -p crunch-rust-cache --lib casita_
```

The `add-store-backend-selection` and `adopt-casita-store-backend` Cairn evidence records each passing run of these rails with the source snapshot it ran against; those runs cover:

- `store-backend-mismatch` from the CLI and in both directions at the library level, with `store-identity.json` unchanged; `store-backend-unknown` before any state exists; and a `MANTLE_STORE_BACKEND` variable that leaves the `snix` default in place; the passing `store_gc_cli` run (14/14, `tests/store_gc_cli.rs`) also shows wrong-backend file and project/inline builds and a Snix build with a read-only Casita base refusing before generating `signing-key`, leaving the writable state and, for the base case, the base byte-identical, with the original Snix store still opening;
- an unrecorded directory with real Snix databases, blobs, and PathInfo that `snix` claims: writing the `mantle-store-state-v2` record changes no other file, and after a real open the earlier PathInfo and an extra file are intact;
- unrecorded directories that hold a `casita` entry, alone or next to Snix files, under either backend, and unrecorded unknown content under `casita`, which fail with `store-backend-mismatch`, change nothing, and create no output directory;
- `snix` profile fields in JSON and human `store info` output, read through an overlay base that stays unchanged;
- rejection of `--base-store`, `store archive import --trust-unsigned`, and `--nario-trust-unsigned` under `casita` before any state exists;
- the migration above with one fixture key: `store info` profile fields and the original signatures, `store roots`, `store verify` with `trusted_signatures=1/1` for the archive signer and `trusted_signatures=0/1` for another key, a byte-identical re-export, an unchanged destination policy, no `signing-key` created in `NEW`, and no change to `OLD` after the export;
- equal `store usage` and GC plan facts, GC execution, a guarded `store usage` that clears an empty pending fence, `gc-plan-stale` without a fence or root registry change, and removal after `store unpin`;
- a CLI `store gc` under `casita` with one pinned and one unpinned signed output: the plan lists only the unpinned output, and executing that plan reports `casita-collection`, removes the unpinned output and its export, and leaves the pinned output's NAR SHA-256, NAR size, signatures, export, and `store verify` result of `trusted_signatures=1/1` unchanged;
- historical interim-gate rejection of Rust cache modes under `casita`, before durable castore payload roots were implemented; the current capability profile and focused Rust cache reuse fixtures supersede that gate;
- a signed fixed-output `file://` fetch under each backend: after only its physical export is deleted, a fresh `mantle --json build` process reports `built_total: 0` and `cached_total: 1`, restores identical bytes with the same store path, NAR SHA-256, NAR size, and signature under both backends, and passes `store verify` with `trusted_signatures=1/1`; under `casita`, the policy file stays byte-identical and no `pathinfo.redb`, `directories.redb`, or `blobs` appears in the state directory;
- an independently built, signed two-path offline source closure under `snix`
  and `casita`: one local HTTP response is captured before its server closes;
  both backends then report equal full source-preflight JSON, build the root
  with the fetched input, and export byte-identical closure archives with
  equal listed paths while the source endpoint is unavailable;
- a real Nix-compatible binary-cache substitute signed by a cache key that
  transport trusts but the destination Casita policy does not: admission
  fails with `casita-signer-untrusted` and no output root, then adding the
  full cache key to that policy admits the same cached bytes without a build;
- a `casita` build whose policy file omits the local signer, which fails with `casita-signer-untrusted`; after the key is listed, the build publishes; removing the key makes a fresh `store info` fail with `casita-signer-untrusted` while the export stays in place, and listing it again restores the read with the same signature, with Mantle leaving the policy file unchanged throughout;
- a new OS process that reads a signed output after its physical export was deleted, with an `ActionResultPort::probe_outputs` check that reuses it, and a new process that reads a signed output without a content address;
- second-client root races across the real batch admission adapter:
  absent-root publication and identical-root idempotence without a
  revision-changing write, then a competing client changes the target
  between unrooted staging and conditional commit; Mantle reports
  `casita-root-conflict`, publishes neither selected root, preserves
  the competitor's signed target, and collection reclaims both failed
  staged envelopes. A separately stopped staging process publishes
  neither root nor PathInfo, and collection allows a signed retry;
  a later bad NAR batch also leaves its earlier valid path unpublished
  from a newly opened process, with permanent roots surviving Casita
  disk-pressure collection;
- a second Casita client that replaces content or metadata, writes non-canonical or oversized `pathinfo.json`, writes signed facts that do not match the content, points the root at another path's envelope, or removes the root, each rejected with its blocker, also in a new process;
- `gc-recovery-required` while a fence is pending, with the root registry unchanged by fenced pin, unpin, and migration attempts;
- library-level `store sign`, which republishes the envelope with the same content, leaves no root on the old envelope, and verifies in a new process;
- the `store sign` CLI command in the signed two-backend conformance rail:
  signing an imported output with an independently provisioned second key
  adds its signature; `store info` reports both and `store verify` accepts
  both trusted signatures under each backend;
- library-level `store sign` against a root that a second client changed after Mantle read it, which fails with `casita-root-conflict` and keeps that client's root;
- `store repair-final-nar` under `casita`: the dry run and `--execute` fail with `casita-repair-final-nar-unsupported` without creating the state or output directory, and against existing state and output directories each leaves every file and directory entry in both unchanged; the library inspection and execution calls on an open Casita store fail with the same blocker, publish nothing, and leave every file and directory entry in its state and output directories unchanged;
- default `snix` final-NAR repair: a dry run reports `would-repair` without changes, `--execute` repairs the stale facts, and a second `--execute` reports `current`;
- a key added to the policy file after the store was opened, which the open handle then accepts for admission and reads; removing it makes that handle's reads, library `store verify`, and GC planning fail with `casita-signer-untrusted` with the root unchanged, and listing it again restores reads;
- policy files that are empty, malformed, over 64 keys, over 65,536 bytes, or a dangling symlink, which fail with `casita-trust-policy-invalid` before the `casita` directory exists and stay unchanged;
- without a policy file, an output signed by the local signing key that a new process reads back while no policy file appears;
- `mantle bootstrap --fetch` under `casita` with `CRUNCH_CONFIG_DIR` unset, from a copy of `bootstrap/seed-legacy.ncl` that points at a local fixture tarball instead of the pinned musl.cc tarball: in a new state directory without a policy file, it creates `<state-dir>/signing-key` with mode `0600`, writes a seed file with `/crunch/store/` paths, and publishes one raw seed that `store verify` in a new process accepts with `trusted_signatures=1/1`; a second run reports the raw seed and the reduced provider cached with the key and the seed file unchanged; in a new state directory whose policy file lists only another key, it fails with `casita-signer-untrusted`, writes no seed file, leaves the policy file unchanged, and neither publishes nor exports the raw seed;
- CLI import rejections for a missing policy, a key that the policy lacks, a policy key with the same name but different bytes, an archive signer that the import does not name, and an unsigned archive record under a valid policy and key, each without changes to the source state directory or to the destination's Mantle-owned state files, output directory, or policy file;
- library import rejection of unsigned archive and Nario v2 records;
- Nario v2 batches of 1,024 and 1,025 new paths, a conflicting root,
  and a later invalid signed envelope after the first was staged: the
  accepted batch is visible with all 1,024 envelopes verified in one
  reopened Casita snapshot, while the late failure publishes no root and
  collection reclaims its unrooted envelope. The pinned Casita
  `publish_if_roots_match` contract applies all those root changes in
  **one metadata publication**; the global repository revision may also
  advance during unrooted staging, and there is no historical per-root
  revision API (see Cairn Run 61);
- GC plan identity, a GC root inventory that verifies 1,025 signed roots
  published in two atomic batches, a clean plan-then-execute that
  collects an unretained signed output and keeps the retained one's
  PathInfo and NAR facts, stale plans including an output root repointed
  after planning, a busy collector, recovery after partial root removal,
  symlinked or oversized outcome journals, and equal candidates with
  `snix` for identical outputs; the guarded recovery fixture additionally
  checks unchanged roots, fence and indexes for unguarded planning and
  execution, fresh-process read/export/admission/registration blocked by
  a pending fence, and a real guarded CLI recovery that clears that fence.

The current Rust cache parity fixtures use real `rustc` through a counted wrapper: a fresh `rust-plan` process restores an evicted unit with the same artifact digest and **no new compiler invocation**. A second Casita client can repoint its retained `node.postcard` or remove the root; fresh `rust-plan` and GC processes then reject the payload before recompilation or a GC fence. Local and shared-origin cache library fixtures restore from a fresh process. The PathInfo-only Builder fixture independently reuses a signed action result after its physical export disappears and checks that neither build published a `mantle/castore/` root.
The `adopt-casita-store-backend` Cairn evidence also keeps the commands and output of these one-off smokes from 2026-09-30, which are not checked-in rails:

- Runs 30 and 31 built `examples/hello.ncl` in the local `bwrap` sandbox under each backend, Run 30 as shipped, which is content-addressed, and Run 31 from a temporary copy with `addressing_mode = 'input-addressed`, each with one explicit signing key and a matching Casita policy: after only the physical output was deleted, a new `mantle --json build` process reported `built_total: 0` and `cached_total: 1` and restored `Hello, mantle!`, and in each smoke both backends produced the same store path, NAR SHA-256, NAR size, and signature, and `store verify` reported `trusted_signatures=1/1`;
- Run 40 built the shipped `examples/hello.ncl` under `casita` in a new state directory with `CRUNCH_CONFIG_DIR` unset, no `--signing-key`, and no policy file: Mantle generated `<state-dir>/signing-key` and published the output under that key without creating a policy file; after only the physical output was deleted, a new process reported `built_total: 0` and `cached_total: 1` and restored `Hello, mantle!`; `store verify` with the generated key's public half reported `trusted_signatures=1/1`; and no Snix database or blob directory appeared in the state directory;
- Run 52 ran `mantle bootstrap --fetch` under `casita` with the same kind of fixture as the bootstrap rail above: `store list` without `--store-prefix /crunch/store` failed with `store-identity-mismatch`, `store info` showed the raw seed signed by the generated key, a second run kept the PathInfo unchanged, `--store-backend snix` failed with `store-backend-mismatch` without writing a seed file or changing the identity file, and after a policy file that lists only another key was added, the next run failed with `casita-signer-untrusted` while reading the already published raw seed, wrote no seed file, and left the policy file and the key unchanged.

These paths have no passing validation yet:

- `mantle build` for derivation shapes beyond the fixed-output fetch, the
  two `examples/hello.ncl` variants, and the signed offline two-path source
  closure above;
- `mantle bootstrap --fetch` under `snix`, and under `casita` with the pinned
  musl.cc tarball rather than the local fixture tarball;
- `store push`, `store pull`, and remote substitution;
- a policy replacement concurrent with an in-flight trust check (removal
  between two operations on one open handle is covered);
- moves from `casita` back to `snix`.

## Portability and non-claims

Casita evidence comes from local Linux filesystems only; no evidence covers network filesystems or other hosts.
A Casita state directory is bound to the on-disk format of the pinned revision, so move content between revisions or hosts with the store archive, not with file copies.

Passing rails do not prove Casita correctness, stability of the experimental API, key ownership, signer honesty, revocation freshness beyond the policy file that each check read, durability under power loss, crash safety beyond the tested interruptions, interchangeability beyond the declared profiles, output correctness, GC safety in general, sandboxing, or release eligibility.
