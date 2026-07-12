# Genuine release rebuild proof evidence

## Outcome

The focused implementation rail passes for commit `dd79b120` and the current
uncommitted lifecycle/documentation update. Exact command output is recorded in
[`validation.txt`](validation.txt).

Observed focused results:

- `crunch-release-core`: 186 passed.
- `release_reproduce_`: 13 passed, including source-derived positive runs and
  direct-copy/alias/symlink/hardlink/output-authority negatives.
- `release_verify_`: 54 passed.
- `release_nix_witness_`: 6 passed.
- Bootstrap-parity `self_build_`: 13 passed, including partial-on-genuine-v2 and
  blocked-on-legacy/missing authority behavior.
- Receipt checker, summary renderer, production recipe, and operator-guide
  self/drift checks passed.
- Focused Rust formatting, `crunch-release-core` Clippy with warnings denied,
  Mantle binary check, Cairn validation, and proposal/design/tasks gates passed.

## Decision evidence

- The v2 receipt binds the accepted content descriptor and authority-plan BLAKE3.
- Only separately materialized regular-file capabilities enter proof sandboxes;
  the complete release bundle is absent.
- Published target bytes, content-identical aliases, hardlinks, symlinks, prior
  proof outputs, and ordinary outputs are non-authorities.
- Release verification, standalone checking, summaries, Nix witnesses, and
  bootstrap parity all reject legacy or incomplete genuine-rebuild evidence.
- The checked bootstrap parity descriptor remains legacy v1, so the current
  project-root row is intentionally blocked until a genuine v2 release run
  refreshes that evidence.

## Bounded non-claims

No full production rebuild, compiler/verifier soundness proof, self-hosting proof,
full-bootstrap reproducibility proof, Guix parity proof, or StageX parity proof was
run or claimed by this focused validation.
