# ADR 0082: Select store backends explicitly and admit Casita as a pinned backend

## Status

Proposed (2026-09-30)

Implementation is partial, and this record is not Accepted. In
`adopt-casita-store-backend`, T2.5, T2.10, T2.13, T2.15, T3.3, T3.4, T3.5,
and T3.6 are checked from recorded runs; 28 tasks remain open. In
`add-store-backend-selection`, T1.1–T1.4 and T4.1–T4.2 are checked from the
pre-selection goldens, the selection/profile contracts, and operator-facing
documentation. Its remaining tasks stay open until their combined-tree proof
is recorded in that change's `tasks.md`. The record claims only tested behavior
within [Evidence scope](#evidence-scope). The final repository quality gates
for both changes have not all passed: the 2026-09-30 runs recorded strict
Clippy, the first-party workspace suite, and `cargo deny` failures. Targeted
2026-10-04 results do not imply workspace gates or release eligibility.
This record can become Accepted only after both changes complete with the
evidence named under [Evidence scope](#evidence-scope).

## Context

Mantle persists admitted outputs in one engine, and the choice is implicit.
`StoreHandle::open` opens Snix services under `--state-dir`: `blobs/`,
`directories.redb`, and `pathinfo.redb` (`crates/crunch-store/src/handle.rs`).
GC rewrites both databases and sweeps `blobs/`
(`crates/crunch-store/src/gc.rs`). The identity record `store-identity.json`
(`mantle-store-state-v1`) binds the logical prefix and trust policy but not
the engine, so a second engine could open the same directory undetected.
About 30 files build `StoreConfig` or call `StoreHandle::open`, and five
launchers pass `--state-dir` or `--store-prefix` to child processes.

Casita (<https://github.com/cachix/casita>) is a pre-release
content-addressed object repository. It verifies object graphs before
publication, keeps named roots, collects unreachable data, and leaves root
lifecycle decisions to the application. That split fits Mantle: Mantle keeps
build execution, NAR measurement, signing, trust, retention, and GC planning,
and Casita keeps verified bytes and named roots. Casita has no release tag.
The reviewed revision is `90404fcb1cfb3d83f2233715448dfefe913f5fd1` (`main`,
2026-09-30).

At that revision the supported API (`native` feature) imports a directory
under a name with `FilesystemImport`, and the import's final commit sets that
root without a precondition. The supported API can also set or remove a root
conditionally (`compare_and_set_root`, `remove_root`, `commit` with root
checks), but a new target must already exist, so new content first needs
another named root. Session-protected unrooted staging and conditional
publication inside that session exist only behind the `experimental`
feature. Upstream documents that API as one that "may change between
revisions".

## Decision Drivers

- Each state directory has one durable authority, and a wrong-engine open
  fails before any effect.
- The operator chooses the engine. Library code cannot choose one by
  omission.
- Backend choice changes no Mantle identity: store paths, derivation hashes,
  NAR facts, signed PathInfo, trusted-key admission, action refs, report
  schemas, or the capability views of
  [ADR 0058](0058-limit-store-access-with-concrete-capability-views.md).
- Mantle keeps admission, trust, retention, and GC authority
  ([ADR 0064](0064-explain-store-retention-before-garbage-collection.md)).
- Mantle never overwrites a root that another writer published, including
  another Casita client.
- Trust for later reads comes only from state that the destination operator
  provisions, never from a command-time key or from the source.
- A backend declares the optional behavior it lacks, and a request for that
  behavior fails closed.
- Upstream instability stays behind one exact pin and a listed API surface.

## Decision

Mantle selects its store backend explicitly and admits Casita as a second,
pinned durable backend. `snix` stays the default and keeps its formats. The
Cairn changes `add-store-backend-selection` and `adopt-casita-store-backend`
carry this decision.

### Backend selection

1. Mantle owns a closed identifier set: `snix` and `casita`. The global
   option `--store-backend <id>` selects one, and only the CLI composition
   root applies the default `snix`. `StoreConfig` requires the identifier,
   and no library constructor supplies one. Environment variables, directory
   contents, and Cargo features never select a backend. An unknown identifier
   fails with `store-backend-unknown` before any state is read or created.
2. Every launcher that starts a Mantle child against the same state
   directory forwards the identifier. This covers the local remote worker,
   bootstrap validation, the source-built fixed-point shell, the transcript
   command, and the Rust cache daemon.
3. A new state directory gets a versioned identity record with a `backend`
   field. A legacy `mantle-store-state-v1` record means `snix`, and ordinary
   commands never rewrite it. A non-Snix backend never writes the legacy
   schema. The rule that v1 means `snix` is permanent.
4. A pure open decision runs before directory creation, lock acquisition,
   identity writes, and service opens. It fails with `store-backend-mismatch`
   and changes no file when the recorded backend differs from the selection,
   when a directory without an identity record holds the `casita` repository
   marker (whichever backend is selected), when `casita` is selected and a
   directory without an identity record holds anything besides the mutation
   lock, the signing key, and trust-policy files, or when an overlay layer
   records another backend. With `snix` selected, a populated directory without
   an identity record or `casita` marker is legacy Snix state: before the Snix
   services open, Mantle adds a `snix` identity and changes no existing file,
   as the pre-selection code did. The Snix services may then update their own
   redb files while opening.
5. No local fallback exists. Lookup, closure resolution, export, GC, repair,
   and inspection use only the selected backend and same-backend overlay
   bases. A miss or integrity failure is reported for that backend. Remote
   substitution keeps its policy and admits into the selected backend.
   Content moves between local backends only through explicit operator
   commands.
6. Each backend declares a Mantle-owned capability profile. Every backend
   implements the core capabilities: output admission, lookup, fresh-process
   reopen, closure resolution, export, store archive export and import, GC
   planning, and plan-bound GC execution. Final-NAR repair is listed per
   backend: `snix` lists `store-repair-final-nar` among its core capabilities,
   and `casita` omits it and fails closed. `overlay-composition` and
   `atomic-batch-import` are optional. A request for an undeclared optional
   capability fails with a backend-specific blocker before any state access,
   and `store info` lists the profile. Backends are interchangeable only within
   their declared profiles.
7. One backend-parameterized conformance rail covers the core capabilities,
   stale-plan rejection, identity checks, and, per profile, each optional
   capability's fixtures or its fail-closed fixture. The `snix` run must
   equal goldens recorded before the change.

| Backend | `overlay-composition` | `atomic-batch-import` | `unsigned-admission` | `rust-unit-cache` |
| --- | --- | --- | --- | --- |
| `snix` (CLI default) | Declared | Declared (`put_batch_atomic`), no backend bound | Declared | Declared |
| `casita` | Not declared; `casita-overlay-unsupported` | Declared, at most 1,024 PathInfo records per batch; Nario v2 counts only paths it finds absent (`casita-batch-limit`) | Not declared; `casita-trust-unsigned-unsupported` | Declared unsupported while the interim guards persist; `casita-rust-cache-unsupported` |

### Casita backend

1. **Pin.** Mantle depends on package `casita` 0.1.0 from
   `https://github.com/cachix/casita` at revision
   `90404fcb1cfb3d83f2233715448dfefe913f5fd1`, with default features
   disabled and exactly `native` and `experimental`. Cargo manifests and
   `Cargo.lock` record the exact revision, never a branch or tag. A Nix
   admission assertion checks the revision and the feature set. The Crane
   `vendorCargoDeps` closure and a repository-owned generation and check path
   for `vendor-deps/` vendor the exact locked sources. The `cli` (which
   includes `casita ipc`), `git`, `git-fetch`, `git-http`, `ssh`, `s3`, `oci`,
   and `fuzzing` features stay off, and Casita mode opens no network
   connection. Changing the revision or the features is a reviewed change
   that updates this ADR.
2. **Repository location.** The Casita repository uses Casita's standard
   local profile in a dedicated subdirectory of `--state-dir`.
   `Repository::local` creates `blobs/` and `casita.sqlite` under its root,
   so a shared root would collide with the Snix `blobs/` marker. Mantle-owned
   files stay beside it: `store-identity.json`, `casita-trusted-public-keys`,
   root and retention records, CA mappings, action-result records,
   attestations, source bundles, and the GC fence. Their store-path
   references resolve only through Casita roots.
3. **One root binds content and signed PathInfo.** Each admitted output has
   one permanent root, `mantle/outputs/<40 lowercase hex of the store-path
   digest>`. Its target is a directory envelope with exactly two entries:
   `content` (the output file, directory, or symlink) and `pathinfo.json`
   (the signed PathInfo in serde JSON encoding; see the postcard alternative
   below). Casita mode keeps no other durable PathInfo mapping. One
   publication makes content and PathInfo visible together, and one root
   removal hides both. The envelope layout and encoding are a Mantle-owned
   durable format. Equal PathInfo values must encode to equal bytes, because
   idempotent admission and replacement compare envelope identities.
4. **Admission stages, then publishes conditionally.** Admission starts only
   after the existing build, NAR measurement, and signature admission succeed
   in session scratch. Under `StoreMutationGuard`, Mantle opens one Casita
   mutation session, stages the envelope unrooted, checks its NAR SHA-256 and
   size against the signed PathInfo, and publishes with one conditional
   publication that expects the root name to be absent. A mismatch commits
   nothing. If the existing root targets an identical envelope, admission
   succeeds without a write. Otherwise it fails with `casita-root-conflict`,
   and the other writer's target stays. An unpublished envelope stays
   unrooted and becomes collectible once the session ends.
5. **Batches publish all-or-none.** Nario v2 import, and any import whose
   paths must appear together, stages every envelope in one session and
   publishes every new root with one multi-root conditional publication. A
   staging, verification, or root-mismatch error on any path commits no root
   of the batch. A path whose root already targets an identical envelope is
   verified and left unchanged. At the pinned revision one commit accepts at
   most 1,024 root changes and 1,024 expectations
   (`FormatLimits::max_root_changes`), while Mantle's Nario v2 reader accepts
   up to 100,000 records (`MAX_RECORDS` in `crates/crunch-store/src/nario.rs`).
   A larger Casita-mode batch fails with `casita-batch-limit` before staging,
   because splitting it would break all-or-none. The bound counts every
   PathInfo passed to one batch. Nario v2 import checks and skips paths it
   finds already present, so only paths it found absent count.
6. **Every read verifies.** Before a lookup, closure walk, export, or cache hit
   uses an output, in the same process or a fresh one, Mantle checks the
   envelope entries and types, decodes `pathinfo.json` (at most 1 MiB, and only
   if its bytes are the canonical JSON encoding), matches the root-name digest
   to the store path, checks content-address facts, and compares the NAR
   SHA-256 and size of `content` with the PathInfo. It verifies signatures
   under the trust set as read for that verification, comparing full key
   material. When the state directory's `casita-trusted-public-keys` file
   exists, the trust set is exactly its keys, and the local signing key counts
   only if the file lists it. Without the file, the trust set is only the
   verifying key of `<state-dir>/signing-key`, so a new state directory can
   build and read its own outputs as long as its builds sign with that key,
   which is the default. When `CRUNCH_CONFIG_DIR` places the signing key
   outside the state directory, builds and `bootstrap --fetch` sign with a key
   outside the trust set and fail closed with `casita-signer-untrusted` unless
   the policy file lists that key; Mantle does not move, copy, or trust it on
   its own. This follows from the source (`src/signing_key.rs:89-91` and
   `:143-145`, `crates/crunch-store/src/overlay.rs:414-415`); no run covers it,
   and every recorded Casita run removes `CRUNCH_CONFIG_DIR`. The parent chose
   on 2026-09-30 to document this condition instead of changing the key
   location or the trust rule. Mantle never enrolls a key. Mantle publishes an
   envelope only when its PathInfo carries a valid signature from that set;
   otherwise admission or the read fails with `casita-signer-untrusted`,
   including for substituted outputs. Mantle validates the file at store open
   and reads it again, with the same parser and 65,536-byte bound, for every
   admission, read, verification, and import preflight. It keeps no trust set
   between verifications, so an added or removed key applies at the next
   verification, in an open handle too. A malformed, empty, oversized, or
   over-limit file fails the open, or the verification that reads it, with
   `casita-trust-policy-invalid`. Mantle does not lock the file, so an
   operation that verifies several outputs while the operator edits it may
   observe more than one version. Each read checks the root out into per-read
   scratch, re-ingests `content`, requires the resulting node to equal the
   PathInfo node, measures the NAR again, and confirms that the root still has
   the target it read. Every read is therefore a full content audit, and
   `store verify` needs no separate scrub. Failures report
   `casita-envelope-invalid`, `casita-nar-mismatch`, or, for a root that
   changed during the read, `casita-root-conflict`.
7. **Snix is session scratch, not a second authority.** Snix blob,
   directory, and PathInfo services run in memory or in a per-session
   directory outside `--state-dir`. They are deleted when the session ends
   and never answer a lookup. Sandbox inputs are rehydrated from Casita into
   that scratch. Mantle never creates `pathinfo.redb`, `directories.redb`, or
   a Snix `blobs/` directory in a Casita state directory.
8. **Mantle is the only retention authority.** Every Mantle root is
   permanent. Mantle never marks a root evictable and never uses Casita
   access times or disk pressure as retention input. The retained set comes
   from the same root records, pins, and project roots that `snix` uses, so
   equal observations give equal GC decisions.
9. **GC releases only planned outputs.** GC keeps Mantle's two-step flow:
   `store gc` writes a plan, and `store gc --execute --plan-id <id>` runs it.
   `crunch-gc-core` plans over verified envelope facts and records each
   candidate's exact root target. Execution takes `StoreMutationGuard` and
   re-observes roots, envelopes, references, and targets. Any drift fails
   with `gc-plan-stale` before a mutation (the `snix` backend keeps its
   existing `stale-gc-plan` for the same condition). Execution then durably
   writes a fence record (plan identity, root names, expected targets, store
   paths), removes each candidate root through its own conditional change
   that expects the planned target, deletes dead exports and index entries,
   records every outcome in the fence, and only then runs Casita collection.
   A mismatch leaves that root published and blocks a success claim. A busy
   collector leaves reclaim incomplete, and the report says so. Casita GC,
   including a dry run, runs only under `StoreMutationGuard`; a call without
   it fails with `casita-gc-guard-required`. While a fence is pending, lookup,
   listing, export, admission, root registration, and rehydration fail with
   `gc-recovery-required` unless the command first recovers the fence.
   Recovery runs only under the guard, at the start of a guarded command and
   before it plans, admits, or registers anything: `store gc`, `store usage`,
   guarded store commands such as `store pin`, the bootstrap build, provider
   adoption, and the remote executor. `store gc` is the explicit remedy.
   Recovery finishes cleanup only for fenced roots that are already absent,
   fails with `casita-root-conflict` on a root with another target, and never
   removes a root. Mantle removes no root outside `mantle/outputs/` and
   `mantle/castore/`.
10. **Migration is explicit, verified, and provisioned first.** A
    Snix-to-Casita move uses the existing verified archive transport:

    ```bash
    printf '%s\n' '<name>:<base64-ed25519-public-key>' \
      > NEW/casita-trusted-public-keys
    mantle --store-backend snix --state-dir OLD --store OLD_OUT \
      store archive export --to closure.mnar <selector>...
    mantle --store-backend casita --state-dir NEW --store NEW_OUT \
      store archive import --from closure.mnar \
      --trusted-public-keys '<name>:<base64-ed25519-public-key>'
    mantle --store-backend casita --state-dir NEW --store NEW_OUT \
      store pin <path>
    mantle --store-backend casita --state-dir NEW --store NEW_OUT store verify
    ```

    The operator provisions `casita-trusted-public-keys` in the destination
    state directory before the import; Mantle never creates, modifies, or
    extends it. Once it exists it is the whole trust set, so a destination
    that also builds must list its own signing key's verifying key there
    (`attest key-show` prints it). Until then its local builds fail with
    `casita-signer-untrusted`. The file uses the existing
    `overlay-trusted-public-keys` format and limits: Nix-style
    `<name>:<base64-ed25519-public-key>` entries separated by whitespace or
    commas, `#` comments, and 1 to 64 keys.
    `--trusted-public-keys` is an import-time policy only and never becomes
    durable signer authority. Before any Casita mutation session opens, the
    import fails with `casita-trust-policy-missing` when the file is absent
    and with `casita-import-key-unauthorized` when a named key is not in it.
    Because import adds no signature, `--trust-unsigned` and
    `--nario-trust-unsigned` fail with `casita-trust-unsigned-unsupported`
    before any state access: an unsigned path could never pass a later read.
    Nothing crosses from the source. Mantle copies no signing key, trust
    file, root record, or retention record, does not trust the source's local
    signing key implicitly, and keeps the source directory byte-identical.
    Original signatures are kept, and none are added. Opening, lookup, and GC
    never migrate content.
11. **Overlay composition is unsupported.** `--base-store` with
    `--store-backend casita` fails with `casita-overlay-unsupported` before
    any layer opens. Admitting it needs a separate change.
12. **Casita stays in the store shell.** Only the Casita adapter in
    `crunch-store` names Casita types. Capability views, application ports,
    and reports expose no Casita repository, session, reader, key, report, or
    error, and `tools/check_store_capability_boundary.rs` rejects escapes.

### Experimental Casita API

The adapter and its tests use exactly the experimental items below; test-only
items are marked. An item counts as experimental when the `experimental`
feature gates it or when it is reachable only through a type exported from
`casita::experimental`. Using any other experimental item needs a reviewed
change that amends this table. Source paths are relative to
`crates/casita/src/` at the pinned revision. The table matches the adapter
source that the runs in the Casita change's `evidence/test-runs-2026-09-30.md`
exercised.

| Item | Source | Mantle use |
| --- | --- | --- |
| `experimental::Repository::local(root)`, typed `Repository<ChunkedBlobStore, TursoMetadataStore>` | `repository/open.rs` | The adapter's repository handle. The supported `casita::Repository` exposes no mutation session and cannot be built from this handle. |
| `Repository::metadata()` with the `MetadataStore` trait: `snapshot()`, then `root(name)` and `roots()` on the snapshot | `repository/mod.rs`, `metadata.rs` | Read a root's target before and after each checkout, list roots for GC planning and verification, and recheck fenced roots during recovery. |
| `Repository::checkout(key, target_dir)` | `repository/read.rs` | Check an envelope out into per-read scratch. Every read verifies that copy. |
| `Repository::limits()`, field `FormatLimits::max_root_changes` | `repository/mod.rs`, `format.rs` | The batch bound. A batch with more paths fails with `casita-batch-limit` before a session opens. |
| `Repository::mutation_session()`, returning `MutationSession` | `repository/mutation.rs` | One session per admission, batch, envelope replacement, and castore payload root. |
| `MutationSession::import` with `import::UnrootedFilesystemImport::new(envelope_dir)` | `repository/mutation.rs`, `importers/filesystem.rs`, `import.rs` | Stage an envelope without a name. It returns the envelope `ObjectKey` and always rereads files. The session protects the graph until the session ends; a published root keeps it afterward. |
| `MutationSession::publish_if_roots_match(staged, expectations, root_changes)` | `repository/mutation.rs` | Admission (one root), batch (many roots in one revision), envelope replacement, and castore payload roots. `staged` is empty because the session import already committed the envelope graph unrooted. |
| `RootExpectation { name, target }` | `repository/mutation.rs` | `target: None` for a new root. `target: Some(current)` for a batch path whose root already holds an identical envelope (checked, not changed), and for a replacement of a target that the process read and verified (`store sign`). |
| `RootChange::Set { name, target }`; test only: `RootChange::Remove { name }` | `metadata.rs` | Publish an output root, a castore payload root, or a replacement envelope. In tests, `Remove` plays another client that removes a Mantle root. |
| `ConditionalPublishResult::Committed(CommitResult)` and `ConditionalPublishResult::RootMismatch { .. }` | `repository/mutation.rs`, `metadata.rs` | A mismatch commits nothing and fails with `casita-root-conflict`, naming the first mismatched root. |
| `Repository::remove_root_if_matches(name, expected)` | `repository/mutation.rs` | GC removes each fenced root only while it still has the planned target. `None` means the root was absent or repointed. |
| `Repository::try_collect()` | `repository/collection.rs` | Collection after the fence and after recovery. An error, including a busy collector, leaves reclaim incomplete. |
| Test only: `Repository::root_retention(name)` | `repository/root_policy.rs` | Asserts that every Mantle root reads `RootRetention::Permanent`. |
| Test only: `Repository::open_payload(key)` | `repository/read.rs` | Asserts that collection removed a released payload and kept a retained one. |
| Test only: `DiskUsage::new(total_bytes, free_bytes)`, `DiskPressurePolicy::new(used_percent)`, `DiskPressurePolicy::collect_if_needed(&repository, usage)`, and `DiskPressureOutcome::Collected` | `collection.rs` | Force a real pressure-triggered collection from a supplied usage for T3.4. It runs `try_vacuum`, which keeps every named root. Casita's eviction of evictable roots is crate-private and not reachable. |

The adapter also uses the supported types `RootName` and `ObjectKey`, and its
tests use `RootRetention`. NAR facts come from Mantle's own measurement of
the re-ingested `content`, not from Casita's NAR API.

## Alternatives Considered

### Infer the backend, or select it by environment or Cargo feature

Rejected. Detection from directory contents lets partial or leftover state
pick the wrong engine. An environment variable selects outside the operator
contract. A Cargo feature fixes one engine per binary, but one binary must
open either backend.

### Default the backend in `StoreConfig`, or add the option to `store *` only

Rejected. A constructor default lets library code bypass the operator's
selection. Build, attest, remote, and cache commands also open stores.

### Require full Snix parity before admitting a backend

Rejected. It would block a durable backend on unrelated optional features.
Declared profiles keep each gap visible and fail closed.

### Rooted `FilesystemImport` after a lookup

Rejected. Its final commit sets the root with no expectation. If another
Casita client publishes the same name between Mantle's lookup and that
commit, the import replaces that target, and a later check cannot see what
was lost. In the local profile it also skips rereading files whose stat
identity matches Casita's ingest cache unless `reread(true)` is set.
`MultiRootFilesystemImport` publishes batches with the same unconditional
root changes.

### Rooted import under a Mantle staging root, then supported promotion

Rejected for now. Importing under a staging root and promoting it with
`compare_and_set_root` or `commit` uses only the supported API. An
interrupted admission, however, leaves a permanent staging root that holds
content until Mantle removes it. That needs a second root namespace, a
recovery sweep, and GC rules for it, and other Casita clients can see and
change staging roots. It is the first fallback to evaluate if a bump removes
or changes the session API.

### `CasitarImport` with absent destinations

Rejected. It requires absent destinations by default but adds an archive
transform with no gain over session publication.

### Casita IPC daemon or CLI subprocess

Rejected. The IPC daemon exposes import, restore, and checkout only, with no
root listing, removal, collection, or blob reads. A CLI subprocess adds an
ambient binary and coarse errors.

### Casita as castore only, or a PathInfo mapping beside roots

Rejected. Keeping `pathinfo.redb` creates two durable authorities with
separate failure domains. A mapping file or application-record namespace
could outlive or precede its content.

### Postcard encoding for the envelope PathInfo

Rejected. `PathInfo` carries `ca: Option<CAHash>`, and `CAHash` deserializes
through a `serde_json::Map` (`vendor/nix-compat/src/nixhash/ca_hash.rs`),
which needs self-describing deserialization that postcard does not implement.
The adapter's admission test failed to decode a postcard envelope with that
error before the cutover (`evidence/test-runs-2026-09-30.md` in
`adopt-casita-store-backend`, Run 2). JSON reads every `PathInfo` back at the
cost of larger envelopes. Mantle reads and writes only `pathinfo.json`, with
no postcard fallback.

### Enroll import keys for later reads

Rejected. If `--trusted-public-keys` enrolled its keys, one import command
would widen trust for every later read, cache hit, and closure walk in that
state directory without a separate, reviewable provisioning step. Only an
operator edit to `casita-trusted-public-keys` changes the destination trust
set.

### Copy trust from the source state directory

Rejected. The source's trust files and local signing key express the
source's policy, not the destination's, and copying a signing key would
duplicate signing authority. The destination names every signer it accepts.

### Trust the local signing key beside the policy file

Rejected. This was the earlier rule. It kept the local key trusted even after
the operator removed it from `casita-trusted-public-keys`, so the policy file
could not revoke it. A present file is now the whole trust set. Without a file,
only `<state-dir>/signing-key` is trusted, so a new state directory still
builds and reads its own outputs when its builds sign with that key.

### A dedicated `store migrate` command

Rejected. The verified archive transport already carries closures, NAR
facts, signatures, and trust checks.

### Casita evictable roots or disk-pressure eviction as retention

Rejected. Mantle is the only retention authority. Casita's pressure pass
releases only roots marked evictable, and Mantle never marks one.

## Consequences

- Every store-opening command and child launcher carries a backend, the
  operator command contract regenerates for the new global option, and about
  30 construction sites change. Active changes that touch those sites
  (`thin-cli-composition-root`,
  `resolve-content-addressed-inputs-before-dispatch`,
  `adopt-fault-injection-io-tests`, `add-retention-interest-records`) need
  ordered rebases.
- Existing Snix state keeps working without migration, and the rule that v1
  means `snix` never expires. That includes a populated directory with no
  identity record at all, as long as it has no `casita` marker: under `snix` it
  gains a `snix` identity, as at HEAD
  `7ec5177718a6950297e04eb4eb957a10b02e23ce`, where `ensure_store_identity`
  wrote a v1 record into any directory without one
  (`crates/crunch-store/src/overlay.rs:156-193`, called from
  `handle.rs:679-697`). The rule has a disclosed risk. Mantle cannot tell
  legacy Snix state from unrelated content, so selecting `snix` claims any
  identity-less directory without a `casita` marker. The store overlay policy
  only adds `mantle-store-state-v2` to its allowed schemas, and the trust
  policy identity is unchanged, so v1 records written at HEAD still pass the
  prefix and trust checks. The parent chose this rule on 2026-09-30 so that
  existing Snix stores can still be exported. Operators are not asked to copy
  state or run an older binary. Casita Run 27 shows the earlier code rejecting
  every populated identity-less directory. The cutover has since landed in
  `crates/crunch-store/src/overlay.rs` (`preflight_store_identity`). Its first
  fixture run (selection Run 6) showed that reopening the Snix services changes
  18 header bytes of both `pathinfo.redb` and `directories.redb`, so the
  no-change guarantee covers only the identity step before the services open,
  and after the open only files other than the Snix databases. Selection Run 9
  passes the revised fixtures: the identity step changes no existing member,
  the public open mints a v2 `snix` identity on identity-less populated state
  and still resolves its PathInfo, and a `casita` marker (alone or beside Snix
  files, under either backend) or unknown content under `casita` fails with
  `store-backend-mismatch` and changes nothing. Selection tasks T2.4, T3.2, and
  T3.4 stay open for their other fixtures.
- Decision 4 applies to the whole command, not only to the store open.
  Selection Run 12 shows `mantle build` under `casita`, against a state
  directory recorded as `snix`, writing `<state-dir>/signing-key` before the
  open failed with `store-backend-mismatch`. The fix calls the read-only
  identity check, `StoreConfig::preflight_backend_identity_for` over the
  selected directory and every base, before the signing key is loaded or
  generated on the build, self-build, and foreign-realization paths. Selection
  Runs 13 and 14 pass the regression: a file target and a project target each
  fail with `store-backend-mismatch`, the state tree keeps every path and byte,
  and no signing key appears. A `casita` base under a `snix` writable state
  fails the same way, with both trees unchanged. The runs cover those three
  cases only. Selection tasks T2.4, T2.5, and T3.2 stay open.
- Casita links into every Mantle binary, including Snix-only use. The graph
  gains, among others, `fastcdc` 5 beside 3.2.1, `nix-archive` 0.6.0 beside
  the 0.1.0 adopted in
  [ADR 0071](0071-adopt-nix-archive-at-the-filesystem-nar-boundary.md),
  `bao-tree`, `iroh-io`, and a git-pinned `turso`. `astral-tokio-tar` moves
  from 0.6.3 to exactly 0.6.4. `blake3` must stay at `=1.8.2`, which Casita's
  `1.8` requirement allows.
- The vendor rail reports that `deny.toml` now admits the Casita and `turso`
  git sources, and that its `cargo deny check` still fails advisories and
  sources. In the lock, only `proc-macro-error 0.4.12` (unmaintained, not a
  security advisory) is new, and it comes only from Casita's graph through
  `genawaiter 0.99.1`. The `h2 0.4.13` and `rustls 0.23.37` advisories are
  unchanged from HEAD and also reachable from `casita` through
  `object_store 0.14.0`. `bitmaps 3.2.1`, `imbl-sized-chunks 0.1.3`, and
  `rsa 0.9.10` are unchanged and off the Casita path, and four missing
  `allow-git` sources predate this change. The dependency gate stays open,
  with no waivers.
- Every read is a full content audit: it checks the envelope out, re-ingests
  `content`, and measures the NAR again, so read cost grows with output size.
- Operators must provision `casita-trusted-public-keys` before a migration.
  Outputs from a signer stay unreadable in Casita mode until its key is listed
  there. Provisioning the file also ends the implicit trust in the local
  signing key: local builds fail with `casita-signer-untrusted` until the
  operator lists that key, and removing it from the file revokes it at the next
  verification, in running processes too. Each verification reads the file
  again, and Mantle does not lock it, so operators replace it by writing a new
  file and renaming it into place. The adapter fix has landed. Run 33 in the
  Casita evidence passes the listed, unlisted, and removed local-signer checks
  in fresh processes; Run 24 is the failing-before counterpart. T2.3 and T2.14
  stay open for the other trust fixtures.
- `bootstrap --fetch` signed its fetch build with a key generated for each run
  at HEAD (`src/bootstrap.rs:873-874`), and no Casita read could verify such an
  output unless the operator listed that key. The parent chose on 2026-09-30
  that under `casita` it signs with the durable signing key that `mantle build`
  uses by default, written only after the backend identity check, while under
  `snix` it keeps the per-run key. Bootstrap outputs of the two backends
  therefore carry different signatures. Casita Run 53 passes its permanent
  regression test: in a new state directory without a policy file, a fresh
  process verifies the bootstrap output and a second run reuses it, and with a
  policy file that omits the key, bootstrap fails with
  `casita-signer-untrusted` and publishes nothing. Run 52 is a one-off smoke of
  the same change.
- Remote substitution into a Casita state directory meets the same read
  rule. An output whose only signer is outside the destination trust set
  would be admitted and then rejected by every later read, so substitution
  must check that set too.
- Casita mode has no overlay composition and, at the pinned defaults, a
  1,024-path ceiling for atomic batches.
- After publication, other Casita clients can still change or remove Mantle
  roots. Mantle cannot prevent that. It detects the change on the next read
  and fails closed, and a retained path without a root reports
  `casita-root-missing`.
- After an interrupted GC, Casita reads and admission fail with
  `gc-recovery-required` until a guarded command recovers the fence, such as
  `store gc`, `store usage`, or `store pin`. Unguarded reads such as
  `store list` do not recover. This gives up availability so that no output
  is served whose content was collected, and it makes recovery, with its
  collection, a side effect of any guarded store command.
- Opening a Casita repository is not read-only. The local profile may
  initialize state, migrate its payload catalog, and reclaim abandoned
  metadata on open. A handle's first mutation may run a disk-pressure
  collection at 80% disk use. Commands that open the repository can also
  change Casita's reader-pin files under `<state-dir>/casita`
  (`casita.sqlite.online-pins`, its `.readers` file, and `reader-*.lock`
  files) even when they fail and Mantle changes nothing. A byte comparison of
  that directory is therefore not a no-mutation check, and fixtures compare
  roots, fence files, and Mantle-owned files instead. In Casita mode
  `store usage` takes `StoreMutationGuard` and may recover a pending fence,
  so it is not read-only either.
- Casita releases evictable roots under disk pressure only through
  crate-private code at the pinned revision (`evict_roots_under_pressure` and
  `evict_next_root` in `repository/root_policy.rs`), so no Mantle test can
  drive that step. Mantle tests that every Mantle root is `Permanent` and that
  a pressure-triggered collection keeps them. That the local profile's
  eviction pass never selects a permanent root rests on Casita's documented
  rule, not on a Mantle test.
- `store sign` replaces an envelope through a conditional change that expects a
  target the process read and verified (`RootExpectation` with `Some(current)`
  and `RootChange::Set`). A mismatch fails with `casita-root-conflict`, and a
  GC plan that saw the old target fails with `gc-plan-stale`. The replacement
  fixtures pass in Casita Runs 35, 38, and 47.
- `store repair-final-nar` is unsupported under `casita`. An executed repair
  writes the repaired PathInfo, renames the staged artifact-attestation sidecar
  into place, and restores the original PathInfo if the rename fails
  (`persist_repaired_state`, `crates/crunch-store/src/repair.rs`). Under Casita
  the original envelope's signed facts do not match `content`, so that restore
  cannot yield a root that verifies, and publishing the sidecar first has the
  opposite race. The parent decided on 2026-09-30 to fail closed until a
  durable repair fence can recover the root and the sidecar together. The
  `casita` profile omits `store-repair-final-nar` from its core list, and
  `store info` lists the rest. The command fails with
  `casita-repair-final-nar-unsupported` before it creates or opens the state
  directory, as a dry run and with `--execute`. `inspect_final_nar_repair` and
  `execute_final_nar_repair` fail with the same blocker for an open Casita
  store before any effect. `snix` repair is unchanged. The refusal fixtures
  pass in Casita Runs 44 and 55 (library; Run 55 also finds the Casita state
  and output trees unchanged after each call) and 45 and 56 (CLI; Run 56 also
  finds an existing state directory unchanged after the dry run and after
  `--execute`), and the unchanged `snix` repair in Run 46.
- `ActionResultPort` outputs backed by PathInfo stay core: they are admitted as
  output roots and rehydrated by a fresh process. Standalone Rust unit cache
  nodes (ADR 0035) are castore-only and are not equivalent. While the interim
  guards persist, `rust-unit-cache` is declared unsupported and Rust cache use
  fails with `casita-rust-cache-unsupported` before any effect. `store gc`
  stays operable for output roots when no Rust cache state exists and fails
  closed before any fence when it does; core output GC is not claimed until the
  no-cache scenario passes. Casita Runs 48 and 49 pass it through the CLI: with
  no Rust unit cache state, `store gc` plans and executes, removes the
  unretained output, and leaves the retained one verifiable; with a
  `rust-unit-cache` entry, planning and execution fail with
  `casita-rust-cache-unsupported` before any fence and change nothing. `casita`
  claims no Rust cache parity or full swap with `snix`. When the proposed
  `mantle/castore/<hex>` roots are wired and a fresh-process reuse test passes,
  the guards are removed and the capability becomes supported.
- The pinned revision does not compile unmodified in Mantle's graph:
  `crates/casita/src/nar.rs:88` resolves `hash.finalize()` to
  `sha2::Digest::finalize`. One tracked, repository-owned patch changes the
  call to `blake3::Hasher::finalize(&hash).as_bytes()`. It must be applied by
  `overrideVendorGitCheckout`, by the `vendor-deps/` generator, and by the
  default `nix develop` shell's Cargo source replacement, because a plain
  `cargo build` in that shell otherwise compiles the unpatched git checkout.
  Each path needs compile and drift checks. Inside that shell an explicit
  `--config .cargo/vendor-config.toml` defines the same git sources a second
  time, and Cargo stops before compiling, so commands in the shell drop the
  flag or unset `CARGO_HOME`. No vendor snapshot is hand-edited, no fork is
  pinned, and no evidence claims an unmodified upstream build.

### Portability

- The local profile expects payloads and state on the filesystem that holds
  the repository root, and it coordinates processes through lock files under
  that root. Casita's state engine `turso` relies on a multi-process WAL that
  Casita's manifest calls experimental. No Mantle evidence covers network
  filesystems or non-Linux hosts in Casita mode.
- A Casita state directory is bound to Casita's on-disk format. Opening it
  with a later revision may migrate that format, and older revisions are not
  promised to read the result. State moves across revisions or hosts through
  the verified archive transport, not file copies.
- Casita declares `rust-version` 1.94.1 and edition 2024. Every toolchain
  that builds Mantle, including self-build source-bundle profiles, must meet
  that floor.
- Store paths, NAR facts, signatures, action refs, source bundles, and store
  archives are specified to be identical across backends, so archives can
  move between `snix` and `casita` state directories. Rail evidence must
  still show that.

### Upstream instability

- Casita is pre-release with no tag, and the `experimental` API may change
  between revisions. The exact pin and the table above bound Mantle's
  exposure.
- `turso` is `=0.8.0-pre.7` from `https://github.com/cachix/turso.git` at
  `dca55133caa690f90dcdd58d3c4329fb0703659c`, an integration revision that
  carries WAL fixes not yet available upstream.
- A bump must recheck, at the new revision, everything this ADR relies on:
  the listed signatures; all-or-none conditional publication with
  first-mismatch reporting and bounded retry of unrelated revision races;
  unrooted-session protection and later collectability; the
  `max_root_changes` default; permanent-by-default roots and the
  pressure-pass rules; the envelope key identity used for idempotent
  admission; and format migrations on open.

## Evidence scope

Evidence that exists on 2026-09-30:

- Proposal, design, and tasks gates pass for both Cairn changes
  (`evidence/gates-2026-09-30.md` in each). `cairn validate --root .`
  reports `valid=False` with one issue from the unrelated active change
  `thin-cli-composition-root`.
- `evidence/baseline-snix-2026-09-30.md` in `add-store-backend-selection`
  records a worker report, not rerun, of two focused `crunch-store` tests
  passing at `7ec5177718a6950297e04eb4eb957a10b02e23ce`. It is partial input
  to task T1.1 and holds no goldens.
- `evidence/casita-review.md` in `adopt-casita-store-backend` records
  upstream source and documentation facts at the pinned revision. The
  source facts cited in this ADR were re-read from a fetch of that exact
  commit on 2026-09-30. The ADR author compiled and ran no Casita code.
- `evidence/test-runs-2026-09-30.md` in `adopt-casita-store-backend` copies the
  runs that fixture owners reported, verbatim and not rerun, all at
  `7ec5177718a6950297e04eb4eb957a10b02e23ce` on a dirty tree. Some failed:
  compile or Cargo configuration failures, the pre-cutover envelope decode, two
  GC tests before their fixes, a byte comparison broken by Casita's reader-pin
  files, the local-signer trust test before the trust fix, one adapter test
  step before its fix, and one repair build in the shared target. Others
  passed: the archive and Nario unit tests, the Casita GC unit tests (finally
  with a clean output-root execution that keeps a retained output verifiable),
  three CLI tests of the migration and GC paths, the adapter's fresh-process
  test, the local-signer trust test after the trust fix, the adapter tests with
  fresh-process read-tamper negatives, replacement races, invalid policy files,
  a local signer without a policy file, and same-handle policy changes, the CLI
  archive import negatives with source and policy byte checks, including an
  unsigned record, the Casita repair refusals in the CLI and the library, later
  with full state and output comparisons, the unchanged `snix` repair, the
  adapter tests again at the current source after the repair cut, including a
  GC inventory of more roots than one atomic mutation allows, the `store gc`
  CLI with and without Rust unit cache state, the CLI archive migration and
  import tests and the CLI cache regression test again at the current source,
  the `bootstrap --fetch` regression test with the durable signer,
  `cargo check` and `cargo build` of `crunch-store` in the default dev shell
  without `--config`, and a fresh-process `mantle build` cache hit under both
  backends for one fixed-output `file://` fetch after its physical export was
  deleted, once as a script and once as a permanent CLI test, and the same for
  one sandboxed content-addressed build and one sandboxed input-addressed
  build, both from `examples/hello.ncl`, plus one-off smokes of a sandboxed
  build with a generated signer and no policy file and of `bootstrap --fetch`
  with the durable signer. Tasks T3.4 (retention fixtures), T2.15 (profile
  rejections), T3.3 (GC fixtures), T2.13 (read-side fixtures), T3.6 (`store gc`
  reachability), T3.5 (migration fixtures), and T2.10 (replacement and
  repair-refusal fixtures) are checked from those runs, and T2.5 (`store sign`
  replacement and the repair refusal) from those runs and a recorded code
  review. Every other task stays open, and the file names each task's remaining
  gap.
- `evidence/test-runs-2026-09-30.md` in `add-store-backend-selection` copies
  its runs the same way: the library backend-mismatch test, the root-registry
  tests with a foreign Casita fence (and their first, non-compiling attempt),
  the CLI selection tests (unknown identifier, recorded mismatch, an ignored
  environment variable, the `snix` profile output), the overlay identity
  fixtures for state without an identity record (a first version that failed on
  the redb files, a compile failure, an intermediate pass, and the final pass),
  the CLI selection tests again at the current source, a one-off smoke of
  `bootstrap --fetch` under `snix` against a state directory recorded with
  another prefix, the wrong-backend `build` regression before the source fix
  (failed: the build wrote a signing key before the mismatch) and after it
  (passed), the `store_gc_cli` suite and one-off smokes after the fix, post-fix
  formatting, Clippy, and TigerStyle checks, and a passing operator command
  contract check. Task T4.1 is checked from that check; every other selection
  task stays open.
- `docs/dependency-audit.md` (Casita admission run) records the vendor rail's
  results. They are not copied into Cairn evidence, and the ADR author did not
  rerun them; the session logs confirm them. At 16:25 UTC
  `nix build --offline --no-link --print-out-paths .#checks.x86_64-linux.casita-vendor-closure`
  printed `/nix/store/skq3d11jy1b67by802kq0d5mmkdhzkmi-vendor-cargo-deps`.
  `scripts/vendor-deps.py generate` then wrote `vendor-deps/` without replacing
  existing data, and `scripts/vendor-deps.py check` matched a fresh Cargo
  generation (53,491 entries) and resolved 766 external package names from
  locked offline metadata, again at 19:12 UTC. The Nix build read the dirty
  working tree (the untracked patch became visible to it only after
  `git add -N`), so it is not the clean-checkout proof that T1.2 asks for.

Evidence added on 2026-10-04 in `add-store-backend-selection`:

- `evidence/prechange-snix-golden-2026-10-04.{md,json}` records an executed
  isolated pre-selection Snix baseline at `7ec51777`, including the exact
  same-key signed PathInfo bytes, NAR hashes, fresh identity bytes, and a
  fixed-physical-root GC report. The original JSON is preserved unchanged.
  `evidence/finish-inventory-2026-10-04.md` inventories store constructors
  and launchers; `docs/store-backends.md` documents the explicit selector,
  identity migration, capability bounds and blocker catalog.
- `evidence/finish-conformance-2026-10-04.md` records a combined source and
  evidence run of all 13 `store_archive_cli` tests: both default and explicit
  Snix match that historical golden with the same root and signer. A
  parameterized Snix/Casita core rail exercises real signed admission,
  strict closure, ActionResult reuse, archive transfer, independently verified
  second signer, no mixed backend, and stale/accepted plan-bound GC. The
  supplemental 7ec rail golden preserves its **first unsorted** execution,
  every recorded rerun/order, and the original red selected observation.
  A selected Snix rail test passes against same-key signed paths, NARs,
  reuse, archive and canonicalized GC consumer facts at the golden's
  identical absolute root and at portable roots. The old 7ec numerical
  execution IDs were nondeterministic; the selected canonical ID differed
  from the retained old unsorted ID, and no assertion says all prechange
  runs emitted the selected ID. T3.1 still needs remaining optional/bound
  fixtures. The source-only capability-boundary checker exited 1 with
  three existing authority escapes, so T4.3 remains open; neither targeted
  suite is a passing repository-wide quality gate.

Evidence that does not exist yet: the remaining selection negatives,
forwarding, and profile fixtures in one passing combined tree; root-race
fixtures, remaining batch fixtures, and trust-policy fixtures for Casita;
castore payload-root fixtures; for T1.2, a vendor build
from a clean checkout and the confirmation that every toolchain, including
self-build source-bundle profiles, meets Casita's `rust-version`; for T4.6,
`cargo check` with the tracked Casita patch in the Nix build and with the
`vendor-deps/` closure, and the patch drift checks; and a passing `cargo deny`
result for the new dependency closure. The vendor rail's run failed advisories
and sources, as recorded under Consequences. At the pin, `casita` itself
declares Apache-2.0, which `deny.toml` allows. The advisory review belongs in
[`docs/dependency-audit.md`](../docs/dependency-audit.md).

When complete, the evidence will cover only the tested behavior for the pinned
revision, feature set, and declared profiles. It will not prove that a state
directory reads its own outputs without a policy file when `CRUNCH_CONFIG_DIR`
places the signing key outside it, Casita correctness, experimental API
stability across revisions, durability under power loss, crash safety beyond
the tested interruptions, interchangeability beyond the declared profiles,
store content or output correctness, GC safety in general, sandboxing, or
release eligibility.

## Ownership and repeatability

- **Owners.** The Mantle store lifecycle owner (`crunch-store`) owns the
  selection seam, identity record, profiles, conformance rail, and Casita
  adapter. The CLI composition-root owner owns option plumbing and child
  forwarding. The dependency owner owns the Casita pin, feature set, Nix
  assertion, both vendor closures, the `deny.toml` entries, and the
  dependency audit record.
- **Repeat the admission.** Check the pin in the Cargo manifests,
  `Cargo.lock`, and the Nix assertion. Build from clean source with the Crane
  vendor closure. Regenerate and check `vendor-deps/` with the
  repository-owned path and locked offline Cargo metadata. Run
  `cargo deny check`, the conformance rail on `snix` and `casita`, and the
  Casita fixtures. Keep exact transcripts in each Cairn change's `evidence/`.
- **Bump.** A new revision or feature set needs its own reviewed Cairn
  change. That change re-reviews the upstream source at the new revision
  against the list above, updates this ADR's API table and
  `docs/dependency-audit.md`, and reruns every step above.

## Rollback

`snix` stays the default, so rolling back Casita removes the dependency, the
adapter, and the `casita` identifier without touching Snix state. A binary
without `casita` rejects a state directory that records it. Before rolling
back, operators move content out with `store archive export` under
`--store-backend casita` and `store archive import --trusted-public-keys
<key>` under `--store-backend snix`.

## References

- <https://github.com/cachix/casita/tree/90404fcb1cfb3d83f2233715448dfefe913f5fd1>
- [ADR 0012](0012-overlay-store-composition.md)
- [ADR 0035](0035-cache-rust-units-through-castore-action-results.md)
- [ADR 0054](0054-select-snix-backports-by-mantle-compatibility-boundary.md)
- [ADR 0058](0058-limit-store-access-with-concrete-capability-views.md)
- [ADR 0064](0064-explain-store-retention-before-garbage-collection.md)
- [ADR 0071](0071-adopt-nix-archive-at-the-filesystem-nar-boundary.md)
- [ADR 0073](0073-read-nario-v2-without-transferring-nix-authority.md)
- [Overlay trust file format](../docs/operator-workflows.md#use-an-ordered-read-only-base-stack)
