## Implementation

- [x] [serial] I1 Inventory Crunch-vs-Mantle occurrences in user-facing docs, examples, help text, and status surfaces, classifying each as stale prose or required exact identifier. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: `docs/mantle-naming.md` records stale-prose rules and exact required contexts; pueue task 15 showed the pre-change unclassified script/status occurrences.
- [x] [serial] I2 Update stale user-facing prose to Mantle while preserving required `crunch-*` identifiers and historical archive content. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: updated proof script/help/non-claim text to Mantle while preserving serialized compatibility tokens such as `not-crunch-bootstrap` and `crunch.self-build`.
- [x] [serial] I3 Add or update a naming drift guard with explicit allowed contexts for exact Crunch identifiers. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: `scripts/check-stale-branding.rs` now has a pure classifier, `--self-test`, and explicit allowances for compatibility tokens and embedded bootstrap markers.
- [x] [serial] I4 Record the classification rules in README or contributor-facing docs. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: `README.md` links `docs/mantle-naming.md`, which records replacement rules and allowed exact identifiers.

## Verification

- [x] [serial] V1 Positive: run the naming guard on allowed exact `crunch-*` identifiers and assert they remain accepted. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: pueue task 34, `nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding'` -> `stale branding check passed`.
- [x] [serial] V2 Negative: add or fixture stale user-facing Crunch prose and assert the guard rejects it. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: pueue task 35, `nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding --self-test'` -> `stale branding checker self-test passed`.
- [x] [serial] V3 Positive: run docs/examples checks that cover updated Mantle prose. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: pueue task 38, `cargo test -p mantle --test examples_inventory -- --nocapture` -> `10 passed; 0 failed`.
- [x] [serial] V4 Run `cargo fmt -p mantle --check` if Rust changed, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[build_tool_boundary.mantle_naming_consistency]
  - Evidence: pueue task 32 `cargo fmt -p mantle --check` passed; task 33 `git diff --check` passed. Post-task Cairn validation/gates were rerun after this checklist update.
