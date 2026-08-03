## ADDED Requirements

### Requirement: Explicit Rust compiler cache daemon boundary

r[rustc_cache_adapter.daemon_boundary] Mantle MUST provide the Cargo Rust compiler cache as an explicit thin-wrapper and local-daemon workflow. Wrapper processes MUST NOT own store databases or remote credentials.

#### Scenario: Authorized wrapper uses the local daemon

GIVEN an operator starts the Rust cache daemon with explicit typed policy
AND an authorized wrapper peer connects through the configured Unix socket
WHEN Cargo invokes `mantle-rustc-wrapper` with the real compiler and compiler arguments
THEN the wrapper MUST send one bounded versioned request to the daemon
AND the daemon MUST own cache lookup, storage, compiler execution, result publication, and receipt effects.

#### Scenario: Protocol and peer policy fail closed

r[rustc_cache_adapter.daemon_boundary.protocol]

GIVEN a client is unauthorized or sends a malformed, oversized, truncated, unsupported, timed-out, or replay-inconsistent request
WHEN the daemon evaluates the request
THEN it MUST reject the request before cache admission or compiler execution
AND diagnostics MUST use a stable bounded reason without exposing credentials or unbounded request bytes.

#### Scenario: Typed policy controls daemon authority

r[rustc_cache_adapter.daemon_boundary.policy]

GIVEN the daemon starts with human-authored cache policy
WHEN Mantle loads that policy
THEN the policy MUST pass a typed Nickel contract and deterministic export
AND it MUST define cache mode, failure mode, roots, peers, effects, limits, result sources, publication, and redaction before serving requests.

### Requirement: Declared-input eligibility for strong wrapper reuse

r[rustc_cache_adapter.strong_eligibility] Mantle MUST require a verified declared-input invocation manifest before strong wrapper cache lookup or publication. Compiler arguments alone MUST NOT establish strong identity.

#### Scenario: Complete manifest permits cache evaluation

GIVEN a Mantle-generated invocation manifest binds source, compiler, sysroot or provider closure, platform, arguments, admitted environment, dependencies, proc macros, build-script outputs, native-link inputs, output contract, effects, schema, and policy
WHEN the daemon verifies the manifest BLAKE3 and all current declared inputs
THEN the daemon MAY compute the common Rust unit action reference and evaluate cache candidates.

#### Scenario: Missing or changed input prevents strong reuse

GIVEN an invocation manifest is absent, malformed, unsupported, digest-mismatched, incomplete, or names a missing, changed, out-of-root, or unclassified input
WHEN the wrapper requests cache service
THEN Mantle MUST NOT report a strong cache hit or publish a result
AND it MUST use the selected pass-through or fail-closed policy.

#### Scenario: Eligible miss enforces the declared boundary

r[rustc_cache_adapter.strong_eligibility.enforcement]

GIVEN a cache-eligible invocation has no admitted result
WHEN the daemon invokes the real compiler
THEN it MUST use explicit argv, environment, working directory, readable roots, writable staging, resource limits, and effect policy
AND failure to enforce that boundary MUST cause pass-through or a policy blocker rather than cache publication.

### Requirement: Cargo wrapper behavior and bypass parity

r[rustc_cache_adapter.wrapper_parity] Mantle MUST preserve the observable compiler process contract for supported and pass-through Cargo wrapper invocations.

#### Scenario: Compiler result parity is preserved

GIVEN the wrapper invokes or delegates to the real compiler
WHEN the compiler completes
THEN the wrapper MUST preserve its exit success or failure, bounded stdout, bounded stderr, and required output visibility
AND the wrapper MUST NOT report success before required artifacts and receipts are committed.

#### Scenario: Unsupported invocation passes through without publication

r[rustc_cache_adapter.wrapper_parity.bypass]

GIVEN Cargo invokes a compiler query, incremental compilation, unsupported response form, unsupported output shape, missing manifest, unsupported compiler effect, or another declared bypass class
WHEN fail-open developer policy is selected
THEN the wrapper MUST invoke the real compiler without strong cache lookup or result publication
AND the receipt or diagnostic MUST identify the stable bypass class.

#### Scenario: Daemon loss follows explicit failure policy

GIVEN the daemon is absent, disconnects, times out, or returns an invalid response
WHEN the wrapper handles the failure
THEN fail-open policy MUST invoke the real compiler without cache publication
AND fail-closed policy MUST return a stable wrapper failure without pretending the compiler ran.

### Requirement: Atomic Cargo artifact restoration

r[rustc_cache_adapter.atomic_output_commit] Mantle MUST verify a complete admitted compiler artifact set before committing cached files to Cargo output paths.

#### Scenario: Cache hit commits exact expected artifacts

GIVEN the daemon admits a complete Rust unit result for the invocation action reference
WHEN it restores the result
THEN it MUST materialize and verify the full artifact set in private staging
AND it MUST commit only the expected Cargo output paths before returning wrapper success
AND the receipt MUST bind current artifact digests and a local or remote cache-hit reason.

#### Scenario: Restore failure leaves no successful partial result

GIVEN cached artifact restoration starts
WHEN materialization, verification, destination validation, or commit fails
THEN the wrapper MUST return failure or use an explicit pre-commit fallback
AND it MUST NOT return success with a partial artifact set
AND it MUST clean or quarantine private staging data.

#### Scenario: Restoration uses ordinary files

GIVEN Cargo owns a writable target directory
WHEN Mantle restores compiler artifacts
THEN it MUST materialize ordinary files into approved output paths
AND it MUST NOT mount read-only castore FUSE or virtiofs over Cargo's writable target directory.

### Requirement: Optional shared results and proof boundary

r[rustc_cache_adapter.shared_results] Mantle MUST use the common signed Rust unit result contract for optional remote wrapper cache reads and publication.

#### Scenario: Wrapper never receives remote credentials

GIVEN the daemon has explicit signed remote result policy
WHEN a wrapper requests an eligible compilation
THEN only the daemon MAY access remote result sources, object sources, signing keys, or credentials
AND the wrapper and compiler environments MUST NOT receive that secret material.

### Requirement: Strict evidence lanes exclude ambient wrapper caching

r[rustc_cache_adapter.proof_boundary] Mantle MUST keep ambient Rust wrapper cache state outside strict self-build, witness, and release proof claims unless a separate accepted change declares and admits it.

#### Scenario: Strict proof scrubs wrapper state

GIVEN a strict self-build, witness rebuild, or release proof starts without an admitted wrapper-cache contract
WHEN Mantle constructs the proof environment
THEN it MUST remove ambient `RUSTC_WRAPPER`, daemon socket, and wrapper-cache policy variables
AND the proof receipt MUST NOT claim wrapper cache reuse.

#### Scenario: Adapter evidence preserves non-claims

GIVEN Mantle reports a successful local or shared wrapper cache hit
WHEN a consumer reviews the evidence
THEN the claim MUST remain limited to the declared invocation, authority, content, policy, effect, and materialization facts
AND it MUST NOT claim full Cargo compatibility, compiler correctness, hermeticity for pass-through invocations, universal reproducibility, or release eligibility.

#### Scenario: Performance evidence includes wrapper overhead

r[rustc_cache_adapter.performance]

GIVEN the daemon-backed adapter is ready for lifecycle review
WHEN Mantle runs its bounded performance rail
THEN evidence MUST compare daemon round-trip, cold miss overhead, local restoration, remote restoration, and pass-through overhead
AND it MUST record named workload, sample, concurrency, byte, and timeout limits plus median and tail latency.
