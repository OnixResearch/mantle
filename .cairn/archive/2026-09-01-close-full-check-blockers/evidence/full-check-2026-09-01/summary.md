# Full-check closeout evidence

## Verdict

Focused repair succeeded. The lexical bootstrap inventory is clean and remains
fail-closed. SpaceWasm and the Octet-bearing component toolchain rebuild from
current pinned inputs. Durable-publication freshness passes. Nine repaired core
packages pass tests, strict Clippy, and formatting.

Full `nix flake check -L` remains blocked. Local and ordinary runs stop at the
same strict Tiger Style debt in `crunch-store`: 139 location-backed findings
across 13 files. No gate was disabled or downgraded.

## Key counts

- inventory before: 115 findings, 355 classifications;
- inventory after: zero findings, 472 classifications;
- repaired package tests: 390 across nine suites;
- store Tiger blocker: 139 findings across 13 files;
- Tracey: 155/155;
- Cairn gates: proposal, design, and tasks pass.

## Repaired transport and fixed outputs

- SpaceWasm uses Crane vendoring and Crane's generated Cargo configuration.
- Fresh SpaceWasm build and `--rebuild` pass with output
  `/nix/store/s10dpbgap6vk219sgvfqlmnp8kj3x9rf-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3`.
- The wasm-component toolchain `--rebuild`, including pinned Octet evidence,
  passes.
- Earlier remote SpaceWasm and Octet hash mismatches do not recur.

## Review checkpoint

- **Question:** Did the work close the first full-check blockers without hiding
  the next one?
- **Inspected evidence:** exact V98 identities, mutation negatives, focused Nix
  rebuilds, Nickel freshness checks, package tests, Clippy, formatting, local
  and ordinary full checks, Cairn, and Tracey.
- **Decision:** accept the focused repairs and preserve the store Tiger debt as
  the next separate change.
- **Owner:** `close-full-check-blockers`.
- **Next action:** push and integrate the verified archive while keeping the
  store findings actionable.

## Non-claims

This evidence does not claim a green full flake check, store Tiger conformance,
compiler correctness, seed correctness, release reproducibility, deployment
success, or broader bootstrap parity.
