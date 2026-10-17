# Design: Add lock-driven vendor fetches

## Goal and scope

The bounded producer will read a fetched source's lock and emit fixed-output
fetches, so source bundles can stop shipping vendored trees. Cargo is the first
family. Pure registry-lock planning is implemented; dynamic admission, Git
hash-table acquisition, assembling, and profile cutover are not yet implemented.

## Current behavior

Existing profiles carry an explicitly supplied, generated `vendor-deps/`
directory with Cargo `.cargo-checksum.json` SHA-256 metadata. This directory
is ignored by Git, so a clean checkout does not have it; the isolated
origin/main baseline generated and checked its own input before profiling.
The self-build source guard validates it from `Cargo.lock` without host cargo.
Fresh-clone profiles carry the unpacked provider archive plus `vendor-deps/`;
the existing recorded profiles run to gigabytes, and the prepare phase
hex-decodes and re-hashes them repeatedly
(perf fixes already reduced passes). The accepted `source-transports` spec
owns bundle formats, canonical payloads, and verified import; the accepted
`dynamic-derivation-admission` spec owns admission of derivations produced at
build time with complete parent identity.

Using the preserved first executable on that isolated checked input, the
emitted `fresh-clone-inputs` source manifest's
`vendored-cargo-inputs.payload_bytes` is **896,632,634 bytes** across
45,231 files; the serialized source-bundle JSON is 1,806,345,932 bytes.
Those are two different measurements. The former came from the emitted
`records[]` field, not checkout `du`; no excluded-vendor profile exists yet,
so this establishes an old-profile baseline, **not** a size reduction.
`evidence/baseline-2026-10-01.json` contains exact inputs and commands.

The external reference implements exactly this pattern on stock Nix
(`fetch.cargoVendor` and siblings; a producer talks to the Nix daemon through
a worker-protocol client, one `builtin:fetchurl` per crate using the lock's
sha256, plus a collecting derivation; `evidence/repkgs-review.md`). Mantle
needs no worker-protocol client: the orchestrator already chains
fixed-output derivations and admits dynamic derivations natively.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Vendored trees in bundle | Ship `vendor-deps/` in source bundles | Rejected direction: derived state as source, gigabyte payloads | Bundle-size receipt comparison |
| Lock parsed at evaluation | Read lock in Nickel | Rejected: evaluation-time purity and cost; a 3,000-line lock must cost nothing |
| Producer derivation | Bounded producer emits per-artifact FODs plus assembler | Selected direction | Parent-identity admission, per-artifact verification |
| Fetch-then-rehash | Download once, hash ourselves | Rejected: invents a second hash source; the lock already has one | Tampered-lock denial fixture |

## Contract and component ownership

- Pure core: lock parsing, artifact admission, layout planning, and typed
  denials in a bounded core module; no network, no filesystem.
- Shell: the producer derivation body running under the fetch service, the
  assembling derivation, and the profile/hydration wiring in the source-bundle
  surface.
- Admission: emitted derivations flow through the accepted
  dynamic-derivation admission; no parallel mechanism.
- Policy: typed Nickel for profile modes and bounds; deterministic export.

### Cargo producer contract (first family)

The pure core accepts only UTF-8 Cargo.lock version 3/4 and bounded literal
`[[package]]` name, version, source, checksum fields (plus dependency lists).
Limits are 4 MiB of lock input, 4,096 fetched artifacts, and 2 MiB of planned
output; caller-supplied smaller bounds can further restrict each dimension.
Local workspace packages have no source and are not fetched. Registry packages
must use the canonical crates.io source and a lowercase 64-hex SHA-256 supplied
by their lock entry. The fetch URL is
`https://static.crates.io/crates/<name>/<name>-<version>.crate`. Git packages in
the current lock do not have archive hashes and are denied until the shared,
version-pinned table supplies reviewed content hashes; a Git commit id alone
is not an artifact checksum. The first complete source profile therefore
requires those table entries before normal acquisition can succeed.

The fallback table is a single UTF-8 file headed
`mantle-shared-lock-hashes-v1`, followed by sorted, unique
`<ecosystem/package@version/source><TAB><flat-sha256|recursive-sha256><TAB><64 lowercase hex>` rows,
ending in a newline. It is limited to 4 MiB and 4,096 entries. Cargo Git
identities use `cargo/<name>@<version>/<exact Cargo.lock source>`, selecting a
reviewed SHA-256 of the fetched tree's NAR serialization, the exact recursive
hash format used by `crunch-build::fetcher::verify_recursive_hash` through
`crunch_store::hash_host_path`; this is neither the Git commit ID nor Cargo's
per-file checksum manifest. The fetch URL and 40- or 64-digit revision come
from that lock source, never from a guessed digest.
Conflicting hashes deny a union merge. The pure plan retains exact consumed
Git table identities and `selected_shared_hashes` returns only those rows,
denying a later changed or missing hash. Shell wiring MUST bind these selected
bytes—not the whole mutable table—to an admitted package producer identity;
otherwise an unrelated table edit changes the producer and violates the
cache invariant.
No reviewed Git hash table entries have been installed in the root yet.

The assembler emits one package directory per artifact, matching Cargo vendor:
for each package name, the highest semver occupies `<name>/` and older
versions occupy `<name>-<version>/`. Relative paths reject absolute, empty,
`.`, `..`, backslash, colon, and repeated-separator components; the shell must also
reject symlinks and escaping unpacked archive entries. Registry archives and
Git trees are verified against their respective *admitted lock/table hash*
before layout. The shell generates Cargo's per-file `.cargo-checksum.json`
from verified file bytes without substituting those hashes for artifact
identity. Only after all
artifacts and their layout verify may the shell atomically publish output.

Git checkout bytes are **not** a Cargo vendor directory: at the exact
`casita` lock revision `90404fcb1cfb3d83f2233715448dfefe913f5fd1`,
`crates/casita/Cargo.toml` hashes
`d7ca68d7385e56a49f3a48789cced893f27a346accb53548d926135d8b52bfca`,
whereas the checked Cargo-generated vendor `casita/Cargo.toml` hashes
`67c626934d5f2a3d7d6e84c7eb508587e285e588b4be02e1610315eb658675ed`.
The latter begins with Cargo's automatic-normalization notice; it cannot
be obtained by cloning and copying the Git package directory. The assembler
must use Cargo's offline vendoring/normalization semantics on verified
inputs before comparing to the checked closure. The existing baseline
`scripts/vendor-deps.py` also applies its tracked
`patches/casita-blake3-finalize.patch` **after** Cargo vendor: the pristine
locked Git `src/nar.rs` SHA-256 is
`bed3012bed878a81b19348a2b927b95ea7e736b6888229d41a6342918a813e09`;
the independently checked vendor file SHA-256 is
`e88c332c3bb0605e5684e303c17e755adc66d2d525b80672dd7e16015f1be0b1`.
Equivalent assembly must bind that declared, tracked patch as an input, verify
the fetched Git tree before any transform, then check the patched vendor
contents. The generated manifest, Git revision, and per-file checksum manifest
are never substituted for the repository-tree fetch digest. Evidence records
the exact commands and independently regenerated vendor check.

Typed denials distinguish lock-byte, artifact-count, and planned-output
bounds; invalid UTF-8/grammar/field; duplicate field/identity; contradictory
checksum; missing hash; unsupported source; and unsafe layout. Denials name
the artifact when its name was parsed. The producer returns no partial plan.
Its output must enter normal dynamic-plan admission with its fetched-source
parent identity; neither Nickel evaluation nor the producer core reads a
lock path or opens a network connection. Fixed-fetch overrides can change
where verified bytes arrive from, not their admitted expected digest.

## Decisions

### Decision: Lock hashes are the only hash source

**Choice:** The producer never recomputes artifact hashes.

**Rationale:** A single hash source keeps integrity checkable end to end;
recomputing would create a second, drift-prone authority and re-opens the
fake-hash churn problem the reference explicitly removed.

### Decision: Cargo first, contract general

**Choice:** The contract is ecosystem-neutral; the first family is Cargo.

**Rationale:** Mantle's own dependency closure is Cargo; npm-style integrity
records can adopt the same contract without redesign, but each family lands
with its own negative controls.

## Risks / Trade-offs

- Producer runs add one build step before compilation; bounded admission
  keeps it small and cacheable.
- Hash-less ecosystems need shared tables; union-merge discipline must be
  documented and tested or parallel edits will conflict.
- The offline fixed-fetch override seam must keep working: override plans
  must be able to supply producer artifacts without live acquisition, as
  fixed-point stages already require.
