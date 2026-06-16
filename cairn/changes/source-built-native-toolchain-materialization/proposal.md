## Why

Mantle now has honest promotion semantics for explicit source-built toolchain closures, but no current proof bundle can claim them because the explicit closure still lacks a zero-seed native host/target toolchain. The next step is to materialize or fail-closed on that native closure rather than manually assembling unverifiable manifests.

## What Changes

- Add an operator path that derives a source-built toolchain closure manifest from a concrete provider/root directory only when required executable and sysroot/runtime members exist and are digest-bound.
- Fail closed with deterministic diagnostics when host C/linker/libc/startup/runtime or target musl helpers are absent.
- Capture the current source-built Rust provider + native closure proof frontier and keep `not-source-built-toolchain-closure` until a zero-seed fixed point passes.

## Impact

- **Files**: native closure manifest generation/validation code, tests, Cairn spec and evidence.
- **Validation**: focused positive/negative unit tests, formatting, diff whitespace, and a real materialization/proof attempt transcript.
