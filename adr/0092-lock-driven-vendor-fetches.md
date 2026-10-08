# ADR 0092: Acquire Cargo vendor inputs from lock-driven fixed-output fetches

## Status

Proposed (2026-10-01). Pure planning and shared-table parsing are implemented;
dynamic admission, reviewed Git hashes, assembler, hydration proof, and acceptance remain open.

## Context

Fresh-clone and fixed-point source-bundle profiles currently carry a generated
`vendor-deps/` tree. `Cargo.lock` already records SHA-256 for registry archives,
while Cargo's vendor directory also contains per-file checksum manifests. A
checked-out `vendor-deps/` is ignored by Git and cannot serve as proof that a
clean-clone profile has any vendored payload. The current lock contains 18 Git
packages without archive checksums. The accepted dynamic-derivation admission
boundary provides a way to produce build requests after source realization
without allowing Nickel evaluation to inspect the lock or fetch network data.

## Decision Drivers

- Each registry fetch must verify the *exact* upstream lock checksum, not a hash
  guessed by a second producer or copied from a mutable vendor checkout.
- A missing checksum must deny acquisition until a reviewable pinned hash source
  exists. An immutable Git revision alone does not supply an artifact checksum.
- Fetch decisions, bounds, and vendor layout must be pure; filesystem,
  decompression, network, and publication remain in the build shell.
- An offline fixed-fetch override must supply the same pinned bytes and pass the
  same verifier; it cannot bypass hash admission.

## Decision

Use a bounded, no-std Cargo-lock producer core. It accepts v3/v4 package lists,
validates registry identities and lowercase SHA-256 checksums, and plans one
fixed-output request per registry package plus a deterministic Cargo vendor
layout. The highest semver of each name occupies `vendor-deps/<name>`; older
versions occupy `vendor-deps/<name>-<version>`. Local packages are not vendored.
Reject malformed, duplicate, conflicting, over-bound, and unsafe entries with
an artifact-specific typed denial. Git entries have no artifact digest in
Cargo.lock: until reviewed rows exist in the shared, sorted, version-pinned
table, they are denied. A row binds the exact lock source and package identity
to the reviewed SHA-256 of its fetched Git tree's NAR serialization, matching
the existing recursive fetch verifier, not merely its revision.
The table must feed each producer only its consumed subset bytes, so unrelated
entries cannot alter its input identity. No producer invents artifact hashes
or mutates the lock.

The normal build scheduler must run the producer only after its source input is
realized, admit its emitted fetches and one assembler through the existing
parent-bound dynamic admission, verify registry archive flat SHA-256 or Git
tree NAR SHA-256 against the respective admitted lock/table digest,
then assemble safely and generate Cargo's per-file `.cargo-checksum.json`
as a *derived file-integrity manifest*, not an alternative artifact hash
authority. Assembly publishes only after all artifacts
and the complete layout verify. Source-bundle excluded-vendor profiles must
record the producer identity and excluded *profile* payload bytes, and hydrate
and verify the assembled tree before self-build preflight. Existing profiles
retaining vendored inputs remain valid until real parity evidence allows a
reviewed cutover.

Raw Git checkout copy does not reproduce Cargo's vendor tree. At the locked
`casita` revision, the upstream `crates/casita/Cargo.toml` SHA-256 is
`d7ca68d7385e56a49f3a48789cced893f27a346accb53548d926135d8b52bfca`;
the independently regenerated vendor `casita/Cargo.toml` SHA-256 is
`67c626934d5f2a3d7d6e84c7eb508587e285e588b4be02e1610315eb658675ed`
and begins with Cargo's automatic-normalization notice. The assembler must
produce Cargo-equivalent normalized manifests from verified Git inputs rather
than substituting copied upstream files or inventing hashes from revisions.

The existing baseline also applies the tracked
`patches/casita-blake3-finalize.patch` *after* offline `cargo vendor`;
independent SHA-256 measurements change `casita/src/nar.rs` from upstream
`bed3012bed878a81b19348a2b927b95ea7e736b6888229d41a6342918a813e09`
to checked vendor
`e88c332c3bb0605e5684e303c17e755adc66d2d525b80672dd7e16015f1be0b1`.
The checked vendor output therefore requires this declared patch as an
assembler input, applied only after verifying the pristine fetched Git tree;
the patch is not a replacement for its reviewed recursive fetch hash.

## Consequences

Registry archive hashes protect artifact integrity, not dependency trust or
compiler correctness. An offline override can replace transport but cannot
replace checksums. Native dynamic admission and source-bundle receipt wiring
are prerequisites; pure planning alone does not realize a dependency or prove
bundle reduction. A fresh-clone profile with no explicitly supplied checked
vendor input cannot establish an old-payload size baseline.
