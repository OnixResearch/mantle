# Design: Verify Wasm source-closure trees

## Context and Success Contract

The producer `crunch-wasm-component/src/files.rs::copy_source_tree` already
publishes a tree identity. The consumer must match those exact admitted tree
bytes, not flatten or archive a tree, recompute a bundle identity, or substitute
its source role. An unchanged real pipeline bundle must verify through the
published facade; the same declared source must be copyable into a newly created
logical path under a caller-opened private destination.

## Existing Boundary, Not a New Identity

Keep `mantle-wasm-component-materialization-bundle-v1`,
`mantle-wasm-consumer-verification-report-v1`, pure structural delegation, exact
member observations, and fixed non-claims. Only source-closure gains directory
acceptance. Shared logical paths require both equal digest and equal length;
a directory cannot acquire another role by sharing its path. File-valued source
closures retain their original unframed byte identity.

Mantle owns the existing tree frame. Visit directories breadth-first and each
directory's direct native filenames in sorted order. Each directory contributes
`dir\0`, u64 LE UTF-8-lossy relative-path length, path bytes, and u64 LE zero.
Each regular file contributes `file\0`, that path frame, u64 LE actual length,
and exact content. The root has no record; empty directories participate and
size is only the sum of regular-file content. Sorting exact byte components by
component count then component-vector order reconstructs producer BFS over the
bounded plan without introducing a second filesystem walker.

## Shell and Dependency Reuse

`ConsumerRoot` holds a cap-std `Dir`, opened once from the explicit caller root.
That root may use a `/proc/self/fd` bridge or caller-supplied symlink. Retargeting
its ambient path does not retarget the capability. Every descendant parent is
opened separately with `open_dir_nofollow`; leaf opens are no-follow and
nonblocking, with opened regular-file kind and length checks and one EOF probe.
No ambient source reopens, registry/store search, whole-member buffering, or
unsafe code is needed.

Use workspace-pinned `bounded-tree-cap`/`bounded-tree-core` for bounded
observation, pure admission, prepared source binding, execution, and source
revalidation. Mantle hashes the accepted facts in v1 order, streams exact files,
compares their hashes against prepared facts, and repeats bounded observation
to detect name, empty-directory, kind, size, mode, and content drift. Directory
member root bindings are checked against their opened capability. The pinned
bounded-tree file-open helper must also use nonblocking no-follow opens so a
metadata-to-open FIFO substitution cannot hang. Publication requires that
immutable dependency fix; no vendor snapshot is edited.

Limits preserve producer policy: 10,000 entries, directory depth 64, and
536,870,912 aggregate content bytes. Producer checks depth when dequeuing a
directory, so regular files directly in depth-64 directories may have 65
components. Shared observation uses depth 65 and rejects directories deeper
than 64 afterward. A 65 × 256-byte path limit covers Linux native component
lengths. Shared lexical admission rejects backslash-containing native names;
these are explicitly unsupported and fail closed, without rewriting producer
paths or claiming universal producer-native filename acceptance. Non-UTF-8
names retain native ordering and producer lossy-UTF-8 hash framing.

## Source-Only Copying API

```rust
pub fn copy_source_closure(
    &self,
    bundle: &MaterializationBundle,
    destination: &ConsumerRoot,
) -> Result<ConsumerMemberObservation, ShellError>
```

The facade directly re-exports `MaterializationBundle` from the pure core.
The helper validates declarations/duplicate identities but does not claim
whole-bundle structural admission or measure unrelated members. Molten must
separately validate the bundle and require complete member-report agreement.
For trees, prepare and measure the declared identity before creating a target,
create a fresh matching logical path through destination capabilities, execute
the bounded plan, revalidate source facts, and remeasure the copied tree. Files
use create-new streaming and source remeasurement. Existing paths are never
clobbered. Return the declared source role, digest, size, and `Some(Matched)`
only on success; failures are typed. The caller owns cleanup of partial private
destinations. This helper has no publication or durability authority.

## Evidence and Maintenance

Implementation markers are in verifier `src/source_tree.rs` and `src/shell.rs`;
verification markers are in `tests/consumer_fixtures.rs` and the root
`tests/wasm_component_cli.rs` production pipeline rail. The small fixture uses
the real producer to establish identity, including nested/empty directories
and native non-UTF-8 filenames. Negative cases cover byte/name/empty-dir/size
drift, missing members, leaf/parent/root links, FIFO/socket nodes, bounds,
duplicate-path length conflict, shared non-source directory roles, destination
links, and create-new behavior. The production rail checks the unchanged actual
pipeline bundle and producer-remeasures the consumer's copied source.

Mantle's Wasm producer/verifier maintainers own framing and compatibility;
Molten is the current private-snapshot intake consumer. Commands and pending
verification are recorded separately; authoring tests is not passing evidence.
No new ADR is required: this repairs the existing identity/capability contract
using dependencies already adopted by Mantle rather than selecting a new
architecture.

## Risks and Non-Claims

Observe/copy/revalidate is not an atomic source snapshot. A caller must control
its private destination and decide admission only from complete reports. Mode
checks inside bounded-tree are conservative source-drift checks, not additional
Mantle digest framing. Reports remain path/payload-free. No source trust,
compiler correctness, component correctness, runtime isolation, runtime
authority, reproducibility, deployment safety, release eligibility, or runtime
adoption/cutover is claimed. Molten installation/effect linking/deadline/rollback
work remains outside this provider repair.
