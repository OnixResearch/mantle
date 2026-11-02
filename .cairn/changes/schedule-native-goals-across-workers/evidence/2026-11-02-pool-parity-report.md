# Retained local/single-builder/pool parity report (2026-11-02)

Scope: record one captured comparison run for S12. The fixture is the
`production-remote-replay` derivation (`/bin/sh -c 'printf replay-ok > $out'`,
input-addressed, native sandbox). Only the output bytes of that derivation are
compared.

## Commands

Run from the repository dev shell with the ordinary test environment
(`CARGO_TARGET_DIR=/var/tmp/mantle-test-target`, pinned busybox as
`SNIX_BUILD_SANDBOX_SHELL`):

```text
cargo test -p mantle --test remote_transfer_production production_single_builder_and_pool_outputs_match -- --nocapture
cargo test -p mantle --test remote_transfer_production production_pool_output_matches_local_build -- --nocapture
```

## Captured output

```text
pool-parity single-builder: bytes=9 blake3=be6783f7a2a3be2305328be6871e90bbf07d5b9f08be219b93e0ffb9609e5075
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 1.68s

pool-parity local: bytes=9 blake3=be6783f7a2a3be2305328be6871e90bbf07d5b9f08be219b93e0ffb9609e5075
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 1.62s
```

Both tests assert byte equality; the printed digests are the retained
comparison values.

## Claim

For this derivation and this environment, the local build, the single-builder
remote path, and the pool path each produce a 9-byte output with BLAKE3
`be6783f7a2a3be2305328be6871e90bbf07d5b9f08be219b93e0ffb9609e5075`.

## Non-claims

- This is one fixture, not general build reproducibility.
- It does not prove compiler correctness, source reproducibility, release
  eligibility, CI success, or physical-target determinism.
- It does not cover multi-output or multi-goal derivations.
