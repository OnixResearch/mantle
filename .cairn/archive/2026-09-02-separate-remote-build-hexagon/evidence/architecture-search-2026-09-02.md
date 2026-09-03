# Remote-build architecture search

## Question

What is the weakest extraction that gives Mantle a compiler-enforced remote-build hexagon without changing accepted remote behavior?

## Inspected evidence

- `src/remote_build.rs` contains about 17,700 code lines and mixes deterministic policy with filesystem, process, clock, random, credential, store, Tokio, wire, and rendering effects.
- `crates/crunch-build/src/distributed.rs` contains the realization seam. Its accepted pure identity and fallback logic was mixed with public Snix build request and result types.
- The seven specialized distributed modules contain about 9,600 code lines. Remote attempt, transfer, resource, telemetry, external-batch, attempt-log, and failure-debug requirements have separate active owners.
- Host Control owns transport-independent host topology and observations. Bounded Exec owns bounded process mechanics. Choregraph owns effect-intent graph semantics. None owns Mantle remote protocol, fencing, retry, output admission, or receipt meaning.

## Candidate portfolio

1. **Move every remote module in one change.** Rejected. It combines architecture extraction with active resource, transfer, telemetry, external-batch, and failure-debug policy changes.
2. **Keep the existing module and add facade traits.** Rejected. Vendor types and host authority would remain in application contracts.
3. **Adopt a generic sibling effect engine.** Rejected. No inspected component owns the required remote-build semantics, and a generic port layer would obscure authority.
4. **Add a strict core plus a narrow application shell, then move central decisions behind compatibility adapters.** Accepted. It changes the smallest semantic surface while making the inward dependency direction enforceable.

## Decision

Create `crunch-remote-core` and `crunch-remote`. Move realization identity, request admission, fallback, protocol hello, missing-input, output-trust, normalized-build identity, state transition, effect planning, and receipt-preimage meaning into the core. Isolate Snix in `distributed::snix_adapter`. Keep specialized active behavior behind explicit outer adapters.

## Owner

Mantle maintainers own remote-build meaning and the application contracts. Existing gateway, resource, transfer, telemetry, external-batch, nominal-type, failure-debug, and Trellis changes retain their policy authority.

## Next action

Run compatibility, architecture, `wasm32`, Clippy, Tiger Style, and Cairn gates. Accept only if existing focused remote tests remain unchanged.
