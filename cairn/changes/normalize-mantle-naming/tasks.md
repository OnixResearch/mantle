## Implementation

- [ ] [serial] I1 Inventory Crunch-vs-Mantle occurrences in user-facing docs, examples, help text, and status surfaces, classifying each as stale prose or required exact identifier. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] I2 Update stale user-facing prose to Mantle while preserving required `crunch-*` identifiers and historical archive content. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] I3 Add or update a naming drift guard with explicit allowed contexts for exact Crunch identifiers. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] I4 Record the classification rules in README or contributor-facing docs. r[build_tool_boundary.mantle_naming_consistency]

## Verification

- [ ] [serial] V1 Positive: run the naming guard on allowed exact `crunch-*` identifiers and assert they remain accepted. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] V2 Negative: add or fixture stale user-facing Crunch prose and assert the guard rejects it. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] V3 Positive: run docs/examples checks that cover updated Mantle prose. r[build_tool_boundary.mantle_naming_consistency]
- [ ] [serial] V4 Run `cargo fmt -p mantle --check` if Rust changed, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[build_tool_boundary.mantle_naming_consistency]
