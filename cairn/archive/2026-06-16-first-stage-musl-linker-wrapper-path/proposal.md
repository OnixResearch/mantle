# Proposal: Put first-stage musl linker wrappers on PATH

## Summary

Ensure the generated first-stage Rust provider script prepends its private musl target-linker wrapper directory to PATH before running mrustc's `run_rustc` target build.

## Motivation

A real musl-host Rust provider attempt with the source-root musl toolchain reached the first-stage `run_rustc` link step and failed because `rustc` invoked plain `cc`. Mantle had generated a private `target-linker-bin/cc` wrapper, but it was not on PATH, so the host GNU `cc` handled the link and could not find musl CRT/runtime inputs.

## Scope

- Add the private target linker alias directory to PATH inside the generated first-stage script when the musl target wrapper branch is active.
- Keep target tool exposure scoped to the generated private wrapper directory.
- Preserve source-root musl and Nix-wrapper target tool discovery.

## Non-goals

- Do not claim a completed musl-host Rust provider proof.
- Do not change default GNU-host route semantics.
- Do not expose target tools as ambient host aliases outside the generated first-stage script.
