# Validation evidence

## Baseline

- Repository Tiger derivation: status 1, with 139 location-backed findings in
  the first failing `crunch-store` target surface.
- Focused package command: status 1, with 183 findings across 13 files.
- Pre-change tests: 357 unit tests and 2 integration tests pass.

The focused package result is the stronger implementation baseline. Completion
requires zero findings there and in the repository derivation.

## Accepted result

- Focused and repository Tiger transcripts contain zero `crunch-store` finding.
- Post-change tests pass 357 unit and 2 integration cases.
- Strict Clippy, Mantle binary compatibility, formatting, and no-build Nix
  evaluation pass.
- Local and ordinary full checks advance to the later 20-finding build boundary.

## Review checkpoint

- **Question:** Did strict store conformance return without changing store
  meaning or weakening the gate?
- **Inspected evidence:** both Tiger baselines, nine repair rounds, 359
  pre-change and post-change tests, strict Clippy, formatting, public-caller
  compilation, local and ordinary Nix checks, and the no-suppression diff.
- **Decision:** accept the store repair. Preserve the 19 `crunch-build` and one
  `crunch-rustc-wrapper` findings as the next separate change.
- **Owner:** `repair-crunch-store-tigerstyle`.
- **Next action:** commit the synchronized archive, push the verified branch,
  and integrate it without altering the later blocker.
