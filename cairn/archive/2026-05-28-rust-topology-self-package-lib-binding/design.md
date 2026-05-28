# Design

## Functional core

Add a pure target-normalization helper:

- input: package id, target kind, package-level selected dependency artifacts, package targets
- output: target-specific dependency artifacts

Rules:

1. External package artifacts are preserved.
2. Same-package artifacts are removed by default.
3. `bin` targets with a same-package `lib` target get exactly one lib artifact re-added with the lib crate name.

This keeps the supported Cargo shape explicit: package bins may consume the package lib, but the lib does not consume itself.

## Imperative shell

No new shell behavior. Existing `rust-plan` orchestration and topology execution consume corrected native derivation facts.

## Risks / Trade-offs

- Cargo has more target shapes than Mantle's bounded fragment. Keep this change scoped to supported `lib` / `bin` build targets.
- Filtering all same-package artifacts could hide a future supported target edge. For now, host/build-script same-package edges already flow through `consumed_host_artifacts`, and bin-to-lib is re-added explicitly.
