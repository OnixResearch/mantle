# Proposal: Adopt Casita as a durable store backend

## Why

Mantle persists admitted outputs only in its Snix store: `blobs/`,
`directories.redb`, and `pathinfo.redb` under `--state-dir`. The physical
export under `--store` is a projection of that state, and GC rewrites both
databases and sweeps blob files (`crates/crunch-store/src/gc.rs`). Rust unit
action-result payloads live only in that castore.

Casita (`https://github.com/cachix/casita`, reviewed in
`evidence/casita-review.md` and ADR 0082) is a pre-release content-addressed
object repository. It verifies object graphs before publication, keeps named
roots, collects unreachable data, and assigns root lifecycle decisions to the
application. That division fits Mantle: Mantle keeps admission, trust, and
retention authority, and Casita keeps durable verified bytes.

Casita verifies object identity, not signer trust. The Snix PathInfo service
also verifies nothing on read, and store archive import trusts only the keys
given for that one run. A durable backend that reopens and reverifies outputs
needs a durable signer policy that the destination owns.

`add-store-backend-selection` supplies the selection seam, the recorded backend
identity, the capability profile, and the conformance rail. This change admits
`casita` as a second backend through that seam and keeps every core capability,
including in-place signature updates through `store sign` and PathInfo-backed
action-result outputs. Final-NAR repair stays unsupported under `casita` until
a durable repair fence exists. The profile declares `rust-unit-cache` after
durable castore payload roots and fresh-process no-recompile reuse are proven.

## What Changes

- Depend on Casita through one exact git revision with default features
  disabled and exactly `native` and `experimental`, which provides the
  conditional publication API. Represent every locked git dependency and lock
  update in `deny.toml`, the Nix vendor closure, and an owner-controlled
  `vendor-deps/` generation and check path.
  r[mantle.casita_store_backend.pinned_dependency]
- Add a destination-owned `casita-trusted-public-keys` policy in the existing
  `overlay-trusted-public-keys` format. When the file exists, the trust set is
  exactly its keys, and the local signing key is trusted only if the file lists
  it. Without the file, the trust set is only the state directory's
  `signing-key`; a key that `CRUNCH_CONFIG_DIR` places elsewhere is trusted
  only if the file lists it. The operator writes the file; Mantle never
  creates, extends, or copies it, and never enrolls a key. Mantle validates the
  file at store open and reads it again for every verification and import
  preflight, so publication and every read use the current file, and a removed
  key fails closed at the next verification, in a running process too.
  r[mantle.casita_store_backend.trust_policy]
- Admit an output only after the existing Snix-side build, NAR measurement,
  and signature admission succeed and a key in the trust set signed it. Stage
  the envelope with `content` and `pathinfo.json` (the deterministic serde
  JSON encoding of the signed PathInfo) unrooted in one Casita mutation
  session, then publish the permanent root
  `mantle/outputs/<40-hex store-path digest>` with one conditional publication
  that requires the name to be absent. A conflicting root, including one
  published concurrently by another Casita client, stays unchanged.
  r[mantle.casita_store_backend.output_admission]
- Keep `store sign` working in Casita mode by replacing the envelope through
  one conditional change that expects the current target.
  `store repair-final-nar` fails under `casita` with
  `casita-repair-final-nar-unsupported` before any state access, in the CLI and
  the library, until a durable repair fence exists; `snix` repair is unchanged.
  r[mantle.casita_store_backend.envelope_replacement]
- Publish Nario v2 and other atomic batches all-or-none through one
  multi-root conditional publication in one mutation session, bounded to
  1,024 paths by Casita's per-commit limit. A larger batch fails before
  staging and is never split. r[mantle.casita_store_backend.atomic_batch_import]
- Keep PathInfo-backed `ActionResultPort` outputs core as output roots with
  fresh-process rehydration. Give castore-only payloads, including standalone
  Rust unit cache nodes, permanent roots under `mantle/castore/`; publish
  each payload root before the result, index, or retention metadata that
  names it. The Casita profile declares `rust-unit-cache` after verified
  fresh-process reuse without another compiler invocation.
  r[mantle.casita_store_backend.castore_payload_roots]
- Complete the castore parity gate: a missing or changed retained payload
  root fails closed before compiler fallback or a GC fence; `store gc` still
  plans output GC without opening Rust cache state when none exists.
  Neither payload-root coverage nor the declared capability claims full
  interchangeability. r[mantle.casita_store_backend.castore_parity_gate]
- Carry one tracked, repository-owned patch for the pinned revision's
  `nar.rs:88` call, applied by both vendor paths with compile and drift
  checks, and never claim that unmodified upstream builds.
  r[mantle.casita_store_backend.pinned_patch]
- Keep signed PathInfo only inside the output root. Validate the root name,
  entries, trusted signature, content-address facts, and measured NAR facts on
  every read, and fail closed on an externally changed root.
  r[mantle.casita_store_backend.durable_pathinfo]
- Use Snix castore services in Casita mode only as per-session scratch outside
  `--state-dir`, with the Casita repository in a dedicated state-directory
  subdirectory. r[mantle.casita_store_backend.session_intermediates]
- Serve a verified cache hit from Casita in a fresh process after the physical
  export was deleted. r[mantle.casita_store_backend.fresh_process_reuse]
- Execute GC from a Mantle plan: recheck under the store mutation guard, write
  a recovery fence, remove each dead output or payload root through a
  conditional change at its exact planned target, then run Casita collection.
  GC without the guard fails with `casita-gc-guard-required`. While a fence is
  pending, reads and admission fail with `gc-recovery-required` until guarded
  `store gc` recovers it. r[mantle.casita_store_backend.plan_bound_gc]
- Keep Mantle the only retention authority: permanent roots only, no
  evictable roots, no reliance on Casita disk-pressure eviction.
  r[mantle.casita_store_backend.retention_authority]
- Document and test the Snix-to-Casita move: the operator writes the reviewed
  signer key into the destination policy, then uses the existing verified
  store archive transport. Import fails before any Casita mutation when the
  policy is missing or an import key is not in it, and `--trusted-public-keys`
  never becomes durable trust. r[mantle.casita_store_backend.verified_migration]
- Keep offline source and store archive formats unchanged and compile no
  Casita network profile. r[mantle.casita_store_backend.offline_boundary]
- Keep Casita types inside the store shell, extend the capability boundary
  checker, and declare the Casita profile: every core capability, bounded
  atomic batch import, and `rust-unit-cache` backed by durable payload roots,
  but no `store-repair-final-nar`, overlay composition, or unsigned admission.
  `--trust-unsigned` and `store repair-final-nar` fail closed under `casita`.
  r[mantle.casita_store_backend.capability_boundary]
- Record bounded non-claims. r[mantle.casita_store_backend.claim_boundary]

## Impact

- **Immediate consumer**: `mantle --store-backend casita build`, `store gc`,
  `store verify`, `store sign`, `store archive import`, Nario v2 import,
  PathInfo-backed action-result reuse, and `store roots`/`store pin` on a
  Casita state directory. The conformance rail from
  `add-store-backend-selection` runs on `casita`.
- **Immediate outcome**: admitted outputs, including PathInfo-backed
  action-result outputs, survive deletion of the physical export and session
  scratch and are reused by a fresh process without a rebuild, and only
  outputs signed by keys the destination trusts are published or served.
- **Durable capability**: a second verified persistence engine behind the
  same capability views, with a recorded upstream pin, a declared capability
  profile with bounds, a reviewable destination trust policy, and a repeatable
  vendor closure.
- **Maintenance owner**: Mantle store lifecycle owner (`crunch-store`) and the
  dependency owner for the Casita pin, feature set, `deny.toml` entries,
  vendor closure, and audit record.
- **Repeatability evidence**: the conformance rail on `snix` and `casita`,
  fresh-process reuse after export deletion, root-race, replacement, and
  interrupted-staging fixtures, all-or-none batch fixtures at 1,024 and 1,025
  paths, PathInfo-backed action-result reuse, `store gc` reachability with and
  without Rust unit cache state, durable castore payload reuse and tamper
  fixtures, missing, invalid, unauthorized, removed-key, and unsigned trust
  fixtures, NAR-mismatch, envelope-tamper, stale-plan, interrupted-GC,
  eviction fixtures, and a clean-source Nix vendor build.
- **Compatibility**: `snix` stays the default and keeps its formats and trust
  behavior. Store paths, NAR facts, signatures, action refs, source bundles,
  and store archives are identical across backends for the same signing key.
  Casita mode has no overlay composition, no unsigned admission, no final-NAR
  repair until a durable repair fence exists, and a 1,024-path batch bound;
  missing or changed retained cache payload roots fail closed before compiler
  fallback or a GC fence. `casita` is not fully interchangeable with `snix`.

## Scope

The change covers the dependency pin, feature set, `deny.toml` entries, and
vendor closure, the Casita adapter inside `crunch-store`, the destination
trust policy, conditional output, replacement, payload, and batch
publication, verification on read, per-session Snix scratch, GC execution and
recovery, retention rules, the migration procedure, the Casita capability
profile, offline and capability boundaries, and documentation. ADR 0082
records the decision.

The dependency closure additionally carries the exact validation-preserving
Bao candidate `eecfbbb458cc684fd85e056881580d307a1d1868` (upstream base
`2be9abd144783455606424424c29bd3a57f926f8`) as an in-repo path patch
with tracked file identities, while Casita's original pin and native feature
remain unchanged. This removes Bao's unmaintained proc-macro path locally;
it is not an upstream-reviewed Bao/Casita publication. In the combined
workspace on 2026-10-06, the configured locked offline `cargo-deny` audit
passed without new waivers (see `evidence/dependency-admission-2026-10-04.md`).
This does not qualify a source-built Rust compiler, hosted CI, or release.

## Non-Goals

- Casita S3, SSH, Git, OCI, IPC, CLI, or FUSE features.
- Overlay composition with Casita layers, and unsigned admission. Both are
  declared unsupported and fail closed until a separate change admits them.
- Atomic batches over Casita's per-commit root limit.
- Enrolling import, substitution, or source keys into the destination policy
  automatically, or a policy editing command.
- Moving source-bundle records, retention records, action-result records, or
  attestations into Casita. They stay Mantle-owned files under the state
  directory; PathInfo-backed outputs live under output roots, and castore
  payloads live under durable Casita roots.
- A dedicated `store migrate` command. The verified store archive transport
  already carries closures, signatures, and per-run trust checks.
- Preventing other Casita clients from changing roots after publication.
  Mantle never replaces a concurrently published target and detects later
  changes on every read.
- Changing GC decision rules or the Snix backend.

## Success Criteria

- A fresh process with `--store-backend casita` reuses an admitted output
  after the physical export is deleted, with no rebuild and equal NAR facts.
- A fresh process reuses a PathInfo-backed action-result output from its
  output root without a rebuild, and `store gc` works for outputs when no Rust
  unit cache state exists. A retained Rust unit payload rooted in Casita
  restores in a fresh process without invoking the compiler again; changed
  or missing retained payload roots block reuse and GC before fallback.
- A second Casita client that publishes under the same root name between
  staging and commit, or repoints a root during `store sign`, makes Mantle's
  change commit nothing and keeps the other client's target.
- A 1,024-path Nario v2 batch publishes in one revision. A 1,025-path batch,
  a batch with one conflicting root, and a batch with one failing later path
  commit no root.
- Import without a destination policy, with an import key that the policy
  lacks, or with `--trust-unsigned` fails before any Casita mutation.
- Removing a key from the policy makes outputs that carry only that key's
  signature fail closed at the next verification, in a running process or a new
  one.
- A tampered envelope or payload, a NAR mismatch, or an externally replaced
  root fails closed with its stable blocker.
- An interrupted GC execution never leaves a published PathInfo without its
  content: reads and admission fail with `gc-recovery-required` until guarded
  `store gc` recovers the fence, and recovery removes no root.
- A clean-source Nix build vendors the exact locked Casita revision and
  feature set, and `cargo deny check` passes.
