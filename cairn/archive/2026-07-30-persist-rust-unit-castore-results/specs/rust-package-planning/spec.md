## ADDED Requirements

### Requirement: Castore-backed Rust unit result cache

r[rust_package_planning.unit_execution.topology.castore_result_cache] Mantle MUST support bounded local castore reuse for supported Rust topology unit results. Object presence alone MUST NOT authorize reuse.

#### Scenario: Canonical action identity binds every declared compiler input

r[rust_package_planning.unit_execution.topology.castore_result_cache.identity]

GIVEN a supported Rust unit has all producer artifacts and build-script facts ready
WHEN Mantle computes the Rust unit action reference
THEN the BLAKE3 identity MUST bind the unit, source, compiler, toolchain closure, platform, target, profile, features, normalized arguments, admitted environment, dependency artifacts, host artifacts, build-script facts, native-link facts, and compiler policy
AND it MUST bind the action-schema and policy versions
AND it MUST exclude the physical execution output root and classified temporary paths
AND any unclassified absolute path or missing required identity MUST block strong reuse and result publication.

#### Scenario: Identity changes invalidate local reuse

GIVEN Mantle has a local Rust unit result for one canonical action reference
WHEN any source, compiler, sysroot or provider closure, platform, target, profile, feature, semantic argument, admitted environment, dependency, host artifact, build-script, native-link, or policy identity changes
THEN Mantle MUST compute a different action reference or reject the candidate
AND it MUST NOT report a local castore hit for the prior result.

### Requirement: Verified local Rust unit restoration

r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse] Mantle MUST skip a supported Rust compiler invocation only after local result admission and complete artifact restoration succeed.

#### Scenario: Deleted execution outputs restore from castore

GIVEN a prior supported Rust unit result was admitted and published
AND its former execution output directory is absent
WHEN the same canonical action is executed with local Rust unit cache reads enabled
THEN Mantle MUST verify the result record and complete castore tree
AND it MUST restore the declared artifacts without invoking `rustc`
AND it MUST emit a fresh execution receipt with a local-castore-reuse reason and current output digests.

#### Scenario: Cache miss compiles and publishes last

GIVEN no admissible local Rust unit result exists for the canonical action reference
WHEN execution policy permits compiler execution
THEN Mantle MUST invoke the declared compiler
AND successful declared artifacts MUST be ingested and verified before the immutable result record becomes discoverable
AND a failed compiler, ingestion, verification, or publication step MUST NOT publish a candidate.

#### Scenario: Existing output-directory reuse remains first

GIVEN the active execution output root contains a matching receipt and complete matching artifacts
WHEN Mantle prepares the unit
THEN Mantle MUST use the existing output-directory reuse route before castore materialization
AND the receipt MUST distinguish output-directory reuse from local castore reuse.

#### Scenario: Rejected advisory candidate does not fabricate a hit

GIVEN a local result record is malformed, mismatched, over limit, or references incomplete castore content
WHEN Mantle evaluates the candidate
THEN Mantle MUST reject it with a stable reason
AND it MUST NOT report reuse or expose a partial restored output
AND an eligible compiler execution MAY continue only under explicit execution policy.

#### Scenario: Conflicting results remain visible

GIVEN multiple complete result records claim the same canonical Rust unit action reference
AND their admitted artifact sets differ
WHEN Mantle plans strong reuse
THEN Mantle MUST report deterministic nondeterminism evidence
AND it MUST NOT select a result by insertion order, discovery order, or last writer.

### Requirement: Atomic Rust unit materialization

r[rust_package_planning.unit_execution.topology.castore_result_cache.atomic_materialization] Mantle MUST materialize cached Rust unit artifacts through a bounded verified staging directory and an atomic filesystem commit.

#### Scenario: Restore failure leaves no partial output

GIVEN Mantle starts restoration into a new staging directory
WHEN export, path validation, artifact verification, or final commit fails
THEN Mantle MUST remove or quarantine the staging data
AND it MUST NOT publish a successful output directory or reuse receipt.

#### Scenario: Writable target mounts are not used

GIVEN the castore FUSE surface is read-only
WHEN Mantle restores Rust unit artifacts for compiler or Cargo consumption
THEN Mantle MUST use explicit materialization rather than mounting castore FUSE over the writable output root
AND virtiofs transport availability MUST NOT by itself authorize or complete restoration.

### Requirement: Rust unit cache retention and evidence

r[rust_package_planning.unit_execution.topology.castore_result_cache.retention] Mantle MUST keep local Rust unit result references and castore object retention consistent under bounded garbage collection.

#### Scenario: Live result references retain complete trees

GIVEN a retained local result index references an admitted castore tree
WHEN Mantle performs store garbage collection
THEN Mantle MUST retain every reachable object or remove the result reference before object deletion
AND a surviving result reference MUST NOT point to collected content.

#### Scenario: Cache evidence is bounded and redacted

r[rust_package_planning.unit_execution.topology.castore_result_cache.evidence]

GIVEN Mantle reports Rust unit cache behavior
WHEN it emits human or JSON evidence
THEN it MUST use stable hit, miss, rejection, conflict, restoration, and compiler-execution reason codes
AND it MUST report bounded candidate, artifact, restored-byte, and reused-byte facts
AND it MUST omit secrets, raw credentials, and unbounded ambient environment data
AND it MUST preserve non-claims for compiler correctness, full Cargo compatibility, remote trust, and universal reproducibility.

#### Scenario: Performance evidence compares all local routes

r[rust_package_planning.unit_execution.topology.castore_result_cache.performance]

GIVEN the local Rust unit cache implementation is ready for lifecycle review
WHEN Mantle runs the bounded cache benchmark rail
THEN the evidence MUST compare cold compilation, existing output-directory reuse, castore restoration after output deletion, and cache-miss overhead
AND it MUST record named sample limits, workload identity, artifact bytes, median latency, tail latency, restored bytes, and reused bytes.
