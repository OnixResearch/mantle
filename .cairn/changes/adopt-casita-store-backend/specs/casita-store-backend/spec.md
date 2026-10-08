# Specification: Casita store backend

## ADDED Requirements

### Requirement: Casita is pinned and reproducibly vendored

r[mantle.casita_store_backend.pinned_dependency] Mantle MUST depend on Casita through one exact git revision recorded in `Cargo.toml` and `Cargo.lock`, with default features disabled and exactly the `native` and `experimental` features enabled. The ADR MUST record every experimental Casita API that Mantle uses and its instability risk. Nix admission assertions MUST check the pinned revision and the feature set. Every transitive git dependency and every lock update that Casita requires MUST be locked to exact revisions or versions. `deny.toml` MUST admit exactly the pinned Casita and `turso` git sources, and `cargo deny check` MUST pass. Every toolchain that builds Mantle, including self-build source-bundle profiles, MUST meet Casita's declared `rust-version`. A clean-source Nix build MUST vendor the exact locked sources through the flake's Crane `vendorCargoDeps` closure. The checkout-local `vendor-deps/` closure selected by `.cargo/vendor-config.toml` MUST be produced and verified by an owner-controlled, repository-owned generation and check path whose output passes the locked offline Cargo metadata check. A hand-edited vendor snapshot or an unrecorded online Cargo run MUST NOT satisfy this requirement. The revision, license review, and advisory posture MUST be recorded in the ADR and `docs/dependency-audit.md`, and a revision or feature change MUST go through a reviewed change.

#### Scenario: Clean source vendors the pinned revision

- GIVEN a clean checkout with the pinned Casita revision and features in `Cargo.toml` and `Cargo.lock`
- WHEN the Nix build vendors dependencies, the repository vendor check runs, and `cargo deny check` runs
- THEN both closures MUST contain exactly the locked Casita revision and its locked git dependencies, and `cargo deny check` MUST pass
- AND locked offline Cargo metadata MUST resolve every package from the checkout-local directory source

#### Scenario: Floating, drifted, or widened dependency

- GIVEN a Casita dependency declared by branch or tag, a `Cargo.lock` revision that differs from `Cargo.toml`, a feature outside `native` and `experimental`, a git source that `deny.toml` does not admit, or a `vendor-deps/` file that differs from the generated closure
- WHEN the Nix admission assertion, vendor check, `cargo deny check`, or offline metadata check runs
- THEN it MUST fail and name the drifted package, feature, or source
- AND the build MUST NOT fall back to network resolution

### Requirement: A destination-owned policy decides signer trust

r[mantle.casita_store_backend.trust_policy] In Casita mode, when the destination-owned file `casita-trusted-public-keys` exists in the state directory, the signer trust set MUST be exactly the keys in that file; when it does not exist, the trust set MUST be only the verifying key of the state directory's local signing key. The local signing key is the `signing-key` file in the state directory. A signing key that `CRUNCH_CONFIG_DIR` places outside the state directory is not the local signing key: outputs signed only by it MUST fail admission and reads with `casita-signer-untrusted` unless the policy file lists its verifying key, and Mantle MUST NOT move, copy, or trust that key on its own. While the file exists, the local signing key MUST be trusted only if the file lists it, and Mantle MUST NOT add it on its own. The file MUST use the existing `overlay-trusted-public-keys` format and parser rules: UTF-8 text, `#` comments, keys separated by whitespace or commas, each key a Nix-style `name:base64` Ed25519 public key, at most 64 keys, and the overlay descriptor size bound of 65,536 bytes. The operator MUST provision the file explicitly. Mantle MUST NOT create, modify, or extend it, MUST NOT enroll any key into it, including the local signing key, and MUST NOT derive it from a source state directory, a source trust file or signing key, an archive, or a command-line key. Key membership MUST compare full key material and never the key name alone. A present file that is malformed, empty, oversized, or over the key limit MUST fail the store open with `casita-trust-policy-invalid`. Mantle MUST publish an envelope only when its PathInfo carries a valid signature from a key in the trust set. In Casita mode, `bootstrap --fetch` MUST sign the outputs it builds with the durable signing key that `mantle build` uses by default, MUST NOT sign them with a key generated for a single run, which no later read could verify, and MUST NOT generate or write that key before the backend identity check passes. Mantle MUST parse and validate the file when it opens the Casita store, and MUST read and validate it again, under the same rules and bounds, for every PathInfo admission, read, and verification and for every store archive or Nario import preflight. Mantle MUST NOT reuse a trust set read for an earlier verification, so a key added to or removed from the file takes effect at the next verification, including in a store handle that is already open. A file that is invalid when it is read again MUST fail that verification or preflight with `casita-trust-policy-invalid`. Mantle does not lock the file and MUST NOT claim that an operation verifying several PathInfo observes a single version of the file while the operator changes it. Unsigned PathInfo MUST NOT be trusted in Casita mode. An output without a valid signature from the trust set MUST fail with `casita-signer-untrusted`, including after its only trusted key was removed from the file or the file was removed.

#### Scenario: Local build without a policy file

- GIVEN a Casita state directory without `casita-trusted-public-keys` and a build signed with the state directory's local signing key
- WHEN Mantle admits the output and a fresh process resolves it
- THEN publication and read verification MUST succeed under the local signing key
- AND Mantle MUST NOT create a `casita-trusted-public-keys` file

#### Scenario: Bootstrap fetch without a policy file

- GIVEN a new Casita state directory without `casita-trusted-public-keys`
- WHEN `bootstrap --fetch` builds the seed and a fresh process verifies the fetched output with the verifying key of the durable signing key
- THEN verification MUST report the signature trusted, and a second `bootstrap --fetch` MUST reuse the outputs and leave the signing key byte-identical
- AND in a new state directory whose policy file omits that key, `bootstrap --fetch` MUST fail with `casita-signer-untrusted`, write no seed file, and leave the policy file byte-identical

#### Scenario: Policy file omits the local signer

- GIVEN a Casita state directory whose `casita-trusted-public-keys` does not list the verifying key of its local signing key, and a build signed only with that key
- WHEN Mantle admits the output
- THEN it MUST fail with `casita-signer-untrusted` before it stages or publishes an envelope
- AND the policy file MUST remain byte-identical

#### Scenario: Policy file lists the local signer

- GIVEN a Casita state directory whose `casita-trusted-public-keys` lists the verifying key of its local signing key, and a build signed with that key
- WHEN Mantle admits the output and a fresh process resolves it
- THEN publication and read verification MUST succeed
- AND after the operator removes that key from the file, the next process MUST fail with `casita-signer-untrusted` for that output

#### Scenario: Signer outside the trust set

- GIVEN a build signed with a `--signing-key` whose public key is outside the trust set, or a substituted output signed only by a cache key that the trust set does not contain
- WHEN Mantle admits the output in Casita mode
- THEN it MUST fail with `casita-signer-untrusted` before it stages or publishes an envelope
- AND no root for that store path MUST be published

#### Scenario: Malformed policy file

- GIVEN a `casita-trusted-public-keys` file with an unparsable key, no keys, more than 64 keys, or more bytes than the descriptor bound
- WHEN Mantle opens the Casita store
- THEN it MUST fail with `casita-trust-policy-invalid`
- AND it MUST NOT open a Casita mutation session

#### Scenario: Key removed after import

- GIVEN outputs imported and verified under a policy key K that carry no signature from another key in the trust set
- WHEN the operator removes K from the policy file, or removes the file, and a fresh process resolves those outputs
- THEN every lookup, export, and cache hit for them MUST fail with `casita-signer-untrusted`
- AND `store verify` MUST report them, and GC planning MUST reject while any of them is retained

#### Scenario: Policy changed while a store is open

- GIVEN an open Casita store handle and an admitted output whose only valid signature from the trust set is by key K
- WHEN the operator removes K from the policy file
- THEN the next lookup, `store verify`, and GC planning through that handle MUST fail with `casita-signer-untrusted`, and the output root MUST keep its target
- AND after the operator adds K back, the next lookup through that handle MUST return the output without reopening the store, and after the operator makes the file invalid, the next verification MUST fail with `casita-trust-policy-invalid`

### Requirement: Outputs are admitted through conditional Casita publication

r[mantle.casita_store_backend.output_admission] With the `casita` backend, Mantle MUST admit an output only after the existing build, NAR measurement, and signature admission succeed and its PathInfo carries a valid signature from a key in the Casita trust set. Mantle MUST stage a directory envelope containing exactly `content` and `pathinfo.json` as an unrooted object inside one Casita mutation session, and MUST publish the permanent root `mantle/outputs/<40 lowercase hex of the store-path digest>` with one conditional publication whose expectation requires the root name to be absent. A root mismatch MUST commit nothing and MUST leave any existing target unchanged, including a target published concurrently by another Casita client. When the existing root already targets an identical envelope, Mantle MUST report idempotent success without a write; otherwise it MUST fail with `casita-root-conflict`. A staged envelope that is not published MUST remain unrooted and reclaimable by later collection. Mantle MUST verify that the measured NAR SHA-256 and size of `content` equal the signed PathInfo before it reports admission, and a mismatch MUST leave no Mantle root published.

#### Scenario: Built output is admitted

- GIVEN a derivation built and admitted in the session scratch with a PathInfo signed by a key in the trust set
- WHEN Mantle stages the envelope and its conditional publication commits
- THEN the root `mantle/outputs/<digest>` MUST target an envelope with exactly `content` and `pathinfo.json`
- AND the measured NAR SHA-256 and size of `content` MUST equal the signed PathInfo before admission is reported

#### Scenario: Measured NAR facts differ

- GIVEN the staged or published `content` measures a NAR SHA-256 or size that differs from the signed PathInfo
- WHEN verification runs
- THEN Mantle MUST fail with `casita-nar-mismatch`
- AND no Mantle root for that store path MUST remain published

#### Scenario: External client publishes during admission

- GIVEN another Casita client publishes a different target under the same root name after Mantle staged its envelope and before Mantle's conditional publication commits
- WHEN Mantle's conditional publication runs
- THEN it MUST commit nothing and fail with `casita-root-conflict`
- AND the other client's target MUST remain unchanged

#### Scenario: Identical root already exists

- GIVEN the root name already targets an envelope identical to the staged envelope
- WHEN Mantle admits the output
- THEN it MUST report idempotent success without writing a root
- AND the existing target MUST remain unchanged

#### Scenario: Process stops after staging

- GIVEN Mantle staged an envelope and stopped before its conditional publication committed
- WHEN a new process resolves the store path and later collection runs
- THEN no root or PathInfo for that output MUST be visible and the staged envelope MUST be reclaimable
- AND a retried admission MUST publish the output through a new conditional publication

### Requirement: Published PathInfo is replaced only conditionally

r[mantle.casita_store_backend.envelope_replacement] In Casita mode, `store sign` MUST update a published PathInfo only by replacing its output root's envelope through one conditional change that expects the root's current target. Mantle MUST read the current target, stage a new envelope with the same `content` and the updated `pathinfo.json` in one Casita mutation session, verify it as for admission, and commit the change only if the root still targets the envelope that Mantle read. A mismatch MUST commit nothing, fail with `casita-root-conflict`, and leave the current target unchanged. The replaced envelope MUST NOT stay reachable from any Mantle root.

#### Scenario: Signature added in place

- GIVEN a published output whose root targets envelope E and no other writer changes that root
- WHEN the operator runs `store sign` for its path under `--store-backend casita`
- THEN the root MUST target a new envelope with the same `content` and a `pathinfo.json` that carries the added signature
- AND a fresh process MUST verify the new PathInfo, and no Mantle root MUST reach E

#### Scenario: Root changed during the update

- GIVEN another Casita client repoints the root after Mantle read E and before Mantle's conditional change commits
- WHEN `store sign` commits its change
- THEN it MUST commit nothing and fail with `casita-root-conflict`
- AND the other client's target MUST remain unchanged

### Requirement: Batch imports publish all-or-none within a declared bound

r[mantle.casita_store_backend.atomic_batch_import] An import that requires several paths to become visible together, including Nario v2 import, MUST stage every envelope in one Casita mutation session and MUST publish every new root with one multi-root conditional publication whose expectations require each new root name to be absent. A path whose root already targets an identical envelope MUST be verified and left unchanged. The publication MUST be all-or-none: a root mismatch, or a staging, verification, signer-trust, or import error for any path, MUST commit no root of the batch and MUST leave every existing root unchanged. Envelopes staged for a failed batch MUST remain unrooted and reclaimable. The Casita capability profile MUST declare `atomic-batch-import` bounded by the repository's per-commit root-change limit (`max_root_changes`), which is 1,024 paths at the pinned defaults. A batch over that bound MUST fail with `casita-batch-limit` before it opens a Casita mutation session, and Mantle MUST NOT split a batch across commits. Before it opens a Casita mutation session, a Nario v2 import MUST fail with `casita-trust-policy-missing` when the destination policy file is absent and with `casita-import-key-unauthorized` when one of its trusted keys is not in the destination policy.

#### Scenario: Batch at the limit publishes together

- GIVEN a Nario v2 import of exactly 1,024 paths that are all absent in a Casita repository with the pinned default limits and signed by a key in the destination policy
- WHEN Mantle stages every envelope and runs one multi-root conditional publication
- THEN every root of the batch MUST become visible in one Casita revision
- AND every published envelope MUST verify

#### Scenario: Batch over the limit

- GIVEN a Nario v2 import of 1,025 paths under `--store-backend casita` with the pinned default limits
- WHEN Mantle admits the batch
- THEN it MUST fail with `casita-batch-limit` before it opens a Casita mutation session
- AND no envelope MUST be staged and no root MUST be published

#### Scenario: One batch root conflicts

- GIVEN a Nario v2 import where one root name already targets a different envelope
- WHEN the multi-root conditional publication runs
- THEN no root of the batch MUST be committed and every existing root MUST remain unchanged
- AND Mantle MUST fail with `casita-root-conflict` naming the conflicting path

#### Scenario: Staging fails after part of the batch is staged

- GIVEN a Nario v2 import where staging or verification of a later path fails after earlier envelopes were staged
- WHEN Mantle processes the batch
- THEN it MUST commit no root of the batch and MUST report the failing path
- AND the staged envelopes MUST remain unrooted and reclaimable

#### Scenario: Batch trust is not authorized

- GIVEN a Nario v2 import in Casita mode whose trusted key is not in the destination policy
- WHEN Mantle processes the batch
- THEN it MUST fail with `casita-import-key-unauthorized` before it opens a Casita mutation session
- AND no root of the batch MUST be committed

### Requirement: Castore-only payloads get Mantle-owned roots once the Rust unit cache is declared

r[mantle.casita_store_backend.castore_payload_roots] PathInfo-backed action-result outputs are core under `casita`: `ActionResultPort` MUST admit them as output roots and rehydrate them in a fresh process like any other output. The standalone Rust unit cache is the optional capability `rust-unit-cache`, and the rest of this requirement applies only while the Casita profile declares it. Then every castore-only payload that Mantle retains, including standalone Rust unit cache nodes and the castore roots that GC receives from the Rust unit retention policy, MUST be published as a permanent root `mantle/castore/<64 lowercase hex of the BLAKE3 digest of the postcard-encoded castore node>` through the same staged conditional publication as outputs. The envelope MUST contain exactly `content` and `node.postcard`, where `node.postcard` records the castore node's kind, digest, size, and executable bit. Before a read uses a payload, Mantle MUST decode `node.postcard`, recompute the root name from it, re-ingest `content`, and require that the reproduced node equals the decoded node and that the recomputed name equals the root name; otherwise it MUST fail with `casita-envelope-invalid`. A retained payload whose root is absent MUST report `casita-root-missing`, and no castore-only payload MUST depend on session scratch after the session ends. While the profile does not declare `rust-unit-cache`, the castore parity gate applies instead.

#### Scenario: Rust unit cache payload is reused after reopen

- GIVEN a Casita profile that declares `rust-unit-cache` and a Rust unit cache node whose payload was published under `mantle/castore/`
- WHEN a fresh process with `--store-backend casita` resolves the same action
- THEN it MUST reuse the payload without recompiling the unit
- AND the re-ingested payload MUST reproduce the node recorded in `node.postcard` and the root name

#### Scenario: Payload root changed or removed

- GIVEN a Casita profile that declares `rust-unit-cache` and a retained payload root that another writer repointed to an envelope with different `content` or a different `node.postcard`, or removed
- WHEN a fresh process resolves the action or plans GC
- THEN it MUST fail with `casita-envelope-invalid` or `casita-root-missing`
- AND GC planning MUST reject before any root removal

#### Scenario: PathInfo-backed action-result output is rehydrated

- GIVEN an action result whose outputs are PathInfo-backed store paths admitted under `casita`, whether or not Rust unit cache state exists
- WHEN a fresh process resolves the action through `ActionResultPort` after the physical export and session scratch are gone
- THEN it MUST rehydrate the outputs from their `mantle/outputs/` roots without rebuilding
- AND it MUST NOT require any `mantle/castore/` root for those PathInfo-backed outputs

### Requirement: Signed PathInfo lives only in its output root

r[mantle.casita_store_backend.durable_pathinfo] In Casita mode the signed PathInfo MUST be stored only as `pathinfo.json` inside its output root, and Mantle MUST NOT keep a second durable PathInfo mapping or read a persistent Snix PathInfo service. `pathinfo.json` MUST hold the serde JSON encoding of the signed PathInfo, Mantle MUST NOT read or write any other PathInfo encoding in an envelope, and equal PathInfo values MUST encode to equal bytes, so that identical envelopes have identical object keys. Mantle MUST NOT write a `pathinfo.json` larger than the store archive metadata bound (1 MiB), and MUST reject with `casita-envelope-invalid` a `pathinfo.json` that exceeds that bound or whose bytes differ from the canonical re-encoding of the PathInfo it decodes to. Before any lookup, closure walk, export, or cache hit uses an output, Mantle MUST verify the envelope entries and types, decode the PathInfo, match the root-name digest to the store path, verify a signature from a key in the Casita trust set, check content-address facts, and match the measured NAR SHA-256 and size of `content`. A failed check MUST fail closed with `casita-envelope-invalid`, `casita-signer-untrusted`, or `casita-nar-mismatch`.

#### Scenario: Valid envelope is served

- GIVEN a root whose envelope, trusted signature, digest, and NAR facts all verify
- WHEN a lookup resolves the store path
- THEN Mantle MUST serve the output from `content`
- AND the reported PathInfo MUST equal the decoded `pathinfo.json`

#### Scenario: PathInfo with a content address round-trips

- GIVEN an admitted output whose signed PathInfo carries a content address
- WHEN a fresh process resolves the store path
- THEN the decoded `pathinfo.json` MUST equal the signed PathInfo
- AND encoding that PathInfo again MUST produce byte-identical `pathinfo.json`

#### Scenario: Envelope was changed outside Mantle

- GIVEN a root that another writer repointed to an envelope with a swapped `content`, a tampered `pathinfo.json`, a `pathinfo.json` over 1 MiB or not in canonical encoding, a different store-path digest, or a signature from a key outside the trust set
- WHEN a fresh process resolves that store path
- THEN Mantle MUST fail with `casita-envelope-invalid`, `casita-signer-untrusted`, or `casita-nar-mismatch`
- AND it MUST NOT serve, export, or rebuild over the root silently

### Requirement: Snix services are session scratch in Casita mode

r[mantle.casita_store_backend.session_intermediates] In Casita mode Mantle MUST keep Snix blob, directory, and PathInfo services in memory or under a per-session directory outside `--state-dir`, MUST delete that scratch when the session ends, and MUST NOT consult it as output or payload authority after publication. The Casita repository MUST live in the dedicated subdirectory `<state-dir>/casita`, so that no Casita file appears at a Snix marker path. A Casita state directory MUST NOT gain `pathinfo.redb`, `directories.redb`, or a Snix `blobs/` directory.

#### Scenario: Session ends after a build

- GIVEN a Casita-mode build session that admitted outputs
- WHEN the session ends
- THEN the state directory MUST contain no Snix persistence files and no Casita file outside `<state-dir>/casita`
- AND the session scratch MUST be removed

#### Scenario: Scratch is not authority

- GIVEN an output present in session scratch whose Casita publication failed
- WHEN a later lookup in the same or a new session resolves that store path
- THEN Mantle MUST report a miss or the publication failure
- AND it MUST NOT serve the scratch copy

### Requirement: Fresh processes reuse admitted outputs

r[mantle.casita_store_backend.fresh_process_reuse] An output admitted under the `casita` backend MUST remain reusable by a new Mantle process after the physical export under `--store` is deleted. The new process MUST verify the envelope under the current trust policy, report a cache hit without rebuilding, and MUST be able to restore the export with the same NAR SHA-256 and size.

#### Scenario: Export deleted before the next build

- GIVEN an admitted Casita output whose physical export was deleted and an unchanged trust policy
- WHEN a fresh process builds the same derivation with `--store-backend casita`
- THEN Mantle MUST report a verified cache hit without running the builder
- AND the restored export MUST have the signed NAR SHA-256 and size

#### Scenario: Root removed outside Mantle

- GIVEN a retained output whose root was removed by another writer
- WHEN a fresh process plans GC or verifies the store
- THEN Mantle MUST report `casita-root-missing`
- AND GC planning MUST reject before any root removal

### Requirement: GC is plan-bound, fenced, then collected

r[mantle.casita_store_backend.plan_bound_gc] Mantle MUST compute Casita GC plans with its GC core from retained output roots, retained castore payload roots while the profile declares `rust-unit-cache`, and verified envelope facts, and each plan MUST record the exact root target of every candidate. Execution MUST take the store mutation guard, re-observe roots, envelopes, references, and candidate targets, and fail with `gc-plan-stale` on any drift before a mutation. Mantle MUST durably write a fence record naming the plan, root names, and expected targets before it removes any root. It MUST remove each candidate root only through a conditional change that expects the planned target, then delete dead exports and index entries, record every outcome in the fence, and only then run Casita collection. Mantle MUST manage and remove roots only under `mantle/outputs/` and `mantle/castore/`. Content and PathInfo MUST be unpublished together by one root removal, so a partial failure MUST NOT leave a published PathInfo without its content. Casita GC planning, dry runs, and execution MUST run only while holding the store mutation guard, and a call without it MUST fail with `casita-gc-guard-required` before any recovery, planning, or mutation. While a fence record is incomplete, every lookup, PathInfo listing, closure walk, export, admission, root registration, and rehydration that has not first recovered the fence under the store mutation guard MUST fail with `gc-recovery-required` before it serves, admits, or registers anything. Recovery MUST run only while the store mutation guard is held, at the start of a guarded command and before that command plans, admits, or registers anything; it MUST complete cleanup only for roots already absent and MUST NOT remove any root.

#### Scenario: Plan executes without drift

- GIVEN an accepted plan and unchanged roots, envelopes, and references
- WHEN the operator runs `mantle --store-backend casita store gc --execute --plan-id <id>`
- THEN Mantle MUST remove only the planned roots at their expected targets and then run collection
- AND every retained output and payload MUST remain verifiable afterward

#### Scenario: Unretained payload is released by plan

- GIVEN a Casita profile that declares `rust-unit-cache`, a castore payload root that the Rust unit retention policy no longer lists, and a payload root that it still lists
- WHEN an accepted plan executes without drift
- THEN Mantle MUST remove the unlisted payload root at its planned target before collection
- AND the listed payload root MUST remain published and verifiable

#### Scenario: Candidate changed after planning

- GIVEN a candidate root that now targets another envelope, or a candidate that became retained or referenced
- WHEN plan execution rechecks state
- THEN it MUST fail with `gc-plan-stale` before writing a fence or removing a root
- AND it MUST require a new plan

#### Scenario: Pending fence blocks outputs until recovery

- GIVEN execution stopped after some fenced roots were removed and before collection
- WHEN a later process looks up, lists, exports, admits, registers, or rehydrates an output without first recovering the fence under the store mutation guard
- THEN it MUST fail with `gc-recovery-required` before it serves, admits, or registers anything
- AND no store path MUST expose a PathInfo whose content was collected

#### Scenario: Guarded GC recovers an interrupted fence

- GIVEN the same incomplete fence
- WHEN the operator runs `mantle --store-backend casita store gc`
- THEN recovery MUST complete cleanup for the removed roots before planning and leave every other fenced root published
- AND it MUST NOT remove any root, and retained outputs MUST verify on later lookups

#### Scenario: GC without the store mutation guard

- GIVEN a Casita state directory
- WHEN Casita GC planning, a dry run, or execution is called without holding the store mutation guard
- THEN it MUST fail with `casita-gc-guard-required` before any recovery, planning, or mutation
- AND no root, fence record, export, or index entry MUST change

#### Scenario: Collector is busy

- GIVEN every planned root was removed and the Casita collector reports busy
- WHEN execution finishes
- THEN the report MUST mark reclaim incomplete
- AND it MUST NOT claim successful reclaim

### Requirement: Mantle is the only retention authority

r[mantle.casita_store_backend.retention_authority] Every Casita root that Mantle publishes, for outputs and, while the profile declares `rust-unit-cache`, for castore payloads, MUST be permanent. Mantle MUST NOT mark a root evictable, MUST NOT use Casita access times or disk pressure as retention input, and MUST keep an admitted output or payload published until a Mantle GC plan releases it. The retained set MUST come from the same retention roots, pins, and project roots that the `snix` backend uses, plus the Rust unit retention policy while `rust-unit-cache` is declared, so equivalent observations MUST produce equal GC decisions under both backends.

#### Scenario: Disk pressure does not release outputs

- GIVEN a Casita repository above its disk-pressure threshold with admitted outputs and payloads that no plan released
- WHEN Casita runs its automatic collection
- THEN every Mantle root MUST remain published
- AND Mantle MUST NOT mark any root evictable

#### Scenario: Equal observations give equal decisions

- GIVEN equivalent retained roots and PathInfo graphs in a `snix` and a `casita` state directory, with Rust unit retention included only when both profiles declare `rust-unit-cache`
- WHEN each backend plans GC
- THEN both plans MUST contain the same candidates in the same order

#### Scenario: Root made evictable outside Mantle

- GIVEN another tool marked a retained output root evictable and Casita released it
- WHEN Mantle verifies the store or plans GC
- THEN it MUST report `casita-root-missing` for the retained path
- AND it MUST NOT treat the loss as a normal miss for that retained path

### Requirement: Snix-to-Casita moves are explicit and verified

r[mantle.casita_store_backend.verified_migration] Mantle MUST move admitted outputs from a `snix` state directory to a `casita` state directory only through explicit operator steps: writing reviewed signer keys into the destination `casita-trusted-public-keys` file, `store archive export` under `--store-backend snix`, `store archive import --trusted-public-keys` under `--store-backend casita`, and explicit root declarations with `store pin`. Before it opens a Casita mutation session, import MUST fail with `casita-trust-policy-missing` when the destination policy file is absent, and with `casita-import-key-unauthorized` when any `--trusted-public-keys` key is not in the destination policy. The `--trusted-public-keys` keys MUST authorize only that import and MUST NOT become durable signer authority. Import MUST run the full archive verification and Casita admission, MUST keep the original signatures, and MUST NOT add a destination signature. Mantle MUST NOT read, copy, or enroll signing keys, trust files, root records, or retention records from the source state directory. After import, a fresh process MUST verify every imported output under the destination trust set. Opening, lookup, and GC MUST NOT migrate content. The source state directory MUST remain byte-identical.

#### Scenario: Migration with a provisioned signer key

- GIVEN a `snix` state directory with outputs signed by key K and a `casita` state directory whose operator wrote K into `casita-trusted-public-keys`
- WHEN the operator exports a closure, imports it with `--trusted-public-keys K`, pins the roots, and runs `store verify` in a fresh process
- THEN every imported output MUST verify under the destination trust set with its original signatures only
- AND the source state directory and the destination policy file MUST remain byte-identical

#### Scenario: Destination policy is missing

- GIVEN a `casita` state directory without `casita-trusted-public-keys`
- WHEN the operator imports an archive with `--trusted-public-keys K`
- THEN import MUST fail with `casita-trust-policy-missing` before it opens a Casita mutation session
- AND Mantle MUST NOT create the policy file or publish any root

#### Scenario: Import key is not authorized by the destination

- GIVEN a destination policy that lacks K, or that contains a key with K's name but different key bytes
- WHEN the operator imports an archive with `--trusted-public-keys K`
- THEN import MUST fail with `casita-import-key-unauthorized` before it opens a Casita mutation session
- AND the policy file MUST remain byte-identical

#### Scenario: Archive signer is not named by the import

- GIVEN an archive whose outputs are signed by a key that the import does not name
- WHEN the operator imports it into a `casita` state directory whose policy contains every named import key
- THEN the import MUST admit no output from that archive
- AND Mantle MUST NOT trust the source state directory's local signing key implicitly

#### Scenario: Unsigned record in a signed import

- GIVEN an archive in either supported format with a record whose PathInfo carries no signature, imported under `casita` with authorized `--trusted-public-keys` and without `--trust-unsigned`
- WHEN Mantle verifies the record
- THEN import MUST fail with `casita-signer-untrusted` and publish no root for that record or its batch
- AND passing `--trust-unsigned` or `--nario-trust-unsigned` instead MUST fail with `casita-trust-unsigned-unsupported` before any state access

### Requirement: Offline source and transport formats stay backend-neutral

r[mantle.casita_store_backend.offline_boundary] The Casita dependency MUST NOT enable network profiles, and Casita mode MUST NOT open network connections. Offline realization, source-bundle import and preflight, and store archive export and import MUST keep their existing semantics and byte formats under both backends. Source-bundle records and pins MUST stay Mantle-owned files, and store paths materialized from them MUST be admitted and retained like any other output.

#### Scenario: Same inputs give the same archive

- GIVEN equivalent closures admitted in a `snix` and a `casita` state directory with the same explicitly provisioned signing key
- WHEN the operator exports the same selectors
- THEN both archives MUST be byte-identical

#### Scenario: Offline build with imported sources

- GIVEN a pinned source bundle and `--store-backend casita`
- WHEN the operator runs offline source preflight and an offline build
- THEN the preflight report MUST equal the `snix` report for the same inputs
- AND no network access MUST occur

### Requirement: Casita stays inside the store shell and declares its profile

r[mantle.casita_store_backend.capability_boundary] Casita types, including the experimental mutation-session and conditional-publication types, MUST stay inside the `crunch-store` shell. Capability views, application ports, and reports MUST NOT expose Casita repositories, sessions, keys, readers, reports, or errors. The store capability boundary checker MUST reject a Casita type or raw repository access outside the store shell. The Casita capability profile MUST declare every core capability, MUST omit `store-repair-final-nar` from its core list until a durable repair fence keeps a replaced envelope and its artifact-attestation sidecar recoverable together, MUST declare `atomic-batch-import` with the repository's `max_root_changes` bound and `rust-unit-cache` with durable verified castore payload roots, and MUST NOT declare `overlay-composition` or `unsigned-admission`. Before any state access, Casita mode MUST reject `--base-store` with `casita-overlay-unsupported`, and `--trust-unsigned` or `--nario-trust-unsigned` with `casita-trust-unsigned-unsupported`, because a fresh read requires a trusted signature. Casita mode MUST reject `store repair-final-nar`, as a dry run or with `--execute`, with `casita-repair-final-nar-unsupported` before it creates or opens the state directory, and the library repair calls `inspect_final_nar_repair` and `execute_final_nar_repair` MUST reject an open Casita store with the same blocker before any effect. `store repair-final-nar` under `snix` is unchanged.

#### Scenario: Casita type escapes the shell

- GIVEN source outside `crunch-store` names a Casita repository, session, reader, or key type
- WHEN the store capability boundary checker runs
- THEN it MUST fail with the caller and the type
- AND a re-export or alias MUST NOT bypass the failure

#### Scenario: Overlay requested in Casita mode

- GIVEN `--store-backend casita` with one or more `--base-store` layers
- WHEN Mantle composes the store
- THEN it MUST fail with `casita-overlay-unsupported`
- AND it MUST NOT open or modify any layer

#### Scenario: Unsigned trust requested in Casita mode

- GIVEN `--store-backend casita` with `--trust-unsigned` or `--nario-trust-unsigned`
- WHEN Mantle admits the command
- THEN it MUST fail with `casita-trust-unsigned-unsupported`
- AND it MUST NOT create, open, or modify the state directory

#### Scenario: Final-NAR repair requested in Casita mode

- GIVEN `--store-backend casita`
- WHEN the operator runs `store repair-final-nar`, as a dry run or with `--execute`
- THEN it MUST fail with `casita-repair-final-nar-unsupported`
- AND it MUST NOT create, open, or modify the state directory, and a library repair call on an open Casita store MUST fail with the same blocker and change nothing

### Requirement: Casita backend evidence keeps local claims

r[mantle.casita_store_backend.claim_boundary] Casita backend evidence MUST be limited to the pinned revision and feature set, the declared capability profile and its bounds, the trust policy files and signing keys used, the tested admission, conditional publication, envelope replacement, castore payload, trust, verification, reuse, retention, GC, recovery, migration, batch, batch-limit, and rejection fixtures, and the recorded vendor closure. Reports and documentation MUST list the optional capabilities that the Casita profile does not declare, including `overlay-composition` and `unsigned-admission`, and the batch bound. Evidence MUST NOT claim Casita correctness, stability of the experimental API across revisions, key ownership, signer honesty, revocation freshness beyond the policy file as read for each verification, that a state directory can build and read its own outputs without a policy file when `CRUNCH_CONFIG_DIR` places the signing key outside it, durability under power loss, crash safety beyond the tested interruptions, interchangeability with `snix` beyond the declared profile, output correctness, sandboxing, or release eligibility.

#### Scenario: All Casita rails pass

- GIVEN the conformance rail and the Casita fixtures pass on the pinned revision
- WHEN documentation, tasks, or status replies summarize the result
- THEN they MUST name the revision, features, profile and bounds, trust policy fixtures, and rails
- AND they MUST NOT claim key ownership, durability under power loss, full interchangeability with `snix`, or release eligibility

### Requirement: The pinned Casita source carries one reviewed patch

r[mantle.casita_store_backend.pinned_patch] The pinned revision does not pass `cargo check` in Mantle's dependency graph as published: `crates/casita/src/nar.rs:88` calls `hash.finalize().as_bytes()`, which resolves to `sha2::Digest::finalize` instead of the BLAKE3 inherent method. Mantle MUST apply exactly one repository-owned, tracked patch for that revision that changes the call to `blake3::Hasher::finalize(&hash).as_bytes()`. The Nix `overrideVendorGitCheckout` path and the generator for the checkout-local `vendor-deps/` closure MUST both apply that patch file, and each MUST fail when the patch does not apply cleanly to the pinned source or when the vendored result differs from the patched source. The documented developer path, `cargo build` or `cargo check` inside `nix develop` without an explicit `--config` flag, MUST resolve Casita from the patched vendor closure and MUST NOT compile an unpatched git checkout. Mantle MUST NOT hand-edit a vendor snapshot or pin a fork. Evidence MUST NOT claim that unmodified upstream Casita builds in Mantle.

#### Scenario: Patched source compiles in both closures

- GIVEN the pinned revision, the tracked patch, and a clean checkout
- WHEN the Nix vendor build and the `vendor-deps/` generator run, followed by `cargo check` of `crunch-store`
- THEN both closures MUST contain the patched `nar.rs`, and the check MUST pass
- AND the evidence MUST name the patch file and its digest

#### Scenario: Patch drift fails closed

- GIVEN a patch that no longer applies to the pinned source, a vendored `nar.rs` without the patch, or a hand-edited vendored file
- WHEN the Nix vendor build or the `vendor-deps/` check runs
- THEN it MUST fail and name the file
- AND it MUST NOT fall back to the unpatched source or to network resolution

#### Scenario: Default dev shell builds the patched source

- GIVEN a clean checkout and the documented `nix develop` shell
- WHEN a developer runs `cargo check -p crunch-store --locked --offline` without an explicit `--config` flag
- THEN Cargo MUST resolve Casita from the patched vendor closure, and the check MUST pass
- AND no unpatched Casita git checkout MUST be compiled

### Requirement: Casita Rust cache reuse and GC require durable verified payload roots

r[mantle.casita_store_backend.castore_parity_gate] Standalone Rust unit cache nodes are castore-only and are not equivalent to `ActionResultPort` outputs; PathInfo-backed action-result outputs remain core and use output roots. The Casita profile MUST declare `rust-unit-cache` only when `RustCache::open_async`, the wrapper daemon, `rust-cache serve`, and `rust-plan` local and shared cache modes all use durable `mantle/castore/` roots. A fresh process MUST verify the node postcard, content, and root name before reusing a result. Under the store mutation guard, `store usage` and `store gc` MUST recover any pending Casita GC fence before verifying Rust retention live roots; a removed or repointed retained root MUST fail with `casita-root-missing` or `casita-envelope-invalid` before another root is removed or fenced. If no Rust cache state exists, output GC MUST NOT create it. Integrity failures MUST NOT be converted into a compiler run by a FailOpen cache policy.

#### Scenario: Rust cache reuses a verified persistent payload

- GIVEN a Casita Rust unit cache result with a retained payload root and a permitted local cache policy
- WHEN a fresh wrapper process requests the same action after session scratch is gone
- THEN it MUST verify and restore the payload without running the compiler
- AND a guarded GC using the Rust retention live nodes MUST preserve that root

#### Scenario: Output GC without Rust unit cache state

- GIVEN a Casita state directory with admitted outputs, one unretained output root, and no Rust unit cache state
- WHEN the operator runs `store gc` and then `store gc --execute --plan-id <id>`
- THEN planning and execution MUST succeed without opening the Rust unit cache
- AND the unretained output root MUST be removed at its planned target while retained outputs stay verifiable

#### Scenario: Output GC with existing Rust unit cache state

- GIVEN a Casita state directory with signed outputs and an existing Rust cache state directory with no retained nodes
- WHEN the operator runs `store usage`, `store gc`, and an accepted `store gc --execute --plan-id <id>`
- THEN verified retention planning MUST succeed and GC MUST remove only the unretained signed output
- AND the Rust cache state and retained output MUST remain readable with no incomplete fence

#### Scenario: Missing retained cache root blocks CLI GC

- GIVEN a Casita Rust cache retention record whose durable payload root another writer removed
- WHEN the operator runs `store gc`
- THEN it MUST fail with `casita-root-missing` before a GC fence or other root removal
- AND FailOpen wrapper reuse MUST reject without running the compiler
