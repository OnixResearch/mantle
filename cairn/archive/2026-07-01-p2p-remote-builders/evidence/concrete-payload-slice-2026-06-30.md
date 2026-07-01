# Evidence: concrete remote payload slice

Date: 2026-06-30

## Question

Can the remote-build protocol carry a bounded executable concrete payload identity before wiring the fixture server to the real build executor? Should this slice switch the wire codec to Preserves?

## Inspected evidence

- `src/remote_build.rs::ConcreteBuildRequest` now includes `payload: RemoteConcreteBuildPayload` and `expected_outputs: Vec<RemoteExpectedOutput>`.
- `RemoteConcreteBuildPayload` supports action payloads (`action_id`, serialized concrete action JSON) and derivation payloads (`drv_path`, serialized derivation JSON).
- `validate_concrete_request(...)` now rejects raw frontend evaluation, missing or oversized payload bodies, derivation paths outside the request store prefix, empty/duplicate expected output names, empty expected output lists, and expected output paths outside the logical store prefix before ticket redemption.
- The deterministic fixture digest now includes payload identity and expected outputs, so stale action/derivation identity changes affect the output metadata even before real executor wiring lands.
- `tests/remote_stdio_cli.rs` sends the new payload and expected-output fields through the public `mantle remote serve --binding stdio-once` child-process fixture.
- `https://preserves.dev` documents a syntax-neutral data model, Rust implementation, schema tooling, and canonical binary syntax. That makes Preserves plausible later, but not the best immediate change while the protocol semantics are still moving.

## Decision

Progress slice accepted, not full drain. Keep the first wire format as `u32be-length-prefixed-json` for now. Preserves should be revisited after the remote-build request/executor/output semantics stabilize, when the problem is codec/schema generation rather than core behavior. This slice adds executable payload identity but still does not execute it through the real Mantle build executor or persist signed PathInfo/artifact attestations.

## Owner

Mantle remote-build owner.

## Next action

1. Map `RemoteConcreteBuildPayload::Derivation` and `::Action` to a real executor boundary.
2. Return real output PathInfo/artifact metadata instead of deterministic fixture digests.
3. Revisit Preserves/canonical binary schema once field semantics are stable.

## Validation

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

```text
cargo test -p mantle --test remote_stdio_cli

running 2 tests
test remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames ... ok
test remote_serve_stdio_once_exchanges_frames_and_redeems_ticket ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
