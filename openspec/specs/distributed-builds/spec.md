# distributed-builds Specification

## Purpose
TBD - created by archiving change add-distributed-build-interfaces. Update Purpose after archive.
## Requirements
### Requirement: Provider-neutral Distributed Build Interfaces [r[distributed-builds.interfaces]]

Mantle MUST expose distributed-build behavior through provider-neutral interfaces for realization-key derivation, artifact resolution, artifact publication, realization policy, and derivation realization. Core scheduler and build-pipeline logic MUST NOT hard-code concrete remote cache providers, remote realization services, Cargo wrappers, or vendor-specific protocols.

Mantle MUST preserve derivation realization as the distributed unit of work. Distributed-build interfaces MUST NOT require Mantle to model or schedule arbitrary build-system sub-actions such as individual Rust crate compilations, C/C++ translation units, Buck actions, or Bazel actions.

#### Scenario: Default realization path stays local-only [r[distributed-builds.interfaces.default-local]]

- GIVEN no distributed-build resolver, publisher, or remote realizer profile is configured
- WHEN Mantle realizes a derivation
- THEN the existing local PathInfo/substitution checks and local sandbox realizer remain the only active behavior
- AND no remote service, provider SDK, or network endpoint is contacted

#### Scenario: Provider backend is adapter-owned [r[distributed-builds.interfaces.adapter-owned]]

- GIVEN a future backend uses S3, Redis, REAPI, HTTP, SSH, or another transport
- WHEN that backend is registered with Mantle
- THEN backend-specific client libraries, endpoint formats, authentication, and retry policy stay inside the adapter boundary
- AND scheduler/core realization logic depends only on Mantle interfaces and capability data

#### Scenario: Sub-actions stay inside the builder [r[distributed-builds.interfaces.derivation-unit]]

- GIVEN a derivation builder internally runs Cargo, Make, GCC, Buck, Bazel, or another build tool
- WHEN Mantle schedules distributed work for that derivation
- THEN Mantle treats the derivation realization as the schedulable unit
- AND any tool-internal compile, link, test, or genrule sub-actions remain internal to the builder unless represented as separate Mantle derivations

### Requirement: Mantle-native Realization Keys [r[distributed-builds.interfaces.keys]]

Mantle MUST derive realization keys from normalized Mantle derivation realization facts rather than Rust/Cargo compiler-wrapper state or provider-specific cache paths. The key derivation MUST be deterministic and MUST include every modeled fact that can affect the output identity or admissible execution environment.

At minimum the key input model MUST cover derivation identity, declared input closure identities, builder path, arguments, normalized environment, system/platform, relevant toolchain store paths, sandbox/hermeticity mode, store prefix semantics, and realizer profile facts that can affect output.

#### Scenario: Equivalent derivation realization facts produce the same key [r[distributed-builds.interfaces.keys.stable]]

- GIVEN two realization requests with equivalent normalized derivation, input, toolchain, sandbox, platform, store-prefix, and realizer-profile facts
- WHEN Mantle derives realization keys for both requests
- THEN both requests produce the same key

#### Scenario: Output-affecting fact changes the key [r[distributed-builds.interfaces.keys.perturbation]]

- GIVEN a realization request has an output-affecting change in input closure, builder path, normalized environment, platform, sandbox/hermeticity mode, store-prefix semantics, toolchain path, or realizer profile
- WHEN Mantle derives a realization key
- THEN the changed request produces a different key from the original request

### Requirement: Artifact Resolver and Publisher Adapters [r[distributed-builds.interfaces.artifact-adapters]]

Mantle MUST model artifact reuse and artifact publication through resolver and publisher adapters. Resolvers MAY consult local PathInfo, configured binary caches, or future remote artifact stores, but they MUST return structured outcomes without mutating realization state directly. Publishers MUST accept only verified realization artifacts.

#### Scenario: Resolver hit skips realization [r[distributed-builds.interfaces.resolver-order.hit]]

- GIVEN a configured resolver returns a verified artifact candidate for a derivation output key
- WHEN the realization pipeline processes that ready derivation
- THEN Mantle verifies/adopts the artifact through the store finalization boundary
- AND the derivation is not realized locally or remotely

#### Scenario: Resolver miss falls through to realization [r[distributed-builds.interfaces.resolver-order.miss]]

- GIVEN all configured resolvers report a miss or non-fatal unavailable state
- WHEN the realization pipeline processes a ready derivation
- THEN Mantle dispatches the derivation through the configured realization policy
- AND existing local realization remains the fallback when no remote realizer is selected

#### Scenario: Publisher rejects unverified artifact [r[distributed-builds.interfaces.artifact-adapters.publish-verified-only]]

- GIVEN a realization result has not completed Mantle output verification and PathInfo/castore finalization
- WHEN a publisher is asked to publish that result
- THEN the publisher refuses the artifact
- AND the refusal is reported without treating the realization itself as successful publication

### Requirement: Realization Policy and Realizer Boundary [r[distributed-builds.interfaces.realizers]]

Mantle MUST route derivation realization through a `DerivationRealizer` boundary selected by a data-driven realization policy. The existing local sandbox builder MUST be the required baseline realizer. Remote realizers MUST be optional adapters and MUST NOT be required for default builds.

#### Scenario: Local realizer remains reference implementation [r[distributed-builds.interfaces.realizers.local-reference]]

- GIVEN a derivation that can build on the current host
- WHEN the realization policy selects the local realizer
- THEN Mantle runs the existing sandboxed realization path
- AND final output verification, log capture, PathInfo persistence, and scheduler completion semantics remain unchanged

#### Scenario: Remote realizer is explicit opt-in [r[distributed-builds.interfaces.realizers.remote-opt-in]]

- GIVEN a remote realizer adapter is available but not selected by configuration or capability policy
- WHEN Mantle schedules ready derivation goals
- THEN no derivation is sent to the remote realizer
- AND the remote realizer cannot be selected implicitly because it is compiled in or registered

### Requirement: Scheduler-owned Goal Semantics [r[distributed-builds.interfaces.scheduler-policy]]

Mantle MUST keep goal readiness, dependency waiting, deduplication, terminal-state propagation, and `max_jobs` bounds owned by the lazy goal scheduler even when realization placement is delegated to policy. Realizer adapters MUST NOT implement their own competing dependency scheduler.

#### Scenario: Independent goals may be placed differently [r[distributed-builds.interfaces.scheduler-policy.independent]]

- GIVEN two independent ready derivation goals and a realization policy that selects different realizers for them
- WHEN Mantle dispatches work with `max_jobs` greater than one
- THEN the scheduler may place one goal locally and one goal remotely
- AND both goals still complete through the same scheduler terminal-state and waiter-notification path

#### Scenario: Duplicate goal is not executed twice [r[distributed-builds.interfaces.scheduler-policy.dedup]]

- GIVEN two dependents require the same derivation goal
- WHEN that goal is already pending, running, or resolved through a resolver hit
- THEN Mantle MUST NOT dispatch a second local or remote realization for the same goal key

### Requirement: Remote Results Are Verification Candidates [r[distributed-builds.interfaces.remote-verification]]

Remote realization results MUST be treated as untrusted candidates until Mantle verifies returned output content and metadata through the local store finalization boundary or an explicitly equivalent verifier. A remote success receipt MUST NOT by itself create trusted PathInfo.

#### Scenario: Remote candidate accepted after verification [r[distributed-builds.interfaces.remote-verification.accept]]

- GIVEN a remote realizer returns logs, output content references, and metadata for a completed derivation
- WHEN Mantle verifies the returned content, references, hashes, and expected output identity
- THEN Mantle may persist PathInfo and report the derivation as successful
- AND the report records that realization happened through a remote realizer adapter

#### Scenario: Remote candidate rejected on mismatch [r[distributed-builds.interfaces.remote-verification.reject]]

- GIVEN a remote realizer reports success but returned content, references, hashes, output identity, or required metadata do not match Mantle verification expectations
- WHEN Mantle processes the remote result
- THEN Mantle rejects the candidate before PathInfo persistence
- AND the scheduler reports a verification failure or falls back according to explicit policy

### Requirement: Capability Configuration Without Provider Lock-in [r[distributed-builds.interfaces.config]]

Mantle MUST configure distributed-build behavior through logical resolver, publisher, realizer, and capability profiles. Core defaults MUST remain provider-neutral and MUST NOT name concrete services as required dependencies.

#### Scenario: Profile selects capabilities instead of vendor [r[distributed-builds.interfaces.config.capabilities]]

- GIVEN a configuration profile requests trusted artifact resolution, artifact publication, or remote realization capability
- WHEN Mantle loads the profile
- THEN Mantle matches the request against registered adapter capabilities
- AND missing optional adapters fail closed with a clear diagnostic rather than enabling an implicit provider

#### Scenario: Default dependency graph excludes optional providers [r[distributed-builds.interfaces.config.default-deps]]

- GIVEN a default Mantle build with no distributed provider feature enabled
- WHEN dependencies and runtime configuration defaults are inspected
- THEN provider-specific SDKs and remote-realization clients are absent from the default dependency graph
- AND local realizations remain supported

### Requirement: Distributed Build Operator Diagnostics [r[distributed-builds.interfaces.diagnostics]]

Mantle MUST emit structured operator diagnostics for distributed-build decisions, including resolver hit, resolver miss, resolver unavailable, publication skipped, publication succeeded, local realization selected, remote realization selected, remote fallback, and remote verification rejection.

#### Scenario: Cache miss and local fallback are visible [r[distributed-builds.interfaces.diagnostics.local-fallback]]

- GIVEN configured resolvers miss or are unavailable
- WHEN Mantle falls back to local realization
- THEN the realization report includes structured diagnostics identifying the miss/unavailable state and the selected local realizer

#### Scenario: Remote rejection is visible [r[distributed-builds.interfaces.diagnostics.remote-reject]]

- GIVEN a remote realization candidate fails verification
- WHEN Mantle rejects the candidate
- THEN the realization report includes a structured diagnostic with the realizer adapter identity, verification failure class, and fallback/failure decision

### Requirement: Distributed Scheduler Wiring Behind Local Defaults [r[distributed-scheduler-wiring]]
Mantle MUST wire distributed build interface seams into scheduler/orchestration only behind explicit local-only-preserving defaults until a provider implementation is separately accepted.

#### Scenario: Default behavior remains local [r[distributed-scheduler-wiring.1]]
- GIVEN no distributed resolver, publisher, or remote realizer is configured
- WHEN a build is scheduled
- THEN the existing local build path and goal dedup semantics are preserved

#### Scenario: Opt-in resolver decision is visible [r[distributed-scheduler-wiring.2]]
- GIVEN an explicit resolver profile is configured
- WHEN a realization key is available before local dispatch
- THEN resolver hit, miss, fallback, or rejection is represented in distributed diagnostics
