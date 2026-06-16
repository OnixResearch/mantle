## Why

The materializer now knows that source-root musl can serve as a host layout for a future musl-host Rust provider, but coverage only checks classification and path helpers. We need a positive manifest fixture that exercises collection and zero-seed materialization through the same pure path the CLI uses after provider validation.

## What Changes

- Add a synthetic musl-host Rust provider/source-root fixture in native closure tests.
- Prove host `cc`/`ld` and runtime members are collected from source-root musl target-prefixed paths.
- Prove the resulting manifest has no seed exceptions and passes materialization validation.

## Impact

- **Files**: native closure tests, Cairn spec/evidence.
- **Validation**: focused positive fixture plus existing negative/guardrail tests, formatting, and diff checks.
