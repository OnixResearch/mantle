# Specification: C and C++ compile cache

## ADDED Requirements

### Requirement: Declared compiler driver boundary

r[mantle.cc_compile_cache.driver_boundary] Mantle MUST expose the compile
cache to bootstrap C/C++ builds through a declared compiler-driver seam that
is admitted by explicit policy and never by ambient PATH mutation outside
declared tool inventories.

The seam MUST be compatible with the protected-exec supervisor: a driver
install must remain inside the declared executable inventory and audit
decisions. The driver MUST forward invocations it cannot classify to the real
compiler unchanged.

#### Scenario: Unclassifiable invocation forwards unchanged

- GIVEN a driver invocation with arguments outside its classification
- WHEN the driver runs
- THEN the real compiler MUST run with the exact original arguments and the
  receipt MUST record the forward.

#### Scenario: Driver stays inside the protected inventory

- GIVEN a protected build with the driver seam enabled
- WHEN the supervisor audits executions
- THEN every driver and compiler execution MUST map to a declared inventory
  entry with no fallback events.

### Requirement: Content-keyed identity

r[mantle.cc_compile_cache.content_keyed_identity] Cache keys MUST be computed
over content identity only: source bytes, normalized arguments, tool identity,
and the learned set of dependency file identities.

Store paths MUST NOT enter a key; a dependency rebuilt to identical content
under a new path MUST still hit. Dependency sets MUST be learned from compiler
dependency files on first miss and stored as manifests; a manifest whose input
identity changes MUST miss.

#### Scenario: Rebuilt-identical dependency still hits

- GIVEN a cached compile whose only change is a dependency rebuilt to
  identical content at a different path
- WHEN the same compile runs again
- THEN the cache MUST hit and the receipt MUST record reuse.

#### Scenario: Learned dependency changes

- GIVEN a cached compile whose depfile manifest names a header whose content
  identity changed
- WHEN the same compile runs again
- THEN the cache MUST miss and recompile.

### Requirement: The cache is not a derivation input

r[mantle.cc_compile_cache.non_input_cache_boundary] The cache MUST NOT appear
as a derivation input, environment binding that changes derivation hashing, or
any other input that differs between cached and uncached builds.

Outputs MUST be byte-identical with the cache on and off. Cache unavailability
MUST degrade to plain compilation without failing the build. The mapping of
the daemon endpoint into sandboxes MUST be an execution-time concern recorded
outside the derivation graph.

#### Scenario: Identical outputs with and without the cache

- GIVEN the same derivation built once with the daemon available and once with
  no daemon
- WHEN the outputs are compared
- THEN they MUST be byte-identical.

#### Scenario: Daemon unavailable

- GIVEN a build with the cache enabled and the daemon unreachable
- WHEN compiles run
- THEN every compile MUST run plainly and the build MUST succeed with
  degraded-disposition receipts.

### Requirement: Proof lanes exclude the cache

r[mantle.cc_compile_cache.proof_exclusion] Fixed-point and release proof lanes
MUST either run with the cache excluded or verify content identity
independently of cache receipts.

A cache hit MUST NOT be accepted as proof evidence in any strict lane. Proof
transcripts MUST record the cache mode.

#### Scenario: Fixed point runs cache-off

- GIVEN a source-built fixed-point proof run
- WHEN the proof executes
- THEN the transcript MUST record cache exclusion and the stage digests MUST
  come from uncached compiles.

#### Scenario: Cache receipt presented as proof

- GIVEN a review packet that cites only cache receipts as rebuild evidence
- WHEN the strict lane evaluates it
- THEN the lane MUST reject the citation as insufficient.

### Requirement: Probe and failure caching is bounded

r[mantle.cc_compile_cache.probe_and_failure_cache] Optional caching of
configure-probe results and compile failures MUST use keys bounded over the
probe script identity, toolchain and dependency identities, platform, and
flags.

A wrong-key hit MUST be impossible for differing probe inputs; failure replay
MUST only occur when every input identity is known. Every probe-cache read
MUST record a disposition.

#### Scenario: Probe inputs change

- GIVEN a cached configure-probe result whose script or toolchain identity
  changed
- WHEN the probe runs again
- THEN the cached result MUST NOT be used.

#### Scenario: Failure replay with unknown inputs

- GIVEN a cached compile failure whose dependency manifest is incomplete
- WHEN the same compile is requested
- THEN the driver MUST recompile instead of replaying the failure.
