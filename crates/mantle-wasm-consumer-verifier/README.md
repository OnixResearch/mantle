# mantle-wasm-consumer-verifier

Published consumer contract for verifying Mantle WebAssembly component
materialization bundles (`mantle-wasm-component-materialization-bundle-v1`).

Independent consumers such as Molten, Kamacite, Aspen, Animus, and Lattice can
pin this crate to remeasure exact bundle members before runtime admission
without importing Mantle build, store, scheduler, release, or CLI authority.

## API

- `verify_consumer_bundle(MaterializationBundle) -> ConsumerVerificationReport`
  runs the structural layer over in-memory bundle data. Every structural and
  canonical-identity decision delegates to `crunch-wasm-component-core`; this
  facade never duplicates decision logic.
- `MaterializationBundle` is re-exported directly from the pure core.
- `ConsumerRoot::open(&Path)` opens and retains one directory capability,
  including caller-provided `/proc/self/fd/<fd>` bridges. Explicit root opens
  may follow the caller's root; replacing that ambient path later does not
  retarget the opened capability. All descendant components are no-follow.
  `verify_bundle(bundle)` first validates structure, then remeasures every
  required member, with streaming regular-file BLAKE3 and length checks.
  Only `source-closure` may additionally be a directory tree. Other roles
  remain regular-file-only. Shared paths require identical digest AND length;
  sharing a directory with another role is not admitted.
- `ConsumerRoot::copy_source_closure(&MaterializationBundle, &ConsumerRoot)`
  returns `Result<ConsumerMemberObservation, ShellError>`. It observes and
  copies only the declared source closure into a newly created matching
  logical path beneath the destination capability, preserving empty
  directories and exact file bytes. Existing targets are never clobbered.
  It returns role `source-closure`, the matching declared digest and size,
  and `Some(Matched)` only after bounded source/copy revalidation. The caller
  must clean its private destination on any error. This is a source-member
  observation, not whole-bundle structural admission or agreement: separately
  run `verify_bundle` and require the complete expected member report.
- The report (`mantle-wasm-consumer-verification-report-v1`) binds schemas,
  bundle and runtime-profile identities, member roles and BLAKE3 values,
  completed layers, ordered blockers, one status (`verified`, `blocked`,
  `rejected`), and fixed non-claims. It never carries logical store paths,
  payloads, credentials, environment values, or raw tool diagnostics.

Missing required bytes produce `blocked`. Drifted bytes produce the
`consumer-member-bytes-drift` blocker and `rejected`.

## Source-tree identity and bounds

Tree framing is unchanged from producer v1 `copy_source_tree`: breadth-first
directories, each directory's direct entries sorted by native filename.
Each directory contributes `dir\0`, a u64 little-endian UTF-8-lossy relative
path byte length, those path bytes, and u64 little-endian zero. Each file
contributes `file\0`, the same path frame, u64 little-endian actual content
length, and exact file bytes. The root itself has no record. Empty directories
participate; size is the sum of file content bytes only. Trees are not flattened
or archived, and bundle paths/digests are never rewritten. File-valued source
closures retain their existing unframed file identity.

The std shell uses the workspace-pinned `bounded-tree-cap`/`bounded-tree-core`
for observation, planning, copying, and revalidation, and cap-std/cap-fs-ext
for descriptor-relative member opens. Mantle owns only v1 digest framing.
Limits are 10,000 entries, directory depth 64 (files directly in depth-64
directories may have 65 components), and 536,870,912 aggregate file bytes.
The shared path bound is 65 × 256 bytes, covering Linux native component
lengths; shared lexical admission rejects backslash-containing components as
unsupported. Such producer-native names fail closed, never get rewritten.
Compatibility is for admitted tree shapes, not every producer-native name.
Existing regular-file bounds remain `MAX_MEMBER_BYTES` (1 GiB) and
`MAX_TOTAL_REMEASURED_BYTES` (4 GiB of unique declared member content).
Symlinks, sockets, devices, FIFOs, drift, and over-bound inputs never match;
regular-file opens are nonblocking to make FIFO substitution fail safely.

Source observation, copy, and revalidation detect drift but do not create an
atomic source snapshot. The caller must control the private destination;
success grants neither durability, publication, nor admission authority.


## Source acquisition

Pin an immutable revision of this repository and import only
`crates/mantle-wasm-consumer-verifier` plus its path dependency,
`crates/crunch-wasm-component-core`. Both are `no_std + alloc` at the core
edge; the default `std` feature enables the Linux capability shell and its
workspace-pinned bounded-tree and capability dependencies. Consumers must not
depend on sibling worktree paths. The producer crate is a dev-dependency only.

## Consumer flow

1. Receive a bundle document from the producer.
2. Parse it as `MaterializationBundle`.
3. Run `verify_consumer_bundle` for the structural verdict.
4. Open the consumer-owned member root and run
   `ConsumerRoot::verify_bundle` for byte remeasurement.
5. Consume the bounded report; never treat `blocked` or `rejected` output as
   verified, and never extend the report's non-claims.

## Claim boundary

A passing report does not prove source trust, compiler correctness,
component behavior, runtime isolation, authority, reproducibility,
deployment safety, or release eligibility. The report's `non_claims` field
is fixed by `CONSUMER_VERIFIER_NON_CLAIMS` and cannot be extended by
callers.

## Maintenance and verification

Mantle's Wasm component producer/verifier maintainers own this identity and
compatibility boundary. Molten is the current private-snapshot intake consumer;
pinning this provider and admitting a snapshot is not runtime adoption or
cutover. Actor/extension installation, effect linking, deadlines, and rollback
remain consumer work.

Focused checks (run in the checked-in development shell):

```sh
cargo test -p mantle-wasm-consumer-verifier --test consumer_fixtures
cargo check -p mantle-wasm-consumer-verifier --no-default-features
```

The [production operator rail](../../examples/projects/wasm-component-hello/README.md)
also verifies the actual unchanged pipeline bundle through this published
facade and copies/remeasures its directory source closure. These are commands
to run, not claims that checks have already executed.
