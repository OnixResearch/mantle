# store-gc Specification

## Purpose
Define durable retained-root management and manual mark-and-sweep garbage collection for local crunch store state.
## Requirements
### Requirement: Durable GC root registry

The store layer MUST maintain a durable GC root registry for retained logical
store paths.

Each root record MUST survive process restart and include at least the logical
store path, the root source, and creation time. The registry MUST keep at most
one retained-root record per logical store path. Successful top-level outputs
from `crunch build`, `crunch self-build`, and `crunch bootstrap --fetch` MUST
be registered as retained roots after their output metadata is persisted
successfully.

#### Scenario: Top-level build output becomes a retained root

- GIVEN a successful `crunch build` of a requested root output
- WHEN crunch finishes persisting that output
- THEN the logical store path is added to the durable GC root registry
- AND a later process restart still sees that retained root

#### Scenario: Failed build does not create a retained root

- GIVEN a build that fails before final output persistence
- WHEN crunch returns the failure
- THEN no retained root record is created for that failed output

### Requirement: Manual mark-and-sweep collection

The system MUST provide manual garbage collection that marks reachability from
retained roots and sweeps unreachable local store state.

The mark phase MUST start from retained roots and walk `PathInfo.references`
transitively. The sweep phase MUST delete only unreachable exported outputs,
unreachable retained store metadata, unreachable attestation sidecars, and
unreachable castore content that are not reachable from any retained root.

Castore content is unreachable only when no surviving `PathInfo` still
references it.

#### Scenario: Reachable dependency survives collection

- GIVEN retained root `A` whose `PathInfo.references` transitively include `B`
- WHEN `crunch store gc` runs
- THEN both `A` and `B` remain present after collection
- AND exported outputs and castore content reachable from `A` and `B` remain
  present

#### Scenario: Unreachable output is collected

- GIVEN output `C` is not a retained root and is not reachable from any
  retained root
- WHEN `crunch store gc` runs
- THEN `C` is removed from local retained store state
- AND any exported output path for `C` is removed
- AND unreachable metadata, attestation sidecars, and castore content for `C`
  are removed as well

#### Scenario: Shared blob survives while a reachable path still references it

- GIVEN reachable output `A` and unreachable output `C` both reference the same
  castore blob
- WHEN `crunch store gc` runs
- THEN `C` may be removed
- BUT that shared castore blob remains present because surviving `PathInfo` for
  `A` still references it

### Requirement: Attestation sidecars follow reachability

Persisted artifact and closure attestation sidecars MUST follow the same
reachability decision as the local store state they describe.

Artifact attestation sidecars for surviving store paths MUST remain. Artifact
or closure attestation sidecars whose associated store paths or retained roots
are no longer reachable MAY be removed during the same GC run.

#### Scenario: Reachable artifact keeps its sidecar

- GIVEN a retained output whose artifact attestation sidecar exists under the
  state directory
- WHEN `crunch store gc` runs
- THEN the output remains reachable
- AND the corresponding artifact attestation sidecar remains present

### Requirement: Missing reachability metadata aborts collection

`crunch store gc` MUST fail closed before deletion if any retained root lacks
readable reachability metadata.

Missing `PathInfo`, unreadable retained-root metadata, or other retained-root
reachability corruption MUST abort the collection run before the sweep phase
starts.

#### Scenario: Missing retained-root PathInfo blocks collection

- GIVEN a retained root whose `PathInfo` cannot be read
- WHEN `crunch store gc` runs
- THEN the command exits non-zero before deletion begins
- AND no exported outputs, `PathInfo`, or castore content are removed

### Requirement: Operator root management surface

The CLI MUST provide explicit retained-root management commands.

At minimum:
- `crunch store roots` lists retained roots
- `crunch store pin <path>` adds or refreshes a retained root for that logical
  store path
- `crunch store unpin <path>` removes the retained-root record for that logical
  store path

#### Scenario: Operator pins an output explicitly

- GIVEN an existing local store path that should survive collection
- WHEN `crunch store pin <path>` runs
- THEN that path appears in `crunch store roots`
- AND later `crunch store gc` retains it

#### Scenario: Operator unpins an output

- GIVEN a retained root that is no longer needed
- WHEN `crunch store unpin <path>` runs
- THEN that retained-root record no longer appears in `crunch store roots`
- AND later collection may reclaim it if nothing else references it

### Requirement: Pin rejects invalid retained-root inputs

`crunch store pin <path>` MUST reject invalid retained-root inputs before they
enter the root registry.

At minimum nonexistent paths, paths outside the configured logical store, and
paths whose required retained-root metadata is unreadable MUST be rejected.

#### Scenario: Nonexistent path is rejected for pinning

- GIVEN an operator supplies a logical store path that does not exist locally
- WHEN `crunch store pin <path>` runs
- THEN the command exits non-zero
- AND no retained-root record is created

#### Scenario: Unreadable retained-root metadata is rejected for pinning

- GIVEN an operator supplies a logical store path whose required retained-root
  metadata cannot be read
- WHEN `crunch store pin <path>` runs
- THEN the command exits non-zero
- AND no retained-root record is created

### Requirement: Manual GC requires exclusive mutation access

`crunch store gc` MUST require exclusive local store-mutation access for the
lifetime of the run.

If another local build or substitution mutation is active, GC MUST refuse to
start before mark begins.

#### Scenario: Active build blocks manual GC

- GIVEN a local build is actively mutating store state
- WHEN `crunch store gc` starts
- THEN the command exits non-zero before the mark phase begins
- AND no deletion occurs

### Requirement: Dry-run GC reporting

`crunch store gc --dry-run` MUST compute the same reachability result as a real
collection run but MUST NOT mutate local state.

Dry-run output MUST report the retained-root count, candidate deletion count,
and reclaimable byte totals.

#### Scenario: Dry-run reports without mutating

- GIVEN a store containing both retained and unreachable outputs
- WHEN `crunch store gc --dry-run` runs
- THEN crunch reports what would be deleted and how many bytes are reclaimable
- AND no retained roots, `PathInfo`, exported outputs, or castore content are
  deleted

