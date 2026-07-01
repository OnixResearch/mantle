# Evidence: remote input-ref and content-addressed output slice

Date: 2026-07-01

## Question

Can `mantle build --builder ... --ticket ...` stop rejecting non-empty derivation input refs, frame them through the stdio upload manifest, and dispatch content-addressed derivations whose final output paths are only known after remote execution?

## Inspected evidence

- `src/main.rs` no longer rejects planned remote client requests solely because `input_refs` is non-empty.
- `src/remote_build.rs` makes `RemoteExpectedOutput.logical_path` optional. Input-addressed requests still require exact output paths; content-addressed derivations carry `None` and bind the final path from returned signed PathInfo.
- Client request planning now preserves derivation/source input refs and records a bounded upload-byte estimate for the framed `InputUpload` control message.
- The local-build executor now validates that uploaded refs exactly match requested refs instead of rejecting all refs. The derivation registry built from the serialized `CrunchDerivation` payload materializes nested derivation graphs for the remote executor.
- Durable output admission/import still verifies request id, store prefix, output digest, trusted builder key, signed PathInfo binding, artifact-attestation digest, and final store-prefix path for content-addressed outputs.
- This is not full arbitrary input transfer: source/CAS byte upload, full output/CAS byte transfer beyond serialized PathInfo metadata, and production P2P transport remain open.

## Decision

Progress slice accepted, not full drain. The default stdio self-spawn local builder now handles content-addressed no-input derivations and nested derivation-input refs in temp-store smoke coverage. The remaining blocker is true byte materialization for source/CAS refs plus first-class top-level build-report integration and production transport.

## Owner

Mantle remote-build owner.

## Next action

1. Add real source/CAS input artifact frames that carry verifiable bytes/directories into the remote store before sandbox start.
2. Add full output/CAS byte transfer (delta-first with full fallback) beyond serialized PathInfo metadata.
3. Promote the remote client report into the stable top-level `crunch-build-report-v1` JSON schema.

## Validation

```text
Baseline attempt: cargo was absent from the ambient PATH. Rerun used the repo-documented Rust/linker/pkg-config environment.
```

```text
cargo test -p mantle --bin mantle remote_build::

test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 966 filtered out; finished in 0.01s
```

```text
cargo test -p mantle --bin mantle build_cli_accepts_remote_builder_ticket_dispatch_flags

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1026 filtered out; finished in 0.00s
```

```text
cargo test -p mantle --test remote_stdio_cli

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
cargo test -p crunch-glue

test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests crunch_glue: ok. 0 passed; 0 failed.
```

```text
rustfmt --edition 2024 --check src/remote_build.rs src/main.rs
git diff --check
cargo build -p mantle

Result: passed. cargo build finished dev profile after the focused test rail.
```

```text
manual smoke with /home/brittonr/.cargo-target/debug/mantle, SNIX_BUILD_BWRAP=/nix/store/g7svy17fhkg2cq3q4lfzzc0mmsl3d8hq-bubblewrap-0.11.2/bin/bwrap, and SNIX_BUILD_SANDBOX_SHELL=/bin/sh

ca-remote-smoke=passed output=/tmp/tmp.Qa8dpD9CsS/ca.out
nested-input-remote-smoke=passed output=/tmp/tmp.Qa8dpD9CsS/nested.out
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .

final gate output: stage=tasks valid=true verdict=PASS
```
