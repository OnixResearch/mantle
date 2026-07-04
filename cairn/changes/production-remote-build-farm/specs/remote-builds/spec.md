## ADDED Requirements

### Requirement: Production remote realization is scheduler-owned

r[remote_builds.production_scheduler_realization] Mantle MUST route production remote builds through the lazy scheduler and scheduler-compatible build service boundary. The scheduler MUST own goal readiness, dependency waiting, duplicate-goal dedupe, route selection, `max_jobs` accounting, waiter notification, local fallback policy, terminal-state propagation, and build report emission for both local and remote realizations.

#### Scenario: ready goal realizes remotely without bypassing scheduler

GIVEN a ready derivation or action goal has a selected remote-builder route
AND all concrete inputs, upload policy, capability facts, and output-trust facts are available
WHEN Mantle dispatches the goal
THEN the remote realizer MAY execute the goal
AND the scheduler MUST mark the goal done only after verified remote outputs are admitted through the ordinary store path.

#### Scenario: duplicate remote goal attaches instead of rebuilding

GIVEN two dependents request equivalent concrete build inputs with the same normalized realization key
AND the first request is queued, running, transferring, or finished but not delivered
WHEN the second request reaches scheduler dispatch
THEN Mantle MUST attach it to the existing goal or retained result
AND it MUST NOT start a second local or remote realization for that key.

#### Scenario: invalid remote route fails before success propagation

GIVEN a remote route would require raw frontend evaluation, mismatched store prefix, unavailable source/input material, invalid fallback policy, or missing output trust
WHEN the scheduler evaluates remote dispatch
THEN Mantle MUST reject the remote route before marking the goal complete
AND it MUST preserve a phase-classified failure or explicit local fallback decision.

### Requirement: Production output trust is cryptographic

r[remote_builds.production_cryptographic_output_trust] Mantle MUST admit production remote outputs only after cryptographic verification of returned PathInfo signatures and required attestation evidence against configured trust roots. Builder tickets, worker registration, SSH authentication, coordinator assignment, key names without matching material, or successful transport authentication MUST NOT be sufficient to import or persist remote outputs.

#### Scenario: trusted key material admits matching output

GIVEN a remote worker returns PathInfo, output content refs, and artifact-attestation evidence for the requested output
AND the PathInfo signature verifies against configured trusted public key material
AND the returned output name, store prefix, object refs, producer policy, and attestation digest match the concrete request
WHEN Mantle performs remote output admission
THEN Mantle MAY persist and export the output
AND the build report MUST record the trust basis without exposing private key paths or bearer tickets.

#### Scenario: same-name different-key output is rejected

GIVEN a remote worker returns PathInfo signed by a key with the same display name as a trusted key but different key material
WHEN Mantle validates the remote output
THEN Mantle MUST reject the output as an output-trust failure
AND it MUST NOT persist the PathInfo, export the output, or report a remote build success for that output.

#### Scenario: resource access is not output trust

GIVEN a client has a valid remote-build ticket, trusted-client credential, SSH identity, or coordinator assignment
AND the client lacks configured output signing-key or attestation-authority trust for the returned output
WHEN the remote route is planned or completed
THEN Mantle MUST reject the route or output import as an output-trust blocker
AND it MUST NOT treat resource authorization as a cache hit, substitution hit, or successful remote build result.

### Requirement: Production transfer is streaming and resumable

r[remote_builds.production_streaming_transfer] Mantle MUST transfer production remote inputs and outputs through bounded streaming artifacts keyed by declared content identities. The transfer layer MUST support CAS object, NAR, source-bundle, PathInfo, attestation, and delta artifacts; verify BLAKE3 content identities before use; preserve Nix-compatible NAR SHA-256 checks where required; enforce upload/download quotas and privacy policy; provide backpressure; and support safe resume or deterministic failure after interruption.

#### Scenario: large output streams without a single oversized frame

GIVEN a remote output is larger than the maximum control-frame payload
WHEN the client requests the output closure
THEN Mantle MUST stream the output through chunked CAS, NAR, or delta artifacts
AND output admission MUST verify every declared chunk/object identity before persistence.

#### Scenario: delta fallback remains claim-safe

GIVEN the client and worker advertise compatible delta transfer
AND the delta stream cannot be applied or verified before output admission
WHEN full transfer is available under policy
THEN Mantle MUST fall back to full transfer and verify the full output before import
AND the report MUST record the fallback reason instead of claiming a successful delta transfer.

#### Scenario: invalid transfer fails before sandbox or import

GIVEN a transfer has a digest mismatch, stale resume cursor, missing requested object, disallowed secret-descriptor upload, or quota overflow
WHEN Mantle validates the transfer
THEN Mantle MUST reject the transfer before sandbox execution or output import
AND diagnostics MUST identify the transfer phase and reason without revealing uploaded content.

### Requirement: Production coordinator runtime is restart-safe

r[remote_builds.production_coordinator_runtime] Mantle MUST provide a durable coordinator/worker runtime for production remote build farms. Workers MUST register endpoint identity, protocol version, systems, feature labels, sandbox modes, network modes, logical store prefixes, transfer capabilities, concurrency, output signing-key identities, and resumable job summaries. The coordinator MUST persist queue state, live leases, bounded log cursors, recent failures, and result-redelivery facts so restart or reconnect cannot fabricate success or duplicate work.

#### Scenario: worker registration enables deterministic assignment

GIVEN one or more workers have registered capability and trust-admission facts
AND a concrete build request declares required system, sandbox mode, network mode, feature labels, transfer policy, upload limits, logical store prefix, and trusted output keys
WHEN the coordinator plans dispatch
THEN it MAY assign the job only to a worker that satisfies every required fact
AND it MUST reject or explicitly pend the request when no matching worker is available.

#### Scenario: restart redelivers retained result

GIVEN a coordinator or worker restarts after a job has finished but before the client receives the admitted result
AND the worker re-registers a resumable summary for the same normalized build key and result identity
WHEN the client resubmits the request
THEN Mantle SHOULD redeliver the retained result through ordinary output admission
AND it MUST report the exact lost phase when retained state is unavailable.

#### Scenario: live leases prevent conflicting output ownership

GIVEN an active remote job owns a live output lease, transfer id, or result-delivery lease
WHEN a different normalized build request claims the same live output identity
THEN Mantle MUST reject the conflicting claim before dispatch
AND diagnostics MUST distinguish a lease conflict from duplicate-request attachment.

### Requirement: Accepted remote results are publishable artifacts

r[remote_builds.production_verified_publication] Mantle MUST publish production remote results only after local output admission has accepted the PathInfo, castore objects, artifact attestations, and required receipt evidence. Publisher adapters MAY target local archives, Nix-compatible binary caches, HTTP/S3 artifact stores, or future provider backends, but publication MUST remain separate from realization success and MUST NOT bypass output trust.

#### Scenario: admitted remote output is published for reuse

GIVEN a remote build output has been admitted into the local store through verified output trust
AND a publisher profile is configured for the output class
WHEN Mantle runs post-admission publication
THEN the publisher MAY export the verified artifact and metadata
AND a later request MAY resolve the artifact without remote execution after normal resolver verification.

#### Scenario: publication failure does not rewrite build truth

GIVEN a remote output has already been admitted locally
AND a configured publisher is unavailable, rejects credentials, or reports a duplicate artifact
WHEN publication runs after admission
THEN Mantle MUST report the publication failure or skip separately
AND it MUST NOT retroactively claim the build failed unless policy explicitly requires successful publication.

#### Scenario: publisher rejects unverified artifact

GIVEN a remote result has not completed local output admission
WHEN a publisher adapter is asked to publish it
THEN the publisher MUST reject the artifact
AND Mantle MUST report the rejection without treating publication as a verification substitute.

### Requirement: Remote farm configuration is typed and provider-neutral

r[remote_builds.production_operator_configuration] Mantle MUST load production remote build farm configuration through typed, reviewable, provider-neutral data. Mantle-owned config files MUST use Nickel contracts unless an external provider format is required, and runtime Rust MUST consume explicit exported data. Configuration MUST describe builder pools, capability profiles, upload and transfer limits, fallback policy, output trust roots, worker/resource policy, and publisher profiles without embedding provider-specific logic in scheduler/core modules.

#### Scenario: valid builder pool config selects capabilities

GIVEN a remote builder pool config declares endpoint identities, supported systems, sandbox/network modes, transfer capabilities, concurrency limits, upload budgets, and output trust roots
WHEN Mantle loads the config
THEN it MUST produce bounded capability facts for route planning and coordinator matching
AND provider-specific transport parameters MUST remain inside adapter configuration.

#### Scenario: unsafe config fails closed

GIVEN a remote builder config has duplicate endpoint ids, missing output trust roots, invalid upload limits, unsafe fallback defaults, or provider-specific fields in core scheduler configuration
WHEN Mantle validates the config
THEN Mantle MUST reject the config before remote dispatch
AND diagnostics MUST identify the invalid field without exposing credentials or private key paths.
