# 0112: Separate remote decisions from host authority

- Status: Accepted
- Date: 2026-09-02

## Context

Mantle's remote-build path has accepted protocol, retry, fencing, transfer, resource, output-trust, and receipt behavior. However, `src/remote_build.rs` combines those decisions with filesystem, process, transport, clock, random, credential, store, async-runtime, and rendering effects.

The provider-neutral `crunch-build::distributed` module also exposed Snix build and store records in its public seam. That made provider types part of application contracts.

A single move of all remote code was rejected. The current distributed modules contain active gateway, resource, transfer, telemetry, external-batch, and failure-debug behavior owned by separate requirements. A bulk move would combine architecture extraction with unrelated policy changes.

Host Control, Bounded Exec, and Choregraph were inspected as reuse candidates. They own host contracts, bounded process mechanics, and effect-intent graphs. None owns Mantle's remote-build protocol, fencing, retry, output admission, or receipt semantics.

## Decision

Add two narrow crates:

- `crunch-remote-core` is `no_std + alloc`. It owns Mantle remote commands, admitted facts, normalized identities, protocol admission, retry and trust decisions, deterministic state transitions, events, typed effects, observations, outcomes, and receipt preimages.
- `crunch-remote` is the application shell. It defines eight independent capability ports and executes one core-planned effect before it submits the resulting observation to the next transition.

Keep Snix request and result types in `crunch-build::distributed::snix_adapter`. Keep stdio, SSH, local executor, external batch, store, credential, clock, identifier, attempt-persistence, and telemetry adapters outside the core.

Move realization-key derivation, build-service request admission, fallback classification, hello admission, missing-input derivation, output-key trust, and normalized remote-build identity behind the core. Keep compatibility projections for existing Rust APIs and wire bytes.

Active gateway, resource-policy, transfer, telemetry, nominal-type, failure-debug, and Trellis work remains with its current owner. Adapters translate those accepted facts into core observations without redefining their policy.

## Consequences

Remote decisions can replay without host authority. A core effect proves only that an effect is requested. Success requires a matching typed observation from the selected port.

Vendor and host types remain visible in outer adapters, but they cannot enter core or application port contracts. A checked architecture rail enforces this dependency direction and includes one negative fixture for each forbidden authority class.

The existing wire protocol and operator behavior remain unchanged. New wire and receipt fixtures compare compatibility projections byte-for-byte.

The extraction does not prove worker honesty, transport confidentiality, successful effects, compiler correctness, output trust without admitted evidence, or release eligibility.
