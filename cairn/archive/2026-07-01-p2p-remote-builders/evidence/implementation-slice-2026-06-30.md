# Implementation slice evidence — P2P remote builders

Date: 2026-06-30

## Implemented in this slice

- Added `src/remote_build.rs` with pure protocol/access-policy core for `mantle-remote-build/1`.
- Added version/ALPN/endpoint/capability validation, ticket authorization, request-shape validation that rejects raw frontend evaluation, delayed ticket redemption, missing-input set derivation, output-trust separation, and secret-redacted ticket views.
- Added persistent `mantle remote ticket create|list|inspect|reveal|revoke` CLI wiring.
- Added `mantle remote serve` diagnostic surface that reports the protocol metadata but does not start a transport yet.

## Current focused evidence

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-store-archive cargo test -p mantle --bin mantle remote_build::tests
running 7 tests
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 953 filtered out
```

## Remaining gap before archive

This is not yet a complete drain of `p2p-remote-builders`: no concrete P2P transport is bound, `mantle remote serve` does not accept sessions, client-side `mantle build --builder/--ticket` dispatch is not implemented, missing input upload is not wired to CAS/source state, signed output transfer/import is not integrated with real builder execution, and loopback remote-build integration tests still need a transport decision.
