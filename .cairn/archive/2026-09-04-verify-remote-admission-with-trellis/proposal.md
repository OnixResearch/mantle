# Change: Verify remote admission with Trellis

## Why

Mantle has a pure remote-attempt core for durable assignment identity, fence checks, report idempotence, phase transitions, retry policy, and output-admission authorization. Unit and property tests cover many cases, but they do not provide machine-checked proofs over every modeled transition.

Trellis already provides Verus-checked patterns for fencing, durable jobs, idempotence, leases, and rejection preservation. Mantle can use a product-neutral Trellis model and bind its proof evidence through the existing Kamacite and Valence Trellis-proof profile.

The proof must stay narrow. It cannot turn an abstract transition proof into claims about transport, persistence, cryptography, worker behavior, or whole-build correctness.

## What Changes

- Define a product-neutral fenced-attempt admission model in Trellis through a paired Trellis Cairn change.
- Prove stale-fence rejection, terminal-state closure, duplicate-event idempotence, conflicting-event rejection, completion linkage, and rejection preservation.
- Add a pure Mantle projection between `RemoteAttemptState` facts and the Trellis model.
- Add exhaustive finite parity fixtures for Mantle transitions and the proved model.
- Export and bind exact Trellis source, verifier, proof artifact, policy, assumption, and requirement identities through existing Kamacite and Valence evidence profiles.
- Keep Trellis out of Mantle's runtime trust and ordinary Cargo dependency graph.

## Dependencies

- The paired Trellis change `add-fenced-attempt-admission-primitives` owns the verified generic model.
- `extend-nominal-types-to-trust-boundaries` owns final remote identity and value admission.
- Active remote credential, gateway, and resource-policy changes must stabilize the report and fence facts used by the projection.
- ADR 0021 and the accepted Trellis proof sidecar contract define proof-evidence acceptance.

## Non-Goals

- Proving network delivery, durable storage, cryptographic signatures, worker execution, sandbox enforcement, liveness, or availability.
- Parsing Verus source or proof logs in Mantle core.
- Adding a runtime dependency on a sibling checkout or a workspace-relative product path.
- Replacing existing Rust tests, fault tests, or remote integration rails.

## Impact

- **Affected specs:** `remote-builds`
- **Affected code:** Trellis verified logic, Mantle remote-attempt projection, parity fixtures, and evidence adapters
- **Compatibility:** no remote wire or report change without a separate versioned requirement
- **Testing:** Trellis Verus proofs, positive and negative runtime tests, Mantle parity tests, mutation fixtures, sidecar binding, and Cairn gates in both repositories
