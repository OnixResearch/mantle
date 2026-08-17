# Bounded-tree adoption evidence

## Result

Mantle now uses `bounded-tree-core` and `bounded-tree-cap` for release-tree and frontend-tree mechanism work.

Mantle still owns artifact meaning, compatibility identities, diagnostics, transport, storage, publication, rollback, and release decisions.

## Prerequisite and source admission

- Bounded Tree establishment archive: `cairn/archive/2026-08-15-establish-bounded-tree`
- Radicle repository: `rad:git:zqhtZvsteJhxCJE96dMAZSZ9y1PX`
- Reviewed revision: `b0fd0103bc9eed2c1b6d852045959462d105d8f1`
- Mantle base revision: `20349655d5a890b063f6b9e9a488a65c313f0369`
- Cargo and Nix use the same immutable revision.
- The source-admission check rejects sibling-path and GitHub fallbacks.
- `nix build .#checks.x86_64-linux.bounded-tree-source-admission --no-link -L` passed.

The typed adoption receipt is `evidence/bounded-tree-adoption-receipt.ncl`.

## Implementation boundary

The release adapter maps Mantle limits to Bounded Tree limits. Bounded Tree observes the source, plans the tree, revalidates the source, and executes the capability-relative copy.

Mantle converts shared member facts back to its retained DTOs. It keeps UTF-8 relative-path string order because this order is part of existing release artifact identities.

The frontend adapter carries one prepared shared observation through identity calculation and copy. It preserves the `mantle-tree-v1` preimage and the existing executable-bit rule.

## Positive and negative evidence

The final focused and release suites passed:

- `cargo test -p crunch-release-core`: 233 passed.
- `cargo test -p mantle --bin mantle release_evidence::`: 44 passed.
- `cargo test -p mantle --bin mantle release_tree_copy::`: 8 passed.
- `cargo test -p mantle --bin mantle frontend_artifact_store::tests::`: 7 passed.
- `cargo test -p mantle --test release_cli`: 149 passed.
- `cargo clippy -p crunch-release-core --all-targets -- -D warnings`: passed.

The negative coverage includes malformed paths, parent-shape errors, limits, special files, external links, source mutation, destination collisions, and identity substitution.

The first broad release run found five compatibility regressions. Shared component-path order differed from Mantle string-path order, and plan errors lost blocker details. The adapter now restores Mantle order and maps typed blockers to retained diagnostics. All five regression tests pass.

## Octet evidence

The pinned default Octet gate completed with exit status zero and a `warning-only` verdict. Warning-only output is not clean acceptance evidence.

The same command on `origin/main` reported 809 inherited findings. The final change reported 803 findings. Findings in `tree_copy.rs` decreased from 21 to 15. The final file has no `unbounded_collection_growth` finding.

No warning baseline, disabled lint, or warning budget was added.

## Bounded validation blockers

`nix flake check --no-build -L` reached the pre-existing SpaceWasm package. Evaluation then failed because an upstream source is unavailable and a cached derivation path is invalid.

The standalone pinned Rust sysroot does not contain `wasm32-unknown-unknown`. The direct no-std Wasm check could not run. An earlier `nix develop` attempt was also blocked by an unrelated remote-store hash mismatch.

Broad first-party Clippy reaches an existing `nix_free_demo_cmd` enum warning. Broad vendored checks also report existing vendor findings. The focused changed core passes with warnings denied.

## Rollback

Revert the completion commit:

```text
git revert <adopt-bounded-tree-completion-commit>
```

The pre-adoption revision and lock identities remain in the typed receipt.

## Non-claims

This change does not claim source observation truth, storage durability, release eligibility, publication success, or whole-system correctness.
