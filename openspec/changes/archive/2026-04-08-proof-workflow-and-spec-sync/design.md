## Context

The current tree is in an awkward middle state.

The self-hosting proof itself exists: `tests/self_hosting.rs` runs the checkout
binary for stage0, a produced binary for stage1, and verifies stage2. The
README already points at that test.

What is missing is the thin shell around it:

- there is no checked-in helper that sets the known-good build environment
- contributors still have to remember the nightly toolchain path, clang / mold,
  pkg-config / openssl lookup, and `SNIX_BUILD_SANDBOX_SHELL`
- the main specs are behind the code in a few visible places

The follow-up should stay small. We do not need a new proof engine. We need one
maintained entry point for the proof we already have, and we need the main
specs to stop promising behaviors that are not shipped.

## Goals / Non-Goals

**Goals:**
- One repo-local proof command or helper that contributors can run from the
  repo root
- Proof docs that point at that helper instead of scattering shell snippets
- Main specs that match the current worker and runtime behavior
- No ambiguity about what is shipped now versus what is still future work

**Non-Goals:**
- Replacing the ignored self-hosting test with a second proof implementation
- Making self-hosting proof part of every default `cargo test` run
- Implementing WASM, OCI, or remote builders in this change
- Changing the current self-build semantics beyond the helper / docs layer

## Decisions

### 1. Wrap the existing ignored proof test instead of inventing a new proof path

**Choice:** provide one checked-in helper that prepares the environment and then
runs the existing `cargo test -p crunch --test self_hosting -- --ignored --nocapture`
flow.

**Rationale:** the proof logic already lives in one place. A wrapper keeps that
as the single source of truth while removing the repeated environment setup.

**Alternative rejected:** add a second shell script that reimplements stage0,
stage1, and stage2 directly. That would drift from the test quickly.

### 2. Treat the helper as the documented proof entry point

**Choice:** the README and any nearby proof notes should point at the helper,
not at ad hoc command sequences.

**Rationale:** one entry point is easier to maintain, and it keeps the docs from
copying the same PATH / env boilerplate in multiple places.

### 3. Update specs from shipped behavior, not from older aspirations

**Choice:** the `build-pipeline` and `portability` specs should be rewritten to
match the current code and help text.

**Rationale:** contributors read the main specs as truth. If the worker already
runs ready goals concurrently and non-Linux builds still error out, the spec
should say exactly that.

### 4. Keep future portability work clearly labeled as future work

**Choice:** WASM, OCI, and remote builders stay in the design space, but the
main spec must not describe them as available runtime paths until the code
ships.

**Rationale:** this keeps the portability spec honest without abandoning the
architectural boundary around `BuildService`.

## Risks / Trade-offs

**[Helper drift]** A wrapper can go stale if the required build environment
changes. Mitigation: keep it thin and point it at the existing proof test.

**[Spec churn]** Portability wording has moved several times already.
Mitigation: anchor the update in current code paths and current error strings.

**[Long proof runtime]** A better entry point can make it tempting to run the
proof more often. Keep it opt-in and document expected runtime and disk usage.
